use std::sync::{Arc, RwLock};

use chrono::Datelike;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout},
    style::Stylize,
    text::Line,
    widgets::{Clear, List, ListState, StatefulWidget, Widget, WidgetRef},
};
use tokio::{task::JoinHandle, time};

use crate::{
    api::{TMDBId, tmdb::MovieSearchResult},
    core::Core,
    events::Event,
    traits::{Modal, OkOrNotify},
    ui::{
        common::Input,
        util::{HIGHLIGHT, center_area, menu_block},
    },
};

pub struct AddMovie<F>
where
    F: AsyncFnOnce(TMDBId),
{
    core: Arc<Core>,
    input: Input,
    movies: Arc<RwLock<Vec<MovieSearchResult>>>,
    list_state: Arc<RwLock<ListState>>,
    on_confirm: Option<Box<F>>,
    fetch_handle: Option<JoinHandle<()>>,
    pub close: bool,
}

impl<F> AddMovie<F>
where
    F: AsyncFnOnce(TMDBId),
{
    pub fn new(core: Arc<Core>, on_confirm: F) -> Self {
        Self {
            core,
            input: Input::new(""),
            movies: Arc::new(RwLock::new(vec![])),
            list_state: Arc::new(RwLock::new(ListState::default().with_selected(Some(0)))),
            on_confirm: Some(Box::new(on_confirm)),
            fetch_handle: None,
            close: false,
        }
    }

    fn fetch_movies(&mut self) {
        let query = self.input.value.clone();
        // TODO: this
        let tmdb = Arc::clone(self.core.api.tmdb.as_ref().unwrap());
        let movies = Arc::clone(&self.movies);
        let list_state = Arc::clone(&self.list_state);

        if let Some(handle) = &self.fetch_handle {
            handle.abort();
        }

        self.fetch_handle = Some(tokio::spawn(async move {
            time::sleep(time::Duration::from_millis(300)).await;
            let response = tmdb.search_movie(&[("query", query)]).await;
            if let Some(results) = response.ok_or_notify() {
                *movies.write().unwrap() = results;
                list_state.write().unwrap().select(Some(0));
                Event::Render.emit();
            }
        }));
    }
}

#[async_trait::async_trait(?Send)]
impl<F> Modal for AddMovie<F>
where
    F: AsyncFnOnce(TMDBId),
{
    async fn on_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                self.list_state.write().unwrap().select_next();
            }
            (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                self.list_state.write().unwrap().select_previous();
            }
            (_, KeyCode::Char(_)) => {
                self.input.on_key_event(key);
                self.fetch_movies();
            }
            (_, KeyCode::Backspace) => {
                self.input.on_key_event(key);
                self.fetch_movies();
            }
            (_, KeyCode::Enter) => {
                let id = self
                    .list_state
                    .read()
                    .unwrap()
                    .selected()
                    .and_then(|i| self.movies.read().unwrap().get(i).map(|m| m.id));

                if let Some(id) = id
                    && let Some(on_confirm) = self.on_confirm.take()
                {
                    (on_confirm)(id).await;
                    self.close = true;
                }
            }
            (_, KeyCode::Esc) => {
                self.close = true;
            }
            _ => {}
        }
    }

    fn should_close(&self) -> bool {
        self.close
    }
}

impl<F> WidgetRef for AddMovie<F>
where
    F: AsyncFnOnce(TMDBId),
{
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let area = center_area(area, Constraint::Percentage(50), Constraint::Percentage(50));
        let [input_area, list_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(area);

        Clear.render(area, buf);

        self.input.render_ref(input_area, buf);

        let list = List::new(self.movies.read().unwrap().iter().map(|movie| {
            Line::from(vec![
                movie.title.clone().into(),
                " ".into(),
                format!("({})", movie.release_date.year()).yellow(),
            ])
        }))
        .block(menu_block("Results"))
        .highlight_style(HIGHLIGHT);

        StatefulWidget::render(list, list_area, buf, &mut self.list_state.write().unwrap());
    }
}

impl<F> Drop for AddMovie<F>
where
    F: AsyncFnOnce(TMDBId),
{
    fn drop(&mut self) {
        if let Some(handle) = self.fetch_handle.take() {
            handle.abort();
        }
    }
}
