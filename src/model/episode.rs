use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::core::api::tmdb::{EpisodeDetails, TMDBId};

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct Episode {
    pub tmdb_id: TMDBId,
    pub show_tmdb_id: TMDBId,
    pub title: String,
    pub season_number: u64,
    pub episode_number: u64,
    pub overview: String,
    #[serde(with = "crate::util::misc::date_format")]
    pub release_date: NaiveDate,
}

impl From<EpisodeDetails> for Episode {
    fn from(details: EpisodeDetails) -> Self {
        Self {
            tmdb_id: details.tmdb_id,
            show_tmdb_id: details.show_tmdb_id,
            title: details.name,
            season_number: details.season_number,
            episode_number: details.episode_number,
            overview: details.overview,
            release_date: details.air_date,
        }
    }
}
