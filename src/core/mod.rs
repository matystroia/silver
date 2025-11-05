use std::sync::Arc;
use std::sync::RwLock;

use color_eyre::Result;
use futures::future;

use crate::core::tasks::scan::ScanTask;
use crate::core::{
    api::Api, config::Config, data::Data, events::Events, notify::Notify, tasks::Tasks,
};
use crate::model::feature::Feature;
use crate::parser::detective::Detective;
use crate::traits::task::Task;

pub mod api;
pub mod config;
pub mod data;
pub mod events;
pub mod log;
pub mod manager;
pub mod notify;
pub mod tasks;

pub struct Core {
    pub data: RwLock<Data>,
    pub config: Config,
    pub events: Events,
    pub tasks: RwLock<Tasks>,
    pub notify: RwLock<Notify>,
    pub api: Api,
}

impl Core {
    pub fn make() -> Self {
        let config = Config::new().unwrap_or_default();

        let mut api = Api::default();
        api.init(&config);

        Self {
            data: RwLock::new(Data::default()),
            events: Events::default(),
            tasks: RwLock::new(Tasks::default()),
            notify: RwLock::new(Notify::default()),
            config,
            api,
        }
    }
}

pub trait CoreTasks {
    fn scan_directories(&self);
}

impl CoreTasks for Arc<Core> {
    fn scan_directories(&self) {
        let core = self.clone();

        let task = ScanTask::new(core.clone());
        self.tasks.write().unwrap().push_task(task.clone());

        tokio::spawn(async move { task.run(core).await });
    }
}
