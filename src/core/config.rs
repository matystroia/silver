use std::{fs, path::Path, sync::LazyLock};

use color_eyre::eyre::{Result, eyre};
use directories_next::ProjectDirs;
use toml::Table;

const CONFIG_FILE: &str = "config.toml";

static PROJECT_DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    ProjectDirs::from("com", "matystroia", "silver").expect("should find project dirs")
});

#[derive(Default, Debug)]
pub struct Config {
    pub api: ApiConfig,
}

#[derive(Default, Debug)]
pub struct ApiConfig {
    pub ost_api_key: Option<String>,
    pub tmdb_access_token: Option<String>,
}

impl Config {
    pub fn new() -> Result<Self> {
        let project_dirs = &PROJECT_DIRS;
        let config_dir = project_dirs.config_dir();

        if !config_dir.try_exists()? {
            return Err(eyre!("Path doesn't exist: {config_dir:?}"));
        }

        let config_path = config_dir.join(CONFIG_FILE);
        if !config_path.try_exists()? {
            return Err(eyre!("Path doesn't exist: {config_path:?}"));
        }

        let config_str = fs::read_to_string(config_path)?;
        let config_table: Table = config_str.parse()?;

        let mut config = Config::default();
        if let Some(api_table) = config_table["api"].as_table() {
            config.api.ost_api_key = api_table["ost_api_key"].as_str().map(str::to_string);
            config.api.tmdb_access_token =
                api_table["tmdb_access_token"].as_str().map(str::to_string)
        }

        Ok(config)
    }

    pub fn data_dir() -> &'static Path {
        PROJECT_DIRS.data_dir()
    }
}
