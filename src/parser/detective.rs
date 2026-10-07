use std::{
    cmp::Ordering,
    sync::{Arc, LazyLock},
};

use chrono::Datelike;
use color_eyre::eyre::Result;
use itertools::Itertools;
use regex::Regex;

use crate::{
    api::{TMDBId, ost, tmdb},
    core::{Core, log::Logger},
    model::{self, Feature, FileData},
    parser::Filename,
    util::{self, LangString},
};

const TMDB_MIN_VOTE_COUNT: u32 = 5;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Hit {
    Movie(TMDBId),
    Episode(TMDBId, u16, u16),
}

impl Hit {
    async fn resolve(self, tmdb: &tmdb::TMDBClient) -> Result<Feature> {
        match self {
            Hit::Movie(id) => {
                let movie = tmdb.movie_details(id).await?;
                Ok(Feature::Movie(Arc::new(movie)))
            }
            Hit::Episode(id, season, episode) => {
                let episode = tmdb.episode_details(id, season, episode).await?;
                Ok(Feature::Episode(Arc::new(episode)))
            }
        }
    }
}

#[derive(Clone)]
pub struct Detective {
    core: Arc<Core>,
    tmdb: Option<Arc<tmdb::TMDBClient>>,
    ost: Option<Arc<ost::OpenSubtitlesClient>>,
}

impl Detective {
    pub fn new(core: Arc<Core>) -> Self {
        Self {
            tmdb: core.api.tmdb.clone(),
            ost: core.api.ost.clone(),
            core,
        }
    }

    /// Order is:
    /// 1. Parse filename
    /// 2. Hit database
    /// 3. Hit TMDB
    /// 4. Hit OST
    pub async fn r#match(&self, file: &FileData) -> Result<Option<Feature>> {
        let Some(tmdb) = &self.tmdb else {
            return Ok(None);
        };

        let filename = Filename::new(&file.path);
        let hit = 'find: {
            if let Some(f) = &filename {
                if let Some(hit) = self.match_local(f).await? {
                    break 'find Some(hit);
                }
                if let Some(hit) = self.match_tmdb_search(f).await? {
                    break 'find Some(hit);
                }
                Logger::debug(format!(
                    "Didn't find tmdb match for {} ({:?})",
                    file.filename().unwrap(),
                    filename
                ));
            } else {
                Logger::debug(format!(
                    "No filename match for {}",
                    file.filename().unwrap()
                ));
            }
            self.match_ost(file, filename.as_ref()).await?
        };

        match hit {
            Some(hit) => Ok(Some(hit.resolve(tmdb).await?)),
            None => Ok(None),
        }
    }

    async fn match_ost(&self, file: &FileData, filename: Option<&Filename>) -> Result<Option<Hit>> {
        let Some(ost) = &self.ost else {
            return Ok(None);
        };

        let results = ost.subtitles(file, filename).await?;

        if let Some(hash_match) = results.iter().find(|res| res.hash_match)
            && filename.as_ref().is_none_or(|d| hash_match.matches_file(d))
        {
            Ok(Some(match &hash_match.details {
                ost::ResultDetails::Movie(movie) => Hit::Movie(movie.tmdb_id),
                ost::ResultDetails::Episode(episode) => Hit::Episode(
                    episode.parent_tmdb_id,
                    episode.season_number,
                    episode.episode_number,
                ),
            }))
        } else {
            Ok(None)
        }
    }

    async fn match_tmdb_search(&self, filename: &Filename) -> Result<Option<Hit>> {
        let Some(tmdb) = &self.tmdb else {
            return Ok(None);
        };

        let query = filename.to_tmdb_query();

        let results: Vec<Box<dyn TMDBResult>> = match filename {
            Filename::Movie(_) => boxed(tmdb.search_movie(&query).await?.into_iter()),
            Filename::Episode(_) => boxed(tmdb.search_series(&query).await?.into_iter()),
        };

        self.match_tmdb(filename, &results).await
    }

    async fn match_local(&self, filename: &Filename) -> Result<Option<Hit>> {
        let results: Vec<Box<dyn TMDBResult>> = match filename {
            Filename::Movie(_) => boxed(
                self.core
                    .data()
                    .movies()
                    .await?
                    .into_iter()
                    .map(tmdb::MovieSearchResult::from),
            ),
            Filename::Episode(_) => boxed(
                self.core
                    .data()
                    .series()
                    .await?
                    .into_iter()
                    .map(tmdb::SeriesSearchResult::from),
            ),
        };
        self.match_tmdb(filename, &results).await
    }

    async fn match_tmdb(
        &self,
        filename: &Filename,
        results: &[Box<dyn TMDBResult>],
    ) -> Result<Option<Hit>> {
        let hits = results
            .iter()
            .filter(|res| res.vote_count() >= TMDB_MIN_VOTE_COUNT)
            .filter_map(|res| res.to_hit(filename).map(|hit| (res, hit)))
            .collect_vec();

        match hits.as_slice() {
            [] => Ok(None),
            [(_, hit)] => Ok(Some(*hit)),
            too_many => {
                // Max by string similarity
                let max_matches = too_many.iter().max_float_set_by_key(|(res, _)| {
                    util::fuzzy_match(res.name(), filename.name())
                        .max(util::fuzzy_match(res.original_name(), filename.name()))
                });
                if let [(_, hit)] = max_matches[..] {
                    return Ok(Some(*hit));
                }

                // Prefer original title
                let max_matches = max_matches.iter().max_float_set_by_key(|(res, _)| {
                    util::fuzzy_match(res.original_name(), filename.name())
                });
                if let [(_, hit)] = max_matches[..] {
                    return Ok(Some(*hit));
                }

                Logger::debug(format!("Too many matches for {filename:?}"));

                Ok(None)
            }
        }
    }
}

trait MaxSetByKey: Iterator {
    fn max_float_set_by_key<K, F>(self, mut key: F) -> Vec<Self::Item>
    where
        Self: Sized,
        K: PartialOrd + Default,
        F: FnMut(&Self::Item) -> K,
    {
        self.fold((Default::default(), vec![]), |(max, mut max_set), x| {
            let key = key(&x);
            match key.partial_cmp(&max) {
                Some(Ordering::Less) | None => (max, max_set),
                Some(Ordering::Equal) => {
                    max_set.push(x);
                    (max, max_set)
                }
                Some(Ordering::Greater) => (key, vec![x]),
            }
        })
        .1
    }
}

impl<T> MaxSetByKey for core::slice::Iter<'_, T> {}

trait TMDBResult: Send + Sync {
    fn name(&self) -> &str;
    fn original_name(&self) -> LangString;
    fn vote_count(&self) -> u32;
    fn to_hit(&self, file: &Filename) -> Option<Hit>;
}

impl TMDBResult for tmdb::MovieSearchResult {
    fn name(&self) -> &str { &self.title }
    fn original_name(&self) -> LangString {
        LangString::new(&self.original_title, &self.original_language)
    }
    fn vote_count(&self) -> u32 { self.vote_count }
    fn to_hit(&self, file: &Filename) -> Option<Hit> {
        let Filename::Movie(file) = file else {
            return None;
        };
        let matches = self.release_date.year() as u16 == file.year
            && [
                // TODO: THIS
                self.name().into(),
                self.original_name(),
                short_title(self.name()).into(),
                // LangString::new(&short_title(&self.original_title), &self.original_language),
            ]
            .into_iter()
            .any(|title| util::is_fuzzy_match(title, &file.title));
        matches.then_some(Hit::Movie(self.id))
    }
}

impl TMDBResult for tmdb::SeriesSearchResult {
    fn name(&self) -> &str { &self.name }
    fn original_name(&self) -> LangString {
        LangString::new(&self.original_name, &self.original_language)
    }
    fn vote_count(&self) -> u32 { self.vote_count }
    fn to_hit(&self, file: &Filename) -> Option<Hit> {
        let Filename::Episode(file) = file else {
            return None;
        };
        let matches = util::is_fuzzy_match(&self.name, &file.series)
            || util::is_fuzzy_match(
                LangString::new(&self.original_name, &self.original_language),
                &file.series,
            );
        matches.then_some(Hit::Episode(self.id, file.season, file.episode))
    }
}

impl From<Arc<model::LocalMovie>> for tmdb::MovieSearchResult {
    fn from(value: Arc<model::LocalMovie>) -> Self {
        Self {
            id: value.id,
            title: value.title.clone(),
            original_title: value.original_title.clone(),
            original_language: value.original_language.clone(),
            overview: value.overview.clone(),
            release_date: value.release_date,
            vote_count: TMDB_MIN_VOTE_COUNT,
        }
    }
}

impl From<Arc<model::Series<model::LocalEpisode>>> for tmdb::SeriesSearchResult {
    fn from(value: Arc<model::Series<model::LocalEpisode>>) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
            original_name: value.original_name.clone(),
            original_language: value.original_language.clone(),
            overview: value.overview.clone(),
            vote_count: TMDB_MIN_VOTE_COUNT,
        }
    }
}

// TODO: MovieLite

static SUBTITLE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(.+?): .+$").unwrap());
static PAREN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(.+?) \(.+\)$").unwrap());

fn short_title(str: &str) -> String {
    if let Some(caps) = [&SUBTITLE_RE, &PAREN_RE]
        .iter()
        .find_map(|re| re.captures(str))
    {
        caps[1].to_string()
    } else {
        str.to_string()
    }
}

impl ost::SubtitleResult {
    fn matches_file(&self, file: &Filename) -> bool {
        match (&self.details, file) {
            (ost::ResultDetails::Movie(sub), Filename::Movie(file)) => {
                (util::is_fuzzy_match(&sub.title, &file.title)
                    || util::is_fuzzy_match(&self.release, &file.title))
                    && sub.year == file.year
            }
            (ost::ResultDetails::Episode(sub), Filename::Episode(file)) => {
                util::is_fuzzy_match(&sub.parent_title, &file.series)
                    && sub.episode_number == file.episode
                    && sub.season_number == file.season
            }
            _ => false,
        }
    }
}

fn boxed<T: TMDBResult + 'static>(iter: impl Iterator<Item = T>) -> Vec<Box<dyn TMDBResult>> {
    iter.map(|x| Box::new(x) as Box<dyn TMDBResult>).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::filename::{EpisodeDetails, MovieDetails};

    #[test]
    fn it_matches_original_title() {
        assert_eq!(
            tmdb::SeriesSearchResult {
                id: TMDBId(1429),
                original_language: "ja".into(),
                original_name: "進撃の巨人".into(),
                overview: "".into(),
                name: "Attack on Titan".into(),
                vote_count: 7494,
            }
            .to_hit(&Filename::Episode(EpisodeDetails {
                series: "shingeki no kyojin".into(),
                season: 1,
                episode: 3
            })),
            Some(Hit::Episode(TMDBId(1429), 1, 3))
        );
    }

    #[test]
    fn it_matches_movie_subtitle() {
        assert_eq!(
            tmdb::MovieSearchResult {
                id: TMDBId(4539),
                original_language: "en".into(),
                original_title: "Hearts of Darkness: A Filmmaker's Apocalypse".into(),
                overview: "".into(),
                release_date: chrono::NaiveDate::parse_from_str("1991-11-27", "%Y-%m-%d").unwrap(),
                title: "Hearts of Darkness: A Filmmaker's Apocalypse".into(),
                vote_count: 419,
            }
            .to_hit(&Filename::Movie(MovieDetails {
                title: "hearts of darkness".into(),
                year: 1991
            })),
            Some(Hit::Movie(TMDBId(4539)))
        )
    }

    #[test]
    fn it_matches_movie_parens() {
        assert_eq!(
            tmdb::MovieSearchResult {
                id: TMDBId(33691),
                original_language: "en".into(),
                original_title: "Stereo (Tile 3B of a CAEE Educational Mosaic)".into(),
                overview: "".into(),
                release_date: chrono::NaiveDate::parse_from_str("1969-06-23", "%Y-%m-%d").unwrap(),
                title: "Stereo (Tile 3B of a CAEE Educational Mosaic)".into(),
                vote_count: 92,
            }
            .to_hit(&Filename::Movie(MovieDetails {
                title: "stereo".into(),
                year: 1969
            })),
            Some(Hit::Movie(TMDBId(33691)))
        )
    }
}
