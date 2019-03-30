#![warn(clippy::all)]

use std::io;

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Gauge, Widget};
use tui::Terminal;

mod event;
use crate::event::{Event, Events};

mod app;
use crate::app::{App, AppState};

mod config;
use crate::config::Config;

fn main() -> Result<(), failure::Error> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

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

    // Create default app state
    let mut app = App::new(&config);

    let mut last_size = terminal.size()?;

    let on_work_start = config.get_string("on_work_start");
    let on_break_start = config.get_string("on_break_start");

    loop {
        let size = terminal.size()?;

        if size != last_size {
            terminal.resize(size)?;
            last_size = size;
        }

        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Percentage(25), Constraint::Percentage(75)].as_ref())
                .split(size);

            let time_left = app.get_time_left();

            Gauge::default()
                .block(
                    Block::default()
                        .title(&format!(
                            " Timer ({} pomodoros) - {} ",
                            app.get_pomodoros(),
                            app.get_state_name()
                        ))
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::Yellow).bg(Color::Red))
                .percent((app.get_progress() * 100.0).round() as u16)
                .label(&format!(
                    "-{}:{:02} - {}% {}",
                    time_left.0,
                    time_left.1,
                    (app.get_progress() * 100.0).round() as u16,
                    if app.is_paused() { "(Paused)" } else { "" }
                ))
                .render(&mut f, chunks[0]);
        })?;

        match events.next()? {
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
