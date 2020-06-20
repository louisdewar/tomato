use std::time::{Duration, SystemTime};

/// Poll-based timer
pub struct Timer {
    last_start: SystemTime,
    is_paused: bool,
    acc_duration: Duration,
    target_duration: Duration,
}

pub enum State {
    Paused,
    Running,
    /// Timer has finished, this is the amount of excess time that has passed since it finished
    Finished(Duration),
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
        assert!(
            !self.is_paused,
            "Tried to pause a timer that was already paused"
        );
        self.is_paused = true;
        self.acc_duration += self
            .last_start
            .elapsed()
            .expect("SystemTime::elapsed failed");
    }

    pub fn start(&mut self) {
        assert!(
            self.is_paused,
            "Tried to start a timer that was already running"
        );

        self.last_start = SystemTime::now();
        self.is_paused = false;
    }

    pub fn get_state(&self) -> State {
        let elapsed = if self.is_paused {
            Duration::new(0, 0)
        } else {
            self.last_start
                .elapsed()
                .expect("SystemTime::elapsed failed")
        };

        let total_duration = self.acc_duration + elapsed;
        if total_duration < self.target_duration {
            if self.is_paused {
                State::Paused
            } else {
                State::Running
            }
        } else {
            State::Finished(total_duration - self.target_duration)
        }
    }

    /// If the timer is unpaused it takes the elapsed time and accumulates it resetting the
    /// last_start, otherwise does nothing
    fn accumulate_elapsed_time(&mut self) {
        if !self.is_paused {
            self.pause();
            self.start();
        }
    }

    /// Returns fraction of the way through the time (between 0.0 and 1.0), time elapsed, and time remaining
    pub fn get_progress_data(&self) -> (f64, Duration, Duration) {
        let current_elapsed = if self.is_paused {
            Duration::new(0, 0)
        } else {
            self.last_start
                .elapsed()
                .expect("SystemTime::elapsed failed")
        };

        let total_duration = self.acc_duration + current_elapsed;
        (
            total_duration.as_millis() as f64 / self.target_duration.as_millis() as f64,
            total_duration,
            self.target_duration - total_duration,
        )
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn reset(&mut self, paused: bool) {
        self.last_start = SystemTime::now();
        self.acc_duration = Duration::new(0, 0);
        self.is_paused = paused;
    }

    pub fn forward_timer(&mut self, duration: Duration) {
        self.acc_duration += duration;
    }

    pub fn rewind_timer(&mut self, duration: Duration) {
        self.accumulate_elapsed_time();

        // Subtract from the accumulated duration, can't go below 0
        self.acc_duration = self
            .acc_duration
            .checked_sub(duration)
            .unwrap_or_else(|| Duration::new(0, 0));
    }
}
