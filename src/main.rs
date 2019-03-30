#![warn(clippy::all)]

mod event;
use crate::event::{Event, Events};

mod app;
use crate::app::{App, AppState};

mod config;
use crate::config::Config;

mod ui;
use ui::Ui;

fn main() -> Result<(), failure::Error> {
    // Setup event handlers
    let events = Events::new();

    let config_file_path = dirs::home_dir()
        .map(|mut home| {
            home.push(".config/tomato_timer.conf");
            home
        })
        .unwrap_or_else(|| {
            let mut path = std::path::PathBuf::new();
            path.push("/etc/tomato_timer/timer.conf");
            path
        });

    let config = Config::new_from_config_file(config_file_path.as_path()).unwrap_or_else(|_| Config::new());

    let on_work_start = config.get_string("on_work_start");
    let on_break_start = config.get_string("on_break_start");

    // Create default app state
    let mut app = App::new(&config);

    let mut ui = Ui::new_with_termion()?;

    loop {
        ui.render(&app)?;

        use termion::event::Key;
        match events.next()? {
            // TODO: Consider moving the input logic to it's own file
            Event::Input(input) => match input {
                Key::Char('q') | Key::Esc => break,
                Key::Char('p') => app.toggle_pause(),
                Key::Right => {
                    app.transition_to_next_state(std::time::SystemTime::now());
                    handle_next_state(app.get_state(), on_work_start, on_break_start);
                },
                Key::Left => {
                    let (minutes, seconds) = app.get_time_elapsed();

                    if minutes == 0 && seconds < 2 {
                        app.transition_to_prev_state(std::time::SystemTime::now());
                        handle_next_state(app.get_state(), on_work_start, on_break_start);
                    } else {
                        app.reset_timer(false);
                    }
                }
                Key::Char('l') => {
                    if app.get_state() == &AppState::ShortBreak {
                        app.transition_to_state(
                            AppState::LongBreak(true),
                            std::time::SystemTime::now(),
                        )
                    }
                }
                _ => {}
            },
            Event::Tick => {
                app.update(&|next_state| {
                    handle_next_state(next_state, on_work_start, on_break_start);
                });
            }
        }
    }

    Ok(())
}

fn handle_next_state(next_state: &AppState, on_work_start: Option<&String>, on_break_start: Option<&String>) {
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
            },
            AppState::ShortBreak => {
                if let Some(script) = on_break_start {
                    Command::new("sh")
                                .arg("-c")
                                .arg(script)
                                .env("BREAK_TYPE", "short")
                                .spawn()
                                .expect("failed to execute break (short) start script");
                }
            },
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
