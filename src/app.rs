use crate::config::Config;

use std::sync::Arc;
use std::time::Duration;

mod timer;
use self::timer::{State, Timer};

// 25 mins
const DEFAULT_WORK_TIME: u64 = 60 * 25;
// 5 mins
const DEFAULT_SHORT_BREAK_TIME: u64 = 60 * 5;
// 20 mins
const DEFAULT_LONG_BREAK_TIME: u64 = 60 * 20;

const DEFAULT_POMODOROS_BEFORE_LONG_BREAK: u64 = 4;

pub struct App {
    state: AppState,
    timer: Timer,
    progress: f64,
    time_left: (u64, u64),
    time_elapsed: (u64, u64),
    pomodoros: u64,
    settings: AppSettings,
}

struct AppSettings {
    work_time: u64,
    short_break_time: u64,
    long_break_time: u64,
    pomodoros_before_long_break: u64,
}

#[derive(PartialEq)]
pub enum AppState {
    ShortBreak,
    LongBreak(bool),
    Work,
}

impl App {
    pub fn new(config: Arc<Config>) -> App {
        let settings = AppSettings {
            work_time: config
                .get_int("work_time")
                .map(|x| x as u64)
                .unwrap_or(DEFAULT_WORK_TIME),
            short_break_time: config
                .get_int("short_break_time")
                .map(|x| x as u64)
                .unwrap_or(DEFAULT_SHORT_BREAK_TIME),
            long_break_time: config
                .get_int("long_break_time")
                .map(|x| x as u64)
                .unwrap_or(DEFAULT_LONG_BREAK_TIME),
            pomodoros_before_long_break: config
                .get_int("pomodoros_before_long_break")
                .map(|x| x as u64)
                .unwrap_or(DEFAULT_POMODOROS_BEFORE_LONG_BREAK),
        };

        App {
            state: AppState::Work,
            timer: Timer::new(Duration::from_secs(settings.work_time), false),
            progress: 0.0,
            time_left: (0, 0),
            time_elapsed: (0, 0),
            pomodoros: 0,
            settings,
        }
    }

    pub fn progress(&self) -> f64 {
        self.progress
    }

    pub fn is_paused(&self) -> bool {
        self.timer.is_paused()
    }

    /// Returns minutes and seconds
    pub fn time_left(&self) -> (u64, u64) {
        self.time_left
    }

    /// Returns minutes and seconds
    pub fn time_elapsed(&self) -> (u64, u64) {
        self.time_elapsed
    }

    pub fn pomodoros(&self) -> u64 {
        self.pomodoros
    }

    /// Returns `(hours, minutes)` of total work time including all previous sessions and the
    /// current running time.
    /// This is entirely based on the number of pomodoros + current running time so if the user has
    /// skipped through a work session it will still count as the total time (this is the intended
    /// behaviour).
    pub fn total_work_time(&self) -> (u64, u64) {
        let historic_minutes = (self.pomodoros() * self.settings.work_time) / 60;
        let (running_minutes, running_seconds) = if self.get_state() == &AppState::Work {
            self.time_elapsed()
        } else {
            (0, 0)
        };

        let total_minutes = historic_minutes + running_minutes + (running_seconds / 60);
        let hours = total_minutes / 60;
        let minutes = total_minutes % 60;

        (hours, minutes)
    }

    pub fn get_state_name(&self) -> &'static str {
        match self.state {
            AppState::LongBreak(elongated) => {
                if elongated {
                    "Long Break (elongated)"
                } else {
                    "Long Break"
                }
            }
            AppState::ShortBreak => "Short Break",
            AppState::Work => "Work",
        }
    }

    pub fn transition_to_next_state(&mut self, last_finished: Duration) {
        let next_state = match self.state {
            AppState::LongBreak(_) | AppState::ShortBreak => AppState::Work,
            AppState::Work => {
                self.pomodoros += 1;

                if self.pomodoros % self.settings.pomodoros_before_long_break == 0 {
                    AppState::LongBreak(false)
                } else {
                    AppState::ShortBreak
                }
            }
        };

        self.transition_to_state(next_state, last_finished);
    }

    pub fn transition_to_prev_state(&mut self, last_finished: Duration) {
        let next_state = if self.pomodoros == 0 {
            // Don't keep subtracting pomodoros, there are no more states to transition between
            AppState::Work
        } else {
            match self.state {
                AppState::LongBreak(_) | AppState::ShortBreak => {
                    self.pomodoros -= 1;
                    AppState::Work
                }
                AppState::Work => {
                    let state = if self.pomodoros % self.settings.pomodoros_before_long_break == 0 {
                        AppState::LongBreak(false)
                    } else {
                        AppState::ShortBreak
                    };

                    state
                }
            }
        };

        self.transition_to_state(next_state, last_finished);
    }

    /// Resets timer to 0 (same target duration)
    pub fn reset_timer(&mut self, paused: bool) {
        self.timer.reset(paused);
    }

    pub fn forward_timer(&mut self, delta_secs: u64) {
        self.timer.forward_timer(Duration::from_secs(delta_secs));
    }

    /// Rewinds the timer without transitioning to a previous state
    pub fn rewind_timer(&mut self, delta_secs: u64) {
        self.timer.rewind_timer(Duration::from_secs(delta_secs));
    }

    pub fn transition_to_state(&mut self, next_state: AppState, elapsed_duration: Duration) {
        let time = match next_state {
            AppState::LongBreak(_) => self.settings.long_break_time,
            AppState::ShortBreak => self.settings.short_break_time,
            AppState::Work => self.settings.work_time,
        };

        self.timer =
            Timer::new_with_acc_duration(Duration::from_secs(time), false, elapsed_duration);
        self.state = next_state;
    }

    pub fn toggle_pause(&mut self) {
        if self.is_paused() {
            self.timer.start();
        } else {
            self.timer.pause();
        }
    }

    pub fn get_state(&self) -> &AppState {
        &self.state
    }

    fn update_progress_data(&mut self) {
        let (progress, time_elapsed, time_left) = self.timer.get_progress_data();

        self.progress = progress;

        let seconds_elapsed = time_elapsed.as_secs();
        self.time_elapsed = (seconds_elapsed / 60, seconds_elapsed % 60);

        let seconds_left = time_left.as_secs();
        self.time_left = (seconds_left / 60, seconds_left % 60);
    }

    pub fn update<F>(&mut self, on_new_state: &F)
    where
        F: Fn(&AppState),
    {
        match self.timer.get_state() {
            State::Paused | State::Running => self.update_progress_data(),
            State::Finished(last_finished) => {
                // All timing is state based so by using the last_finished & the recursive
                // calling of update, any lag won't cause issues with the correctness of the timer
                self.transition_to_next_state(last_finished);
                on_new_state(self.get_state());
                self.update(on_new_state);
            }
        }
    }
}
