use std::sync::LazyLock;

use color_eyre::eyre::{OptionExt, Result};
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use nonzero_ext::nonzero;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;

use crate::{
    core::api::{imdb::IMDBId, tmdb::TMDBId},
    model::file::FileData,
    util::misc::{field, optional_field},
};

const URL_BASE: &str = "https://api.opensubtitles.com/api/v1";

static LIMITER: LazyLock<DefaultDirectRateLimiter> =
    LazyLock::new(|| RateLimiter::direct(Quota::per_second(nonzero!(3u32))));

#[derive(Clone)]
pub struct OpenSubtitlesClient {
    client: reqwest::Client,
}

impl OpenSubtitlesClient {
    pub fn new(api_key: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("Api-Key", HeaderValue::from_str(api_key)?);
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("User-Agent", HeaderValue::from_static("Silver v0.1"));

        let client = Self {
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()?,
        };

        Ok(client)
    }

    pub async fn subtitles(&self, file: &FileData) -> Result<Vec<SubtitleResult>> {
        LIMITER.until_ready().await;

        let response = self
            .client
            .get(format!("{URL_BASE}/subtitles"))
            .query(&[
                ("moviehash", &file.hash),
                ("query", &file.clean_filename().to_string()),
            ])
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let response: Value = serde_json::from_str(&response)?;
        let data = response["data"].as_array().ok_or_eyre("no data")?;
        Ok(data
            .iter()
            .filter_map(|val| SubtitleResult::from(val.as_object().unwrap()).ok())
            .collect())
    }
}

#[derive(Clone, Debug)]
pub struct SubtitleResult {
    pub feature_type: String,
    pub title: String,
    pub year: u16,

    pub tmdb_id: TMDBId,
    pub imdb_id: IMDBId,
    pub hash_match: bool,

    pub parent_tmdb_id: Option<TMDBId>,
    pub season_number: Option<u64>,
    pub episode_number: Option<u64>,
}

impl SubtitleResult {
    fn from(value: &serde_json::Map<String, Value>) -> Result<Self> {
        let attributes = value["attributes"].as_object().unwrap();
        let details = attributes["feature_details"].as_object().unwrap();
        Ok(Self {
            feature_type: field(details, "feature_type")?,
            title: field(details, "title")?,
            year: field(details, "year")?,
            tmdb_id: field(details, "tmdb_id")?,
            imdb_id: field(details, "imdb_id")?,

            parent_tmdb_id: optional_field(details, "parent_tmdb_id")?,
            season_number: optional_field(details, "season_number")?,
            episode_number: optional_field(details, "episode_number")?,

            hash_match: field(attributes, "moviehash_match")?,
        })
    }
}
