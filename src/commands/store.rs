use std::{
    io::{Read, stdin},
    path::Path,
};

use content_inspector::ContentType;
use miette::{Context, IntoDiagnostic, Result, miette};
use tracing::instrument;

use crate::{
    cli::StoreArgs,
    database::{
        init_db,
        queries::{delete_all_entries, delete_entries_older_than, trim_entries, upsert_entry},
    },
    utils::now,
};

#[instrument]
pub fn execute(path_db: &Path, args: StoreArgs) -> Result<()> {
    let StoreArgs {
        max_entries,
        max_entry_age: max_age,
        max_entry_length: max_bytes,
        min_entry_length: min_bytes,
        store_sensitive,
        ignore_pattern,
    } = args;

    // Min conflicts with max
    if min_bytes > max_bytes {
        return Err(miette!(
            "minimum entry length ({min_bytes}) exceeds maximum entry length ({max_bytes})"
        ));
    }

    // Set by `wl-clipboard`
    if let Ok(s) = std::env::var("CLIPBOARD_STATE") {
        tracing::debug!("CLIPBOARD_STATE={s}");
        match s.as_str() {
            // Clipboard contains a sensitive value - skip if not storing sensitive values
            // As of writing, the latest release of `wl-clipboard` does not include the changes for
            // marking sensitive values using x-kde-passwordManagerHint.
            "sensitive" if !store_sensitive => {
                tracing::trace!("sensitive - not storing");
                return Ok(());
            }
            // Clipboard explicitly cleared - clear history as well.
            // As of writing, "clear" is not yet used by `wl-clipboard`.
            "clear" => {
                tracing::debug!("explicitly cleared clipboard");
                return delete_all_entries(&init_db(path_db)?);
            }
            // Clipboard is empty - nothing to store
            "nil" => return Ok(()),
            _ => {}
        }
    };

    // Read input from STDIN
    let mut buf = vec![];
    stdin()
        .read_to_end(&mut buf)
        .into_diagnostic()
        .context("failed to read from STDIN")?;

    // No content to store
    if buf.is_empty() {
        tracing::trace!("no content to store");
        return Ok(());
    }

    // Ignore content larger than the max size or smaller than the min size in bytes
    let gt_max = buf.len() > max_bytes && max_bytes != 0;
    let lt_min = buf.len() < min_bytes;
    if gt_max || lt_min {
        tracing::debug!(
            "content length ({}) is outside the bounds {min_bytes}->{max_bytes}",
            buf.len()
        );
        return Ok(());
    }

    // Ignore purely whitespace content
    if buf.trim_ascii().is_empty() {
        tracing::debug!("only ASCII whitespace content");
        return Ok(());
    }

    // Check user-provided ignore pattern
    if let Some(regexes) = ignore_pattern
        && matches!(
            content_inspector::inspect(&buf),
            ContentType::UTF_8 | ContentType::UTF_8_BOM
        )
        && regexes
            .iter()
            .any(|re| re.is_match(&String::from_utf8_lossy(&buf)))
    {
        tracing::debug!("content matched an ignore pattern");
        return Ok(());
    }

    // Only get DB connection after parsing STDIN - avoid locking
    let conn = &init_db(path_db)?;

    // Delete old entries
    let max_age = max_age.as_secs();
    if max_age != 0 {
        let timestamp = now() - max_age;
        delete_entries_older_than(conn, timestamp)?;
    }

    // Upsert new entry
    upsert_entry(conn, &buf)?;

    // Trim entries if over limit
    if max_entries != 0 {
        trim_entries(conn, max_entries)?;
    }

    Ok(())
}
