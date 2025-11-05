use std::{
    borrow::Cow,
    ffi::OsStr,
    fs::File,
    hash::Hash,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use color_eyre::Result;
use color_eyre::eyre::OptionExt;

use crate::util::file::{clean_filename, file_hash};

#[derive(Debug, Clone, Eq)]
pub struct FileData {
    pub path: PathBuf,
    pub filename: String,
    pub extension: String,
    pub size: u64,
    pub hash: String,
}

impl FileData {
    pub fn new(path: &Path) -> Result<Self> {
        let extension = path
            .extension()
            .map(OsStr::to_string_lossy)
            .ok_or_eyre("no file extension")?;
        let filename = path
            .file_name()
            .map(|filename| {
                OsStr::to_string_lossy(filename)[..filename.len() - extension.len() - 1].to_string()
            })
            .ok_or_eyre("non-canonical path")?;
        Ok(Self {
            path: PathBuf::from(path),
            filename,
            extension: extension.into_owned(),
            size: path.metadata()?.size(),
            hash: file_hash(File::open(path)?)?,
        })
    }

    pub fn clean_filename(&self) -> Cow<'_, str> {
        clean_filename(&self.filename)
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed(&self.filename))
    }
}

impl Hash for FileData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

impl PartialEq for FileData {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
