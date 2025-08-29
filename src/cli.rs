use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueHint, command};

use crate::defaults;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to the SQLite database.
    ///
    /// Automatically created if it does not exist at the given path.
    #[arg(
        short,
        long,
        default_value = defaults::DB_PATH.to_str(),
        value_hint = ValueHint::FilePath,
        env = "CLIPVAULT_DB",
        global = true
    )]
    pub database: PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List the currently stored entries.
    #[command()]
    List(ListArgs),
    /// Store an entry. Only accepts STDIN.
    #[command()]
    Store(StoreArgs),
    /// Get the content of an entry.
    #[command()]
    Get(GetDelArgs),
    /// Delete an entry.
    #[command()]
    Delete(GetDelArgs),
    /// Delete all entries.
    #[command()]
    Clear,
}

#[derive(Debug, clap::Args)]
pub struct StoreArgs {
    /// Maximum number of entries to store.
    ///
    /// Setting this value to 0 disables the limit.
    #[arg(long, default_value_t = defaults::MAX_ENTRIES, env = "CLIPVAULT_MAX_ENTRIES")]
    pub max_entries: usize,

    /// Entries older than this value will be deleted. Only accurate to the second.
    ///
    /// Setting this value to 0s or less disables the limit.
    #[arg(long, default_value = defaults::MAX_ENTRY_AGE, env = "CLIPVAULT_MAX_AGE")]
    pub max_entry_age: humantime::Duration,

    /// Entries that are larger than this value in bytes will not be stored.
    ///
    /// Setting this value to 0 disables the limit.
    #[arg(long, default_value_t = defaults::MAX_ENTRY_LEN, env = "CLIPVAULT_MAX_LENGTH")]
    pub max_entry_length: usize,

    /// Entries that are smaller than this value in bytes will not be stored.
    ///
    /// This value must be less than `max_entry_length`.
    #[arg(long, default_value_t = defaults::MIN_ENTRY_LEN, env = "CLIPVAULT_MIN_LENGTH")]
    pub min_entry_length: usize,

    /// Store sensitive values, ignoring e.g. CLIPBOARD_STATE="sensitive" set by wl-clipboard.
    #[arg(long, action, env = "CLIPVAULT_STORE_SENSITIVE")]
    pub store_sensitive: bool,
}

#[derive(Debug, clap::Args)]
pub struct ListArgs {
    /// Maximum width in characters for the previews.
    ///
    /// Setting this value to 0 disables the limit.
    #[arg(
        short,
        long,
        default_value_t = defaults::MAX_PREVIEW_WIDTH,
        env = "CLIPVAULT_MAX_PREVIEW_WIDTH"
    )]
    pub max_preview_width: usize,

    /// Reverse the order of the list of returned entries (oldest first).
    #[arg(short, long, action, env = "CLIPVAULT_REVERSE_LIST")]
    pub reverse: bool,
}

#[derive(Debug, clap::Args)]
pub struct GetDelArgs {
    /// The selected row from `clipvault list`, or just the ID of the entry.
    ///
    /// Can also be provided through STDIN.
    #[arg(default_value(""))]
    pub input: String,
    /// The relative index of the desired entry (starting at 0). Negative
    /// values are interpreted as starting from the oldest entries first.
    /// For example, 0 represents the newest entry, 1 the entry just before that,
    /// and -1 represents the oldest entry.
    ///
    /// *NOTE*: conflicts with positional input, and will ignore
    /// STDIN in the case where input is not provided.
    // TODO: return error instead of ignore STDIN?
    #[arg(long, conflicts_with("input"), allow_hyphen_values(true))]
    pub index: Option<isize>,
}

// impl Cli {
//     fn to_config
// }
