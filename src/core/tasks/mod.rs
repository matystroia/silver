use std::{collections::VecDeque, sync::Arc};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
};
use tokio::sync::mpsc;

use crate::traits::{Task, TaskSequence, TaskState};

mod hash;
mod import_list;
mod scan;

pub use hash::HashTask;
pub use import_list::ImportListTask;
pub use scan::ScanTask;

enum Job {
    One(Arc<dyn Task>),
    Sequence(Vec<Arc<dyn Task>>),
}

pub struct Tasks {
    tasks: VecDeque<Arc<dyn Task>>,
    jobs: mpsc::UnboundedSender<Job>,
}

impl Tasks {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<Job>();
        tokio::spawn(async move {
            while let Some(job) = rx.recv().await {
                match job {
                    Job::One(t) => {
                        tokio::spawn(async move { t.run().await });
                    }
                    Job::Sequence(seq) => {
                        tokio::spawn(async move {
                            for t in seq {
                                t.run().await;
                            }
                        });
                    }
                }
            }
        });

        Self {
            tasks: VecDeque::new(),
            jobs: tx,
        }
    }

    pub fn render(&self, area: Rect, buffer: &mut Buffer) {
        let tasks: Vec<_> = self
            .tasks
            .iter()
            .filter(|task| matches!(*task.state().read().unwrap(), TaskState::Active))
            .collect();

        if !tasks.is_empty() {
            let layout = Layout::vertical(vec![Constraint::Length(3); tasks.len()]).flex(Flex::End);
            let areas = layout.split(area);

            for (&task, &area) in tasks.iter().zip(areas.iter()) {
                task.render(area, buffer);
            }
        }
    }

    pub fn push_task(&mut self, task: Arc<dyn Task>) {
        self.tasks.push_back(Arc::clone(&task));
        let _ = self.jobs.send(Job::One(task));
    }

    pub fn push_task_sequence(&mut self, seq: TaskSequence) {
        let tasks = seq.tasks();
        self.tasks.extend(tasks.iter().cloned());
        let _ = self.jobs.send(Job::Sequence(tasks));
    }

    pub fn active_len(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| *t.state().read().unwrap() == TaskState::Active)
            .count()
    }
}
