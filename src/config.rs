use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub wake_word: String,
    #[serde(default = "default_cooldown_seconds")]
    pub cooldown_seconds: u64,
    pub actions: Vec<ActionConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ActionConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

impl Config {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read config from {}", path.display()))?;

        toml::from_str(&contents)
            .with_context(|| format!("failed to parse config from {}", path.display()))
    }

    pub fn validate(&self) -> Result<()> {
        if self.wake_word.trim().is_empty() {
            bail!("config wake_word must not be empty");
        }

        if self.cooldown_seconds == 0 {
            bail!("config cooldown_seconds must be greater than 0");
        }

        if self.actions.is_empty() {
            bail!("config must define at least one action");
        }

        for (index, action) in self.actions.iter().enumerate() {
            let action_number = index + 1;

            if action.name.trim().is_empty() {
                bail!("config action {action_number} name must not be empty");
            }

            if action.command.trim().is_empty() {
                bail!("config action {action_number} command must not be empty");
            }
        }

        Ok(())
    }
}

fn default_cooldown_seconds() -> u64 {
    5
}
