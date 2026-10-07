use std::{fmt::Display, sync::LazyLock};

use chrono::NaiveDate;
use color_eyre::eyre::{OptionExt, Result};
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use nonzero_ext::nonzero;
use reqwest::{
    Url,
    header::{HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    core::Config,
    model::{Episode, Movie},
    util::{self, json_field},
};

static URL_BASE: LazyLock<Url> =
    LazyLock::new(|| Url::parse("https://api.themoviedb.org/3/").unwrap());

static LIMITER: LazyLock<DefaultDirectRateLimiter> =
    LazyLock::new(|| RateLimiter::direct(Quota::per_second(nonzero!(20u32))));

#[derive(Clone)]
pub struct TMDBClient {
    client: reqwest::Client,
}

impl TMDBClient {
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

    pub async fn movie_details(&self, movie_id: TMDBId) -> Result<Movie> {
        LIMITER.until_ready().await;

        let url = URL_BASE.join(&format!("movie/{movie_id}")).unwrap();
        let response = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        let response: Value = serde_json::from_str(&response)?;
        let movie_details = MovieDetails::from(response.as_object().unwrap())?;

        let tmdb = self.clone();
        let (path, id) = (movie_details.poster_path.clone(), movie_details.id);
        tokio::spawn(async move { tmdb.fetch_poster(path, id).await });

        Ok(Movie::from(movie_details))
    }

    pub async fn episode_details(
        &self,
        series_id: TMDBId,
        season_number: u16,
        episode_number: u16,
    ) -> Result<Episode> {
        LIMITER.until_ready().await;

        let url = URL_BASE
            .join(&format!(
                "tv/{series_id}/season/{season_number}/episode/{episode_number}"
            ))
            .unwrap();
        let response = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        let response: Value = serde_json::from_str(&response)?;
        let episode_details = EpisodeDetails::from(series_id, response.as_object().unwrap())?;

        Ok(Episode::from(episode_details))
    }

    pub async fn series_details(&self, series_id: TMDBId) -> Result<SeriesDetails> {
        LIMITER.until_ready().await;

        let url = URL_BASE.join(&format!("tv/{series_id}")).unwrap();
        let response = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        let response: Value = serde_json::from_str(&response)?;
        let series_details = SeriesDetails::from(response.as_object().unwrap())?;

        let tmdb = self.clone();
        let (path, id) = (series_details.poster_path.clone(), series_details.id);
        tokio::spawn(async move { tmdb.fetch_poster(path, id).await });

        Ok(series_details)
    }

    pub async fn season_details(&self, series_id: TMDBId, season: u64) -> Result<SeasonDetails> {
        LIMITER.until_ready().await;

        let url = URL_BASE
            .join(&format!("tv/{series_id}/season/{season}"))
            .unwrap();
        let response = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        let response: Value = serde_json::from_str(&response)?;
        SeasonDetails::from(response.as_object().unwrap())
    }

    pub async fn search_movie(&self, query: &[(&str, String)]) -> Result<Vec<MovieSearchResult>> {
        LIMITER.until_ready().await;

        let mut url = URL_BASE.join("search/movie").unwrap();
        url.query_pairs_mut()
            .append_pair("include_adult", "true")
            .extend_pairs(query);
        let response = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        let response: Value = serde_json::from_str(&response)?;
        let results: Vec<serde_json::Map<String, Value>> =
            json_field!(response.as_object().unwrap(), "results")?;
        Ok(results
            .iter()
            .filter_map(|res| MovieSearchResult::from(res).ok())
            .collect())
    }

    pub async fn search_series(&self, query: &[(&str, String)]) -> Result<Vec<SeriesSearchResult>> {
        LIMITER.until_ready().await;

        let mut url = URL_BASE.join("search/tv").unwrap();
        url.query_pairs_mut()
            .append_pair("include_adult", "true")
            .extend_pairs(query);
        let response = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        let response: Value = serde_json::from_str(&response)?;
        let results: Vec<serde_json::Map<String, Value>> =
            json_field!(response.as_object().unwrap(), "results")?;
        Ok(results
            .iter()
            .filter_map(|res| SeriesSearchResult::from(res).ok())
            .collect())
    }

    pub async fn fetch_poster(&self, path: String, id: TMDBId) -> Result<()> {
        let posters_path = Config::data_dir().join("posters");
        std::fs::create_dir_all(&posters_path)?;

        LIMITER.until_ready().await;

        let size = "w500";
        let bytes = self
            .client
            .get(format!("https://image.tmdb.org/t/p/{size}{path}"))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .bytes()
            .await
            .unwrap();

        tokio::fs::write(posters_path.join(format!("{id}.jpg")), &bytes).await?;

        Ok(())
    }
}

#[derive(sqlx::Type)]
#[sqlx(transparent)]
#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct TMDBId(pub i64);

impl Display for TMDBId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}

pub struct MovieDetails {
    pub id: TMDBId,
    pub title: String,
    pub original_title: String,
    pub original_language: String,
    pub release_date: NaiveDate,
    pub tagline: String,
    pub overview: String,
    pub poster_path: String,
    pub genres: Vec<String>,
}

impl MovieDetails {
    fn from(json: &serde_json::Map<String, Value>) -> Result<Self> {
        Ok(Self {
            id: json_field!(json, "id")?,
            title: json_field!(json, "title")?,
            original_title: json_field!(json, "original_title")?,
            original_language: json_field!(json, "original_language")?,
            release_date: util::date_format::deserialize(&json["release_date"])?,
            tagline: json_field!(json, "tagline")?,
            overview: json_field!(json, "overview")?,
            poster_path: json_field!(json, "poster_path")?,
            genres: json["genres"]
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

#[derive(Debug)]
pub struct MovieSearchResult {
    pub id: TMDBId,
    pub title: String,
    pub original_title: String,
    pub original_language: String,
    #[allow(dead_code)]
    pub overview: String,
    pub release_date: NaiveDate,
    pub vote_count: u32,
}

impl MovieSearchResult {
    fn from(json: &serde_json::Map<String, Value>) -> Result<Self> {
        Ok(Self {
            id: json_field!(json, "id")?,
            title: json_field!(json, "title")?,
            original_title: json_field!(json, "original_title")?,
            original_language: json_field!(json, "original_language")?,
            overview: json_field!(json, "overview")?,
            release_date: json_field!(json, "release_date")?,
            vote_count: json_field!(json, "vote_count")?,
        })
    }
}

#[derive(Debug)]
pub struct SeriesSearchResult {
    pub id: TMDBId,
    pub name: String,
    pub original_name: String,
    pub original_language: String,
    pub overview: String,
    pub vote_count: u32,
}

impl SeriesSearchResult {
    fn from(json: &serde_json::Map<String, Value>) -> Result<Self> {
        Ok(Self {
            id: json_field!(json, "id")?,
            name: json_field!(json, "name")?,
            original_name: json_field!(json, "original_name")?,
            original_language: json_field!(json, "original_language")?,
            overview: json_field!(json, "overview")?,
            vote_count: json_field!(json, "vote_count")?,
        })
    }
}

pub struct EpisodeDetails {
    pub id: TMDBId,
    pub series_id: TMDBId,
    pub season_number: i64,
    pub episode_number: i64,
    pub name: String,
    pub overview: String,
    pub air_date: NaiveDate,
}

impl EpisodeDetails {
    fn from(series_id: TMDBId, json: &serde_json::Map<String, Value>) -> Result<Self> {
        Ok(Self {
            id: json_field!(json, "id")?,
            series_id,
            season_number: json_field!(json, "season_number")?,
            episode_number: json_field!(json, "episode_number")?,
            name: json_field!(json, "name")?,
            overview: json_field!(json, "overview")?,
            air_date: util::date_format::deserialize(&json["air_date"])?,
        })
    }
}

#[derive(Clone)]
pub struct SeriesDetails {
    pub id: TMDBId,
    pub name: String,
    pub original_name: String,
    pub original_language: String,
    #[allow(dead_code)]
    pub num_seasons: u64,
    #[allow(dead_code)]
    pub num_episodes: u64,
    pub tagline: String,
    pub overview: String,
    pub poster_path: String,
    pub genres: Vec<String>,
    pub seasons: Vec<SeasonSummary>,
}

impl SeriesDetails {
    fn from(json: &serde_json::Map<String, Value>) -> Result<Self> {
        Ok(Self {
            id: json_field!(json, "id")?,
            name: json_field!(json, "name")?,
            original_name: json_field!(json, "original_name")?,
            original_language: json_field!(json, "original_language")?,
            num_seasons: json_field!(json, "number_of_seasons")?,
            num_episodes: json_field!(json, "number_of_episodes")?,
            poster_path: json_field!(json, "poster_path")?,
            seasons: json["seasons"]
                .as_array()
                .ok_or_eyre("invalid seasons")?
                .iter()
                .filter_map(|season| SeasonSummary::from(season.as_object().unwrap()).ok())
                .collect(),
            genres: json["genres"]
                .as_array()
                .map(|gs| {
                    gs.iter()
                        .map(|g| g.as_object().unwrap()["name"].as_str().unwrap().into())
                        .collect()
                })
                .ok_or_eyre("empty genres")?,
            tagline: json_field!(json, "tagline")?,
            overview: json_field!(json, "overview")?,
        })
    }
}

#[derive(Clone)]
pub struct SeasonSummary {
    #[allow(dead_code)]
    pub id: TMDBId,
    #[allow(dead_code)]
    pub name: String,
    pub number: u64,
    #[allow(dead_code)]
    pub num_episodes: u64,
    #[allow(dead_code)]
    pub overview: String,
}

impl SeasonSummary {
    fn from(json: &serde_json::Map<String, Value>) -> Result<Self> {
        Ok(Self {
            id: json_field!(json, "id")?,
            name: json_field!(json, "name")?,
            number: json_field!(json, "season_number")?,
            num_episodes: json_field!(json, "episode_count")?,
            overview: json_field!(json, "overview")?,
        })
    }
}

pub struct SeasonDetails {
    pub id: TMDBId,
    pub name: String,
    pub number: i64,
    #[allow(dead_code)]
    pub overview: String,
    pub episodes: Vec<EpisodeDetails>,
}

impl SeasonDetails {
    fn from(json: &serde_json::Map<String, Value>) -> Result<Self> {
        Ok(Self {
            id: json_field!(json, "id")?,
            name: json_field!(json, "name")?,
            number: json_field!(json, "season_number")?,
            overview: json_field!(json, "overview")?,
            episodes: json["episodes"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|ep| {
                    EpisodeDetails::from(
                        TMDBId(ep.as_object().unwrap()["show_id"].as_i64().unwrap()),
                        ep.as_object().unwrap(),
                    )
                    .ok()
                })
                .collect(),
        })
    }
}
