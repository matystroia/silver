use crossterm::event::KeyEvent;
use ratatui::widgets::WidgetRef;

#[async_trait::async_trait(?Send)]
pub trait Modal: WidgetRef {
    async fn on_key_event(&mut self, key: KeyEvent);
    fn should_close(&self) -> bool;
}
