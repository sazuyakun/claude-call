use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

const DEFAULT_WAKE_DETECTOR_BACKEND: WakeDetectorBackend = WakeDetectorBackend::Stdin;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub wake_word: String,
    pub cooldown_seconds: u64,
    #[serde(default)]
    pub wake_detector: WakeDetectorConfig,
    pub routing: Option<RoutingConfig>,
    pub actions: Vec<ActionConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WakeDetectorConfig {
    #[serde(default = "default_wake_detector_backend")]
    pub backend: WakeDetectorBackend,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WakeDetectorBackend {
    Stdin,
    Microphone,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RoutingConfig {
    pub opencode: OpencodeRoutingConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OpencodeRoutingConfig {
    pub project_path: PathBuf,
    pub session_id: String,
    pub command: Option<String>,
    pub agent: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
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

        self.wake_detector.validate()?;

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

        if let Some(routing) = &self.routing {
            routing.validate()?;
        }

        Ok(())
    }
}

impl Default for WakeDetectorConfig {
    fn default() -> Self {
        Self {
            backend: DEFAULT_WAKE_DETECTOR_BACKEND,
        }
    }
}

impl WakeDetectorConfig {
    fn validate(&self) -> Result<()> {
        match self.backend {
            WakeDetectorBackend::Stdin | WakeDetectorBackend::Microphone => Ok(()),
        }
    }
}

fn default_wake_detector_backend() -> WakeDetectorBackend {
    DEFAULT_WAKE_DETECTOR_BACKEND
}

impl RoutingConfig {
    fn validate(&self) -> Result<()> {
        if self.opencode.project_path.as_os_str().is_empty() {
            bail!("config routing.opencode.project_path must not be empty");
        }

        if self.opencode.session_id.trim().is_empty() {
            bail!("config routing.opencode.session_id must not be empty");
        }

        if let Some(command) = &self.opencode.command {
            if command.trim().is_empty() {
                bail!("config routing.opencode.command must not be empty when set");
            }
        }

        if let Some(agent) = &self.opencode.agent {
            if agent.trim().is_empty() {
                bail!("config routing.opencode.agent must not be empty when set");
            }
        }

        Ok(())
    }
}

impl OpencodeRoutingConfig {
    pub fn command(&self) -> &str {
        self.command.as_deref().unwrap_or("opencode")
    }
}
