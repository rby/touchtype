use std::time::{Duration, Instant};

use crate::msg::Msg;

pub(crate) struct Stats {
    duration_sum: Duration,
    last_key: Option<Instant>,
    count: u32,
}

impl Stats {
    pub(crate) fn add(&mut self, msg: Msg) {
        match msg {
            Msg::KeyPressed(_, _, _, ts) => {
                self.count += 1;
                self.duration_sum += match self.last_key {
                    Some(last_key) => ts.duration_since(last_key),
                    None => Duration::ZERO,
                };
                self.last_key = Some(ts);
            }
        }
    }

    pub(crate) fn avg_key_s(&self) -> f32 {
        if self.duration_sum.is_zero() {
            0.0
        } else {
            self.count as f32 / self.duration_sum.as_secs_f32()
        }
    }
    pub(crate) fn new() -> Self {
        Stats {
            duration_sum: Duration::ZERO,
            last_key: None,
            count: 0,
        }
    }
}
