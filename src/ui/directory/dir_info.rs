use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Cell, Row},
};

use crate::{core::data::Directory, util};

#[derive(Clone)]
pub struct DirectoryMetadata {
    file_name: String,
    base_path: String,
    exists: bool,
    size: u64,
}

#[derive(Clone)]
pub struct DirectoryWithMetadata {
    pub directory: Directory,
    pub metadata: Option<DirectoryMetadata>,
}

pub fn get_metadata(dir: &Directory) -> DirectoryMetadata {
    let exists = dir.0.try_exists().unwrap_or(false);
    DirectoryMetadata {
        file_name: dir.0.file_name().unwrap().to_string_lossy().into_owned(),
        base_path: dir.0.parent().unwrap().to_string_lossy().into_owned(),
        exists,
        size: util::dir_size(&dir.0),
    }
}

impl DirectoryWithMetadata {
    pub fn to_row(&self) -> Row<'_> {
        let mut left = Line::from(vec![
            self.metadata
                .as_ref()
                .map_or(" ", |m| m.file_name.as_str())
                .into(),
            " ".into(),
            self.metadata
                .as_ref()
                .map_or(" ", |m| m.base_path.as_str())
                .into(),
        ]);
        let right = self
            .metadata
            .as_ref()
            .map(|m| m.size)
            .map_or_else(|| "?".to_string(), util::human_size);

        if !self.metadata.as_ref().is_some_and(|m| m.exists) {
            left = left.red();
        }

        Row::new(vec![Cell::new(left), Cell::new(Line::from(right))])
    }
}
