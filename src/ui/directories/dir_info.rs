use ratatui::{
    style::Stylize,
    text::{Line, Span},
    widgets::{Cell, Row},
};

use crate::{
    core::data::directory::Directory,
    util::directory::{dir_size, human_size},
};

pub struct DirInfo {
    file_name: String,
    base_path: String,
    size: Option<u64>,
}

impl From<&Directory> for DirInfo {
    fn from(value: &Directory) -> Self {
        Self {
            file_name: value.0.file_name().unwrap().to_string_lossy().into_owned(),
            base_path: value.0.parent().unwrap().to_string_lossy().into_owned(),
            size: dir_size(&value.0).ok(),
        }
    }
}

impl From<&DirInfo> for Row<'_> {
    fn from(value: &DirInfo) -> Self {
        Self::new(vec![
            Cell::new(Line::from(vec![
                value.file_name.clone().into(),
                " ".into(),
                value.base_path.clone().dark_gray(),
            ])),
            Cell::new(
                Span::from(value.size.map(human_size).unwrap_or("?".into()))
                    .into_right_aligned_line(),
            ),
        ])
    }
}
