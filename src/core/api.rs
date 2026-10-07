use std::sync::Arc;

use crate::{
    api::{LetterboxdClient, OpenSubtitlesClient, TMDBClient},
    core::Config,
};

#[derive(Default, Clone)]
pub struct Api {
    pub ost: Option<Arc<OpenSubtitlesClient>>,
    pub tmdb: Option<Arc<TMDBClient>>,
    pub letterboxd: Option<Arc<LetterboxdClient>>,
}

impl Api {
    pub fn init(&mut self, config: &Config) -> color_eyre::Result<()> {
        if let Some(ost_api_key) = &config.api.ost_api_key {
            self.ost = Some(Arc::new(OpenSubtitlesClient::new(ost_api_key)?));
        }

        if let Some(tmdb_access_token) = &config.api.tmdb_access_token {
            self.tmdb = Some(Arc::new(TMDBClient::new(tmdb_access_token)?));
        }

        self.letterboxd = Some(Arc::new(LetterboxdClient::new()?));

        Ok(())
    }
}
