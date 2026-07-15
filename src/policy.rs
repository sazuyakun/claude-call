use crate::event::WakeEvent;

#[derive(Debug)]
pub enum WakeDecision {
    Accept,
}

#[derive(Debug, Default)]
pub struct WakePolicy;

impl WakePolicy {
    pub fn new() -> Self {
        Self
    }

    pub fn decide(&mut self, _event: &WakeEvent) -> WakeDecision {
        WakeDecision::Accept
    }
}
