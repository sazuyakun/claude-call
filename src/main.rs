mod actions;
mod config;
mod detector;

use anyhow::Result;
use config::Config;

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

    Ok(())
}
