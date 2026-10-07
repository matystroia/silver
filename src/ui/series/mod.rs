use std::{
    cell::RefCell,
    sync::{Arc, RwLock},
};

use crossterm::event::KeyCode;
use ratatui::{
    layout::Constraint,
    style::Stylize,
    text::Line,
    widgets::{Row, StatefulWidget, TableState, WidgetRef},
};
use series_details::SeriesDetails;

use crate::{
    api::TMDBId,
    core::{Core, data::Data},
    events::{Event, Navigate},
    model::{LocalEpisode, Series},
    traits::{Menu, Modal},
    ui::{
        common::{ImageCache, InputModal, Keymap, Listable, View},
        util::{detail_split, menu_block, menu_table},
    },
    util,
};

mod episode_overview;
mod episodes_table;
mod series_details;
pub mod single_series;

impl Listable for Series<LocalEpisode> {
    fn title(&self) -> &str { &self.name }

    fn year(&self) -> Option<i32> { Series::year(self) }

    fn row(&self) -> Row<'static> {
        Row::new([Line::from(vec![
            Series::year(self)
                .map_or("?".into(), |y| y.to_string())
                .yellow(),
            " ".into(),
            self.name.clone().into(),
        ])])
    }
}

pub struct SeriesList {
    view: Arc<RwLock<View<Series<LocalEpisode>>>>,
    table_state: RefCell<TableState>,
    details: Option<SeriesDetails>,
    poster_cache: Arc<ImageCache<TMDBId>>,
    modal: Option<Box<dyn Modal>>,
    #[allow(unused)]
    watch_handle: util::AbortOnDrop,
}

impl SeriesList {
    pub async fn new(core: Arc<Core>) -> Self {
        let series = core.data().series().await.unwrap();

        let poster_cache = Arc::new(ImageCache::new(Data::get_poster));

        let view = View::new(series);

        let (mut table_state, mut details) = (TableState::default(), None);
        if let Some(first) = view.get(0) {
            table_state = table_state.with_selected(Some(0));
            details = Some(SeriesDetails::new(first, Arc::clone(&poster_cache)));
        }

        let view = Arc::new(RwLock::new(view));

        let mut series_rx = core.data().series_tx.subscribe();
        let view_ref = Arc::clone(&view);
        let watch_handle = tokio::spawn(async move {
            while series_rx.changed().await.is_ok() {
                let new = series_rx.borrow_and_update().iter().cloned().collect();
                view_ref.write().unwrap().set_items(new);
            }
        });

        Self {
            view,
            table_state: RefCell::new(table_state),
            details,
            poster_cache,
            modal: None,
            watch_handle: util::AbortOnDrop(watch_handle),
        }
    }

    fn selected_series(&self) -> Option<Arc<Series<LocalEpisode>>> {
        let i = self.table_state.borrow().selected()?;
        self.view.read().unwrap().get(i)
    }

    fn filter_series(&mut self) {
        let filter = self.view.read().unwrap().filter().to_string();
        let view = Arc::clone(&self.view);
        self.modal = Some(Box::new(InputModal::new(filter, async move |filter| {
            view.write().unwrap().set_filter(filter);
        })))
    }

    fn cycle_sort(&self) { self.view.write().unwrap().cycle_sort(); }

    fn update_details(&mut self) {
        if let Some(series) = self.selected_series() {
            self.details = Some(SeriesDetails::new(series, Arc::clone(&self.poster_cache)))
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Menu for SeriesList {
    async fn on_key_event(&mut self, key: crossterm::event::KeyEvent) {
        if Self::dispatch_modal(key, &mut self.modal).await {
            return;
        }
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.table_state.borrow_mut().select_next();
                self.update_details();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.table_state.borrow_mut().select_previous();
                self.update_details();
            }
            KeyCode::Char('m') => self.cycle_sort(),
            KeyCode::Char('/') => self.filter_series(),
            KeyCode::Enter => {
                if let Some(series) = self.selected_series() {
                    Event::Navigate(Navigate::Series(series.id)).emit()
                }
            }
            KeyCode::Esc => Event::Navigate(Navigate::Back).emit(),
            _ => {}
        }
    }

    fn keymaps(&self) -> Vec<Keymap> {
        vec![
            Keymap::new(KeyCode::Enter, "enter"),
            Keymap::new(KeyCode::Char('m'), "sort"),
            Keymap::new(KeyCode::Char('/'), "filter"),
        ]
    }

    fn has_modal(&self) -> bool { self.modal.is_some() }
}

impl WidgetRef for SeriesList {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let [table_area, details_area] = detail_split(area, self.details.is_some());

        let filter = Line::from(self.view.read().unwrap().filter_display());
        let sort = Line::from(self.view.read().unwrap().sort_display());

        let table = menu_table(
            "Series",
            self.view.read().unwrap().rows(),
            [Constraint::Fill(1)],
        )
        .block(
            menu_block("Series")
                .title(filter.right_aligned())
                .title(sort.right_aligned()),
        );
        table.render(table_area, buf, &mut self.table_state.borrow_mut());

        if let Some(details) = &self.details {
            details.render_ref(details_area, buf);
        }

        if let Some(modal) = &self.modal {
            modal.render_ref(area, buf);
        }
    }
}
