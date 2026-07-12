mod actions;
mod cli;
mod config;
mod detector;

use actions::run_actions;
use anyhow::Result;
use cli::Cli;
use config::Config;
use detector::wait_for_wake_word;

fn main() -> Result<()> {
    let _cli = Cli::parse_args();

    tracing_subscriber::fmt::init();

    let config = Config::load_from_file("config/claude-call.toml")?;

    tracing::info!(
        wake_word = %config.wake_word,
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

    loop {
        tracing::info!("listening for wake word");
        wait_for_wake_word(&config.wake_word)?;
        run_actions(&config.actions)?;
    }
}
