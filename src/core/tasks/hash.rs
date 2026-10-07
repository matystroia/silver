use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{
        Arc, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use color_eyre::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    core::Core,
    events::Event,
    model::FileData,
    traits::{OkOrNotify, Task, TaskState},
    util,
};

pub struct HashTask {
    core: Arc<Core>,
    num_files: AtomicU64,
    num_done: AtomicU64,
    last_file: RwLock<Option<PathBuf>>,
    state: RwLock<TaskState>,
}

impl HashTask {
    pub fn new(core: Arc<Core>) -> Arc<Self> {
        Arc::new(Self {
            core,
            num_files: Default::default(),
            num_done: Default::default(),
            last_file: Default::default(),
            state: RwLock::new(TaskState::default()),
        })
    }

    async fn hash_files(&self) -> Result<Vec<FileData>> {
        let directories = self.core.data().directories().await?;

        let cur_paths: HashSet<_> = self
            .core
            .data()
            .files()
            .await?
            .into_iter()
            .map(|f| f.path)
            .collect();

        let paths: Vec<PathBuf> = directories
            .iter()
            .flat_map(|dir| {
                walkdir::WalkDir::new(dir)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| e.file_type().is_file())
                    .map(|e| e.into_path())
                    .filter(|p| !cur_paths.contains(p) && util::is_video(p) && !util::is_sample(p))
            })
            .collect();

        self.num_files.store(paths.len() as u64, Ordering::Relaxed);
        Event::Render.emit();

        let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build()?;
        let files: Vec<_> = pool.install(|| {
            paths
                .par_iter()
                .filter_map(|path| {
                    let file = FileData::new(path).ok();
                    self.num_done.fetch_add(1, Ordering::Relaxed);
                    Event::Render.emit();
                    file
                })
                .collect()
        });

        Ok(files)
    }
}

impl HashTask {}

#[async_trait::async_trait]
impl Task for Arc<HashTask> {
    async fn run(&self) {
        *self.state.write().unwrap() = TaskState::Active;
        let Ok(files) = self.hash_files().await else {
            *self.state.write().unwrap() = TaskState::Error;
            return;
        };

        for file in files {
            // TODO: bulk
            self.core.data().add_file(&file).await.ok_or_notify();
        }

        *self.state.write().unwrap() = TaskState::Success;
        Event::Render.emit();
    }

    fn progress(&self) -> f32 {
        match self.num_files.load(Ordering::Relaxed) {
            0 => 0.0,
            num_files => self.num_done.load(Ordering::Relaxed) as f32 / num_files as f32,
        }
    }

    fn summary(&self) -> String {
        format!(
            "Hashing ({}/{})",
            self.num_done.load(Ordering::Relaxed),
            self.num_files.load(Ordering::Relaxed)
        )
    }

    fn subtitle(&self) -> Option<String> {
        self.last_file
            .read()
            .unwrap()
            .as_ref()
            .map(|f| f.to_string_lossy().into())
    }

    fn state(&self) -> &RwLock<TaskState> {
        &self.state
    }
}
