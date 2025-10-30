use std::{env::temp_dir, io::BufRead, os::unix::fs::MetadataExt, sync::LazyLock, time::Duration};

use assert_cmd::{Command, cargo_bin};
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
    let mut cmd = Command::new(cargo_bin!());
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
fn test_store_basic() {
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
        assert_eq!(limit as usize, output.stdout.lines().count());
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
fn test_store_ignore_pattern() {
    let db = &get_db();

    let store_ignore = |patterns: &[&str], input: &[u8]| {
        let mut args = Vec::from(["store"]);
        for pat in patterns {
            args.push("--ignore-pattern");
            args.push(pat);
        }
        get_cmd(db).args(args).write_stdin(input).assert().success();
    };
    let count_stored = || {
        get_cmd(db)
            .arg("list")
            .output()
            .expect("couldn't list entries")
            .stdout
            .lines()
            .count()
    };

    let mut expected = 0;

    store_ignore(&["test"], b"testing");
    assert_eq!(count_stored(), expected);

    expected += 1;
    store_ignore(&["test"], b"hello");
    assert_eq!(count_stored(), expected);

    store_ignore(&["^abc"], b"abcdefg");
    assert_eq!(count_stored(), expected);

    expected += 1;
    store_ignore(&["^abc"], b"def");
    assert_eq!(count_stored(), expected);

    store_ignore(&["world$"], b"hello world");
    assert_eq!(count_stored(), expected);

    expected += 1;
    store_ignore(&["world$"], b"goodbye earth");
    assert_eq!(count_stored(), expected);

    let http_image = r#"<meta http-equiv="content-type" content="text/html; charset=utf-8"><img src="https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Ftse4.mm.bing.net%2Fth%2Fid%2FOIP.NHEHZBx37DjUVrFOwDUNugHaE8%3Fpid%3DApi&amp;f=1&amp;ipt=dcffa9260288ea62423251c252ecf228037d6317b7911a5365b03d36ca0590fb&amp;ipo=images" alt="alsdkf dlas by dannicalifornia on DeviantArt" style="width: 100%; height: 180px; min-height: 180px; max-height: 180px;" loading="lazy">"#;
    store_ignore(
        &[r"^<meta http-equiv=", r"ignore\n$"],
        http_image.as_bytes(),
    );
    assert_eq!(count_stored(), expected);

    store_ignore(
        &[r"^<meta http-equiv=", r"ignore\n$"],
        b"\nplease\tignore\n",
    );
    assert_eq!(count_stored(), expected);

    expected += 1;
    store_ignore(&[r"^<meta http-equiv=", r"ignore\n$"], b"control");
    assert_eq!(count_stored(), expected);

    // Fails with invalid regex
    get_cmd(db)
        .args(["store", "--ignore-pattern", "[["])
        .assert()
        .failure()
        .stderr(contains("regex parse error"));
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
    assert_eq!(stdout.lines().count(), (u8::MAX - 3) as usize);
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

#[test]
fn test_creates_db_even_for_readonly() {
    let dir = temp_dir();
    let path_db = dir.as_path().join("test.db");
    let mut cmd = Command::new(cargo_bin!());
    cmd.args([
        "--database",
        path_db.to_str().expect("couldn't get DB path str"),
    ]);
    cmd.arg("list").assert().success();
    assert!(path_db.is_file());

    let db = &get_db();
    get_cmd(db).arg("list").assert().success();
}

#[test]
fn test_db_shrinks() {
    let db = &get_db();

    let get_size = || {
        db.as_file()
            .metadata()
            .expect("can't read temp DB metadata")
            .size()
    };
    let store_random = || {
        get_cmd(db)
            .arg("store")
            // Store enough random data to make the DB grow past the initial size
            .write_stdin("random_data".repeat(200))
            .assert()
            .success();
    };

    // Initialise DB with a call don't won't actually add any entries
    get_cmd(db).arg("list").assert().success();
    let initial_size = get_size();

    // DELETE WITH ID
    store_random();
    assert!(initial_size < get_size(), "DB size did not increase");

    get_cmd(db).args(["delete", "1"]).assert().success();
    assert_eq!(
        initial_size,
        get_size(),
        "DB size did not shrink after deleting an entry by ID"
    );

    // DELETE WITH RELATIVE INDEXING
    store_random();
    assert!(initial_size < get_size(), "DB size did not increase");

    get_cmd(db)
        .args(["delete", "--index", "0"])
        .assert()
        .success();
    assert_eq!(
        initial_size,
        get_size(),
        "DB size did not shrink after deleting an entry using relative indexing"
    );

    // DELETE WITH EXCEEDED ENTRY COUNT
    store_random();
    let max_size = get_size();

    get_cmd(db)
        .args(["store", "--max-entries", "1"])
        .write_stdin("test")
        .assert()
        .success();
    assert!(
        max_size > get_size(),
        "DB size did not shrink after deleting entries by exceeding the max entry count"
    );
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
