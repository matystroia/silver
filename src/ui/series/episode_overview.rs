use std::{path::PathBuf, sync::Arc};

use ratatui::{
    layout::{Constraint, Layout},
    style::Stylize,
    widgets::WidgetRef,
};

use crate::{
    model::LocalEpisode,
    ui::common::{ImageCache, ParagraphStack},
};

pub struct EpisodeOverview<'a> {
    episode: &'a LocalEpisode,
    thumb_cache: Arc<ImageCache<PathBuf>>,
}

impl<'a> EpisodeOverview<'a> {
    pub fn new(episode: &'a LocalEpisode, thumb_cache: Arc<ImageCache<PathBuf>>) -> Self {
        Self {
            episode,
            thumb_cache,
        }
    }
}

impl WidgetRef for EpisodeOverview<'_> {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let thumb_height = area.width as f64 * (9.0 / 16.0) / 2.0;
        let [thumb_area, overview_area] = Layout::vertical([
            Constraint::Length(thumb_height.floor() as u16),
            Constraint::Fill(1),
        ])
        .areas(area);

        let release_date = self
            .episode
            .release_date
            .format("%d/%m/%Y")
            .to_string()
            .yellow();
        let overview = self.episode.overview.clone().into();

        // TODO: DRY
        let files = if self.episode.paths.is_empty() {
            "No local files".into()
        } else {
            format!(
                "{} local file{}",
                self.episode.paths.len(),
                if self.episode.paths.len() == 1 {
                    ""
                } else {
                    "s"
                }
            )
        }
        .yellow();

        ParagraphStack::new(vec![release_date, overview, files]).render_ref(overview_area, buf);

        if let Some(file) = self.episode.paths.first()
            && let Some(image) = self.thumb_cache.get(file)
        {
            image.render_ref(thumb_area, buf);
        }
    }
}
