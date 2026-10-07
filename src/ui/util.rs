use std::fmt::Display;

use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Block, Padding, Row, Table},
};

pub const HIGHLIGHT: Style = Style::new().bg(Color::DarkGray);

pub enum Menu {
    Directory,
    Movie,
    Series,
}

impl Display for Menu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Directory => f.write_str("Directories"),
            Self::Movie => f.write_str("Movies"),
            Self::Series => f.write_str("Series"),
        }
    }
}

pub fn center_area(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

/// Split `area` into `[main, details]`, giving the details pane a fixed width
/// when `show_details` is set and zero width otherwise.
pub fn detail_split(area: Rect, show_details: bool) -> [Rect; 2] {
    Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(if show_details { 50 } else { 0 }),
    ])
    .areas(area)
}

/// Indicator cell for whether an item has any local paths on disk.
pub fn presence_dot(has_paths: bool) -> &'static str {
    if has_paths { "●" } else { " " }
}

pub fn menu_block(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(ratatui::widgets::BorderType::Rounded)
        .padding(Padding::horizontal(1))
        .title(title.bold().yellow())
}

pub fn menu_table<'a, R, C>(title: &'a str, rows: R, widths: C) -> Table<'a>
where
    R: IntoIterator,
    R::Item: Into<Row<'a>>,
    C: IntoIterator,
    C::Item: Into<Constraint>,
{
    Table::new(rows, widths)
        .block(menu_block(title))
        .row_highlight_style(HIGHLIGHT)
}
