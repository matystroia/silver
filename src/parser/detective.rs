use crate::{
    core::api::{Api, ost::SubtitleResult, tmdb::TMDBId},
    model::{episode::Episode, feature::Feature, file::FileData, movie::Movie},
};
use color_eyre::eyre::{Result, eyre};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Detective {
    api: Api,
}

impl Detective {
    pub fn new(api: &Api) -> Self {
        Self { api: api.clone() }
    }

    pub async fn r#match(&self, file: &FileData) -> Result<Option<Feature>> {
        let sub_result = match self.match_ost(file).await? {
            Some(sub_result) => sub_result,
            None => return Ok(None),
        };

        let tmdb_api = match &self.api.tmdb {
            Some(tmdb_api) => tmdb_api,
            None => return Err(eyre!("No TMDB client")),
        };

        match sub_result.feature_type.as_str() {
            "Movie" => {
                let movie_details = tmdb_api.movie_details(sub_result.tmdb_id).await?;
                Ok(Some(Feature::Movie(Movie::from(movie_details))))
            }
            "Episode" => {
                let episode_details = tmdb_api
                    .episode_details(
                        sub_result.parent_tmdb_id.unwrap(),
                        sub_result.season_number.unwrap(),
                        sub_result.episode_number.unwrap(),
                    )
                    .await?;
                Ok(Some(Feature::Episode(Episode::from(episode_details))))
            }
            _ => Err(eyre!("unknown feature_type")),
        }
    }

    pub async fn match_ost(&self, file: &FileData) -> Result<Option<SubtitleResult>> {
        let ost_api = match &self.api.ost {
            Some(ost_api) => ost_api,
            None => return Err(eyre!("No OpenSubtitles client")),
        };

        let results = ost_api.subtitles(&file).await?;

        if let Some(hash_match) = results.iter().find(|res| res.hash_match) {
            return Ok(Some(hash_match.clone()));
        }

        let mut count: HashMap<TMDBId, Vec<SubtitleResult>> = HashMap::new();
        results.iter().for_each(|res| {
            count
                .entry(res.tmdb_id)
                .and_modify(|c| c.push(res.clone()))
                .or_insert(vec![res.clone()]);
        });

        // Majority wins
        Ok(count.iter().find_map(|(_, lst)| {
            if lst.len() > results.len() / 2 {
                Some(lst[0].clone())
            } else {
                None
            }
        }))
    }
}
