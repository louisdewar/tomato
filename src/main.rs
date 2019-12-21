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

use clap::{crate_authors, crate_version, load_yaml, App as Arguments};

fn main() -> Result<(), failure::Error> {
    let yaml = load_yaml!("cli.yml");
    let matches = Arguments::from(yaml)
        .version(crate_version!())
        .author(crate_authors!())
        .get_matches();

    // Setup event handlers
    let events = Events::new(250);

    use std::path::PathBuf;

    let config = Arc::new(
        matches
            .value_of("config")
            .map(PathBuf::from)
            .map(|path| Config::new_from_config_file(path).expect("Couldn't find your config file"))
            .or_else(|| {
                // If there wasn't a config specified try $HOME/.config/tomato_timer.conf
                dirs::home_dir()
                    .map(|mut home| {
                        home.push(".config/tomato_timer.conf");
                        home
                    })
                    // Silently try to get a default config file
                    .and_then(|path| Config::new_from_config_file(path).ok())
            })
            // If default oconfig file couldn't be found then silently just use an empty one
            .unwrap_or_else(Config::new),
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
