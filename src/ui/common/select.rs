use std::cell::RefCell;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Flex},
    prelude::*,
    widgets::{Clear, List, ListState, Widget, WidgetRef},
};

use crate::{
    traits::Modal,
    ui::util::{HIGHLIGHT, menu_block},
};

pub struct Select<T, F> {
    options: Vec<T>,
    labels: Vec<String>,
    list_state: RefCell<ListState>,
    on_confirm: Option<F>,
    close: bool,
}

impl<T, F> Select<T, F>
where
    F: AsyncFnOnce(&T),
{
    pub fn new(options: Vec<T>, label: impl Fn(&T) -> String, on_confirm: F) -> Self {
        let labels = options.iter().map(label).collect();
        Self {
            options,
            labels,
            list_state: RefCell::new(ListState::default().with_selected(Some(0))),
            on_confirm: Some(on_confirm),
            close: false,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<T, F> Modal for Select<T, F>
where
    F: AsyncFnOnce(&T),
{
    async fn on_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Char('j') | KeyCode::Down) => {
                self.list_state.borrow_mut().select_next();
            }
            (KeyModifiers::NONE, KeyCode::Char('k') | KeyCode::Up) => {
                self.list_state.borrow_mut().select_previous()
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                let Some(i) = self.list_state.borrow().selected() else {
                    return;
                };
                if let Some(on_confirm) = self.on_confirm.take() {
                    (on_confirm)(&self.options[i]).await;
                    self.close = true;
                }
            }
            (KeyModifiers::NONE, KeyCode::Esc) => {
                self.close = true;
            }
            _ => {}
        }
    }

    fn should_close(&self) -> bool { self.close }
}

impl<T, F> WidgetRef for Select<T, F> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [list_area] = Layout::vertical([Constraint::Length(self.options.len() as u16 + 2)])
            .flex(Flex::End)
            .areas(area);

        let list = List::new(self.labels.iter().map(Span::from))
            .block(menu_block("Select"))
            .highlight_style(HIGHLIGHT);

        Clear.render(list_area, buf);
        StatefulWidget::render(list, list_area, buf, &mut self.list_state.borrow_mut());
    }
}
