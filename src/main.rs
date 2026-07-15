mod actions;
mod cli;
mod config;
mod detector;
mod event;
mod policy;

use actions::run_actions;
use anyhow::Result;
use cli::{Cli, CliCommand, ConfigCommand};
use config::Config;
use detector::wait_for_wake_word;
use policy::{WakeDecision, WakePolicy};

fn main() -> Result<()> {
    let cli = Cli::parse_args();

    tracing_subscriber::fmt::init();

    let config = Config::load_from_file(&cli.config)?;
    config.validate()?;

    if matches!(
        cli.command,
        Some(CliCommand::Config {
            command: ConfigCommand::Check
        })
    ) {
        tracing::info!(config_path = %cli.config.display(), "config ok");
        return Ok(());
    }

    tracing::info!(
        config_path = %cli.config.display(),
        wake_word = %config.wake_word,
        cooldown_seconds = config.cooldown_seconds,
        actions = config.actions.len(),
        "loaded config"
    );

    for action in &config.actions {
        tracing::debug!(
            name = %action.name,
            command = %action.command,
            args = ?action.args,
            "loaded action"
        );
    }

    if matches!(cli.command, Some(CliCommand::Trigger)) {
        tracing::info!("manual trigger requested");
        run_actions(&config.actions)?;
        return Ok(());
    }

    let mut wake_policy = WakePolicy::new();

    loop {
        tracing::info!("listening for wake word");
        let wake_event = wait_for_wake_word(&config.wake_word)?;
        tracing::debug!(wake_word = %wake_event.wake_word, "wake event received");

        match wake_policy.decide(&wake_event) {
            WakeDecision::Accept => run_actions(&config.actions)?,
        }
    }
}
