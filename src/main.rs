mod actions;
mod cli;
mod config;
mod detector;
mod event;
mod policy;
mod runtime;

use anyhow::Result;
use cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse_args();

    tracing_subscriber::fmt::init();

    runtime::run(&cli.config, cli.command)
}
