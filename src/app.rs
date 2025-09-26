use crate::{install, ui};
pub use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use ratatui::{DefaultTerminal, widgets::*};
use std::{sync::mpsc, thread};

pub enum Event {
    Key(KeyEvent),
    Progress(f64),
}

pub fn send_key_event(tx: mpsc::Sender<Event>) {
    loop {
        if let crossterm::event::Event::Key(key_event) = event::read().unwrap() {
            tx.send(Event::Key(key_event)).unwrap()
        }
    }
}

pub fn send_progress_event(ratio: f64, tx: mpsc::Sender<Event>) {
    tx.send(Event::Progress(ratio)).unwrap()
}

#[derive(Clone)]
pub enum Item {
    Obs(&'static str),
    Vmb(&'static str),
    Ja2(&'static str),
    Khs(&'static str),
}

impl Item {
    pub fn as_str(&self) -> &'static str {
        match self {
            Item::Obs(s) => s,
            Item::Vmb(s) => s,
            Item::Ja2(s) => s,
            Item::Khs(s) => s,
        }
    }
}

pub struct App {
    pub exit: bool,
    pub event_tx: mpsc::Sender<Event>,
    pub event_rx: mpsc::Receiver<Event>,
    pub list: ui::StatefulList<'static>,
    pub progress: ui::ProgressBar,
}

impl Default for App {
    fn default() -> Self {
        let (event_tx, event_rx) = mpsc::channel::<Event>();
        Self {
            exit: false,
            event_tx,
            event_rx,
            list: Default::default(),
            progress: Default::default(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::channel::<Event>();

        let items = vec![
            Item::Obs("Install OBS (Open Broadcast Software)"),
            #[cfg(target_os = "windows")]
            Item::Vmb("Install Voicemeeter Banana"),
            #[cfg(target_os = "linux")]
            Item::Ja2("Install Jack Audio Connection Kit"),
            Item::Khs("Install Kilohearts Bundle"),
        ];
        let state = ListState::default().with_selected(Some(0));
        let header = Line::from(" OBS Install Manager ").centered();
        let footer = Line::from(vec![
            " Up ".green().into(),
            "<↑>".green().bold(),
            " -".bold(),
            " Down ".green().into(),
            "<↓>".green().bold(),
            " -".bold(),
            " Accept ".blue().into(),
            "<Enter>".blue().bold(),
            " -".bold(),
            " Exit ".red().into(),
            "<Esc> ".red().bold(),
        ])
        .centered();

        let list = ui::StatefulList {
            items,
            state,
            header,
            footer,
        };

        let progress = ui::ProgressBar {
            title: " Downloading ",
            ..Default::default()
        };

        Self {
            exit: false,
            event_tx,
            event_rx,
            list,
            progress,
        }
    }

    pub fn run(&mut self, mut term: DefaultTerminal) -> Result<()> {
        let key_tx = self.event_tx.clone();
        thread::spawn(move || {
            send_key_event(key_tx);
        });

        while !self.exit {
            match self.event_rx.recv()? {
                Event::Key(key_event) => self.handle_key_event(key_event),
                Event::Progress(ratio) => self.progress.set_ratio(ratio),
            }

            term.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if KeyEventKind::Press == key_event.kind {
            match key_event.code {
                KeyCode::Up => self.list.state.select_previous(),
                KeyCode::Down => self.list.state.select_next(),
                KeyCode::Enter => self.select_accept(),
                KeyCode::Esc => self.exit(),
                _ => {}
            }
        }
    }

    fn select_accept(&mut self) {
        if let Some(selected) = self.list.state.selected() {
            let tx = self.event_tx.clone();
            let item = self.list.items[selected].clone();

            thread::spawn(move || match item {
                Item::Obs(_) => install::install_obs(tx),
                Item::Vmb(_) => install::install_vmb(tx),
                Item::Ja2(_) => install::install_ja2(tx),
                Item::Khs(_) => install::install_khs(tx),
            });
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let width = self.list.width(area);
        let layout = Layout::horizontal([Constraint::Length(width)]).flex(layout::Flex::Center);
        let [cell] = layout.areas(area);

        let height = self.list.height(area);
        let layout = Layout::vertical([Constraint::Length(height), Constraint::Length(3)])
            .flex(layout::Flex::Center);
        let [top, btm] = layout.areas(cell);

        match self.progress.ratio {
            0.0 => {
                self.list.render(top, buf);
            }
            _ => {
                self.list.render(top, buf);
                self.progress.render(btm, buf);
            }
        }
    }
}
