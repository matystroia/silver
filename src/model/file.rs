use std::{
    fs::File,
    hash::Hash,
    path::{Path, PathBuf},
};

use color_eyre::Result;

use crate::util;

#[derive(Debug, Clone, Eq)]
pub struct FileData {
    pub path: PathBuf,
    pub hash: String,
    pub scanned: bool,
}

impl FileData {
    pub fn new(path: &Path) -> Result<Self> {
        let hash = util::file_hash(File::open(path)?).unwrap_or_default();

        Ok(Self {
            path: path.to_path_buf(),
            hash,
            scanned: false,
        })
    }

    pub fn filename(&self) -> Option<String> {
        self.path
            .file_stem()
            .map(|x| x.to_string_lossy().into_owned())
    }
}

pub trait Playable {
    fn play(&self) -> Result<()>;
}

impl Playable for PathBuf {
    fn play(&self) -> Result<()> { Ok(open::that_detached(self)?) }
}

impl Hash for FileData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.path.hash(state); }
}

impl PartialEq for FileData {
    fn eq(&self, other: &Self) -> bool { self.path == other.path }
}
