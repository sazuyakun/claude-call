use std::{
    io::{self, BufRead, BufReader, Write},
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};

use crate::app::config::{PythonWakeDetectorConfig, WakeDetectorBackend, WakeDetectorConfig};

use super::event::WakeEvent;

pub fn wait_for_wake_event(config: &WakeDetectorConfig, wake_word: &str) -> Result<WakeEvent> {
    match config.backend {
        WakeDetectorBackend::Stdin => wait_for_stdin_wake_word(wake_word),
        WakeDetectorBackend::Microphone => wait_for_python_wake_event(config, wake_word),
    }
}

fn wait_for_python_wake_event(config: &WakeDetectorConfig, wake_word: &str) -> Result<WakeEvent> {
    let python = config.python.as_ref().context(
        "config wake_detector.python is required when wake_detector.backend is microphone",
    )?;

    wait_for_python_stdout_wake_event(python, wake_word)
}

fn wait_for_python_stdout_wake_event(
    python: &PythonWakeDetectorConfig,
    wake_word: &str,
) -> Result<WakeEvent> {
    tracing::info!(
        command = %python.command,
        args = ?python.args,
        "starting python wake detector"
    );

    let mut child = Command::new(&python.command)
        .args(&python.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("failed to start python wake detector: {}", python.command))?;

    let stdout = child
        .stdout
        .take()
        .context("python wake detector stdout was not captured")?;
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line.context("failed to read python wake detector output")?;
        let event = line.trim();

        match event {
            "wake" => {
                stop_python_wake_detector(child)?;
                tracing::info!(wake_word = %wake_word, "python wake detector emitted wake event");
                return Ok(WakeEvent::new(wake_word));
            }
            "" => {}
            other => {
                stop_python_wake_detector(child)?;
                bail!("invalid python wake detector event: {other}");
            }
        }
    }

    let status = child
        .wait()
        .context("failed to wait for python wake detector")?;
    bail!("python wake detector exited before emitting wake event with status {status}")
}

fn stop_python_wake_detector(mut child: std::process::Child) -> Result<()> {
    if child
        .try_wait()
        .context("failed to inspect python wake detector process")?
        .is_none()
    {
        child
            .kill()
            .context("failed to stop python wake detector process")?;
    }

    child
        .wait()
        .context("failed to wait for python wake detector shutdown")?;

    Ok(())
}

fn wait_for_stdin_wake_word(wake_word: &str) -> Result<WakeEvent> {
    let normalized_wake_word = normalize_input(wake_word);

    loop {
        print!("> ");
        io::stdout().flush().context("failed to flush stdout")?;

        let mut input = String::new();
        let bytes_read = io::stdin()
            .read_line(&mut input)
            .context("failed to read from stdin")?;

        if bytes_read == 0 {
            bail!("stdin closed");
        }

        let normalized_input = normalize_input(&input);

        if normalized_input == normalized_wake_word {
            tracing::info!(wake_word = %wake_word, "wake word detected");
            return Ok(WakeEvent::new(wake_word));
        }

        tracing::debug!(input = %normalized_input, "ignored input");
    }
}

fn normalize_input(input: &str) -> String {
    input.trim().to_lowercase()
}
