use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TranscriptEvent {
    pub text: String,
}

impl TranscriptEvent {
    pub fn new(text: String) -> Result<Self> {
        ensure!(!text.trim().is_empty(), "transcript text cannot be empty");

        Ok(Self { text })
    }
}
