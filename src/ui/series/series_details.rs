use std::sync::Arc;

use itertools::Itertools;
use ratatui::{prelude::*, widgets::WidgetRef};

use crate::{
    PICKER,
    api::TMDBId,
    model::{LocalEpisode, Series},
    ui::common::{ImageCache, ParagraphStack},
};

pub(super) struct SeriesDetails {
    series: Arc<Series<LocalEpisode>>,
    poster_cache: Arc<ImageCache<TMDBId>>,
}

impl SeriesDetails {
    pub const fn new(
        series: Arc<Series<LocalEpisode>>,
        poster_cache: Arc<ImageCache<TMDBId>>,
    ) -> Self {
        Self {
            series,
            poster_cache,
        }
    }
}

impl WidgetRef for SeriesDetails {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let name = self.series.name.clone().yellow().bold();
        let tagline = self.series.tagline.clone().yellow().italic();
        let genres = self
            .series
            .genres
            .iter()
            .map(|g| g.0.as_str())
            .collect_vec()
            .join(", ")
            .yellow();
        let overview = self.series.overview.clone().into();

        let font_size = PICKER.get().unwrap().font_size();
        let poster_height = (f32::from(area.width * font_size.width) * 1.5
            / f32::from(font_size.height))
        .ceil() as u16;

        let [poster_area, details_area] =
            Layout::vertical([Constraint::Length(poster_height), Constraint::Fill(1)]).areas(area);

        ParagraphStack::new(vec![name, tagline, genres, overview]).render_ref(details_area, buf);

        if let Some(poster) = self.poster_cache.get(&self.series.id) {
            poster.render_ref(poster_area, buf);
        }
    }
}
