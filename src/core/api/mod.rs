use color_eyre::eyre::Result;

use crate::core::{
    api::{ost::OpenSubtitlesClient, tmdb::TMDBClient},
    config::Config,
    events::event::Event,
};

pub mod imdb;
pub mod ost;
pub mod tmdb;

#[derive(Default, Clone)]
pub struct Api {
    pub ost: Option<OpenSubtitlesClient>,
    pub tmdb: Option<TMDBClient>,
}

impl Api {
    pub fn init(&mut self, config: &Config) -> Result<()> {
        if let Some(ost_api_key) = &config.api.ost_api_key {
            self.ost = Some(OpenSubtitlesClient::new(ost_api_key)?);
            Event::Notify("Initialised OpenSubtitles API".into()).emit();
        }

        if let Some(tmdb_access_token) = &config.api.tmdb_access_token {
            self.tmdb = Some(TMDBClient::new(tmdb_access_token)?);
            Event::Notify("Initialised TMDB API".into()).emit();
        }

        Ok(())
    }
}
