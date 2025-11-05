use crossterm::event::KeyCode;
use derive::Close;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::{Block, Clear, Paragraph, Widget, WidgetRef},
};

use crate::traits::modal::{Close, Modal};
use crate::ui::util::center_area;

#[derive(Close)]
pub struct Confirm {
    text: String,
    on_confirm: Option<Box<dyn FnOnce()>>,
    pub close: bool,
}

impl Confirm {
    pub fn new(text: impl Into<String>, on_confirm: impl FnOnce() + 'static) -> Self {
        Self {
            text: text.into(),
            on_confirm: Some(Box::new(on_confirm)),
            close: false,
        }
    }
}

impl Modal for Confirm {
    fn on_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('y') => {
                if let Some(on_confirm) = self.on_confirm.take() {
                    on_confirm();
                }
            }
            KeyCode::Char('n') => self.close = true,
            _ => {}
        }
    }
}

impl WidgetRef for Confirm {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let area = center_area(area, Constraint::Percentage(50), Constraint::Length(7));
        let [text_area, button_area] = Layout::new(
            Direction::Vertical,
            vec![Constraint::Fill(1), Constraint::Length(3)],
        )
        .areas(area);
        let [yes_area, no_area] =
            Layout::new(Direction::Horizontal, vec![Constraint::Fill(1); 2]).areas(button_area);

        Clear.render(area, buf);
        let text = Paragraph::new(self.text.as_str()).block(Block::bordered());
        text.render_ref(text_area, buf);

        let yes_button = Paragraph::new("[Y]es")
            .alignment(Alignment::Center)
            .block(Block::bordered());
        let no_button = Paragraph::new("[N]o")
            .alignment(Alignment::Center)
            .block(Block::bordered());
        yes_button.render_ref(yes_area, buf);
        no_button.render_ref(no_area, buf);
    }
}
