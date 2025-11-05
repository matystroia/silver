use crossterm::event::KeyEvent;
use ratatui::widgets::WidgetRef;

pub trait Close {
    fn should_close(&self) -> bool;
}

pub trait Modal: WidgetRef + Close {
    fn on_key_event(&mut self, key: KeyEvent);
}
