use std::ops::{Deref, DerefMut};

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Clear, Paragraph, WidgetRef},
};

use crate::{
    traits::Modal,
    ui::util::{center_area, menu_block},
};

pub struct Input {
    pub value: String,
}

impl Input {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn on_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(ch)) => {
                self.value.push(ch);
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                self.value.pop();
            }
            _ => {}
        }
    }
}

impl WidgetRef for Input {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let text = Paragraph::new(Line::from(vec![
            self.value.as_str().into(),
            Span::raw("█").style(Style::new().yellow()),
        ]))
        .block(menu_block("Input"));

        text.render(area, buf);
    }
}

pub struct InputModal<F> {
    pub input: Input,
    pub on_confirm: Option<F>,
    pub close: bool,
}

impl<F> InputModal<F>
where
    F: AsyncFnOnce(String),
{
    pub fn new(value: impl Into<String>, on_confirm: F) -> Self {
        Self {
            input: Input::new(value),
            on_confirm: Some(on_confirm),
            close: false,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<F> Modal for InputModal<F>
where
    F: AsyncFnOnce(String),
{
    async fn on_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc) => {
                self.close = true;
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                if let Some(on_confirm) = self.on_confirm.take() {
                    on_confirm(self.input.value.clone()).await;
                    self.close = true;
                }
            }
            _ => self.input.on_key_event(key),
        }
    }

    fn should_close(&self) -> bool {
        self.close
    }
}

impl<F> WidgetRef for InputModal<F> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let area = center_area(area, Constraint::Percentage(50), Constraint::Length(3));
        Clear.render(area, buf);
        self.input.render_ref(area, buf);
    }
}

impl<F> Deref for InputModal<F> {
    type Target = Input;
    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

impl<F> DerefMut for InputModal<F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.input
    }
}
