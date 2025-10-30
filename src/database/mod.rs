use std::{path::Path, sync::LazyLock};

use include_dir::{Dir, include_dir};
use miette::{Context, IntoDiagnostic, Result};
use rusqlite::{Connection, OpenFlags};
use rusqlite_migration::Migrations;
use tracing::instrument;

pub mod data;
pub mod queries;

// DB MIGRATIONS, DEFINED IN ./migrations
static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/database/migrations");
static MIGRATIONS: LazyLock<Migrations<'static>> =
    LazyLock::new(|| Migrations::from_directory(&MIGRATIONS_DIR).unwrap());

/// Get a database connection. Make sure the DB is initialised somewhere first
/// before calling this function.
pub fn get_db_connection(path_db: &Path) -> Result<Connection> {
    Connection::open(path_db)
        .into_diagnostic()
        .context("failed to get a read/write connection to the database")
}

pub fn get_db_connection_readonly(path_db: &Path) -> Result<Connection> {
    Connection::open_with_flags(
        path_db,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .into_diagnostic()
    .context("failed to get readonly connection to the database")
}

#[instrument]
/// Initialise the database, creating the file and applying migrations if needed
pub fn init_db(path_db: &Path) -> Result<Connection> {
    tracing::debug!("initialising DB");
    let mut conn = get_db_connection(path_db)?;

    tracing::trace!("applying PRAGMA");
    conn.pragma_update(None, "journal_mode", "WAL")
        .into_diagnostic()
        .context("failed to apply PRAGMA: journal mode")?;

    tracing::trace!("applying migrations");
    MIGRATIONS
        .to_latest(&mut conn)
        .into_diagnostic()
        .context("failed to apply migrations")?;

    Ok(conn)
}

#[instrument]
/// Initialise the database in readonly mode (but still creating the file and applying migrations
/// if it does not exist).
pub fn init_readonly_db(path_db: &Path) -> Result<Connection> {
    tracing::debug!("initialising readonly DB");

    if !path_db.is_file() {
        tracing::debug!("DB file does not exist - creating with read/write connection first");
        init_db(path_db)?;
    }

    let conn = get_db_connection_readonly(path_db)?;
    if !conn
        .table_exists(None, "clipboard")
        .into_diagnostic()
        .context("failed to check if table exists")?
    {
        tracing::debug!("DB table does not exist - setting up with read/write connection");
        init_db(path_db)?;
    };
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations() {
        assert!(MIGRATIONS.validate().is_ok());
        insta::assert_debug_snapshot!(MIGRATIONS);
    }
}
