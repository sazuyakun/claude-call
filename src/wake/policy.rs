use std::time::{Duration, Instant};

use super::event::WakeEvent;

#[derive(Debug)]
pub enum WakeDecision {
    Accept,
    Ignore { reason: &'static str },
}

#[derive(Debug)]
pub struct WakePolicy {
    cooldown: Duration,
    last_accepted_at: Option<Instant>,
}

impl WakePolicy {
    pub fn new(cooldown: Duration) -> Self {
        Self {
            cooldown,
            last_accepted_at: None,
        }
    }

    pub fn decide(&mut self, _event: &WakeEvent) -> WakeDecision {
        let now = Instant::now();

        if let Some(last_accepted_at) = self.last_accepted_at {
            if now.duration_since(last_accepted_at) < self.cooldown {
                return WakeDecision::Ignore {
                    reason: "cooldown active",
                };
            }
        }

        self.last_accepted_at = Some(now);
        WakeDecision::Accept
    }
}
