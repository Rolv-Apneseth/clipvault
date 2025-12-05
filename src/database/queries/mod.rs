use miette::{Context, IntoDiagnostic, Result, miette};
use rusqlite::{Connection, fallible_iterator::FallibleIterator, params};

use crate::{database::data::ClipboardEntry, utils::now};

#[tracing::instrument(skip(conn))]
pub fn count_entries(conn: &Connection) -> Result<usize> {
    tracing::debug!("getting count of total entries");

    conn.query_one(include_str!("./count_entries.sql"), params![], |row| {
        row.get::<usize, usize>(0)
    })
    .into_diagnostic()
    .context("failed to query: count of clipboard entries")
}

#[tracing::instrument(skip(conn))]
pub fn get_all_entries(conn: &Connection, preview_width: usize) -> Result<Vec<ClipboardEntry>> {
    tracing::debug!("getting all entries");

    // NOTE: query truncates the blob content based on the allowed preview width (with a minimum
    // for ensuring file signatures are present)
    let max_blob_width = (preview_width.saturating_add(preview_width)).max(50);

    let mut stmt = conn
        .prepare(include_str!("./get_all.sql"))
        .into_diagnostic()
        .context("failed to prepare: get all entries")?;

    let entries: Vec<ClipboardEntry> = stmt
        .query(params![max_blob_width])
        .into_diagnostic()
        .context("failed to query: get all entries")?
        .map(|c| ClipboardEntry::try_from(c))
        .collect()
        .into_diagnostic()
        .context("failed to create clipboard entries from database rows")?;

    Ok(entries)
}

#[tracing::instrument(skip(conn))]
pub fn delete_all_entries(conn: &Connection) -> Result<()> {
    tracing::debug!("deleting all entries");

    conn.execute(include_str!("./delete_all.sql"), params![])
        .map(|_| ())
        .into_diagnostic()
        .context("failed to execute: wipe entries")
}

/// Perform a `VACUUM` on the DB, reducing its size by clearing deleted entries and defragmenting.
#[tracing::instrument(skip(conn))]
fn vacuum(conn: &Connection) -> Result<()> {
    tracing::debug!("vacuuming DB");

    let estimated_free = get_estimated_free_space(conn)?;
    if estimated_free < 1_000_000 {
        tracing::debug!(
            "estimated freed space ({estimated_free}) under the threshold - skipping VACUUM"
        );
        return Ok(());
    }

    conn.execute("VACUUM;", params![])
        .map(|_| ())
        .into_diagnostic()
        .context("failed to execute: vacuum")
}

#[tracing::instrument(skip(conn))]
pub fn delete_entries_older_than(conn: &Connection, timestamp: u64) -> Result<usize> {
    tracing::debug!("deleting old entries");

    let changed = conn
        .execute(include_str!("./delete_old.sql"), params![timestamp])
        .into_diagnostic()
        .context("failed to execute: delete old entries")?;

    if changed > 0 {
        vacuum(conn).map(|_| changed)
    } else {
        Ok(changed)
    }
}

#[tracing::instrument(skip(conn))]
pub fn trim_entries(conn: &Connection, limit: usize) -> Result<usize> {
    tracing::debug!("trimming entries over limit");

    let count = count_entries(conn)?;
    if count <= limit {
        tracing::trace!("not over limit");
        return Ok(0);
    }

    let del = count - limit;
    let changed = conn
        .execute(include_str!("./trim_entries.sql"), params![del])
        .into_diagnostic()
        .context("failed to execute: trim clipboard entries")?;
    assert_eq!(
        del, changed,
        "should only delete specified number of entries"
    );

    vacuum(conn).map(|_| changed)
}

#[tracing::instrument(skip(conn))]
pub fn get_entry_by_id(conn: &Connection, id: u64) -> Result<ClipboardEntry> {
    tracing::debug!("trimming entries over limit");

    conn.query_one(include_str!("./get_entry.sql"), params![id], |row| {
        ClipboardEntry::try_from(row)
    })
    .into_diagnostic()
    .context("couldn't get entry by ID")
}

#[tracing::instrument(skip(conn))]
pub fn get_estimated_free_space(conn: &Connection) -> Result<u64> {
    tracing::debug!("getting estimate of space that can be freed");

    conn.query_one(
        include_str!("./estimated_free_space.sql"),
        params![],
        |row| row.get("freelist_size"),
    )
    .into_diagnostic()
    .context("couldn't get entry by ID")
}

#[tracing::instrument(skip(conn))]
pub fn delete_entry_by_id(conn: &Connection, id: u64) -> Result<()> {
    tracing::debug!("deleting specific entry by ID");

    let changed = conn
        .execute(include_str!("./delete_entry.sql"), params![id])
        .into_diagnostic()
        .context("failed to execute: delete specific entry")?;

    if changed == 0 {
        return Err(miette!("entry not found"));
    }
    assert_eq!(changed, 1, "should only delete specified entry");

    vacuum(conn)
}

#[tracing::instrument(skip(conn))]
pub fn get_entry_by_position(conn: &Connection, index: usize) -> Result<ClipboardEntry> {
    tracing::debug!("getting entry by position");

    conn.query_one(include_str!("./get_nth_entry.sql"), params![index], |row| {
        ClipboardEntry::try_from(row)
    })
    .into_diagnostic()
    .context("couldn't get entry by position")
}

#[tracing::instrument(skip(conn))]
pub fn delete_entry_by_position(conn: &Connection, index: usize) -> Result<()> {
    tracing::debug!("deleting entry by position");

    let changed = conn
        .execute(include_str!("./delete_nth_entry.sql"), params![index])
        .into_diagnostic()
        .context("couldn't delete entry by position")?;

    if changed == 0 {
        return Err(miette!("database is empty"));
    }
    assert_eq!(changed, 1, "should only delete a single entry");

    vacuum(conn)
}

#[tracing::instrument(skip_all)]
pub fn upsert_entry(conn: &Connection, content: &[u8]) -> Result<()> {
    tracing::debug!("creating entry");
    tracing::debug!(
        "entry content preview: {}",
        String::from_utf8_lossy(&content[..16.min(content.len())])
    );

    let timestamp = now();
    tracing::trace!("current_timestamp={timestamp}");

    conn.execute(
        include_str!("./upsert_post.sql"),
        params![content, timestamp],
    )
    .map(|_| ())
    .into_diagnostic()
    .context("failed to execute: upsert clipboard entry")
}
