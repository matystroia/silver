use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Cell, Row},
};

use crate::{core::data::Directory, util};

pub struct DirectoryMetadata {
    file_name: String,
    base_path: String,
    exists: bool,
    size: Option<u64>,
}

pub struct DirectoryWithMetadata<'a> {
    pub directory: &'a Directory,
    pub metadata: Option<&'a DirectoryMetadata>,
}

pub fn get_metadata(dir: &Directory) -> DirectoryMetadata {
    let exists = dir.0.try_exists().unwrap_or(false);
    DirectoryMetadata {
        file_name: dir.0.file_name().unwrap().to_string_lossy().into_owned(),
        base_path: dir.0.parent().unwrap().to_string_lossy().into_owned(),
        exists,
        size: util::dir_size(&dir.0).ok(),
    }
}

impl<'a> DirectoryWithMetadata<'a> {
    pub fn to_row(&self) -> Row<'a> {
        let mut left = Line::from(vec![
            self.metadata
                .map(|m| m.file_name.as_str())
                .unwrap_or(" ")
                .into(),
            " ".into(),
            self.metadata
                .map(|m| m.base_path.as_str())
                .unwrap_or(" ")
                .into(),
        ]);
        let right = self
            .metadata
            .and_then(|m| m.size)
            .map(util::human_size)
            .unwrap_or("?".to_string());

        if !self.metadata.map(|m| m.exists).unwrap_or(false) {
            left = left.red();
        }

        Row::new(vec![Cell::new(left), Cell::new(Line::from(right))])
    }
}
