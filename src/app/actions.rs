use std::process::Command;

use anyhow::{Context, Result, bail};

use super::config::ActionConfig;

pub fn run_actions(actions: &[ActionConfig]) -> Result<()> {
    for action in actions {
        run_action(action)?;
    }

    Ok(())
}

fn run_action(action: &ActionConfig) -> Result<()> {
    tracing::info!(name = %action.name, command = %action.command, "running action");

    let status = Command::new(&action.command)
        .args(&action.args)
        .status()
        .with_context(|| format!("failed to run action {}", action.name))?;

    if !status.success() {
        bail!("action {} exited with {}", action.name, status);
    }

    tracing::info!(name = %action.name, "action completed");

    Ok(())
}
