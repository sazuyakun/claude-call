mod app;
mod cli;
mod daemon;
mod wake;

use anyhow::Result;
use cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse_args();

    tracing_subscriber::fmt::init();

    app::runtime::run(&cli.config, cli.command)
}
