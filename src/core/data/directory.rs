use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub struct Directory(pub PathBuf);

impl From<&Directory> for PathBuf {
    fn from(value: &Directory) -> Self {
        value.0.clone()
    }
}

impl AsRef<Path> for Directory {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
