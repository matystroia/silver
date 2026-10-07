use std::sync::Arc;

use itertools::Itertools;
use ratatui::{
    layout::{Constraint, Layout},
    prelude::*,
    style::Stylize,
    widgets::WidgetRef,
};

use crate::{
    PICKER,
    api::TMDBId,
    model::LocalMovie,
    ui::common::{ImageCache, ParagraphStack},
};

pub struct MovieDetails {
    pub movie: Arc<LocalMovie>,
    pub poster_cache: Arc<ImageCache<TMDBId>>,
}

impl WidgetRef for MovieDetails {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let title = self.movie.title.clone().yellow().bold();
        let tagline = self.movie.tagline.clone().yellow().italic();
        let genres = self
            .movie
            .genres
            .iter()
            .map(|g| g.0.as_str())
            .collect_vec()
            .join(", ")
            .yellow();
        let overview = self.movie.overview.clone().into();

        let files = if self.movie.paths.is_empty() {
            "No local files".into()
        } else {
            format!(
                "{} local file{}",
                self.movie.paths.len(),
                if self.movie.paths.len() == 1 { "" } else { "s" }
            )
        }
        .yellow();

        let font_size = PICKER.get().unwrap().font_size();
        let poster_height = (f32::from(area.width * font_size.width) * 1.5
            / f32::from(font_size.height))
        .ceil() as u16;

        let [poster_area, details_area] =
            Layout::vertical([Constraint::Length(poster_height), Constraint::Fill(1)]).areas(area);

        ParagraphStack::new(vec![title, tagline, genres, overview, files])
            .render_ref(details_area, buf);

        if let Some(poster) = self.poster_cache.get(&self.movie.id) {
            poster.render_ref(poster_area, buf);
        }
    }
}
