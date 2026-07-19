use std::process::Command;

use anyhow::{Context, Result};

use crate::app::{config::OpencodeRoutingConfig, transcript::TranscriptEvent};

pub fn route_transcript_to_opencode(
    transcript: &TranscriptEvent,
    route: &OpencodeRoutingConfig,
) -> Result<()> {
    tracing::info!(
        project_path = %route.project_path.display(),
        session_id = %route.session_id,
        agent = route.agent.as_deref().unwrap_or("default"),
        "routing transcript to opencode"
    );

    let mut command = Command::new(route.command());
    command
        .arg("run")
        .arg("--session")
        .arg(&route.session_id)
        .arg("--dir")
        .arg(&route.project_path);

    if let Some(agent) = &route.agent {
        command.arg("--agent").arg(agent);
    }

    let status = command
        .arg(&transcript.text)
        .status()
        .with_context(|| format!("failed to run {}", route.command()))?;

    if !status.success() {
        anyhow::bail!("opencode routing failed with status {status}");
    }

    tracing::info!(session_id = %route.session_id, "transcript routed to opencode");
    println!("routed transcript to opencode session {}", route.session_id);

    Ok(())
}
