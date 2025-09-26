use crate::{app, install};
use color_eyre::eyre::Result;
use ratatui::prelude::*;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, palette::tailwind},
    symbols::border,
    text::Line,
    widgets::*,
};
use std::sync::mpsc;

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
        let width = self.items.iter().map(|s| s.desc.len()).max().unwrap_or(0);
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
            .title_top(self.header.clone().centered())
            .title_bottom(self.footer.clone().centered())
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .padding(Padding::uniform(1));

        let items: Vec<String> = self.items.iter().map(|i| i.desc.clone()).collect();

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

#[derive(Clone)]
pub enum InstallItem {
    Obs,
    Vmb,
    Ja2,
    Khs,
    Sbs,
}

#[derive(Clone)]
pub enum ItemKind {
    Install(InstallItem),
    Release,
}

#[derive(Clone)]
pub struct Item {
    pub kind: ItemKind,
    pub desc: String,
}

impl Item {
    pub fn new(kind: ItemKind, desc: String) -> Self {
        Self { kind, desc }
    }
}

pub trait ListItemExecution {
    type Output;

    fn execute(&self, tx: mpsc::Sender<app::Event>) -> Result<Self::Output>;
}

impl ListItemExecution for InstallItem {
    type Output = ();

    fn execute(&self, tx: mpsc::Sender<app::Event>) -> Result<Self::Output> {
        match self {
            InstallItem::Obs => install::obs(tx)?,
            InstallItem::Vmb => install::vmb(tx)?,
            InstallItem::Ja2 => install::ja2(tx)?,
            InstallItem::Khs => install::khs(tx)?,
            InstallItem::Sbs => install::sbs(tx)?,
        }
        Ok(())
    }
}
