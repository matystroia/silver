use ratatui::{layout::Flex, prelude::*};
use strum::Display;

#[derive(Display)]
pub enum Menu {
    Movies,
    Directories,
}

pub fn center_area(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
