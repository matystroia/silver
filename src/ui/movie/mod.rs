use std::{
    cell::RefCell,
    sync::{Arc, RwLock},
};

use chrono::Datelike;
use crossterm::event::KeyCode;
use ratatui::{
    prelude::*,
    widgets::{Cell, Row, TableState, WidgetRef},
};

use crate::{
    api::TMDBId,
    core::{Core, Output, data::Data, notify::Message},
    events::{Event, Navigate},
    model::{LocalMovie, Playable},
    traits::{Menu, Modal, OkOrNotify},
    ui::{
        common::{ImageCache, InputModal, Keymap, Listable, Select, View},
        movie::{add_movie::AddMovie, movie_details::MovieDetails},
        util::{detail_split, menu_block, menu_table, presence_dot},
    },
    util,
};

mod add_movie;
mod movie_details;

impl Listable for LocalMovie {
    fn title(&self) -> &str { &self.title }

    fn year(&self) -> Option<i32> { Some(self.release_date.year()) }

    fn row(&self) -> Row<'static> {
        Row::new([
            Cell::from(self.release_date.year().yellow()),
            Cell::from(self.title.clone()),
            Cell::from(presence_dot(!self.paths.is_empty())),
        ])
    }
}

pub struct MoviesMenu {
    core: Arc<Core>,
    view: Arc<RwLock<View<LocalMovie>>>,
    table_state: RefCell<TableState>,
    details: Option<MovieDetails>,
    poster_cache: Arc<ImageCache<TMDBId>>,
    #[allow(unused, reason = "struct implements drop")]
    watch_handle: util::AbortOnDrop,
    modal: Option<Box<dyn Modal>>,
}

impl MoviesMenu {
    pub async fn new(core: Arc<Core>) -> Self {
        let movies = core.data().movies().await.unwrap();

        let poster_cache = Arc::new(ImageCache::new(Data::get_poster));

        let view = View::new(movies);

        let (mut table_state, mut details) = (TableState::default(), None);
        if let Some(first) = view.get(0) {
            table_state = table_state.with_selected(Some(0));
            details = Some(MovieDetails {
                movie: first,
                poster_cache: Arc::clone(&poster_cache),
            })
        }

        let view = Arc::new(RwLock::new(view));

        let mut movies_rx = core.data().movies_tx.subscribe();
        let view_ref = Arc::clone(&view);
        let watch_handle = tokio::spawn(async move {
            while movies_rx.changed().await.is_ok() {
                let new = movies_rx.borrow_and_update().clone();
                view_ref.write().unwrap().set_items(new);
            }
        });

        Self {
            core,
            view,
            watch_handle: util::AbortOnDrop(watch_handle),
            table_state: RefCell::new(table_state),
            details,
            poster_cache,
            modal: None,
        }
    }

    fn add_movie(&mut self) {
        let Some(tmdb) = self.core.api.tmdb.clone() else {
            Output::notify(Message::error("No TMDb API"));
            return;
        };
        let core = Arc::clone(&self.core);
        self.modal = Some(Box::new(AddMovie::new(
            Arc::clone(&self.core),
            async move |id| {
                if let Some(movie) = tmdb.movie_details(id).await.ok_or_notify() {
                    core.data().add_movie(Arc::new(movie)).await.ok_or_notify();
                }
            },
        )))
    }

    fn filter_movies(&mut self) {
        let filter = self.view.read().unwrap().filter().to_string();
        let view = Arc::clone(&self.view);
        self.modal = Some(Box::new(InputModal::new(filter, async move |filter| {
            view.write().unwrap().set_filter(filter);
        })))
    }

    fn selected_movie(&self) -> Option<Arc<LocalMovie>> {
        let i = self.table_state.borrow().selected()?;
        self.view.read().unwrap().get(i)
    }

    fn cycle_sort(&self) {
        let mut view = self.view.write().unwrap();
        view.cycle_sort();
    }
}

#[async_trait::async_trait(?Send)]
impl Menu for MoviesMenu {
    async fn on_key_event(&mut self, key: crossterm::event::KeyEvent) {
        if !Self::dispatch_modal(key, &mut self.modal).await {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.table_state.borrow_mut().select_next();
                    self.update_details().await;
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.table_state.borrow_mut().select_previous();
                    self.update_details().await;
                }
                KeyCode::Enter => {
                    if let Some(movie) = self.selected_movie() {
                        match movie.paths.as_slice() {
                            [] => {}
                            [path] => {
                                path.play().ok_or_notify();
                            }
                            paths => {
                                self.modal = Some(Box::new(Select::new(
                                    paths.to_vec(),
                                    |p| p.to_string_lossy().into_owned(),
                                    async |path| {
                                        path.play().ok_or_notify();
                                    },
                                )));
                            }
                        }
                    }
                }
                KeyCode::Char('a') => self.add_movie(),
                KeyCode::Char('m') => self.cycle_sort(),
                KeyCode::Char('/') => self.filter_movies(),
                KeyCode::Esc => Event::Navigate(Navigate::Back).emit(),
                _ => {}
            }
        }
    }

    fn keymaps(&self) -> Vec<Keymap> {
        vec![
            Keymap::new(KeyCode::Enter, "play"),
            Keymap::new(KeyCode::Char('a'), "add"),
            Keymap::new(KeyCode::Char('m'), "sort"),
            Keymap::new(KeyCode::Char('/'), "filter"),
        ]
    }

    fn has_modal(&self) -> bool { self.modal.is_some() }
}

impl WidgetRef for MoviesMenu {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [table_area, details_area] = detail_split(area, self.details.is_some());

        let filter = Line::from(self.view.read().unwrap().filter_display());
        let sort = Line::from(self.view.read().unwrap().sort_display());

        let table = menu_table("Movies", self.view.read().unwrap().rows(), Self::columns()).block(
            menu_block("Movies")
                .title(filter.right_aligned())
                .title(sort.right_aligned()),
        );
        StatefulWidget::render(table, table_area, buf, &mut self.table_state.borrow_mut());

        if let Some(details) = &self.details {
            details.render_ref(details_area, buf);
        }

        if let Some(modal) = &self.modal {
            modal.render_ref(area, buf);
        }
    }
}

impl MoviesMenu {
    fn columns() -> impl IntoIterator<Item = impl Into<Constraint>> {
        [
            Constraint::Length(4),
            Constraint::Fill(1),
            Constraint::Length(2),
        ]
    }

    async fn update_details(&mut self) {
        if let Some(movie) = self.selected_movie() {
            self.details = Some(MovieDetails {
                movie: Arc::clone(&movie),
                poster_cache: Arc::clone(&self.poster_cache),
            });
        }
    }
}
