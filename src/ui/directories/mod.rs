use std::{cell::RefCell, rc::Rc, sync::RwLock};

use crate::{
    core::{
        Core,
        data::{Data, directory::Directory},
        events::{event::Event, navigate::Navigate},
    },
    traits::{menu::Menu, modal::Modal},
    ui::{
        common::confirm::Confirm,
        directories::{add_dir::AddDir, dir_info::DirInfo},
    },
};
use crossterm::event::KeyCode;
use ratatui::{
    prelude::*,
    widgets::{Block, Row, Table, TableState, WidgetRef},
};

mod add_dir;
mod dir_info;

pub struct Directories {
    directories: Vec<(Directory, DirInfo)>,
    table_state: Rc<RefCell<TableState>>,
    modal: Option<Box<dyn Modal>>,
}

impl Directories {
    fn dirs_with_info(data: &RwLock<Data>) -> Vec<(Directory, DirInfo)> {
        let data = data.read().unwrap();
        data.library
            .directories()
            .map(|dir| (dir.clone(), DirInfo::from(dir)))
            .collect()
    }

    fn add_dir(&mut self) {
        self.modal = Some(Box::new(AddDir::new("", |dir| {
            Event::AddDir(dir.into()).emit()
        })));
    }

    fn rm_dir(&mut self) {
        let i = match self.table_state.borrow().selected() {
            Some(i) => i,
            None => return,
        };

        let dir = self.directories[i].0.clone();
        self.modal = Some(Box::new(Confirm::new("are you sure", move || {
            Event::RmDir(dir).emit()
        })));
    }
}

impl Menu for Directories {
    fn new(core: &Core) -> Self {
        Self {
            directories: Directories::dirs_with_info(&core.data),
            table_state: Rc::new(RefCell::new(TableState::default().with_selected(Some(0)))),
            modal: None,
        }
    }

    fn on_key_event(&mut self, key: crossterm::event::KeyEvent) {
        if let Some(confirm) = &mut self.modal {
            confirm.on_key_event(key);
            if confirm.should_close() {
                self.modal = None;
            }
        } else {
            match (key.modifiers, key.code) {
                (_, KeyCode::Char('j')) => self.table_state.borrow_mut().select_next(),
                (_, KeyCode::Char('k')) => self.table_state.borrow_mut().select_previous(),
                (_, KeyCode::Char('a')) => self.add_dir(),
                (_, KeyCode::Char('d')) => self.rm_dir(),
                (_, KeyCode::Char('s')) => Event::Scan.emit(),
                (_, KeyCode::Backspace) => {
                    Event::Navigate(Navigate::Back).emit();
                }
                _ => {}
            }
        }
    }
}

impl WidgetRef for Directories {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("Directories");
        let table = Table::new(
            self.directories
                .iter()
                .map(|(_, dir_info)| Row::from(dir_info)),
            [Constraint::Fill(1), Constraint::Max(10)],
        )
        .highlight_symbol("* ")
        .block(Block::bordered().title(title.left_aligned()));
        StatefulWidget::render(table, area, buf, &mut self.table_state.borrow_mut());

        if let Some(confirm) = &self.modal {
            confirm.render_ref(area, buf);
        }
    }
}
