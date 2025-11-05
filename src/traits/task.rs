use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use color_eyre::eyre::Result;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Gauge, Widget},
};

use crate::core::Core;

#[async_trait]
pub trait Task: Send + Sync {
    async fn run(&self, core: Arc<Core>);
    fn progress(&self) -> f32;
    fn summary(&self) -> String;
    fn state(&self) -> &RwLock<TaskState>;
    fn render(&self, area: Rect, buf: &mut Buffer) {
        Gauge::default()
            .block(Block::bordered())
            .label(self.summary())
            .percent((self.progress() * 100f32).round() as u16)
            .render(area, buf);
    }
}

// impl<T: Task> Task for Arc<T> {
//     fn run(&self, core: &Core) {
//         self.as_ref().run(core);
//     }
//
//     fn progress(&self) -> f32 {
//         self.as_ref().progress()
//     }
//
//     fn summary(&self) -> String {
//         self.as_ref().summary()
//     }
//
//     fn state(&self) -> TaskState {
//         self.as_ref().state()
//     }
//
//     fn render(&self, area: Rect, buf: &mut Buffer) {
//         self.as_ref().render(area, buf);
//     }
// }

pub enum TaskState {
    Active,
    Done(Result<()>),
}
