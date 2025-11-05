use ratatui::{
    text::Line,
    widgets::{Paragraph, Widget, Wrap},
};
use tokio::time::Instant;

use crate::traits::notification::Notification;

#[derive(Hash, PartialEq, Eq)]
pub struct Message(String, Instant);

impl Notification for Message {
    fn should_show(&self) -> bool {
        self.1.elapsed().as_secs() < 3
    }

    fn render(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        Paragraph::new(self.0.clone())
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}

impl From<&Message> for Line<'_> {
    fn from(value: &Message) -> Self {
        Self::from(value.0.clone())
    }
}

impl From<&str> for Message {
    fn from(value: &str) -> Self {
        Message(value.to_string(), Instant::now())
    }
}

impl From<String> for Message {
    fn from(value: String) -> Self {
        Message(value.clone(), Instant::now())
    }
}
