use std::cell::RefCell;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Cell, Row, Table, TableState, WidgetRef};

use crate::core::Core;
use crate::core::data::Data;
use crate::core::data::library::LocalMovie;
use crate::core::events::event::Event;
use crate::core::events::navigate::Navigate;
use crate::model::movie::Movie;
use crate::traits::menu::Menu;
use crate::ui::movies::movie_details::MovieDetails;

mod movie_details;

pub struct Movies {
    movies: Arc<RwLock<Vec<LocalMovie>>>,
    table_state: Rc<RefCell<TableState>>,
    details: Option<MovieDetails>,
    watch_handle: tokio::task::JoinHandle<()>,
}

impl Menu for Movies {
    fn new(core: &Core) -> Self {
        let mut movies_rx = core.data.read().unwrap().movies_tx.subscribe();

        // FIXME: This might outlive the menu (test it)
        let movies = Arc::new(RwLock::new(
            core.data.read().unwrap().library.movies().collect(),
        ));
        let movies_ref = movies.clone();

        let watch_handle = tokio::spawn(async move {
            while movies_rx.changed().await.is_ok() {
                let new_movies = movies_rx.borrow().clone();
                *movies_ref.write().unwrap() = new_movies;
            }
        });

        Self {
            movies,
            watch_handle,
            table_state: Rc::new(RefCell::new(TableState::default().with_selected(Some(0)))),
            details: None,
        }
    }

    fn on_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc if self.details.is_some() => self.details = None,
            KeyCode::Char('j') => self.table_state.borrow_mut().select_next(),
            KeyCode::Char('k') => self.table_state.borrow_mut().select_previous(),
            KeyCode::Tab => {
                if let Some(i) = self.table_state.borrow().selected() {
                    let local_movie = &self.movies.read().unwrap()[i];
                    self.details = Some(MovieDetails::from(local_movie));
                }
            }
            KeyCode::Enter => {
                if let Some(i) = self.table_state.borrow().selected() {
                    let (_, paths) = &self.movies.read().unwrap()[i];
                    Command::new("xdg-open")
                        .arg(paths.iter().next().unwrap())
                        .spawn();
                }
            }
            KeyCode::Backspace => Event::Navigate(Navigate::Back).emit(),
            _ => {}
        }
    }
}

impl WidgetRef for Movies {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("Movies");
        let table = Table::new(
            self.movies
                .read()
                .unwrap()
                .iter()
                .map(|(movie, paths)| Row::from(movie)),
            [Constraint::Fill(1), Constraint::Length(1)],
        )
        .highlight_symbol("* ")
        .block(Block::bordered().title(title.left_aligned()));
        StatefulWidget::render(table, area, buf, &mut self.table_state.borrow_mut());

        if let Some(details) = &self.details {
            details.render_ref(area, buf);
        }
    }
}

impl From<&Movie> for Row<'_> {
    fn from(movie: &Movie) -> Self {
        let mut spans = vec![movie.title.clone().into()];
        if let Some(year) = movie.year {
            spans.extend(vec![" ".into(), format!("({})", year).gray()]);
        }
        Self::new(vec![
            Cell::from(Text::from(Line::from(spans))),
            Cell::from("✓".green()),
        ])
    }
}

impl Drop for Movies {
    fn drop(&mut self) {
        self.watch_handle.abort();
    }
}
