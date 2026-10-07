use std::{cell::RefCell, path::PathBuf, sync::Arc};

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    style::Stylize,
    widgets::{Row, StatefulWidget, TableState, WidgetRef},
};

use crate::{
    api::TMDBId,
    core::{Core, data::Data},
    events::{Event, Navigate},
    model::{LocalEpisode, Playable, Season},
    traits::{Menu, OkOrNotify},
    ui::{
        common::{ImageCache, Keymap},
        series::{episode_overview::EpisodeOverview, episodes_table::EpisodesTable},
        util::{detail_split, menu_table},
    },
};

pub struct MenuState {
    pub seasons: TableState,
    pub episodes: TableState,
}

pub struct SeriesMenu {
    seasons: Vec<Season<LocalEpisode>>,
    state: RefCell<MenuState>,
    thumb_cache: Arc<ImageCache<PathBuf>>,
}

impl SeriesMenu {
    pub async fn new(core: Arc<Core>, id: TMDBId) -> Self {
        let series = core.data().get_series(id).await.unwrap();
        let thumb_cache = Arc::new(ImageCache::<PathBuf>::new(|path| Data::get_thumb(path)));

        let seasons = series.regular_seasons();
        let selected = (!seasons.is_empty()).then_some(0);

        Self {
            seasons,
            state: RefCell::new(MenuState {
                seasons: TableState::default().with_selected_column(selected),
                episodes: TableState::default().with_selected(selected),
            }),
            thumb_cache,
        }
    }
}

impl SeriesMenu {
    fn selected_season(&self) -> Option<&Season<LocalEpisode>> {
        let i = self.state.borrow().seasons.selected_column()?;
        self.seasons.get(i)
    }

    fn selected_episode(&self) -> Option<&LocalEpisode> {
        let season = self.selected_season()?;
        let i = self.state.borrow().episodes.selected()?;
        season.episodes.get(i)
    }

    fn select_season_next(&self) {
        let last = match self.seasons.len() {
            0 => return,
            n => n - 1,
        };
        let mut guard = self.state.borrow_mut();
        let next = guard
            .seasons
            .selected_column()
            .map_or(0, |i| (i + 1).min(last));
        guard.seasons.select_column(Some(next));
    }

    fn select_episode_next(&self) {
        let Some(last) = self
            .selected_season()
            .filter(|s| !s.episodes.is_empty())
            .map(|s| s.episodes.len() - 1)
        else {
            return;
        };
        let mut guard = self.state.borrow_mut();
        let next = guard.episodes.selected().map_or(0, |i| (i + 1).min(last));
        guard.episodes.select(Some(next));
    }
}

#[async_trait::async_trait(?Send)]
impl Menu for SeriesMenu {
    async fn on_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('h') | KeyCode::Left => {
                let mut guard = self.state.borrow_mut();
                guard.seasons.select_previous_column();
            }
            KeyCode::Char('l') | KeyCode::Right => self.select_season_next(),
            KeyCode::Char('j') | KeyCode::Down => self.select_episode_next(),
            KeyCode::Char('k') | KeyCode::Up => self.state.borrow_mut().episodes.select_previous(),
            KeyCode::Esc => Event::Navigate(Navigate::Back).emit(),
            KeyCode::Enter => {
                if let Some(episode) = self.selected_episode() {
                    match episode.paths.as_slice() {
                        [] => {}
                        [path] => {
                            path.play().ok_or_notify();
                        }
                        // TODO: This
                        _ => {}
                    }
                }
            }
            _ => return,
        }
    }

    fn keymaps(&self) -> Vec<Keymap> { vec![] }

    fn has_modal(&self) -> bool { false }
}

impl WidgetRef for SeriesMenu {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let [left_area, right_area] = detail_split(area, self.selected_episode().is_some());

        let [seasons_area, episodes_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(left_area);

        let sel = self.state.borrow().seasons.selected_column().unwrap_or(0);

        let visible = ((seasons_area.width.saturating_sub(2)) as usize / 3).max(1);

        let start = sel
            .saturating_sub(visible.saturating_sub(1))
            .min(self.seasons.len().saturating_sub(visible));
        let end = (start + visible).min(self.seasons.len());

        let seasons = menu_table(
            "Seasons",
            [Row::new(
                self.seasons[start..end]
                    .iter()
                    .map(|season| format!("{:02}", season.number).yellow()),
            )],
            vec![Constraint::Length(2); end - start],
        );

        let mut local_state = self.state.borrow_mut().seasons;
        local_state.select_column(Some(sel - start));
        StatefulWidget::render(seasons, seasons_area, buf, &mut local_state);

        if let Some(season) = self.selected_season() {
            EpisodesTable::new(&self.state, season).render_ref(episodes_area, buf)
        }

        if let Some(episode) = self.selected_episode() {
            EpisodeOverview::new(episode, Arc::clone(&self.thumb_cache))
                .render_ref(right_area, buf);
        }
    }
}
