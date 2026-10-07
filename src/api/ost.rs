use std::sync::LazyLock;

use color_eyre::eyre::{OptionExt, Result, bail};
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use nonzero_ext::nonzero;
use reqwest::{
    Url,
    header::{HeaderMap, HeaderValue},
};
use serde_json::Value;

use crate::{api::TMDBId, model::FileData, parser::Filename, util::json_field};

static URL_BASE: LazyLock<Url> =
    LazyLock::new(|| Url::parse("https://api.opensubtitles.com/api/v1/").unwrap());

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

    pub async fn subtitles(
        &self,
        file: &FileData,
        parsed: Option<&Filename>,
    ) -> Result<Vec<SubtitleResult>> {
        if file.hash.is_empty() {
            return Ok(vec![]);
        }

        LIMITER.until_ready().await;

        let mut url = URL_BASE.join("subtitles").unwrap();
        url.query_pairs_mut().append_pair("moviehash", &file.hash);

        match parsed {
            Some(details) => {
                url.query_pairs_mut().extend_pairs(&details.to_ost_query());
            }
            None => {
                url.query_pairs_mut()
                    .append_pair("query", &file.filename().unwrap());
            }
        };

        let response = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let response: Value = serde_json::from_str(&response)?;
        let data = response["data"].as_array().ok_or_eyre("no data")?;
        Ok(data
            .iter()
            .filter_map(|val| SubtitleResult::new(val.as_object().unwrap()).ok())
            .collect())
    }
}

#[derive(Clone, Debug)]
pub struct SubtitleResult {
    pub release: String,
    pub hash_match: bool,
    pub details: ResultDetails,
}

#[derive(Clone, Debug)]
pub enum ResultDetails {
    Movie(MovieDetails),
    Episode(EpisodeDetails),
}

#[derive(Clone, Debug)]
pub struct MovieDetails {
    pub tmdb_id: TMDBId,
    pub title: String,
    pub year: u16,
}

#[derive(Clone, Debug)]
pub struct EpisodeDetails {
    #[allow(dead_code)]
    pub tmdb_id: TMDBId,
    pub parent_tmdb_id: TMDBId,
    #[allow(dead_code)]
    pub title: String,
    pub parent_title: String,
    pub season_number: u16,
    pub episode_number: u16,
}

impl SubtitleResult {
    fn new(json: &serde_json::Map<String, Value>) -> Result<Self> {
        let attributes = json["attributes"].as_object().unwrap();
        let details = attributes["feature_details"].as_object().unwrap();

        Ok(Self {
            release: json_field!(attributes, "release")?,
            hash_match: json_field!(attributes, "moviehash_match")?,
            details: ResultDetails::new(details)?,
        })
    }
}

impl ResultDetails {
    fn new(details: &serde_json::Map<String, Value>) -> Result<Self> {
        let feature_type: String = json_field!(details, "feature_type")?;
        match feature_type.as_str() {
            "Movie" => Ok(Self::Movie(MovieDetails {
                tmdb_id: json_field!(details, "tmdb_id")?,
                title: json_field!(details, "title")?,
                year: json_field!(details, "year")?,
            })),
            "Episode" => Ok(Self::Episode(EpisodeDetails {
                tmdb_id: json_field!(details, "tmdb_id")?,
                parent_tmdb_id: json_field!(details, "parent_tmdb_id")?,
                title: json_field!(details, "title")?,
                parent_title: json_field!(details, "parent_title")?,
                season_number: json_field!(details, "season_number")?,
                episode_number: json_field!(details, "episode_number")?,
            })),
            _ => bail!("unknown feature_type: {feature_type}\n{details:?}"),
        }
    }
}
