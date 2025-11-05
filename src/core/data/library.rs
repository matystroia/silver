use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use color_eyre::eyre::{Result, eyre};
use serde::{Deserialize, Serialize};

use crate::{
    core::{api::tmdb::TMDBId, config::PROJECT_DIRS, data::directory::Directory},
    model::{episode::Episode, feature::Feature, movie::Movie, show::Show},
};

const DATA_FILE: &str = "data.json";

pub type LocalFeature = (Feature, HashSet<PathBuf>);
pub type LocalMovie = (Movie, HashSet<PathBuf>);
pub type LocalEpisode = (Episode, HashSet<PathBuf>);

#[derive(Serialize, Deserialize, Default)]
pub struct Library {
    pub directories: HashSet<Directory>,
    // Should I also store FileData?
    pub local_features: HashMap<Feature, HashSet<PathBuf>>,

    pub shows: HashMap<TMDBId, Show>,
}

impl Library {
    pub(super) fn load() -> Result<Self> {
        let project_dirs = &PROJECT_DIRS;
        let data_dir = project_dirs.data_local_dir();

        if !data_dir.try_exists()? {
            return Err(eyre!("Path doesn't exist: {data_dir:?}"));
        }

        let data_path = data_dir.join(DATA_FILE);
        if !data_path.try_exists()? {
            return Err(eyre!("Path doesn't exist: {data_path:?}"));
        }

        let library_json = fs::read_to_string(data_path)?;
        let mut library: Library = serde_json::from_str(&library_json)?;

        library
            .directories
            .retain(|dir| dir.0.try_exists().unwrap_or(false));

        Ok(library)
    }

    pub fn save(&self) -> Result<()> {
        let project_dirs = &PROJECT_DIRS;
        let data_dir = project_dirs.data_local_dir();

        if !data_dir.try_exists()? {
            fs::create_dir_all(data_dir)?;
        }

        let data_path = data_dir.join(DATA_FILE);
        fs::write(data_path, serde_json::to_string_pretty(&self)?)?;

        Ok(())
    }
}

// TODO: Shouldn't iterate?
impl Library {
    pub fn directories(&self) -> impl Iterator<Item = &Directory> {
        self.directories.iter()
    }

    pub fn movies(&self) -> impl Iterator<Item = LocalMovie> {
        self.local_features
            .iter()
            .filter_map(|(feat, paths)| match feat {
                Feature::Movie(movie) => Some((movie.clone(), paths.clone())),
                _ => None,
            })
    }

    pub fn episodes(&self) -> impl Iterator<Item = LocalEpisode> {
        self.local_features
            .iter()
            .filter_map(|(feat, paths)| match feat {
                Feature::Episode(episode) => Some((episode.clone(), paths.clone())),
                _ => None,
            })
    }
}
