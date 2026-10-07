use std::sync::{
    Arc, RwLock,
    atomic::{AtomicU64, Ordering},
};

use futures::{StreamExt, stream};

use crate::{
    core::Core,
    events::Event,
    traits::{OkOrNotify, Task, TaskState},
};

pub struct ImportListTask {
    core: Arc<Core>,
    username: String,
    list: String,
    num_movies: AtomicU64,
    num_done: AtomicU64,
    state: RwLock<TaskState>,
}

impl ImportListTask {
    pub fn new(core: Arc<Core>, username: String, list: String) -> Arc<Self> {
        Arc::new(Self {
            core,
            username,
            list,
            num_movies: AtomicU64::new(0),
            num_done: AtomicU64::new(0),
            state: RwLock::new(TaskState::default()),
        })
    }
}

#[async_trait::async_trait]
impl Task for Arc<ImportListTask> {
    async fn run(&self) {
        *self.state.write().unwrap() = TaskState::Active;

        let (Some(letterboxd), Some(tmdb)) = (
            self.core.api.letterboxd.as_ref(),
            self.core.api.tmdb.as_ref(),
        ) else {
            *self.state.write().unwrap() = TaskState::Error;
            return;
        };

        let Some(movies) = letterboxd
            .movie_list(&self.username, &self.list)
            .await
            .ok_or_notify()
        else {
            *self.state.write().unwrap() = TaskState::Error;
            return;
        };

        self.num_movies
            .store(movies.len() as u64, Ordering::Relaxed);

        stream::iter(movies)
            .for_each_concurrent(16, |(title, year)| {
                let core = Arc::clone(&self.core);
                async move {
                    let results = tmdb
                        .search_movie(&[("query", title), ("year", year.to_string())])
                        .await;

                    if let Some([result]) = results.ok_or_notify().as_deref()
                        && let Some(movie) = tmdb.movie_details(result.id).await.ok_or_notify()
                    {
                        core.data().add_movie(Arc::new(movie)).await.ok_or_notify();
                    }

                    self.num_done.fetch_add(1, Ordering::Relaxed);
                    Event::Render.emit();
                }
            })
            .await;

        *self.state.write().unwrap() = TaskState::Success;
        Event::Render.emit();
    }

    fn progress(&self) -> f32 {
        match self.num_movies.load(Ordering::Relaxed) {
            0 => 0.0,
            num_movies => self.num_done.load(Ordering::Relaxed) as f32 / num_movies as f32,
        }
    }

    fn summary(&self) -> String {
        format!(
            "Adding ({}/{})",
            self.num_done.load(Ordering::Relaxed),
            self.num_movies.load(Ordering::Relaxed)
        )
    }

    fn subtitle(&self) -> Option<String> { None }

    fn state(&self) -> &RwLock<TaskState> { &self.state }
}
