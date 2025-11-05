use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::core::api::tmdb::TMDBId;

#[derive(Serialize, Deserialize)]
pub struct Show {
    pub tmdb_id: TMDBId,
    pub title: String,
    pub seasons: Vec<Season>,
}

#[derive(Serialize, Deserialize)]
pub struct Season {
    pub tmdb_id: TMDBId,
}

impl Hash for Show {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tmdb_id.hash(state);
    }
}

impl PartialEq for Show {
    fn eq(&self, other: &Self) -> bool {
        self.tmdb_id.eq(&other.tmdb_id)
    }
}

impl Eq for Show {}
