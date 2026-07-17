#[derive(Debug)]
pub struct WakeEvent {
    pub wake_word: String,
}

impl WakeEvent {
    pub fn new(wake_word: impl Into<String>) -> Self {
        Self {
            wake_word: wake_word.into(),
        }
    }
}
