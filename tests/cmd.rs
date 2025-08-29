use std::{sync::LazyLock, time::Duration};

use assert_cmd::Command;
use base64::{
    Engine, alphabet,
    engine::{self, GeneralPurposeConfig},
};
use clipvault::database::init_db;
use predicates::{
    prelude::PredicateBooleanExt,
    str::{contains, is_empty},
};
use proptest::prelude::*;
use tempfile::NamedTempFile;

/// Used to get a new database file for each test.
fn get_db() -> NamedTempFile {
    NamedTempFile::new().expect("couldn't create tempfile")
}

/// Builds the command to be run, pointing at the given temporary file for the database.
fn get_cmd(db: &NamedTempFile) -> Command {
    let mut cmd = Command::cargo_bin("clipvault").expect("failed to build cmd");

    cmd.args(["--database", &db.path().to_string_lossy()]);
    cmd
}

const ENCODED_BINARY: &[(&str, &[u8])] = &[
    (
        "image/png",
        b"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABAQAAAAA3bvkkAAAACklEQVR4AWNgAAAAAgABc3UBGAAAAABJRU5ErkJggg==",
    ),
    (
        "image/jpeg",
        b"/9j/2wBDAAMCAgICAgMCAgIDAwMDBAYEBAQEBAgGBgUGCQgKCgkICQkKDA8MCgsOCwkJDRENDg8QEBEQCgwSExIQEw8QEBD/yQALCAABAAEBAREA/8wABgAQEAX/2gAIAQEAAD8A0s8g/9k=",
    ),
    (
        "image/bmp",
        b"Qk1+AAAAAAAAAHoAAABsAAAAAQAAAAEAAAABACAAAwAAAAQAAADEDgAAxA4AAAAAAAAAAAAAAAD/AAD/AAD/AAAAAAAA/yBuaVcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD/////",
    ),
    (
        "image/gif",
        b"R0lGODlhAQABAIABAP///wAAACH5BAEKAAEALAAAAAABAAEAAAICTAEAOw==",
    ),
    (
        "image/x-icon",
        b"AAABAAEAAQEAAAEAIAAwAAAAFgAAACgAAAABAAAAAgAAAAEAIAAAAAAABAAAAAAAAAAAAAAAAAAAAAAAAAD/////AAAAAA==",
    ),
    (
        "image/webp",
        b"UklGRhYAAABXRUJQVlA4TAoAAAAvAAAAAEX/I/of",
    ),
    (
        "image/tiff",
        b"SUkqAAoAAACAAA0AAAEDAAEAAAABAAAAAQEDAAEAAAABAAAAAgEDAAEAAAABAAAAAwEDAAEAAAABAAAABgEDAAEAAAABAAAACgEDAAEAAAABAAAAEQEEAAEAAAAIAAAAEgEDAAEAAAABAAAAFQEDAAEAAAABAAAAFgEDAAEAAAABAAAAFwEEAAEAAAABAAAAHAEDAAEAAAABAAAAKQEDAAIAAAAAAAEAAAAAAA==",
    ),
];

#[test]
fn test_cmd_store() {
    let db = &get_db();

    // TEXT
    for str in [
        "testing",
        "123",
        "clipvault",
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
        "lorem ipsum",
    ] {
        let assert = get_cmd(db).arg("store").write_stdin(str).assert();
        assert.success();

        let assert = get_cmd(db).arg("list").assert();
        assert.success().stdout(contains(str));
    }

    // BINARY
    let decoder = engine::GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::default());

    for (mime, encoded) in ENCODED_BINARY {
        let bytes = decoder.decode(encoded).unwrap();
        get_cmd(db)
            .arg("store")
            .write_stdin(bytes)
            .assert()
            .success();
        get_cmd(db)
            .arg("list")
            .assert()
            .success()
            .stdout(contains(*mime));
    }
}

#[test]
fn test_store_max_entries() {
    let db = &get_db();
    for limit in [1, 100, u8::MAX] {
        for i in 0..u8::MAX {
            get_cmd(db)
                .args(["store", "--max-entries", &limit.to_string()])
                .write_stdin(i.to_string().as_bytes())
                .assert()
                .success();
        }
        let assert = get_cmd(db).arg("list").assert();
        let output = assert.get_output();
        assert_eq!(
            limit as usize,
            String::from_utf8_lossy(&output.stdout).lines().count()
        );
        assert.success();
    }

    get_cmd(db).arg("clear").assert().success();
    let assert = get_cmd(db).arg("list").assert();
    assert.stdout(is_empty()).success();
}

#[test]
fn test_store_max_age() {
    let db = &get_db();
    get_cmd(db)
        .args(["store", "--max-entry-age", "1s"])
        .write_stdin("max_age")
        .assert()
        .success();
    let assert = get_cmd(db).arg("list").assert();
    assert.success().stdout(contains("max_age"));

    std::thread::sleep(Duration::from_secs(2));

    get_cmd(db)
        .args(["store", "--max-entry-age", "1s"])
        .write_stdin("abc")
        .assert()
        .success();
    let assert = get_cmd(db).arg("list").assert();
    assert
        .success()
        .stdout(contains("abc").and(contains("max_age").not()));
}

#[test]
fn test_store_max_length() {
    let db = &get_db();

    get_cmd(db)
        .args(["store", "--max-entry-length", "5"])
        .write_stdin("bytes")
        .assert()
        .success();
    let assert = get_cmd(db).arg("list").assert();
    assert.success().stdout(contains("bytes"));

    get_cmd(db)
        .args(["store", "--max-entry-length", "1"])
        .write_stdin("bytes2")
        .assert()
        .success();
    let assert = get_cmd(db).arg("list").assert();
    assert.success().stdout(contains("bytes2").not());
}

#[test]
fn test_store_min_length() {
    let db = &get_db();

    get_cmd(db)
        .args(["store", "--min-entry-length", "5"])
        .write_stdin("bytes")
        .assert()
        .success();
    let assert = get_cmd(db).arg("list").assert();
    assert.success().stdout(contains("bytes"));

    get_cmd(db)
        .args(["store", "--min-entry-length", "5"])
        .write_stdin("abcd")
        .assert()
        .success();
    let assert = get_cmd(db).arg("list").assert();
    assert.success().stdout(contains("abcd").not());
}

#[test]
fn test_store_min_max_conflict() {
    let db = &get_db();

    get_cmd(db)
        .args([
            "store",
            "--min-entry-length",
            "5",
            "--max-entry-length",
            "4",
        ])
        .write_stdin("bytes")
        .assert()
        .failure();

    get_cmd(db)
        // Exceeds default value for max entry length
        .args(["store", "--min-entry-length", &usize::MAX.to_string()])
        .write_stdin("bytes")
        .assert()
        .failure();
}

#[test]
fn test_get_del() {
    let db = &get_db();

    for i in 0..u8::MAX {
        let bytes = i.to_string().as_bytes().to_owned();
        let assert = get_cmd(db).arg("store").write_stdin(bytes).assert();
        assert.success();
    }

    let assert = get_cmd(db).arg("get").write_stdin("5").assert();
    assert.success().stdout(contains("4"));

    let assert = get_cmd(db).arg("get").write_stdin("255").assert();
    assert.success().stdout(contains("254"));

    let assert = get_cmd(db).args(["get", "--index", "0"]).assert();
    assert.success().stdout(contains("254"));
    let assert = get_cmd(db).args(["get", "--index", "1"]).assert();
    assert.success().stdout(contains("253"));
    let assert = get_cmd(db).args(["get", "--index", "-1"]).assert();
    assert.success().stdout(contains("0"));
    let assert = get_cmd(db).args(["get", "--index", "-2"]).assert();
    assert.success().stdout(contains("1"));

    let assert = get_cmd(db).arg("delete").write_stdin("5").assert();
    assert.success();
    get_cmd(db).arg("get").write_stdin("5").assert().failure();

    let assert = get_cmd(db).arg("delete").write_stdin("254").assert();
    assert.success();
    get_cmd(db).arg("get").write_stdin("254").assert().failure();

    let assert = get_cmd(db).args(["delete", "--index", "-1"]).assert();
    assert.success();
    get_cmd(db).arg("get").write_stdin("1").assert().failure();

    let stdout = get_cmd(db).arg("list").output().unwrap().stdout;
    let str = String::from_utf8_lossy(&stdout);
    assert_eq!(str.lines().count(), (u8::MAX - 3) as usize);
}

#[test]
fn test_get_del_input_index_conflict() {
    let db = &get_db();

    // Avoid empty DB
    get_cmd(db)
        .arg("store")
        .write_stdin("test")
        .assert()
        .success();

    // Can't have relative index and input to parse ID from defined
    get_cmd(db)
        .args(["get", "--index", "5", "7"])
        .assert()
        .failure()
        .stderr(contains("cannot be used with"));
    get_cmd(db)
        .args(["get", "--index", "2", "9\ttext"])
        .assert()
        .failure()
        .stderr(contains("cannot be used with"));

    // STDIN ignored
    get_cmd(db)
        .args(["get", "--index", "2"])
        .write_stdin("12")
        .assert()
        .success();
}

// PROP TESTS
/// Re-use DB for prop tests as it is not necessary for each one to have its own.
static PROPTEST_DB: LazyLock<NamedTempFile> = LazyLock::new(|| {
    let db = get_db();
    // Solves sporadic failures due to locked DB
    init_db(db.path()).expect("failed to init proptest DB");
    db
});

proptest! {
    #[test]
    fn prop_test_store_bytes(bytes: Vec<u8>) {
        get_cmd(&PROPTEST_DB).arg("store").write_stdin(bytes).assert().success();
    }

    #[test]
    fn prop_test_store_text(s: String) {
        get_cmd(&PROPTEST_DB).arg("store").write_stdin(s).assert().success();
    }

    #[test]
    fn prop_test_store_max_length(bytes: usize) {
        let s = bytes.to_string();
        get_cmd(&PROPTEST_DB).args(["store", "--max-entry-length", &s]).write_stdin(s.as_bytes()).assert().success();
    }

    #[test]
    fn prop_test_store_min_length(bytes: usize) {
        let s = bytes.to_string();
        get_cmd(&PROPTEST_DB).args(["store", "--min-entry-length", &s, "--max-entry-length", &s]).write_stdin(s.as_bytes()).assert().success();
    }

    #[test]
    fn prop_test_store_max_entries(entries: usize) {
        let s = entries.to_string();
        get_cmd(&PROPTEST_DB).args(["store", "--max-entries", &s]).write_stdin(s.as_bytes()).assert().success();
    }

    #[test]
    fn prop_test_get_by_relative_index(index: isize) {
        let s = index.to_string();
        get_cmd(&PROPTEST_DB).arg("store").write_stdin(s.as_bytes()).assert().success();
        get_cmd(&PROPTEST_DB).args(["get", "--index", &s]).write_stdin(s.as_bytes()).assert().success();
    }
}
