use std::{fs::create_dir_all, path::PathBuf};

use dirs::state_dir;
use miette::{Context, IntoDiagnostic, Result};
use tracing::level_filters::LevelFilter;
use tracing_appender::{non_blocking::WorkerGuard, rolling::Rotation};
use tracing_subscriber::{EnvFilter, prelude::*};

fn get_logs_dir() -> Result<PathBuf> {
    let dir = state_dir()
        .expect("could not identify state directory")
        .join("clipvault")
        .join("logs");

    if !dir.exists() {
        create_dir_all(&dir)
            .into_diagnostic()
            .context("failed to create logs directory")?
    }

    Ok(dir)
}

pub fn init_logging() -> Result<WorkerGuard> {
    let dir = get_logs_dir()?;

    // Non-blocking, rolling log file appender
    let file_appender = tracing_appender::rolling::Builder::new()
        .rotation(Rotation::DAILY)
        .filename_suffix("log")
        .max_log_files(8)
        .build(dir)
        .expect("failed to build log appender");

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Layers
    let fmt = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false);
    let env = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env_lossy();

    // Init subscriber
    tracing_subscriber::registry().with(fmt).with(env).init();

    Ok(guard)
}

pub fn trace_err(e: &miette::Error) {
    tracing::error!("{e}");
    let mut source = e.source();
    while let Some(err) = source {
        source = err.source();
        tracing::error!("-> {err}")
    }
}
