use std::time::{Duration, SystemTime};

/// Poll-based timer
pub struct Timer {
    pub last_start: SystemTime,
    pub is_paused: bool,
    pub acc_duration: Duration,
    pub target_duration: Duration,
}

pub enum State {
    Paused,
    /// Fraction of the way through the time (between 0.0 and 1.0), time elapsed, and time remaining
    Running(f64, Duration, Duration),
    /// Timer has finished, this is the instant it should have finished
    Finished(SystemTime),
}

impl Timer {
    pub fn new(target_duration: Duration, paused: bool) -> Timer {
        Timer {
            last_start: SystemTime::now(),
            is_paused: paused,
            acc_duration: Duration::new(0, 0),
            target_duration,
        }
    }

    pub fn new_with_acc_duration(
        target_duration: Duration,
        paused: bool,
        acc_duration: Duration,
    ) -> Timer {
        Timer {
            last_start: SystemTime::now(),
            is_paused: paused,
            acc_duration,
            target_duration,
        }
    }

    pub fn pause(&mut self) {
        assert!(!self.is_paused, "Tried to pause a timer that was already paused");
        self.is_paused = true;
        self.acc_duration += self.last_start.elapsed().expect("SystemTime::elapsed failed");
    }

    pub fn start(&mut self) {
        assert!(self.is_paused, "Tried to start a timer that was already running");

        self.last_start = SystemTime::now();
        self.is_paused = false;
    }

    pub fn get_state(&self) -> State {
        if self.is_paused {
            State::Paused
        } else {
            let total_duration = self.acc_duration + self.last_start.elapsed().expect("SystemTime::elapsed failed");

            if total_duration < self.target_duration {
                State::Running(
                    total_duration.as_millis() as f64 / self.target_duration.as_millis() as f64,
                    total_duration,
                    self.target_duration - total_duration,
                )
            } else {
                State::Finished(self.last_start + (self.target_duration - self.acc_duration))
            }
        }
    }
}
