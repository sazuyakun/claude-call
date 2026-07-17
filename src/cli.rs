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
    /// Inspect and validate Claude Call configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Run the wake listener as the daemon process.
    Daemon,
    /// Run the wake listener in the foreground terminal.
    Foreground,
    /// Ask the running daemon for its status.
    Status,
    /// Ask the daemon to run actions, or use --direct to run locally.
    Trigger {
        #[arg(long)]
        direct: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Validate the config file without running actions.
    Check,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
