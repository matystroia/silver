use chrono::Datelike;
use serde::{Deserialize, Serialize};

use crate::core::api::tmdb::{MovieDetails, TMDBId};

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct Movie {
    pub title: String,
    pub year: Option<u16>,
    pub tmdb_id: TMDBId,
    pub tagline: String,
    pub overview: String,
    pub genres: Vec<String>,
}

impl From<MovieDetails> for Movie {
    fn from(details: MovieDetails) -> Self {
        Self {
            tmdb_id: details.tmdb_id,
            title: details.title,
            year: Some(details.release_date.year() as u16),
            tagline: details.tagline,
            overview: details.overview,
            genres: details.genres,
        }
    }
}
