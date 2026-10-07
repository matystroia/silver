use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use color_eyre::Result;
use futures::{
    FutureExt, StreamExt,
    future::{self, BoxFuture, Shared},
    stream,
};

use crate::{
    api::tmdb::{SeasonDetails, SeriesDetails, TMDBClient, TMDBId},
    core::{Core, Output, notify::Message},
    events::Event,
    model::{Feature, FileData, Series},
    parser::Detective,
    traits::{OkOrNotify, Task, TaskState},
};

type SeriesFetch = Shared<BoxFuture<'static, ()>>;

// TODO: Task list
pub struct ScanTask {
    core: Arc<Core>,
    num_files: AtomicU64,
    num_done: AtomicU64,
    last_file: RwLock<Option<FileData>>,
    series_fetches: Mutex<HashMap<TMDBId, SeriesFetch>>,
    state: RwLock<TaskState>,
}

impl ScanTask {
    pub fn new(core: Arc<Core>) -> Arc<Self> {
        Arc::new(Self {
            core,
            num_files: AtomicU64::new(0),
            num_done: AtomicU64::new(0),
            last_file: RwLock::new(None),
            series_fetches: Mutex::new(HashMap::new()),
            state: RwLock::new(TaskState::default()),
        })
    }

    async fn file_done(self: &Arc<Self>, file: &FileData, result: &Result<Option<Feature>>) {
        self.core
            .data()
            .set_scanned(&file.path, true)
            .await
            .ok_or_notify();

        match result {
            Ok(Some(feature)) => {
                match feature {
                    Feature::Movie(movie) => {
                        self.core
                            .data()
                            .add_movie(Arc::clone(movie))
                            .await
                            .ok_or_notify();
                    }
                    Feature::Episode(ep) => {
                        self.fetch_series(ep.series_id).await;
                    }
                }

                self.core
                    .data()
                    .add_feature(file, feature)
                    .await
                    .ok_or_notify();
            }
            Err(err) => {
                Output::notify(Message::error(err.to_string()));
            }
            _ => {}
        }
        self.num_done.fetch_add(1, Ordering::Relaxed);
        *self.last_file.write().unwrap() = Some(file.clone());
    }

    async fn done(&self) {
        *self.state.write().unwrap() = TaskState::Success;
        Event::Render.emit();
    }

    fn fetch_series(&self, series_id: TMDBId) -> SeriesFetch {
        self.series_fetches
            .lock()
            .unwrap()
            .entry(series_id)
            .or_insert_with(|| {
                let core = Arc::clone(&self.core);
                let tmdb = core.api.tmdb.clone().unwrap();
                async move {
                    if let Ok(series_details) = tmdb.series_details(series_id).await {
                        let series = Series::from_details(
                            series_details.clone(),
                            Self::fetch_series_seasons(&series_details, tmdb).await,
                        );
                        core.data().add_series(series).await.ok_or_notify();
                    }
                }
                .boxed()
                .shared()
            })
            .clone()
    }

    async fn fetch_series_seasons(
        details: &SeriesDetails,
        tmdb: Arc<TMDBClient>,
    ) -> Vec<SeasonDetails> {
        let handles = details.seasons.iter().map(|season| {
            let tmdb = Arc::clone(&tmdb);
            let series_id = details.id;
            let season_num = season.number;
            tokio::spawn(async move { tmdb.season_details(series_id, season_num).await })
        });

        future::join_all(handles)
            .await
            .into_iter()
            .filter_map(|res| res.ok().and_then(Result::ok))
            .collect()
    }
}

#[async_trait::async_trait]
impl Task for Arc<ScanTask> {
    async fn run(&self) {
        *self.state.write().unwrap() = TaskState::Active;

        let Some(files) = self.core.data().files().await.ok_or_notify() else {
            *self.state.write().unwrap() = TaskState::Error;
            return;
        };

        let files: Vec<_> = files.into_iter().filter(|f| !f.scanned).collect();
        self.num_files.store(files.len() as u64, Ordering::Relaxed);

        Output::notify(Message::info(format!(
            "Found {} new file{}",
            files.len(),
            if files.len() == 1 { "" } else { "s" }
        )));

        if self.core.api.tmdb.is_none() || self.core.api.ost.is_none() {
            Output::notify(Message::error("Need both TMDb and OpenSubtitles"));
            *self.state.write().unwrap() = TaskState::Error;
            return;
        }

        let detective = Arc::new(Detective::new(Arc::clone(&self.core)));

        let _results: Vec<_> = stream::iter(files)
            .map(|file| {
                let (task, detective) = (Arc::clone(self), Arc::clone(&detective));
                async move {
                    let result = detective.r#match(&file).await;
                    task.file_done(&file, &result).await;
                    Event::Render.emit();
                    result
                }
            })
            .buffer_unordered(16)
            .collect()
            .await;

        self.done().await;
    }

    fn progress(&self) -> f32 {
        match self.num_files.load(Ordering::Relaxed) {
            0 => 0.0,
            num_files => self.num_done.load(Ordering::Relaxed) as f32 / num_files as f32,
        }
    }

    fn summary(&self) -> String {
        format!(
            "Scanning ({}/{})",
            self.num_done.load(Ordering::Relaxed),
            self.num_files.load(Ordering::Relaxed),
        )
    }

    fn subtitle(&self) -> Option<String> {
        self.last_file
            .read()
            .unwrap()
            .as_ref()
            .map(|f| f.filename())?
    }

    fn state(&self) -> &RwLock<TaskState> { &self.state }
}
