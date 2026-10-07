use crossterm::event::KeyEvent;
use ratatui::widgets::WidgetRef;

use crate::{traits::Modal, ui::common::Keymap};

#[async_trait::async_trait(?Send)]
pub trait Menu: WidgetRef {
    async fn on_key_event(&mut self, key: KeyEvent);
    fn keymaps(&self) -> Vec<Keymap>;
    fn has_modal(&self) -> bool;

    async fn dispatch_modal(
        key: crossterm::event::KeyEvent,
        modal: &mut Option<Box<dyn Modal>>,
    ) -> bool
    where
        Self: Sized,
    {
        if let Some(m) = modal {
            m.on_key_event(key).await;
            if m.should_close() {
                *modal = None;
            }
            return true;
        }
        false
    }
}

impl WidgetRef for &dyn Menu {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        (*self).render_ref(area, buf);
    }
}
