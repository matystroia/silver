use std::collections::VecDeque;

use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Margin},
    widgets::{Block, Widget},
};

use crate::traits::notification::Notification;

pub mod message;

#[derive(Default)]
pub struct Notify {
    notifications: VecDeque<Box<dyn Notification>>,
}

impl Notify {
    pub fn render(&self, frame: &mut Frame) {
        let notifications: Vec<_> = self
            .notifications
            .iter()
            .filter(|notif| notif.should_show())
            .collect();

        if !notifications.is_empty() {
            let [area] = Layout::horizontal([Constraint::Length(40)])
                .flex(Flex::End)
                .areas(frame.area());

            let layout =
                Layout::vertical(vec![Constraint::Length(3); notifications.len()]).flex(Flex::End);

            let areas = layout.split(area);

            for (&notif, &area) in notifications.iter().zip(areas.iter()) {
                Block::bordered().render(area, frame.buffer_mut());
                notif.render(area.inner(Margin::new(1, 1)), frame.buffer_mut());
            }
        }
    }

    pub fn push_notification(&mut self, notif: impl Notification + 'static) {
        self.notifications.push_back(Box::new(notif));
    }
}
