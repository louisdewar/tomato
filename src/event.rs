/// Adapted from tui-rs/examples/util/event
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termion::event::Key;
use termion::input::TermRead;

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
            let stdin = io::stdin();
            for evt in stdin.keys() {
                if let Ok(key) = evt {
                    // Will stop this thread if the main thread has dropped it's receiver
                    if key_tx.send(Event::Input(key)).is_err() {
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
