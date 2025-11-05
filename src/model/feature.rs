use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::{
    core::api::tmdb::TMDBId,
    model::{episode::Episode, movie::Movie},
};

#[derive(Debug, Serialize, Deserialize)]
pub enum Feature {
    Movie(Movie),
    Episode(Episode),
}

impl Feature {
    fn tmdb_id(&self) -> TMDBId {
        match self {
            Feature::Movie(movie) => movie.tmdb_id,
            Feature::Episode(episode) => episode.tmdb_id,
        }
    }
}

impl Hash for Feature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tmdb_id().hash(state);
    }
}

impl PartialEq for Feature {
    fn eq(&self, other: &Self) -> bool {
        self.tmdb_id().eq(&other.tmdb_id())
    }
}

impl Eq for Feature {}
