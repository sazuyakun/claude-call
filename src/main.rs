mod actions;
mod config;
mod detector;

use anyhow::Result;
use config::Config;
use detector::wait_for_wake_word;

fn main() -> Result<()> {
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

    tracing::info!("listening for wake word");
    wait_for_wake_word(&config.wake_word)?;

    Ok(())
}
