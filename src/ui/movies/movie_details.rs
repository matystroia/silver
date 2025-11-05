use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::Stylize,
    text::{Line, Text},
    widgets::{Block, Clear, Paragraph, Widget, WidgetRef, Wrap},
};

use crate::{core::data::library::LocalMovie, model::movie::Movie, ui::util::center_area};

pub struct MovieDetails {
    movie: LocalMovie,
}

impl From<&LocalMovie> for MovieDetails {
    fn from(movie: &LocalMovie) -> Self {
        Self {
            movie: movie.clone(),
        }
    }
}

impl WidgetRef for MovieDetails {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let (movie, paths) = &self.movie;

        let title = Paragraph::new(Line::from(vec![
            movie.title.clone().red().bold(),
            " ".into(),
            movie.year.map_or("?".into(), |x| x.to_string().gray()),
        ]))
        .wrap(Wrap { trim: false });

        let tagline = Paragraph::new(movie.tagline.clone().italic()).wrap(Wrap { trim: false });
        let overview = Paragraph::new(movie.overview.clone()).wrap(Wrap { trim: false });

        // let values = [
        //     ("Title", movie.title.clone()),
        //     ("Year", movie.year.map_or("?".into(), |x| x.to_string())),
        //     ("TMDb", movie.tmdb_id.to_string()),
        //     ("Tagline", movie.tagline.clone()),
        //     ("Overview", movie.overview.clone()),
        //     ("Genres", movie.genres.join(", ")),
        //     ("Files", format!("{paths:?}")),
        // ];

        let width = area.width / 2;
        let inner_width = width - 2;

        let title_height = title.line_count(inner_width) as u16 + 1;
        let tagline_height = tagline.line_count(inner_width) as u16 + 1;
        let overview_height = overview.line_count(inner_width) as u16;

        let inner_height = title_height + tagline_height + overview_height;
        let height = inner_height + 2;

        let area = center_area(area, Constraint::Length(width), Constraint::Length(height));

        let [title_area, tagline_area, overview_area] = Layout::new(
            Direction::Vertical,
            vec![
                Constraint::Length(title_height),
                Constraint::Length(tagline_height),
                Constraint::Length(overview_height),
            ],
        )
        .areas(area.inner(Margin::new(1, 1)));

        Clear.render(area, buf);
        Block::bordered().render(area, buf);

        title.render_ref(title_area, buf);
        tagline.render_ref(tagline_area, buf);
        overview.render_ref(overview_area, buf);

        // for ((label, value), &area) in values.into_iter().zip(areas.iter()) {
        //     let [label_area, value_area] = Layout::new(
        //         Direction::Horizontal,
        //         vec![Constraint::Length(5), Constraint::Fill(1)],
        //     )
        //     .areas(area);
        //     let (label, area) = (
        //         Line::from(label),
        //         Line::from(value).alignment(Alignment::Right),
        //     );
        //
        //     label.render_ref(label_area, buf);
        //     area.render_ref(value_area, buf);
        // }
    }
}
