use std::{
    io::{Read, stdin},
    path::PathBuf,
};

use miette::{Context, IntoDiagnostic, Result, miette};

use super::{extract_id, wrap_index};
use crate::{
    cli::GetDelArgs,
    database::{
        init_db,
        queries::{count_entries, delete_entry_by_id, delete_entry_by_position},
    },
};

#[tracing::instrument(skip(path_db))]
pub fn execute(path_db: PathBuf, args: GetDelArgs) -> Result<()> {
    let GetDelArgs { input, index } = args;

    assert!(
        index.is_none() || index.is_some_and(|_| input.is_empty()),
        "conflicting relative index and input - only one of these should make it to this stage"
    );

    // Use relative index if given, otherwise parse the input for the entry ID
    if let Some(i) = index {
        delete_entry_rel(path_db, i)
    } else {
        delete_entry(path_db, input)
    }
}

fn delete_entry(path_db: PathBuf, mut input: String) -> Result<()> {
    // Read from STDIN if no argument given
    if input.is_empty() {
        stdin()
            .lock()
            .read_to_string(&mut input)
            .into_diagnostic()
            .context("failed to read STDIN")?;
    }

    let id = extract_id(input)?;
    let conn = &init_db(&path_db)?;
    delete_entry_by_id(conn, id)
}

fn delete_entry_rel(path_db: PathBuf, i: isize) -> Result<()> {
    let conn = &init_db(&path_db)?;

    let len = count_entries(conn)?;
    if len == 0 {
        return Err(miette!("there are currently no saved clipboard entries"));
    }

    let index = wrap_index(len, i);
    delete_entry_by_position(conn, index)
}
