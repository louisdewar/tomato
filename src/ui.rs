use std::io;

use tui::layout::Rect;
use tui::Terminal;

use crossterm::{terminal, ExecutableCommand};
use tui::backend::CrosstermBackend;

use crate::app::App;

pub type BackendType = CrosstermBackend<std::io::Stdout>;

pub struct Ui {
    terminal: Terminal<BackendType>,
    last_size: Rect,
}

impl Ui {
    pub fn new_with_termion() -> Result<Ui, crossterm::ErrorKind> {
        // Terminal initialization
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        stdout.execute(terminal::EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);

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

            let time_left = app.time_left();

            let percent_progress = (app.progress() * 100.0).round() as u16;
            let total_work_time = app.total_work_time();

            Gauge::default()
                .block(
                    Block::default()
                        .title(&format!(
                            " Timer - {} pomodoros complete - {}h{}m of work - {} ",
                            app.pomodoros(),
                            total_work_time.0,
                            total_work_time.1,
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

pub fn cleanup() {
    terminal::disable_raw_mode().unwrap();
    io::stdout()
        .execute(terminal::LeaveAlternateScreen)
        .unwrap();
}
