use color_eyre::eyre::{Result, eyre};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::{LazyLock, OnceLock},
};
use tokio::sync::watch;
use walkdir::WalkDir;

use crate::{
    core::{
        config::PROJECT_DIRS,
        data::{directory::Directory, library::Library},
    },
    model::{episode::Episode, feature::Feature, file::FileData, movie::Movie, show::Show},
    util::file::is_video,
};

pub mod directory;
pub mod library;

pub struct Data {
    pub library: Library,
    // TODO: Nope, send references!
    pub movies_tx: watch::Sender<Vec<(Movie, HashSet<PathBuf>)>>,
}

impl Default for Data {
    fn default() -> Self {
        let data = Self {
            library: Library::load().unwrap_or_default(),
            movies_tx: watch::channel(vec![]).0,
        };

        data.movies_tx.send(
            data.library
                .movies()
                .map(|(movie, paths)| (movie.clone(), paths.clone()))
                .collect(),
        );

        data
    }
}

impl Data {
    pub fn add_dir(&mut self, path: PathBuf) -> Result<()> {
        if !path.try_exists()? {
            return Err(eyre!("Invalid path: {path:?}"));
        }
        self.library.directories.insert(Directory(path));
        self.library.save()?;
        Ok(())
    }

    pub fn rm_dir(&mut self, dir: &Directory) -> Result<()> {
        self.library.directories.remove(dir);
        self.library.save()?;
        Ok(())
    }

    pub fn scan(&self) -> Vec<FileData> {
        self.library
            .directories
            .iter()
            .flat_map(|dir| {
                WalkDir::new(dir)
                    .into_iter()
                    .filter_map(Result::ok)
                    .map(|entry| entry.into_path())
                    .filter(|path| path.is_file() && is_video(path))
            })
            .filter_map(|path| FileData::new(&path).ok())
            .collect()
    }

    pub fn add_feature(&mut self, file: FileData, feature: Feature) {
        self.library
            .local_features
            .entry(feature)
            .and_modify(|entry| {
                entry.insert(file.path.clone());
            })
            .or_insert(HashSet::from([file.path.clone()]));

        if let Feature::Episode(episode) = feature {
            let show = self
                .library
                .shows
                .get(&episode.show_tmdb_id)
                .unwrap_or_else(|| Show::from(episode));
            self.library.shows.insert()
        }

        // FIXME: Stinky!!!
        self.movies_tx.send(
            self.library
                .movies()
                .map(|(movie, paths)| (movie.clone(), paths.clone()))
                .collect(),
        );
    }
}
