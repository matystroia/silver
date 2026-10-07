use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout, Spacing},
    style::{Color, Style, Stylize},
    symbols::merge::MergeStrategy,
    widgets::{Block, BorderType, Paragraph, Widget, WidgetRef},
};

use crate::traits::Modal;

pub struct Confirm<F> {
    text: String,
    on_confirm: Option<F>,
    close: bool,
}

impl<F> Confirm<F>
where
    F: AsyncFnOnce(),
{
    pub fn new(text: impl Into<String>, on_confirm: F) -> Self {
        Self {
            text: text.into(),
            on_confirm: Some(on_confirm),
            close: false,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<F> Modal for Confirm<F>
where
    F: AsyncFnOnce(),
{
    async fn on_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('y') => {
                if let Some(on_confirm) = self.on_confirm.take() {
                    on_confirm().await;
                    self.close = true;
                }
            }
            KeyCode::Char('n') | KeyCode::Esc => self.close = true,
            _ => {}
        }
    }

    fn should_close(&self) -> bool {
        self.close
    }
}

impl<F> WidgetRef for Confirm<F> {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .merge_borders(MergeStrategy::Fuzzy)
            .reset();

        let text = Paragraph::new(self.text.as_str())
            .alignment(Alignment::Center)
            .block(block.clone());

        let [text_area, button_area] = Layout::vertical([
            Constraint::Length(text.line_count(area.width) as u16),
            Constraint::Length(3),
        ])
        .spacing(Spacing::Overlap(1))
        .flex(Flex::End)
        .areas(area);

        let [yes_area, no_area] = Layout::horizontal([Constraint::Fill(1); 2])
            .spacing(Spacing::Overlap(1))
            .areas(button_area);

        text.render(text_area, buf);

        let yes_button = Paragraph::new("[Y]es")
            .alignment(Alignment::Center)
            .block(block.clone())
            .style(Style::new().fg(Color::Green));
        yes_button.render(yes_area, buf);

        let no_button = Paragraph::new("[N]o")
            .alignment(Alignment::Center)
            .block(block)
            .style(Style::new().fg(Color::Red));
        no_button.render(no_area, buf);
    }
}
