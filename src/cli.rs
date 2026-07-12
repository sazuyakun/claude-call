use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(long, default_value = "config/claude-call.toml")]
    pub config: PathBuf,

    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Debug, Subcommand)]
pub enum CliCommand {
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    Trigger,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    Check,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
