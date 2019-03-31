use crate::config::Config;

use std::time::{Duration, SystemTime};
use std::sync::Arc;

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

    pub fn get_progress(&self) -> f64 {
        self.progress
    }

    pub fn is_paused(&self) -> bool {
        self.timer.is_paused
    }

    // Returns minutes and seconds
    pub fn get_time_left(&self) -> (u64, u64) {
        self.time_left
    }

    pub fn get_time_elapsed(&self) -> (u64, u64) {
        self.time_elapsed
    }

    pub fn get_pomodoros(&self) -> u64 {
        self.pomodoros
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

    pub fn transition_to_next_state(&mut self, last_finished: SystemTime) {
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

    pub fn transition_to_prev_state(&mut self, last_finished: SystemTime) {
        let next_state = if self.pomodoros == 0 {
            // Don't keep subtracting pomodoros, there are no more states to transition between
            AppState::Work
        } else {
            match self.state {
                AppState::LongBreak(_) | AppState::ShortBreak => AppState::Work,
                AppState::Work => {
                    let state = if self.pomodoros % self.settings.pomodoros_before_long_break == 0 {
                        AppState::LongBreak(false)
                    } else {
                        AppState::ShortBreak
                    };

                    self.pomodoros -= 1;

                    state
                }
            }
        };

        self.transition_to_state(next_state, last_finished);
    }

    /// Resets timer to 0 (same target duration)
    pub fn reset_timer(&mut self, paused: bool) {
        self.timer = Timer::new(self.timer.target_duration, paused)
    }

    pub fn transition_to_state(&mut self, next_state: AppState, last_finished: SystemTime) {
        let time = match next_state {
            AppState::LongBreak(_) => self.settings.long_break_time,
            AppState::ShortBreak => self.settings.short_break_time,
            AppState::Work => self.settings.work_time,
        };

        self.timer = Timer::new_with_acc_duration(
            Duration::from_secs(time),
            false,
            last_finished.elapsed().expect("SystemTime::elapsed failed"),
        );
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

    pub fn update<F>(&mut self, on_new_state: &F)
    where
        F: Fn(&AppState) -> (),
    {
        match self.timer.get_state() {
            State::Paused => self.timer.is_paused = true,
            State::Running(progress, time_elapsed, time_left) => {
                self.progress = progress;

                let seconds_elapsed = time_elapsed.as_secs();
                self.time_elapsed = (seconds_elapsed / 60, seconds_elapsed % 60);

                let seconds_left = time_left.as_secs();
                self.time_left = (seconds_left / 60, seconds_left % 60);
            }
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
