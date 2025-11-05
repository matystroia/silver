use std::fmt::Display;

use chrono::NaiveDate;
use color_eyre::eyre::{OptionExt, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::util::misc::{date_format, field};

#[derive(Clone)]
pub struct TMDBClient {
    client: reqwest::Client,
}

impl TMDBClient {
    const URL_BASE: &str = "https://api.themoviedb.org/3";

    pub fn new(access_token: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", access_token))?,
        );

        let client = Self {
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()?,
        };

        Ok(client)
    }

    pub async fn movie_details(&self, movie_id: TMDBId) -> Result<MovieDetails> {
        let url = format!("{}/movie/{movie_id}", Self::URL_BASE);
        let response = self.client.get(url).send().await?.text().await?;
        let response: Value = serde_json::from_str(&response)?;
        MovieDetails::from(response.as_object().unwrap())
    }

    pub async fn episode_details(
        &self,
        series_id: TMDBId,
        season_number: u64,
        episode_number: u64,
    ) -> Result<EpisodeDetails> {
        let url = format!(
            "{}/tv/{series_id}/season/{season_number}/episode/{episode_number}",
            Self::URL_BASE
        );
        let response = self.client.get(url).send().await?.text().await?;
        let response: Value = serde_json::from_str(&response)?;
        EpisodeDetails::from(series_id, response.as_object().unwrap())
    }

    pub async fn show_details(&self, show_id: TMDBId) -> Result<ShowDetails> {
        let url = format!("{}/tv/{show_id}", Self::URL_BASE);
        let response = self.client.get(url).send().await?.text().await?;
        let response: Value = serde_json::from_str(&response)?;
        ShowDetails::from(response.as_object().unwrap())
    }
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct TMDBId(pub u64);

impl Display for TMDBId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct MovieDetails {
    pub tmdb_id: TMDBId,
    // TODO: IMDB id
    pub title: String,
    pub release_date: NaiveDate,
    pub tagline: String,
    pub overview: String,
    pub genres: Vec<String>,
}

impl MovieDetails {
    fn from(value: &serde_json::Map<String, Value>) -> Result<Self> {
        Ok(Self {
            tmdb_id: field(value, "id")?,
            title: field(value, "title")?,
            release_date: date_format::deserialize(&value["release_date"])?,
            tagline: field(value, "tagline")?,
            overview: field(value, "overview")?,
            genres: value["genres"]
                .as_array()
                .map(|gs| {
                    gs.iter()
                        .map(|g| g.as_object().unwrap()["name"].as_str().unwrap().into())
                        .collect()
                })
                .ok_or_eyre("empty genres")?,
        })
    }
}

pub struct EpisodeDetails {
    pub tmdb_id: TMDBId,
    pub show_tmdb_id: TMDBId,
    pub season_number: u64,
    pub episode_number: u64,
    pub name: String,
    pub overview: String,
    pub air_date: NaiveDate,
}

impl EpisodeDetails {
    fn from(series_id: TMDBId, value: &serde_json::Map<String, Value>) -> Result<Self> {
        Ok(Self {
            tmdb_id: field(value, "id")?,
            show_tmdb_id: series_id,
            season_number: field(value, "season_number")?,
            episode_number: field(value, "episode_number")?,
            name: field(value, "name")?,
            overview: field(value, "overview")?,
            air_date: date_format::deserialize(&value["air_date"])?,
        })
    }
}

pub struct ShowDetails {
    pub tmdb_id: TMDBId,
    pub name: String,
    pub num_seasons: u64,
    pub num_episodes: u64,
    pub seasons: Vec<SeasonDetails>,
    pub genres: Vec<String>,
    pub tagline: String,
}

impl ShowDetails {
    fn from(value: &serde_json::Map<String, Value>) -> Result<Self> {
        Ok(Self {
            tmdb_id: field(value, "id")?,
            name: field(value, "name")?,
            num_seasons: field(value, "number_of_seasons")?,
            num_episodes: field(value, "number_of_episodes")?,
            seasons: value["seasons"]
                .as_array()
                .ok_or_eyre("invalid seasons")?
                .iter()
                .filter_map(|season| SeasonDetails::from(season.as_object().unwrap()).ok())
                .collect(),
            genres: value["genres"]
                .as_array()
                .map(|gs| {
                    gs.iter()
                        .map(|g| g.as_object().unwrap()["name"].as_str().unwrap().into())
                        .collect()
                })
                .ok_or_eyre("empty genres")?,
            tagline: field(value, "tagline")?,
        })
    }
}

pub struct SeasonDetails {
    pub tmdb_id: TMDBId,
    pub name: String,
    pub number: u64,
    pub num_episodes: u64,
    pub overview: String,
}

impl SeasonDetails {
    fn from(value: &serde_json::Map<String, Value>) -> Result<Self> {
        Ok(Self {
            tmdb_id: field(value, "id")?,
            name: field(value, "name")?,
            number: field(value, "season_number")?,
            num_episodes: field(value, "episode_count")?,
            overview: field(value, "overview")?,
        })
    }
}
