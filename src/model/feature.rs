use std::{hash::Hash, sync::Arc};

use crate::{
    api::TMDBId,
    model::{movie::Movie, series::Episode},
};

#[derive(Debug, Clone)]
pub enum Feature {
    Movie(Arc<Movie>),
    Episode(Arc<Episode>),
}

impl Feature {
    pub fn id(&self) -> TMDBId {
        match self {
            Self::Movie(movie) => movie.id,
            Self::Episode(episode) => episode.id,
        }
    }
}

impl Hash for Feature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id().hash(state); }
}

impl PartialEq for Feature {
    fn eq(&self, other: &Self) -> bool { self.id().eq(&other.id()) }
}

impl Eq for Feature {}
