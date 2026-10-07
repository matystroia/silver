use std::{hash::Hash, ops::Deref, path::PathBuf};

use chrono::{Datelike, NaiveDate};

use crate::{
    api::tmdb::{EpisodeDetails, SeasonDetails, SeriesDetails, TMDBId},
    model::Genre,
};

#[derive(Clone)]
pub struct Series<T> {
    pub id: TMDBId,
    pub name: String,
    pub original_name: String,
    pub original_language: String,
    pub tagline: String,
    pub overview: String,
    pub genres: Vec<Genre>,
    pub seasons: Vec<Season<T>>,
}

#[derive(Clone)]
pub struct Season<T> {
    pub id: TMDBId,
    pub number: i64,
    pub name: String,
    pub episodes: Vec<T>,
}

#[derive(Debug, Clone)]
pub struct Episode {
    pub id: TMDBId,
    pub series_id: TMDBId,
    pub title: String,
    pub season_number: i64,
    pub episode_number: i64,
    pub overview: String,
    pub release_date: NaiveDate,
}

impl Series<Episode> {
    pub fn from_details(details: SeriesDetails, seasons: Vec<SeasonDetails>) -> Self {
        Self {
            id: details.id,
            name: details.name,
            original_name: details.original_name,
            original_language: details.original_language,
            tagline: details.tagline,
            overview: details.overview,
            genres: details.genres.iter().cloned().map(Genre).collect(),
            seasons: seasons.into_iter().map(Season::from).collect(),
        }
    }
}

impl<T> Series<T>
where
    T: Deref<Target = Episode> + Clone,
{
    pub fn seasons(&self) -> &[Season<T>] { self.seasons.as_slice() }

    pub fn regular_seasons(&self) -> Vec<Season<T>> {
        self.seasons
            .iter()
            .filter(|s| s.number > 0 && !s.episodes.is_empty())
            .cloned()
            .collect()
    }

    pub fn year(&self) -> Option<i32> {
        self.seasons
            .first()
            .and_then(|s| s.episodes.first())
            .map(|ep| ep.release_date.year())
    }
}

impl From<SeasonDetails> for Season<Episode> {
    fn from(details: SeasonDetails) -> Self {
        Self {
            id: details.id,
            number: details.number,
            name: details.name,
            episodes: details.episodes.into_iter().map(Episode::from).collect(),
        }
    }
}

impl From<EpisodeDetails> for Episode {
    fn from(details: EpisodeDetails) -> Self {
        Self {
            id: details.id,
            series_id: details.series_id,
            title: details.name,
            season_number: details.season_number,
            episode_number: details.episode_number,
            overview: details.overview,
            release_date: details.air_date,
        }
    }
}

impl Hash for Series<Episode> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state); }
}

impl PartialEq for Series<Episode> {
    fn eq(&self, other: &Self) -> bool { self.id.eq(&other.id) }
}

impl Eq for Series<Episode> {}

// TODO: Local<Episode>?
#[derive(Clone)]
pub struct LocalEpisode {
    pub episode: Episode,
    pub paths: Vec<PathBuf>,
}

impl Deref for Episode {
    type Target = Episode;
    fn deref(&self) -> &Self::Target { self }
}

impl Deref for LocalEpisode {
    type Target = Episode;
    fn deref(&self) -> &Self::Target { &self.episode }
}
