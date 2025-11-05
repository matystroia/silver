use crossterm::event::KeyCode;
use derive::Close;
use ratatui::{
    layout::Constraint,
    widgets::{Block, Clear, Paragraph, Widget, WidgetRef},
};

use crate::{
    traits::modal::{Close, Modal},
    ui::util::center_area,
};

#[derive(Close)]
pub struct AddDir {
    value: String,
    on_confirm: Option<Box<dyn FnOnce(String)>>,
    pub close: bool,
}

impl AddDir {
    pub fn new(value: impl Into<String>, on_confirm: impl FnOnce(String) + 'static) -> Self {
        Self {
            value: value.into(),
            on_confirm: Some(Box::new(on_confirm)),
            close: false,
        }
    }
}

impl Modal for AddDir {
    fn on_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char(ch) => {
                self.value.push(ch);
            }
            KeyCode::Backspace => {
                self.value.pop();
            }
            KeyCode::Enter => {
                if let Some(on_confirm) = self.on_confirm.take() {
                    on_confirm(self.value.clone());
                    self.close = true;
                }
            }
            KeyCode::Esc => {
                self.close = true;
            }
            _ => {}
        }
    }
}

impl WidgetRef for AddDir {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let area = center_area(area, Constraint::Percentage(50), Constraint::Length(3));

        Clear.render(area, buf);
        let text = Paragraph::new(self.value.clone()).block(Block::bordered());
        text.render(area, buf);
    }
}
