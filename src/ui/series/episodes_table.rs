use std::cell::RefCell;

use ratatui::{
    layout::Constraint,
    prelude::*,
    style::Stylize,
    widgets::{Row, StatefulWidget, WidgetRef},
};

use crate::{
    model::{LocalEpisode, Season},
    ui::{
        series::single_series::MenuState,
        util::{menu_table, presence_dot},
    },
};

pub struct EpisodesTable<'a> {
    state: &'a RefCell<MenuState>,
    season: &'a Season<LocalEpisode>,
}

impl<'a> EpisodesTable<'a> {
    pub fn new(state: &'a RefCell<MenuState>, season: &'a Season<LocalEpisode>) -> Self {
        Self { state, season }
    }

    fn rows(&self) -> Vec<Row<'_>> {
        self.season
            .episodes
            .iter()
            .map(|ep| {
                Row::new(vec![
                    format!("{:02}", ep.episode_number).yellow(),
                    ep.title.to_string().into(),
                    presence_dot(!ep.paths.is_empty()).into(),
                ])
            })
            .collect()
    }
}

impl WidgetRef for EpisodesTable<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let table = menu_table(
            "Episodes",
            self.rows(),
            [
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Length(2),
            ],
        );

        StatefulWidget::render(table, area, buf, &mut self.state.borrow_mut().episodes);
    }
}
