use std::{cell::RefCell, rc::Rc};

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    text::{Line, Text},
    widgets::{Block, BorderType, List, ListState, StatefulWidget, WidgetRef},
};

use crate::{
    core::{
        Core,
        events::{event::Event, navigate::Navigate},
    },
    traits::menu::Menu,
    ui::util::Menu as MenuEnum,
    ui::util::center_area,
};

pub struct Home {
    list_items: Vec<MenuEnum>,
    list_state: Rc<RefCell<ListState>>,
}

impl Menu for Home {
    fn new(_core: &Core) -> Self {
        Self {
            list_items: vec![MenuEnum::Directories, MenuEnum::Movies],
            list_state: Rc::new(RefCell::new(ListState::default().with_selected(Some(0)))),
        }
    }

    fn on_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Char('j')) => self.list_state.borrow_mut().select_next(),
            (_, KeyCode::Char('k')) => self.list_state.borrow_mut().select_previous(),
            (_, KeyCode::Enter) => {
                if let Some(index) = self.list_state.borrow().selected() {
                    let item = &self.list_items[index];
                    match item {
                        MenuEnum::Movies => {
                            Event::Navigate(Navigate::Movies).emit();
                        }
                        MenuEnum::Directories => {
                            Event::Navigate(Navigate::Directories).emit();
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

impl WidgetRef for Home {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let area = center_area(
            area,
            Constraint::Length(30),
            Constraint::Length(self.list_items.len() as u16 + 2),
        );

        let title = Line::from("Hello, strigoi");
        let list = List::new(self.list_items.iter().map(|x| Text::from(x.to_string())))
            .block(
                Block::bordered()
                    .title(title.centered())
                    .border_type(BorderType::Rounded),
            )
            .highlight_symbol("* ");

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
