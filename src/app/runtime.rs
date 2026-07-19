use std::{path::Path, time::Duration};

use anyhow::{Context, Result};

use crate::{
    app::{
        actions::run_actions, config::Config, routing::route_transcript_to_opencode,
        transcript::TranscriptEvent,
    },
    cli::{CliCommand, ConfigCommand},
    daemon::control::{request_status, request_transcript, request_trigger, start_control_server},
    wake::{
        detector::wait_for_wake_word,
        policy::{WakeDecision, WakePolicy},
    },
};

pub fn run(config_path: &Path, command: Option<CliCommand>) -> Result<()> {
    if matches!(command.as_ref(), Some(CliCommand::Status)) {
        tracing::info!("daemon status requested");
        return request_status();
    }

    if matches!(
        command.as_ref(),
        Some(CliCommand::Trigger { direct: false })
    ) {
        tracing::info!("daemon trigger requested");
        return request_trigger();
    }

    if let Some(CliCommand::Transcript {
        direct: false,
        text,
    }) = command.as_ref()
    {
        let transcript = TranscriptEvent::new(text.clone())?;
        tracing::info!("daemon transcript requested");
        request_transcript(&transcript)?;
        return Ok(());
    }

    let config = load_config(config_path)?;

    if matches!(
        command.as_ref(),
        Some(CliCommand::Config {
            command: ConfigCommand::Check
        })
    ) {
        tracing::info!(config_path = %config_path.display(), "config ok");
        return Ok(());
    }

    log_config(config_path, &config);

    if matches!(command.as_ref(), Some(CliCommand::Trigger { direct: true })) {
        tracing::info!("direct manual trigger requested");
        run_actions(&config.actions)?;
        return Ok(());
    }

    if let Some(CliCommand::Transcript { direct: true, text }) = command.as_ref() {
        let transcript = TranscriptEvent::new(text.clone())?;
        let routing = config
            .routing
            .as_ref()
            .context("config routing.opencode is required for transcript routing")?;

        tracing::info!("direct transcript requested");
        route_transcript_to_opencode(&transcript, &routing.opencode)?;
        return Ok(());
    }

    match command.as_ref() {
        Some(CliCommand::Daemon) => {
            tracing::info!("daemon wake listener requested");
            start_control_server(config.actions.clone(), config.routing.clone())?;
        }
        Some(CliCommand::Foreground) => tracing::info!("foreground wake listener requested"),
        _ => {}
    }

    run_interactive(config)
}

fn load_config(config_path: &Path) -> Result<Config> {
    let config = Config::load_from_file(config_path)?;
    config.validate()?;
    Ok(config)
}

fn log_config(config_path: &Path, config: &Config) {
    tracing::info!(
        config_path = %config_path.display(),
        wake_word = %config.wake_word,
        cooldown_seconds = config.cooldown_seconds,
        actions = config.actions.len(),
        routing = config.routing.is_some(),
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
}

fn run_interactive(config: Config) -> Result<()> {
    let mut wake_policy = WakePolicy::new(Duration::from_secs(config.cooldown_seconds));

    loop {
        tracing::info!("listening for wake word");
        let wake_event = wait_for_wake_word(&config.wake_word)?;
        tracing::debug!(wake_word = %wake_event.wake_word, "wake event received");

        match wake_policy.decide(&wake_event) {
            WakeDecision::Accept => {
                tracing::info!(wake_word = %wake_event.wake_word, "wake event accepted");
                run_actions(&config.actions)?;
            }
            WakeDecision::Ignore { reason } => {
                tracing::info!(wake_word = %wake_event.wake_word, reason, "wake event ignored");
            }
        }
    }
}
