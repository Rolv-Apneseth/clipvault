use std::path::PathBuf;

use miette::Result;

use crate::database::{init_db, queries::delete_all_entries};

#[tracing::instrument(skip(path_db))]
pub fn execute(path_db: PathBuf) -> Result<()> {
    let conn = &init_db(&path_db)?;
    delete_all_entries(conn)?;
    Ok(())
}
