use crate::{
    install::{Installer, Ja2, Khs, Obs, Sbs},
    ui,
};
pub use color_eyre::{Result, eyre::eyre};
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, prelude::*, widgets::*};
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

pub fn send_progress_event(ratio: f64, tx: &mpsc::Sender<Event>) {
    tx.send(Event::Progress(ratio)).unwrap()
}

pub struct App {
    pub evtx: mpsc::Sender<Event>,
    pub evrx: mpsc::Receiver<Event>,
    pub list: ui::StatefulList<'static>,
    pub pbar: ui::ProgressBar,
    pub exit: bool,
}

impl App {
    pub fn new() -> Self {
        let (evtx, evrx) = mpsc::channel::<Event>();

        let items = vec![
            ui::ActionItem::new(
                Installer::Obs(Obs::default()),
                "Install OBS (Open Broadcast Software)".to_string(),
            ),
            // #[cfg(target_os = "windows")]
            // ui::ActionItem::new(
            //     Installer::Vmb(Vmb),
            //     "Install Voicemeeter Banana".to_string(),
            // ),
            ui::ActionItem::new(
                Installer::Ja2(Ja2::default()),
                "Install Jack Audio Connection Kit".to_string(),
            ),
            ui::ActionItem::new(Installer::Khs(Khs), "Install Kilohearts Bundle".to_string()),
            #[cfg(any(target_os = "windows", target_os = "macos"))]
            ui::ActionItem::new(
                Installer::Sbs(Sbs::default()),
                "Install Sonobus".to_string(),
            ),
        ];
        let state = ListState::default().with_selected(Some(0));
        let header = Line::from(" OBS Install Manager ".bold());
        let footer = Line::from(
            [" Up <↑>", "Down <↓>", "Accept <Enter>", "Exit <Esc> "]
                .join(" - ")
                .bold(),
        );

        let list = ui::StatefulList {
            items,
            state,
            header,
            footer,
        };

        let pbar = ui::ProgressBar {
            title: " Downloading ",
            ..Default::default()
        };

        Self {
            evtx,
            evrx,
            list,
            pbar,
            exit: false,
        }
    }

    pub fn run(&mut self, mut term: DefaultTerminal) -> Result<()> {
        term.draw(|frame| self.draw(frame))?;

        let evtx = self.evtx.clone();
        thread::spawn(move || {
            send_key_event(evtx);
        });

        while !self.exit {
            match self.evrx.recv()? {
                Event::Key(key_event) => {
                    if self.pbar.ratio == 0.0 {
                        self.handle_key_event(key_event)?
                    }
                }
                Event::Progress(ratio) => self.pbar.set_ratio(ratio),
            }

            term.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        if KeyEventKind::Press == key_event.kind {
            match key_event.code {
                KeyCode::Up => self.list.state.select_previous(),
                KeyCode::Down => self.list.state.select_next(),
                KeyCode::Enter => self.select_accept()?,
                KeyCode::Esc => self.exit(),
                _ => (),
            }
        };
        Ok(())
    }

    fn select_accept(&mut self) -> Result<()> {
        if let Some(selected) = self.list.state.selected() {
            let tx = self.evtx.clone();
            let item = self.list.items[selected].clone();
            thread::spawn(move || -> Result<()> { item.execute(tx).map_err(|e| eyre!("{e}")) });
        }
        Ok(())
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

        self.list.render(top, buf);

        if self.pbar.ratio != 0.0 {
            self.pbar.render(btm, buf);
        }
    }
}
