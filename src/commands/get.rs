use std::{
    io::{Read, Write, stdin, stdout},
    path::Path,
};

use miette::{Context, IntoDiagnostic, Result, miette};

use super::extract_id;
use crate::{
    cli::GetDelArgs,
    commands::wrap_index,
    database::{
        data::ClipboardEntry,
        init_db,
        queries::{count_entries, get_entry_by_id, get_entry_by_position},
    },
    utils::ignore_broken_pipe,
};

fn get_entry(path_db: &Path, mut input: String) -> Result<ClipboardEntry> {
    // Read from STDIN if no argument given
    if input.is_empty() {
        stdin()
            .lock()
            .read_to_string(&mut input)
            .into_diagnostic()
            .context("failed to read STDIN")?;
    }

    let id = extract_id(input)?;
    let conn = init_db(path_db)?;
    get_entry_by_id(&conn, id)
}

fn get_entry_rel(path_db: &Path, i: isize) -> Result<ClipboardEntry> {
    let conn = &init_db(path_db)?;

    let len = count_entries(conn)?;
    if len == 0 {
        return Err(miette!("there are currently no saved clipboard entries"));
    }

    let index = wrap_index(len, i);
    get_entry_by_position(conn, index)
}

#[tracing::instrument(skip(path_db))]
fn execute_inner(path_db: &Path, args: GetDelArgs, show_output: bool) -> Result<()> {
    let GetDelArgs { input, index } = args;

    assert!(
        index.is_none() || index.is_some_and(|_| input.is_empty()),
        "conflicting relative index and input - only one of these should make it to this stage"
    );

    // Use relative index if given, otherwise parse the input for the entry ID
    let entry = if let Some(i) = index {
        get_entry_rel(path_db, i)
    } else {
        get_entry(path_db, input)
    }?;

    // Used for benchmarks - don't actually write to stdout
    if !show_output {
        return Ok(());
    }

    // Write to STDOUT
    let stdout = stdout();
    let mut stdout = stdout.lock();

    ignore_broken_pipe(stdout.write_all(&entry.content))
        .into_diagnostic()
        .context("failed to write to STDOUT")?;
    ignore_broken_pipe(stdout.flush())
        .into_diagnostic()
        .context("failed to flush STDOUT")?;

    Ok(())
}

#[tracing::instrument(skip(path_db))]
pub fn execute(path_db: &Path, args: GetDelArgs) -> Result<()> {
    execute_inner(path_db, args, true)
}

#[doc(hidden)]
#[tracing::instrument(skip(path_db))]
pub fn execute_without_output(path_db: &Path, args: GetDelArgs) -> Result<()> {
    assert!(
        !cfg!(debug_assertions),
        "Not intended to run in production code"
    );
    execute_inner(path_db, args, false)
}
