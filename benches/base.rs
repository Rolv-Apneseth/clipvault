use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use std::ffi::OsString;
use std::str::FromStr;
use std::sync::LazyLock;
use std::time::Duration;

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
        let bytes = OsString::from_str("0".repeat(n).as_ref()).unwrap();
        let args = StoreArgs {
            bytes: Some(bytes),
            ..Default::default()
        };
        store::execute(db.path(), args).expect("failed to store");
    }
    db
});

fn store(n: usize) {
    let db = get_temp();

    let bytes = OsString::from_str("a".repeat(n).as_ref()).unwrap();
    for _ in 0..n {
        let args = StoreArgs {
            bytes: Some(bytes.clone()),
            ..Default::default()
        };
        store::execute(db.path(), args).expect("failed to store");
    }
}

fn list(n: usize) {
    let path_db = DB.path();

    let args = ListArgs {
        max_preview_width: n,
        ..Default::default()
    };

    list::execute_without_output(path_db, args).expect("failed to list");
}

fn get(n: isize) {
    let path_db = DB.path();

    let args = GetDelArgs {
        input: String::new(),
        index: Some(n),
    };

    get::execute_without_output(path_db, args).expect("failed to get");
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut g = c.benchmark_group("base");
    g.warm_up_time(Duration::from_secs(1))
        .significance_level(0.01);

    // STORE
    for n in [1, 10, 100, 1000] {
        g.bench_with_input(BenchmarkId::new("store", n), &n, |b, i| {
            b.iter(|| store(*i));
        });
    }

    // LIST
    for n in [1, 5, 10, 25, 50, 100, 1000] {
        g.bench_with_input(BenchmarkId::new("list", n), &n, |b, i| {
            b.iter(|| list(*i));
        });
    }

    // GET
    for n in [-100000, -1, 0, 1, 100000] {
        g.bench_with_input(BenchmarkId::new("get", n), &n, |b, i| {
            b.iter(|| {
                for _ in 0..100 {
                    get(*i)
                }
            });
        });
    }

    g.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
