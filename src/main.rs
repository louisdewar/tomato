#![warn(clippy::all)]

mod event;
use crate::event::{Event, Events};

mod app;
use crate::app::{App, AppState};

mod config;
use crate::config::Config;

mod ui;
use ui::Ui;

mod input;
use input::InputManager;

use std::sync::Arc;

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

    let config = Arc::new(
        Config::new_from_config_file(config_file_path.as_path()).unwrap_or_else(|_| Config::new()),
    );

    let on_work_start = config.get_string("on_work_start");
    let on_break_start = config.get_string("on_break_start");

    // Create default app state
    let mut app = App::new(Arc::clone(&config));
    let mut ui = Ui::new_with_termion()?;
    let input_manager = InputManager::new(Arc::clone(&config));

    loop {
        ui.render(&app)?;
        match events.next()? {
            Event::Input(input) => {
                if !input_manager.handle_input(input, &mut app) {
                    // Handle_input has returned false which means that the app should exit
                    break;
                }
            }
            Event::Tick => {
                app.update(&|next_state| {
                    input::handle_next_state(next_state, on_work_start, on_break_start);
                });
            }
        }
    }

    Ok(())
}
