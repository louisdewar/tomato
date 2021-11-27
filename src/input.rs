use crate::{App, AppState, Config};

use crossterm::event::KeyCode as Key;

use std::sync::Arc;

pub struct InputManager {
    on_work_start: Option<String>,
    on_break_start: Option<String>,
}

impl InputManager {
    pub fn new(config: Arc<Config>) -> InputManager {
        let on_work_start = config.get_string("on_work_start").cloned();
        let on_break_start = config.get_string("on_break_start").cloned();

        // TODO: Use config for keys
        InputManager {
            on_work_start,
            on_break_start,
        }
    }

    /// Handles the input and alters the app accordingly.
    /// Returns false when the app should stop
    pub fn handle_input(&self, input: Key, app: &mut App) -> bool {
        match input {
            Key::Char('q') | Key::Esc => return false,
            Key::Char('p') => app.toggle_pause(),
            Key::Right => {
                app.transition_to_next_state(std::time::Duration::new(0, 0));
                handle_next_state(
                    app.get_state(),
                    self.on_work_start.as_ref(),
                    self.on_break_start.as_ref(),
                );
            }
            Key::Left => {
                let (minutes, seconds) = app.time_elapsed();

                if minutes == 0 && seconds < 2 {
                    app.transition_to_prev_state(std::time::Duration::new(0, 0));
                    handle_next_state(
                        app.get_state(),
                        self.on_work_start.as_ref(),
                        self.on_break_start.as_ref(),
                    );
                } else {
                    app.reset_timer(false);
                }
            }
            Key::Char('l') => {
                if app.get_state() == &AppState::ShortBreak {
                    app.transition_to_state(
                        AppState::LongBreak(true),
                        std::time::Duration::new(0, 0),
                    )
                }
            }
            Key::Char('-') => app.rewind_timer(1),
            Key::Char('=') => app.forward_timer(1),
            Key::Char('[') => app.rewind_timer(5),
            Key::Char(']') => app.forward_timer(5),
            Key::Char(',') => app.rewind_timer(60),
            Key::Char('.') => app.forward_timer(60),
            _ => {}
        }

        true
    }
}

pub fn handle_next_state(
    next_state: &AppState,
    on_work_start: Option<&String>,
    on_break_start: Option<&String>,
) {
    use std::process::Command;
    match next_state {
        AppState::LongBreak(_) => {
            if let Some(script) = on_break_start {
                Command::new("sh")
                    .arg("-c")
                    .env("BREAK_TYPE", "long")
                    .arg(script)
                    .spawn()
                    .expect("failed to execute break (long) start script");
            }
        }
        AppState::ShortBreak => {
            if let Some(script) = on_break_start {
                Command::new("sh")
                    .arg("-c")
                    .arg(script)
                    .env("BREAK_TYPE", "short")
                    .spawn()
                    .expect("failed to execute break (short) start script");
            }
        }
        AppState::Work => {
            if let Some(script) = on_work_start {
                Command::new("sh")
                    .arg("-c")
                    .arg(script)
                    .spawn()
                    .expect("failed to execute work start script");
            }
        }
    }
}
