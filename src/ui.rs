use std::io;

use tui::layout::Rect;
use tui::{backend::TermionBackend, Terminal};

use termion::{
    input::MouseTerminal,
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};

use crate::app::App;

// TODO: There has got to be a better way of doing this type signature...
pub type BackendType = TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<std::io::Stdout>>>>;

pub struct Ui {
    terminal: Terminal<BackendType>,
    last_size: Rect,
}

impl Ui {
    pub fn new_with_termion() -> Result<Ui, io::Error> {
        // Terminal initialization
        let stdout = io::stdout().into_raw_mode()?;
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);

        let backend = TermionBackend::new(stdout);

        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        let last_size = terminal.size()?;

        Ok(Ui {
            terminal,
            last_size,
        })
    }

    pub fn render(&mut self, app: &App) -> Result<(), io::Error> {
        let size = self.terminal.size()?;

        if size != self.last_size {
            self.terminal.resize(size)?;
            self.last_size = size;
        }

        self.terminal.draw(|mut f| {
            use tui::layout::{Constraint, Direction, Layout};
            use tui::style::{Color, Style};
            use tui::widgets::{Block, Borders, Gauge, Widget};

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(14), Constraint::Min(0)].as_ref())
                .split(size);

            let time_left = app.get_time_left();

            let percent_progress = (app.get_progress() * 100.0).round() as u16;

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
                .percent(percent_progress)
                .label(&format!(
                    "-{}:{:02} - {}% {}",
                    time_left.0,
                    time_left.1,
                    percent_progress,
                    if app.is_paused() { "(Paused)" } else { "" }
                ))
                .render(&mut f, chunks[0]);
        })?;

        Ok(())
    }
}
