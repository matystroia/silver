use std::{ops::Deref, path::PathBuf};

use chrono::NaiveDate;

use crate::api::tmdb::{MovieDetails, TMDBId};

#[derive(Debug, Clone, Hash, PartialEq, Eq, sqlx::Type)]
#[sqlx(transparent)]
pub struct Genre(pub String);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Movie {
    pub id: TMDBId,
    pub title: String,
    pub original_title: String,
    pub original_language: String,
    pub release_date: NaiveDate,
    pub tagline: String,
    pub overview: String,
    pub genres: Vec<Genre>,
}

impl From<MovieDetails> for Movie {
    fn from(details: MovieDetails) -> Self {
        Self {
            id: details.id,
            title: details.title,
            original_title: details.original_title,
            original_language: details.original_language,
            release_date: details.release_date,
            tagline: details.tagline,
            overview: details.overview,
            genres: details.genres.iter().cloned().map(Genre).collect(),
        }
    }
}

pub struct LocalMovie {
    pub movie: Movie,
    pub paths: Vec<PathBuf>,
}

impl Deref for LocalMovie {
    type Target = Movie;
    fn deref(&self) -> &Self::Target { &self.movie }
}
