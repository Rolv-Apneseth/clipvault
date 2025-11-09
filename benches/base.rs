use std::io::Cursor;
use std::sync::LazyLock;

use clipvault::cli::{GetDelArgs, ListArgs, StoreArgs};
use clipvault::commands::{get, list, store};
use clipvault::defaults;
use tempfile::NamedTempFile;

/// Get temporary file for DB.
fn get_temp() -> NamedTempFile {
    NamedTempFile::new().expect("couldn't create tempfile")
}

static DB: LazyLock<NamedTempFile> = LazyLock::new(|| {
    let db = get_temp();
    for n in 0..defaults::MAX_ENTRIES {
        let args = StoreArgs::default();
        let bytes = "0".repeat(n).into_bytes();
        store::execute_with_source(db.path(), args, Cursor::new(bytes)).expect("failed to store");
    }
    db
});

#[divan::bench(args = [1, 10, 100, 10_000, 100_000, 1_000_000], sample_size=10)]
fn store(n: usize) {
    let db = get_temp();

    let args = StoreArgs::default();
    let bytes = "0".repeat(n).into_bytes();
    store::execute_with_source(db.path(), args, Cursor::new(bytes)).expect("failed to store");
}

#[divan::bench(args = [1, 5, 10, 25, 50, 100, 1000], sample_size=10)]
fn list(n: usize) {
    let path_db = DB.path();

    let args = ListArgs {
        max_preview_width: n,
        ..Default::default()
    };

    list::execute_without_output(path_db, args).expect("failed to list");
}

#[divan::bench(args = [-100000, -1, 0, 1, 100000], sample_size=100)]
fn get(n: isize) {
    let path_db = DB.path();

    let args = GetDelArgs {
        input: String::new(),
        index: Some(n),
    };

    get::execute_without_output(path_db, args).expect("failed to get");
}

fn main() {
    divan::main();
}
