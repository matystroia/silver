use std::sync::Arc;

use ratatui::{buffer::Buffer, layout::Rect};

pub trait Notification: Send + Sync {
    fn should_show(&self) -> bool;
    fn render(&self, area: Rect, buf: &mut Buffer);
}

impl<T: Notification> Notification for Arc<T> {
    fn should_show(&self) -> bool {
        self.as_ref().should_show()
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        self.as_ref().render(area, buf);
    }
}
