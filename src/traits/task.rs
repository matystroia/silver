use std::sync::{Arc, RwLock};

use ratatui::{
    buffer::Buffer,
    layout::{HorizontalAlignment, Rect},
    widgets::{Block, BorderType, Gauge, Widget},
};

#[async_trait::async_trait]
pub trait Task: Send + Sync {
    async fn run(&self);

    fn progress(&self) -> f32;

    fn summary(&self) -> String;

    fn subtitle(&self) -> Option<String>;

    fn state(&self) -> &RwLock<TaskState>;

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().border_type(BorderType::Rounded);
        let block = match self.subtitle() {
            Some(subtitle) => block
                .title_bottom(subtitle)
                .title_alignment(HorizontalAlignment::Center),
            _ => block,
        };
        Gauge::default()
            .block(block)
            .label(self.summary())
            .percent((self.progress() * 100f32).round() as u16)
            .render(area, buf);
    }
}

#[derive(PartialEq, Eq, Default)]
pub enum TaskState {
    #[default]
    Pending,
    Active,
    Success,
    Error,
}

#[derive(Default)]
pub struct TaskSequence {
    tasks: Vec<Arc<dyn Task>>,
}

impl TaskSequence {
    pub fn task(mut self, task: impl Task + 'static) -> Self {
        self.tasks.push(Arc::new(task));
        self
    }

    pub fn tasks(&self) -> Vec<Arc<dyn Task>> {
        self.tasks.iter().map(Arc::clone).collect()
    }
}
