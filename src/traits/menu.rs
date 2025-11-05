use crossterm::event::KeyEvent;
use ratatui::widgets::WidgetRef;

use crate::core::Core;

pub trait Menu: WidgetRef {
    fn new(core: &Core) -> Self
    where
        Self: Sized;
    fn on_key_event(&mut self, key: KeyEvent);
}

impl WidgetRef for &dyn Menu {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        (*self).render_ref(area, buf);
    }
}
