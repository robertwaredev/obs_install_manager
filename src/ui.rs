use crate::app::Item;
use ratatui::prelude::*;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, palette::tailwind},
    symbols::border,
    text::Line,
    widgets::*,
};

pub const HIGHLIGHT_STYLE: Style = Style::new()
    .bg(tailwind::SLATE.c800)
    .add_modifier(Modifier::BOLD);

#[derive(Default)]
pub struct StatefulList<'a> {
    pub items: Vec<Item>,
    pub state: ListState,
    pub header: Line<'a>,
    pub footer: Line<'a>,
}

impl<'a> StatefulList<'a> {
    pub fn width(&self, area: Rect) -> u16 {
        let width = self
            .items
            .iter()
            .map(|s| s.as_str().len())
            .max()
            .unwrap_or(0);
        // +4 to account for padding and borders
        let width = width.max(self.header.width()).max(self.footer.width()) + 4;
        area.width.min(width as u16)
    }

    pub fn height(&self, area: Rect) -> u16 {
        // +4 to account for padding and borders
        let height = self.items.len() + 4;
        area.height.min(height as u16)
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title_top(self.header.clone())
            .title_bottom(self.footer.clone())
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .padding(Padding::uniform(1));

        let items: Vec<&'static str> = self.items.iter().map(|i| i.as_str()).collect();

        let list = List::new(items)
            .block(block)
            .highlight_symbol("▶ ")
            .highlight_style(HIGHLIGHT_STYLE)
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.state);
    }
}

#[derive(Default)]
pub struct ProgressBar {
    pub title: &'static str,
    pub ratio: f64,
}

impl ProgressBar {
    pub fn set_ratio(&mut self, ratio: f64) {
        self.ratio = ratio.min(1.0);
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(self.title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_set(border::ROUNDED);

        let gauge = Gauge::default()
            .block(block)
            .gauge_style(Style::default().blue())
            .ratio(self.ratio);

        Widget::render(gauge, area, buf);
    }
}
