use std::{cell::RefCell, rc::Rc};

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    text::Text,
    widgets::{List, ListState, StatefulWidget, WidgetRef},
};

use crate::{
    core::Core,
    events::{Event, Navigate},
    traits::Menu,
    ui::{
        common::Keymap,
        util::{HIGHLIGHT, Menu as MenuEnum, center_area, menu_block},
    },
};

pub struct Home {
    list_items: Vec<MenuEnum>,
    list_state: Rc<RefCell<ListState>>,
}

impl Home {
    pub fn new(_core: &Core) -> Self {
        Self {
            list_items: vec![MenuEnum::Directory, MenuEnum::Movie, MenuEnum::Series],
            list_state: Rc::new(RefCell::new(ListState::default().with_selected(Some(0)))),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Menu for Home {
    async fn on_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Char('j') | KeyCode::Down) => self.list_state.borrow_mut().select_next(),
            (_, KeyCode::Char('k') | KeyCode::Up) => self.list_state.borrow_mut().select_previous(),
            (_, KeyCode::Enter) => {
                if let Some(index) = self.list_state.borrow().selected() {
                    let item = &self.list_items[index];
                    match item {
                        MenuEnum::Directory => {
                            Event::Navigate(Navigate::Directories).emit();
                        }
                        MenuEnum::Movie => {
                            Event::Navigate(Navigate::Movies).emit();
                        }
                        MenuEnum::Series => {
                            Event::Navigate(Navigate::SeriesList).emit();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn keymaps(&self) -> Vec<Keymap> { vec![] }

    fn has_modal(&self) -> bool { false }
}

impl WidgetRef for Home {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let area = center_area(
            area,
            Constraint::Length(30),
            Constraint::Length(self.list_items.len() as u16 + 2),
        );

        let list = List::new(self.list_items.iter().map(|x| Text::from(x.to_string())))
            .block(menu_block("Silver"))
            .highlight_style(HIGHLIGHT);

        list.render(
            center_area(
                area,
                Constraint::Length(30),
                Constraint::Length(self.list_items.len() as u16 + 2),
            ),
            buf,
            &mut self.list_state.borrow_mut(),
        );
    }
}
