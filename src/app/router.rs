use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{app::App, events::Event};

pub struct Router<'a> {
    app: &'a mut App,
}

impl<'a> Router<'a> {
    pub fn new(app: &'a mut App) -> Self {
        Self { app }
    }

    pub async fn route(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Char('q')) | (KeyModifiers::CONTROL, KeyCode::Char('c'))
                if !self.app.manager.has_modal() =>
            {
                Event::Quit.emit()
            }
            (_, KeyCode::Char('?')) if !self.app.manager.has_modal() => {
                self.app.showing_help = true;
            }
            (_, KeyCode::Esc) if self.app.showing_help => {
                self.app.showing_help = false;
            }
            _ => {
                let menu = self.app.manager.menu_mut();
                menu.on_key_event(key).await;
            }
        }
    }
}
