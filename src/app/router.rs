use crate::{app::App, core::events::event::Event};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct Router<'a> {
    app: &'a mut App,
}

impl<'a> Router<'a> {
    pub fn new(app: &'a mut App) -> Self {
        Self { app }
    }

    pub fn route(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Char('q')) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                Event::Quit.emit()
            }
            _ => {
                let menu = self.app.manager.menu_mut();
                menu.on_key_event(key);
            }
        }
    }
}
