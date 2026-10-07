use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Paragraph, Widget, Wrap},
};
use tokio::time::Instant;

use crate::{events::Event, traits::Notification};

const MESSAGE_DURATION_SEC: u64 = 3;

#[derive(Hash, PartialEq, Eq)]
enum MessageType {
    Info,
    Error,
}

#[derive(Hash, PartialEq, Eq)]
pub struct Message {
    r#type: MessageType,
    pub message: Line<'static>,
    instant: Instant,
}

impl Message {
    fn new(r#type: MessageType, msg: impl Into<Line<'static>>) -> Self {
        tokio::spawn(async {
            tokio::time::sleep(tokio::time::Duration::from_secs(MESSAGE_DURATION_SEC)).await;
            Event::Render.emit();
        });
        Self {
            r#type,
            message: msg.into(),
            instant: Instant::now(),
        }
    }

    pub fn info(msg: impl Into<Line<'static>>) -> Self {
        Self::new(MessageType::Info, msg)
    }

    pub fn error(msg: impl Into<Line<'static>>) -> Self {
        Self::new(MessageType::Error, msg)
    }
}

impl Notification for Message {
    fn should_show(&self) -> bool {
        self.instant.elapsed().as_secs() < MESSAGE_DURATION_SEC
    }

    fn render(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let text = self.message.clone();
        let text = match self.r#type {
            MessageType::Error => text.red(),
            _ => text,
        };

        Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
