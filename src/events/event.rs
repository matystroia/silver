use std::sync::OnceLock;

use crossterm::event::KeyEvent;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{core::notify::Message, events::Navigate};

static TX: OnceLock<UnboundedSender<Event>> = OnceLock::new();

#[derive(PartialEq, Eq, Hash)]
pub enum Event {
    Render,
    Key(KeyEvent),
    Navigate(Navigate),
    Notify(Message),
    Suspend,
    Terminate(i32),
    Quit,
}

impl Event {
    pub fn init() -> UnboundedReceiver<Self> {
        let (tx, rx) = mpsc::unbounded_channel();
        let _ = TX.set(tx);
        rx
    }

    pub fn emit(self) {
        if let Some(tx) = TX.get() {
            let _ = tx.send(self);
        }
    }
}
