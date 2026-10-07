use ratatui::{
    layout::Spacing,
    prelude::*,
    symbols::merge::MergeStrategy,
    widgets::{Block, Padding, Paragraph, WidgetRef, Wrap},
};

pub struct ParagraphStack {
    paragraphs: Vec<Paragraph<'static>>,
}

impl ParagraphStack {
    pub fn new<T>(texts: Vec<T>) -> Self
    where
        Text<'static>: From<T>,
    {
        Self {
            paragraphs: texts
                .into_iter()
                .map(|t| Text::from(t))
                .filter(|t| t.width() > 0)
                .map(|t| Paragraph::new(t).wrap(Wrap { trim: false }))
                .collect(),
        }
    }
}

impl WidgetRef for ParagraphStack {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .padding(Padding::horizontal(1))
            .merge_borders(MergeStrategy::Exact);

        let paragraphs: Vec<_> = self
            .paragraphs
            .iter()
            .map(|p| p.clone().block(block.clone()))
            .collect();

        let inner_width = block.inner(area).width;
        let areas = Layout::vertical(
            paragraphs
                .iter()
                .map(|p| Constraint::Length(p.line_count(inner_width) as u16)),
        )
        .spacing(Spacing::Overlap(1))
        .split(area);

        for (paragraph, paragraph_area) in paragraphs.into_iter().zip(areas.iter()) {
            paragraph.render(*paragraph_area, buf);
        }
    }
}
