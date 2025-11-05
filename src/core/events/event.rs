use std::{path::PathBuf, sync::OnceLock};

use color_eyre::Result;
use crossterm::event::KeyEvent;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::core::{
    data::directory::Directory, events::navigate::Navigate, notify::message::Message,
};

static TX: OnceLock<UnboundedSender<Event>> = OnceLock::new();

#[derive(PartialEq, Eq, Hash)]
pub enum Event {
    Render,
    Resize,
    Key(KeyEvent),
    Navigate(Navigate),
    Notify(Message),
    Scan,
    RmDir(Directory),
    AddDir(PathBuf),
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
