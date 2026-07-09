use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub wake_word: String,
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
}
