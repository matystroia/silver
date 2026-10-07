use std::{
    cell::RefCell,
    collections::HashMap,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, RwLock},
};

use crossterm::event::KeyCode;
use ratatui::{
    prelude::*,
    widgets::{TableState, WidgetRef},
};
use tokio::task;

use crate::{
    core::{Core, CoreTasks, data::Directory},
    events::{Event, Navigate},
    traits::{Menu, Modal, OkOrNotify},
    ui::{
        common::{Confirm, Keymap},
        directory::{
            add_directory::AddDirectory,
            dir_info::{DirectoryMetadata, DirectoryWithMetadata, get_metadata},
        },
        util::menu_table,
    },
    util,
};

mod add_directory;
mod dir_info;

pub struct DirectoriesMenu {
    core: Arc<Core>,
    directories: Arc<RwLock<Vec<Directory>>>,
    metadata: Arc<RwLock<HashMap<Directory, DirectoryMetadata>>>,
    table_state: Rc<RefCell<TableState>>,
    modal: Option<Box<dyn Modal>>,
    #[allow(unused)]
    watch_handle: util::AbortOnDrop,
}

impl DirectoriesMenu {
    pub async fn new(core: Arc<Core>) -> Self {
        let directories = core.data().directories().await.unwrap();
        let dirs = Arc::new(RwLock::new(directories));
        let metadata = Arc::new(RwLock::new(HashMap::new()));

        let dirs_ref = Arc::clone(&dirs);
        let metadata_ref = Arc::clone(&metadata);

        Self::fetch_metadata(Arc::clone(&dirs), Arc::clone(&metadata));

        let mut dirs_rx = core.data().directories_tx.subscribe();
        let watch_handle = tokio::spawn(async move {
            while dirs_rx.changed().await.is_ok() {
                *dirs_ref.write().unwrap() = dirs_rx.borrow_and_update().iter().cloned().collect();
                Self::fetch_metadata(Arc::clone(&dirs_ref), Arc::clone(&metadata_ref));
                Event::Render.emit();
            }
        });

        Self {
            core,
            directories: dirs,
            metadata,
            table_state: Rc::new(RefCell::new(TableState::default().with_selected(Some(0)))),
            modal: None,
            watch_handle: util::AbortOnDrop(watch_handle),
        }
    }

    fn add_dir(&mut self) {
        let core = Arc::clone(&self.core);
        self.modal = Some(Box::new(AddDirectory::new("", async move |dir| {
            core.data()
                .add_directory(PathBuf::from(dir))
                .await
                .ok_or_notify();
        })));
    }

    fn rm_dir(&mut self) {
        let Some(i) = self.table_state.borrow().selected() else {
            return;
        };

        if let Some(path) = self.directories.read().unwrap().get(i).map(|d| d.0.clone()) {
            let core = Arc::clone(&self.core);
            self.modal = Some(Box::new(Confirm::new(
                "Remove directory?",
                async move || {
                    core.data().remove_directory(path).await.ok_or_notify();
                },
            )));
        }
    }

    fn fetch_metadata(
        directories: Arc<RwLock<Vec<Directory>>>,
        metadata: Arc<RwLock<HashMap<Directory, DirectoryMetadata>>>,
    ) {
        // TODO: This should ideally compute difference
        task::spawn_blocking(move || {
            let dirs = directories.read().unwrap().clone();
            for dir in dirs {
                metadata
                    .write()
                    .unwrap()
                    .insert(dir.clone(), get_metadata(&dir));
            }
        });
    }
}

#[async_trait::async_trait(?Send)]
impl Menu for DirectoriesMenu {
    async fn on_key_event(&mut self, key: crossterm::event::KeyEvent) {
        if !Self::dispatch_modal(key, &mut self.modal).await {
            match (key.modifiers, key.code) {
                (_, KeyCode::Char('j') | KeyCode::Down) => {
                    self.table_state.borrow_mut().select_next()
                }
                (_, KeyCode::Char('k') | KeyCode::Up) => {
                    self.table_state.borrow_mut().select_previous()
                }
                (_, KeyCode::Char('a')) => self.add_dir(),
                (_, KeyCode::Char('d')) => self.rm_dir(),
                (_, KeyCode::Char('h')) => {
                    self.core.hash_files();
                }
                (_, KeyCode::Char('s')) => {
                    self.core.scan_directories();
                }
                (_, KeyCode::Esc) => {
                    Event::Navigate(Navigate::Back).emit();
                }
                _ => {}
            }
        }
    }

    fn keymaps(&self) -> Vec<Keymap> {
        vec![
            Keymap::new(KeyCode::Char('a'), "add"),
            Keymap::new(KeyCode::Char('d'), "delete"),
            Keymap::new(KeyCode::Char('s'), "scan"),
        ]
    }

    fn has_modal(&self) -> bool { self.modal.is_some() }
}

impl WidgetRef for DirectoriesMenu {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let dirs_guard = self.directories.read().unwrap();
        let metadata_guard = self.metadata.read().unwrap();
        let dirs_with_metadata: Vec<_> = dirs_guard
            .iter()
            .map(|dir| DirectoryWithMetadata {
                directory: dir,
                metadata: metadata_guard.get(dir),
            })
            .collect();

        let table = menu_table(
            "Directories",
            dirs_with_metadata.iter().map(DirectoryWithMetadata::to_row),
            [Constraint::Fill(1), Constraint::Max(10)],
        );
        StatefulWidget::render(table, area, buf, &mut self.table_state.borrow_mut());

        if let Some(modal) = &self.modal {
            modal.render_ref(area, buf);
        }
    }
}
