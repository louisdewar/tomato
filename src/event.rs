/// Adapted from tui-rs/examples/util/event
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossterm::event::{read, Event as TerminalEvent, KeyCode as Key};

pub enum Event<I> {
    Input(I),
    Tick,
}

/// An small event handler that wraps termion input and tick events. Each event
/// type is handled in its own thread.
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
}

impl Events {
    pub fn new(tick_rate_millis: u64) -> Events {
        let (tx, rx) = mpsc::channel();

        // Transmitter for key events
        let key_tx = tx.clone();
        thread::spawn(move || {
            while let Ok(event) = read() {
                if let TerminalEvent::Key(key) = event {
                    // Will stop this thread if the main thread has dropped it's receiver
                    if key_tx.send(Event::Input(key.code)).is_err() {
                        return;
                    }
                }
            }
        });

        let tick_rate = Duration::from_millis(tick_rate_millis);

        thread::spawn(move || {
            let tx = tx.clone();
            loop {
                if tx.send(Event::Tick).is_err() {
                    return;
                }

                thread::sleep(tick_rate);
            }
        });

        Events { rx }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }
}
