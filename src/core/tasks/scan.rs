use std::sync::{
    Arc, RwLock,
    atomic::{AtomicU64, Ordering},
};

use async_trait::async_trait;
use color_eyre::Result;
use futures::future;

use crate::{
    core::Core,
    model::{feature::Feature, file::FileData},
    parser::detective::Detective,
    traits::task::{Task, TaskState},
};

pub struct ScanTask {
    core: Arc<Core>,
    files: Vec<FileData>,
    num_done: AtomicU64,
    state: RwLock<TaskState>,
}

impl ScanTask {
    pub fn new(core: Arc<Core>) -> Arc<Self> {
        Arc::new(Self {
            core: core.clone(),
            files: core.data.read().unwrap().scan(),
            num_done: AtomicU64::new(0),
            state: RwLock::new(TaskState::Active),
        })
    }

    pub fn file_done(&self, file: &FileData, result: Result<Option<Feature>>) {
        if let Ok(Some(feature)) = result {
            self.core
                .data
                .write()
                .unwrap()
                .add_feature(file.clone(), feature);
        }
        self.num_done.fetch_add(1, Ordering::Relaxed);
    }

    pub fn done(&self) {
        self.core.data.read().unwrap().library.save();
        *self.state.write().unwrap() = TaskState::Done(Ok(()));
    }
}

#[async_trait]
impl Task for Arc<ScanTask> {
    async fn run(&self, core: Arc<Core>) {
        let detective = Arc::new(Detective::new(&core.api));

        let handles = self.files.iter().cloned().map(|file| {
            let (task, detective) = (self.clone(), detective.clone());
            tokio::spawn(async move {
                let result = detective.r#match(&file).await;
                task.file_done(&file, result);
            })
        });

        future::join_all(handles).await;
        self.done();
    }

    fn progress(&self) -> f32 {
        self.num_done.load(Ordering::Relaxed) as f32 / self.files.len() as f32
    }

    fn summary(&self) -> String {
        format!(
            "Scanning ({}/{})",
            self.num_done.load(Ordering::Relaxed),
            self.files.len()
        )
    }

    fn state(&self) -> &RwLock<TaskState> {
        &self.state
    }
}
