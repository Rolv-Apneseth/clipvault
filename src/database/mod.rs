use std::{fs, os::unix::fs::PermissionsExt, path::Path, sync::LazyLock};

use include_dir::{Dir, include_dir};
use miette::{Context, IntoDiagnostic, Result};
use rusqlite::Connection;
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
        .context("failed to connect to the database")
}

/// Ensures that the DB file matches the expected permissions, and modifies
/// if it does not.
///
/// DB file should be read-writeable by the owner only (i.e. `chmod 600`).
pub fn ensure_db_permissions(path_db: &Path) -> Result<()> {
    if !path_db.is_file() {
        return Ok(());
    }

    let mut perms = fs::metadata(path_db)
        .into_diagnostic()
        .context("failed to read DB file metadata")?
        .permissions();

    if perms.mode() != 0o600 {
        perms.set_mode(0o600);
        fs::set_permissions(path_db, perms)
            .into_diagnostic()
            .context("failed to set DB file permissions")?;
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations() {
        assert!(MIGRATIONS.validate().is_ok());
        insta::assert_debug_snapshot!(MIGRATIONS);
    }
}
