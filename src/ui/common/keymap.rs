use crossterm::event::KeyCode;
use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Block, BorderType, Cell, Clear, Row, Table, Widget, WidgetRef},
};

pub struct Keymap {
    key: KeyCode,
    desc: &'static str,
}

impl Keymap {
    pub fn new(key: KeyCode, desc: &'static str) -> Self {
        Self { key, desc }
    }
}

pub struct KeymapHelp {
    keys: Vec<Keymap>,
}

impl KeymapHelp {
    pub fn new(keys: Vec<Keymap>) -> Self {
        Self { keys }
    }
}

impl WidgetRef for KeymapHelp {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        if self.keys.is_empty() {
            return;
        }

        let [table_area] = Layout::vertical([Constraint::Length(self.keys.len() as u16 + 2)])
            .flex(Flex::End)
            .areas(area);

        let table = Table::new(
            self.keys.iter().map(|key| {
                Row::new([
                    Cell::from(
                        Line::from(vec!["<".gray(), key.key.to_string().yellow(), ">".gray()])
                            .alignment(HorizontalAlignment::Center),
                    ),
                    Cell::from(key.desc),
                ])
            }),
            [
                Constraint::Length(
                    self.keys
                        .iter()
                        .map(|k| k.key.to_string().len())
                        .max()
                        .unwrap() as u16
                        + 2,
                ),
                Constraint::Fill(1),
            ],
        )
        .block(Block::bordered().border_type(BorderType::Rounded));

        Clear.render(table_area, buf);
        Widget::render(table, table_area, buf);
    }
}
