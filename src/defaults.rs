use dirs::{config_dir, data_local_dir};
use std::{path::PathBuf, sync::LazyLock};

pub static DB_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    data_local_dir()
        .expect("could not identify user data directory")
        .join("clipvault.db")
});

pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    config_dir()
        .expect("could not identify config directory")
        .join("clipvault")
        .join("config.toml")
});

pub const MAX_ENTRIES: usize = 1000;
pub const MAX_ENTRY_AGE: &str = "14d";
pub const MAX_ENTRY_LEN: usize = 5000000;
pub const MIN_ENTRY_LEN: usize = 0;

pub const MAX_PREVIEW_WIDTH: usize = 100;
