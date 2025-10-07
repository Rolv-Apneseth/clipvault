use clap::Parser;

use clipvault::{
    cli::{Cli, Commands},
    commands,
    logging::{init_logging, trace_err},
};

use miette::{Context, IntoDiagnostic, Result};

fn main() -> Result<()> {
    let _guard = init_logging()?;

    let args = argfile::expand_args_from(
        std::env::args_os(),
        argfile::parse_fromfile,
        argfile::PREFIX,
    )
    .into_diagnostic()
    .context("failed to parse arguments from argfile")
    .inspect_err(trace_err)?;

    let args = Cli::parse_from(args);
    let path_db = args.database;

    match args.command {
        Commands::List(args) => commands::list::execute(&path_db, args),
        Commands::Store(args) => commands::store::execute(&path_db, args),
        Commands::Get(args) => commands::get::execute(&path_db, args),
        Commands::Delete(args) => commands::delete::execute(&path_db, args),
        Commands::Clear => commands::clear::execute(&path_db),
    }
    .inspect_err(trace_err)
}
