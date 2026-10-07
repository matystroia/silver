use std::sync::{Arc, RwLock};

use api::Api;
use data::Data;
use events::Events;
use notify::Notify;

use crate::{
    core::tasks::{HashTask, ScanTask, Tasks},
    traits::{OkOrNotify, TaskSequence},
};

pub mod api;
mod config;
pub mod data;
pub mod events;
pub mod log;
pub mod notify;
mod output;
pub mod tasks;

pub use config::Config;
pub use output::Output;

pub struct Core {
    data: Data,
    #[allow(dead_code)]
    config: Config,
    #[allow(dead_code)]
    events: Events,
    pub tasks: RwLock<Tasks>,
    pub notify: RwLock<Notify>,
    pub api: Api,
}

impl Core {
    pub async fn make() -> Self {
        let config = Config::new().unwrap_or_default();

        let mut api = Api::default();
        api.init(&config).ok_or_notify();

        Self {
            data: Data::new().await.unwrap(),
            events: Events::default(),
            tasks: RwLock::new(Tasks::new()),
            notify: RwLock::new(Notify::default()),
            config,
            api,
        }
    }

    pub fn data(&self) -> &Data {
        &self.data
    }

    #[allow(dead_code)]
    pub fn config(&self) -> &Config {
        &self.config
    }

    #[allow(dead_code)]
    pub fn events(&self) -> &Events {
        &self.events
    }
}

#[async_trait::async_trait]
pub trait CoreTasks {
    fn hash_files(&self);
    fn scan_directories(&self);
}

#[async_trait::async_trait]
impl CoreTasks for Arc<Core> {
    fn hash_files(&self) {
        let task = HashTask::new(Arc::clone(self));
        self.tasks.write().unwrap().push_task(Arc::new(task));
    }

    fn scan_directories(&self) {
        let seq = TaskSequence::default()
            .task(HashTask::new(Arc::clone(self)))
            .task(ScanTask::new(Arc::clone(self)));
        self.tasks.write().unwrap().push_task_sequence(seq);
    }
}
