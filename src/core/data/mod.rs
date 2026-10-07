use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use chrono::{Datelike, NaiveDate};
use color_eyre::eyre::{Result, bail};
use image::DynamicImage;
use ratatui::{style::Stylize, text::Line};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use tokio::sync::watch;

use crate::{
    api::TMDBId,
    core::{Config, notify::Message, output::Output},
    model::{Episode, Feature, FileData, Genre, LocalEpisode, LocalMovie, Movie, Season, Series},
    util::extract_frame,
};

mod directory;
pub use directory::Directory;

pub struct Data {
    pool: SqlitePool,
    pub directories_tx: watch::Sender<Vec<Directory>>,
    pub movies_tx: watch::Sender<Vec<Arc<LocalMovie>>>,
    pub series_tx: watch::Sender<Vec<Arc<Series<LocalEpisode>>>>,
}

impl Data {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            pool: Self::connect().await?,
            directories_tx: watch::channel(Default::default()).0,
            movies_tx: watch::channel(Default::default()).0,
            series_tx: watch::channel(Default::default()).0,
        })
    }
}

impl Data {
    pub async fn directories(&self) -> Result<Vec<Directory>> {
        let directories = sqlx::query!("SELECT path FROM directories")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| Directory(PathBuf::from(row.path)))
            .collect();
        Ok(directories)
    }

    pub async fn add_directory(&self, path: PathBuf) -> Result<()> {
        if path.is_empty() {
            bail!("Empty path")
        }
        if !path.try_exists()? {
            bail!("Invalid path: {path:?}")
        }

        let path = match path.is_absolute() {
            true => path,
            false => path.absolute()?,
        };

        let result = sqlx::query!(
            "INSERT INTO directories (path) values (?)",
            &path.to_string_lossy()
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            let directories = self.directories().await?;
            self.directories_tx.send_replace(directories);
        }

        Ok(())
    }

    pub async fn remove_directory(&self, path: PathBuf) -> Result<()> {
        let result = sqlx::query!(
            "DELETE FROM directories WHERE path = ?",
            path.to_string_lossy()
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            let directories = self.directories().await?;
            self.directories_tx.send_replace(directories);
        }

        Ok(())
    }
}

impl Data {
    #[allow(dead_code)]
    pub async fn features(&self) -> Result<Vec<SqliteFeature>> {
        let features = sqlx::query_as!(
            SqliteFeature,
            r#"SELECT 
                path,
                movie_id as "movie_id: TMDBId",
                episode_id as "episode_id: TMDBId"
            FROM features"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(features)
    }

    #[allow(dead_code)]
    pub async fn paths_by_id(&self, id: &TMDBId) -> Result<Vec<PathBuf>> {
        let paths: Vec<_> = sqlx::query!(
            "SELECT path FROM features WHERE movie_id = ? OR episode_id = ?",
            id,
            id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| PathBuf::from(&row.path))
        .collect();
        Ok(paths)
    }

    pub async fn add_feature(&self, file: &FileData, feature: &Feature) -> Result<()> {
        match feature {
            Feature::Movie(movie) => {
                sqlx::query!(
                    "INSERT INTO features (path, movie_id) VALUES (?, ?)",
                    file.path.to_string_lossy(),
                    movie.id
                )
                .execute(&self.pool)
                .await?
            }
            Feature::Episode(episode) => {
                sqlx::query!(
                    "INSERT INTO features (path, episode_id) VALUES (?, ?)",
                    file.path.to_string_lossy(),
                    episode.id
                )
                .execute(&self.pool)
                .await?
            }
        };
        Ok(())
    }

    pub async fn clear_features(&self) -> Result<()> {
        sqlx::query!("DELETE FROM movies; DELETE FROM series; DELETE FROM files")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

impl Data {
    pub async fn files(&self) -> Result<Vec<FileData>> {
        let files: Vec<_> = sqlx::query_as!(
            FileData,
            r#"SELECT path, hash, scanned AS "scanned: bool" FROM files"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(files)
    }

    pub async fn add_file(&self, file: &FileData) -> Result<()> {
        sqlx::query!(
            "INSERT INTO files (path, hash)
            VALUES (?, ?)",
            file.path.to_string_lossy(),
            file.hash
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn set_scanned(&self, path: &Path, scanned: bool) -> Result<()> {
        sqlx::query!(
            "UPDATE files SET scanned = ? WHERE path = ?",
            scanned,
            &path.to_string_lossy()
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

impl Data {
    pub async fn series(&self) -> Result<Vec<Arc<Series<LocalEpisode>>>> {
        let mut paths: HashMap<TMDBId, Vec<PathBuf>> = HashMap::new();
        for row in sqlx::query!(
            r#"SELECT path, episode_id as "episode_id!: TMDBId" FROM features WHERE episode_id IS NOT NULL"#
        )
        .fetch_all(&self.pool)
        .await?
        {
            paths.entry(row.episode_id).or_default().push(PathBuf::from(&row.path))
        }

        let mut episodes_by_season: HashMap<(TMDBId, i64), Vec<LocalEpisode>> = HashMap::new();
        for row in sqlx::query!("SELECT * FROM episodes ORDER BY series_id, episode_number")
            .fetch_all(&self.pool)
            .await?
        {
            episodes_by_season
                .entry((TMDBId(row.series_id), row.season_number))
                .or_default()
                .push(LocalEpisode {
                    episode: Episode {
                        id: TMDBId(row.id),
                        series_id: TMDBId(row.series_id),
                        season_number: row.season_number,
                        episode_number: row.episode_number,
                        title: row.title,
                        overview: row.overview,
                        release_date: NaiveDate::from_str(&row.release_date)?,
                    },
                    paths: paths.remove(&TMDBId(row.id)).unwrap_or_default(),
                });
        }

        let mut seasons_by_series: HashMap<TMDBId, Vec<Season<LocalEpisode>>> = HashMap::new();
        for row in sqlx::query!("SELECT * FROM seasons ORDER BY series_id, id")
            .fetch_all(&self.pool)
            .await?
        {
            seasons_by_series
                .entry(TMDBId(row.series_id))
                .or_default()
                .push(Season::<LocalEpisode> {
                    id: TMDBId(row.id),
                    name: row.name,
                    number: row.number,
                    episodes: episodes_by_season
                        .remove(&(TMDBId(row.series_id), row.number))
                        .unwrap_or_default(),
                });
        }

        let mut genres: HashMap<TMDBId, Vec<Genre>> = HashMap::new();
        for row in sqlx::query!(
            r#"SELECT series_id as "series_id: TMDBId", genre as "genre: Genre" FROM series_genres"#
        )
        .fetch_all(&self.pool)
        .await?
        {
            genres.entry(row.series_id).or_default().push(row.genre);
        }

        let series: Vec<_> = sqlx::query!("SELECT * FROM series ORDER BY id")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| {
                Arc::new(Series::<LocalEpisode> {
                    id: TMDBId(row.id),
                    name: row.name,
                    original_name: row.original_name,
                    original_language: row.original_language,
                    tagline: row.tagline,
                    overview: row.overview,
                    genres: genres.remove(&TMDBId(row.id)).unwrap_or_default(),
                    seasons: seasons_by_series
                        .remove(&TMDBId(row.id))
                        .unwrap_or_default(),
                })
            })
            .collect();

        Ok(series)
    }

    pub async fn get_series(&self, id: TMDBId) -> Result<Arc<Series<LocalEpisode>>> {
        let mut paths: HashMap<TMDBId, Vec<PathBuf>> = HashMap::new();
        for row in sqlx::query!(
            r#"SELECT 
                path,
                episode_id as "episode_id!: TMDBId"
            FROM features
            JOIN episodes e ON e.id = episode_id
            WHERE e.series_id = ?"#,
            id
        )
        .fetch_all(&self.pool)
        .await?
        {
            paths
                .entry(row.episode_id)
                .or_default()
                .push(PathBuf::from(&row.path))
        }

        let mut episodes_by_season: HashMap<(TMDBId, i64), Vec<LocalEpisode>> = HashMap::new();
        for row in sqlx::query!(
            "SELECT * FROM episodes WHERE series_id = ? ORDER BY episode_number",
            id
        )
        .fetch_all(&self.pool)
        .await?
        {
            episodes_by_season
                .entry((TMDBId(row.series_id), row.season_number))
                .or_default()
                .push(LocalEpisode {
                    episode: Episode {
                        id: TMDBId(row.id),
                        series_id: TMDBId(row.series_id),
                        season_number: row.season_number,
                        episode_number: row.episode_number,
                        title: row.title,
                        overview: row.overview,
                        release_date: NaiveDate::from_str(&row.release_date)?,
                    },
                    paths: paths.remove(&TMDBId(row.id)).unwrap_or_default(),
                });
        }

        let mut seasons_by_series: HashMap<TMDBId, Vec<Season<LocalEpisode>>> = HashMap::new();
        for row in sqlx::query!(
            "SELECT * FROM seasons WHERE series_id = ? ORDER BY number",
            id
        )
        .fetch_all(&self.pool)
        .await?
        {
            seasons_by_series
                .entry(TMDBId(row.series_id))
                .or_default()
                .push(Season::<LocalEpisode> {
                    id: TMDBId(row.id),
                    name: row.name,
                    number: row.number,
                    episodes: episodes_by_season
                        .remove(&(TMDBId(row.series_id), row.number))
                        .unwrap_or_default(),
                });
        }

        let mut genres: HashMap<TMDBId, Vec<Genre>> = HashMap::new();
        for row in sqlx::query!(
            r#"SELECT series_id as "series_id: TMDBId", genre as "genre: Genre" FROM series_genres"#
        )
        .fetch_all(&self.pool)
        .await?
        {
            genres.entry(row.series_id).or_default().push(row.genre);
        }

        let series = sqlx::query!("SELECT * FROM series WHERE id = ?", id)
            .fetch_one(&self.pool)
            .await
            .map(|row| {
                Arc::new(Series::<LocalEpisode> {
                    id: TMDBId(row.id),
                    name: row.name,
                    original_name: row.original_name,
                    original_language: row.original_language,
                    tagline: row.tagline,
                    overview: row.overview,
                    genres: genres.remove(&TMDBId(row.id)).unwrap_or_default(),
                    seasons: seasons_by_series
                        .remove(&TMDBId(row.id))
                        .unwrap_or_default(),
                })
            })?;

        Ok(series)
    }

    pub async fn add_series(&self, series: Series<Episode>) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "INSERT INTO series (id, name, original_name, original_language, tagline, overview) \
             VALUES (?, ?, ?, ?, ?, ?)",
            series.id,
            &series.name,
            &series.original_name,
            &series.original_language,
            &series.tagline,
            &series.overview
        )
        .execute(&mut *tx)
        .await?;

        for genre in &series.genres {
            sqlx::query!("INSERT OR IGNORE INTO genres (genre) VALUES (?)", genre)
                .execute(&mut *tx)
                .await?;
            sqlx::query!(
                "INSERT OR IGNORE INTO series_genres (series_id, genre) VALUES (?, ?)",
                series.id,
                genre,
            )
            .execute(&mut *tx)
            .await?;
        }

        for season in series.seasons() {
            sqlx::query!(
                "INSERT INTO seasons (id, series_id, number, name) VALUES (?, ?, ?, ?)",
                season.id,
                series.id,
                season.number,
                &season.name
            )
            .execute(&mut *tx)
            .await?;

            for episode in &season.episodes {
                sqlx::query!(
                    "INSERT INTO episodes (id, series_id, title, season_number, episode_number, \
                     overview, release_date) VALUES (?, ?, ?, ?, ?, ?, ?)",
                    episode.id,
                    episode.series_id,
                    &episode.title,
                    episode.season_number,
                    episode.episode_number,
                    &episode.overview,
                    &episode.release_date,
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        let msg = Line::from(vec![
            "+".green(),
            " ".into(),
            series.name.clone().into(),
            " ".into(),
            format!("({} seasons)", series.seasons().len()).yellow(),
        ]);
        Output::notify(Message::info(msg));

        let series = self.series().await?;
        self.series_tx.send_replace(series);

        Ok(())
    }
}

impl Data {
    pub async fn movies(&self) -> Result<Vec<Arc<LocalMovie>>> {
        let mut genres: HashMap<TMDBId, Vec<Genre>> = HashMap::new();
        for row in sqlx::query!(
            r#"SELECT movie_id as "movie_id: TMDBId", genre as "genre: Genre" FROM movie_genres"#
        )
        .fetch_all(&self.pool)
        .await?
        {
            genres.entry(row.movie_id).or_default().push(row.genre);
        }

        let mut paths: HashMap<TMDBId, Vec<PathBuf>> = HashMap::new();
        for row in sqlx::query!(
            r#"SELECT path, movie_id as "movie_id!: TMDBId" FROM features WHERE movie_id IS NOT NULL"#
        )
        .fetch_all(&self.pool)
        .await?
        {
            paths.entry(row.movie_id).or_default().push(PathBuf::from(&row.path))
        }

        let movies = sqlx::query!(r#"SELECT * FROM movies"#)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .filter_map(|row| {
                let id = TMDBId(row.id);
                Some(Arc::new(LocalMovie {
                    movie: Movie {
                        id,
                        title: row.title,
                        original_title: row.original_title,
                        original_language: row.original_language,
                        release_date: NaiveDate::from_str(&row.release_date).ok()?,
                        tagline: row.tagline,
                        overview: row.overview,
                        genres: genres.remove(&id).unwrap_or_default(),
                    },
                    paths: paths.remove(&id).unwrap_or_default(),
                }))
            })
            .collect();

        Ok(movies)
    }

    pub async fn add_movie(&self, movie: Arc<Movie>) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        let result = sqlx::query!(
            "INSERT OR IGNORE INTO movies 
                (id, title, original_title, original_language, release_date, tagline, overview) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            movie.id,
            &movie.title,
            &movie.original_title,
            &movie.original_language,
            movie.release_date,
            &movie.tagline,
            &movie.overview,
        )
        .execute(&mut *tx)
        .await?;

        // Movie already in the library; nothing to do.
        if result.rows_affected() == 0 {
            return Ok(());
        }

        for genre in &movie.genres {
            sqlx::query!("INSERT OR IGNORE INTO genres (genre) VALUES (?)", genre)
                .execute(&mut *tx)
                .await?;
            sqlx::query!(
                "INSERT OR IGNORE INTO movie_genres (movie_id, genre) VALUES (?, ?)",
                movie.id,
                genre,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        let msg = Line::from(vec![
            "+".green(),
            " ".into(),
            movie.title.clone().into(),
            " ".into(),
            movie.release_date.year().yellow(),
        ]);
        Output::notify(Message::info(msg));

        let movies = self.movies().await?;
        self.movies_tx.send_replace(movies);

        Ok(())
    }

    async fn connect() -> Result<SqlitePool> {
        let data_dir = Config::data_dir();
        std::fs::create_dir_all(data_dir)?;
        let db_path = data_dir.join("library.db");

        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))?
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .busy_timeout(std::time::Duration::from_secs(5));

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(pool)
    }
}

impl Data {
    pub fn get_poster(id: &TMDBId) -> Option<DynamicImage> {
        let poster_path = Config::data_dir().join("posters").join(format!("{id}.jpg"));
        image::ImageReader::open(poster_path).ok()?.decode().ok()
    }

    pub fn get_thumb(path: &Path) -> Option<DynamicImage> {
        if !std::fs::exists(path).unwrap_or(false) {
            return None;
        }
        extract_frame(path, 0.5).ok().map(|img| img.into())
    }
}

pub struct SqliteFeature {
    pub path: PathBuf,
    pub movie_id: Option<TMDBId>,
    pub episode_id: Option<TMDBId>,
}
