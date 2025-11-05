use std::collections::VecDeque;

use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
};

use crate::traits::task::{Task, TaskState};

pub mod scan;

#[derive(Default)]
pub struct Tasks {
    tasks: VecDeque<Box<dyn Task>>,
}

impl Tasks {
    pub fn render(&self, frame: &mut Frame) {
        let tasks: Vec<_> = self
            .tasks
            .iter()
            .filter(|task| matches!(*task.state().read().unwrap(), TaskState::Done(_)))
            .collect();

        if !tasks.is_empty() {
            let layout = Layout::vertical(vec![Constraint::Length(3); tasks.len()]).flex(Flex::End);
            let areas = layout.split(frame.area());

            for (&task, &area) in tasks.iter().zip(areas.iter()) {
                task.render(area, frame.buffer_mut());
            }
        }
    }

    pub fn push_task(&mut self, task: impl Task + 'static) {
        self.tasks.push_back(Box::new(task));
    }
}
