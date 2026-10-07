use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyModifiers};
use itertools::Itertools;
use ratatui::{prelude::*, widgets::WidgetRef};

use crate::{traits::Modal, ui::common::InputModal};

pub struct AddDirectory<F> {
    complete: Option<Complete>,
    modal: InputModal<F>,
}

impl<F> AddDirectory<F>
where
    F: AsyncFnOnce(String),
{
    pub fn new(value: impl Into<String>, on_confirm: F) -> Self {
        Self {
            complete: None,
            modal: InputModal::new(value, on_confirm),
        }
    }

    fn complete(&mut self, reverse: bool) {
        if let Some(ref mut complete) = self.complete {
            self.modal.value = match reverse {
                false => complete.increment(),
                true => complete.decrement(),
            }
            .to_string_lossy()
            .into_owned();
            return;
        }

        let cur = PathBuf::from(&self.modal.value);

        let candidates = if cur.is_dir() {
            subdirs(&cur)
        } else if let Some(filename) = cur.file_name()
            && let Some(parent) = cur.parent()
        {
            subdirs(parent)
                .into_iter()
                .filter(|p| {
                    p.file_name().is_some_and(|x| {
                        x.to_string_lossy()
                            .starts_with(&*filename.to_string_lossy())
                    })
                })
                .collect()
        } else {
            vec![]
        };

        match candidates.as_slice() {
            [] => {}
            [path] => {
                self.modal.value = path.with_trailing_sep().to_string_lossy().into_owned();
            }
            multiple => {
                let complete = Complete::new(multiple.to_vec());
                self.modal.value = complete.path().to_string_lossy().into_owned();
                self.complete = Some(complete);
            }
        };
    }

    fn delete_word_back(&mut self) {
        self.complete = None;
        if self.modal.value.is_empty() {
            return;
        }
        let trimmed = self.modal.value.trim_end_matches(std::path::MAIN_SEPARATOR);
        let cut = trimmed
            .char_indices()
            .rev()
            .find(|&(_, ch)| ch == std::path::MAIN_SEPARATOR)
            .map(|(i, ch)| i + ch.len_utf8())
            .unwrap_or(0);
        self.modal.value.truncate(cut);
    }
}

#[async_trait::async_trait(?Send)]
impl<F> Modal for AddDirectory<F>
where
    F: AsyncFnOnce(String),
{
    async fn on_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(_)) => {
                self.modal.on_key_event(key).await;
                self.complete = None;
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                self.modal.on_key_event(key).await;
                self.complete = None;
            }
            (KeyModifiers::CONTROL, KeyCode::Char('w') | KeyCode::Backspace) => {
                self.delete_word_back();
            }
            (KeyModifiers::NONE, KeyCode::Tab) => {
                self.complete(false);
            }
            (KeyModifiers::SHIFT, KeyCode::Tab) => {
                self.complete(true);
            }
            _ => {}
        }
    }

    fn should_close(&self) -> bool { self.modal.should_close() }
}

impl<F> WidgetRef for AddDirectory<F> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) { self.modal.render_ref(area, buf); }
}

fn subdirs(path: &Path) -> Vec<PathBuf> {
    let Ok(children) = std::fs::read_dir(path) else {
        return vec![];
    };
    children
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| e.path())
        .sorted()
        .collect()
}

struct Complete {
    index: usize,
    options: Vec<PathBuf>,
}

impl Complete {
    fn new(options: Vec<PathBuf>) -> Self { Self { index: 0, options } }

    fn increment(&mut self) -> PathBuf {
        self.index = (self.index + 1) % self.options.len();
        self.path()
    }

    fn decrement(&mut self) -> PathBuf {
        self.index = (self.index - 1).rem_euclid(self.options.len());
        self.path()
    }

    fn path(&self) -> PathBuf { self.options[self.index].clone() }
}
