use std::collections::VecDeque;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Margin, Rect},
    widgets::{Block, Clear, Widget},
};

use crate::traits::Notification;

mod message;

pub use message::Message;

#[derive(Default)]
pub struct Notify {
    notifications: VecDeque<Box<dyn Notification>>,
}

impl Notify {
    pub fn render(&self, area: Rect, buffer: &mut Buffer) {
        let notifications: Vec<_> = self
            .notifications
            .iter()
            .filter(|notif| notif.should_show())
            .collect();

        if !notifications.is_empty() {
            let [area] = Layout::horizontal([Constraint::Fill(1)])
                .flex(Flex::End)
                .areas(area);

            let layout =
                Layout::vertical(vec![Constraint::Length(3); notifications.len()]).flex(Flex::End);

            let areas = layout.split(area);

            for (&notif, &area) in notifications.iter().zip(areas.iter()) {
                Clear.render(area, buffer);
                Block::bordered().render(area, buffer);
                notif.render(area.inner(Margin::new(1, 1)), buffer);
            }
        }
    }

    pub fn push_notification(&mut self, notif: impl Notification + 'static) {
        self.notifications.push_back(Box::new(notif));
    }
}
