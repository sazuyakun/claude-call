use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(long, default_value = "config/claude-call.toml")]
    pub config: PathBuf,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
