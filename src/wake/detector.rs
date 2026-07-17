use std::io::{self, Write};

use anyhow::{Context, Result, bail};

use super::event::WakeEvent;

pub fn wait_for_wake_word(wake_word: &str) -> Result<WakeEvent> {
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
