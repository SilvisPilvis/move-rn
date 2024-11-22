use nix::unistd::Uid;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    style::{Color, Style, Stylize},
    widgets::{self, Block, List, ListDirection, ListState},
    DefaultTerminal,
};
use std::fs;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    // if !IsRoot() {
    //     println!("Please run as root");
    //     return Ok(());
    // }

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

fn IsRoot() -> bool {
    Uid::effective().is_root()
}

#[derive(Debug, Default)]
pub struct App {
    list: Vec<String>,
    state: ListState,
    selected: Vec<String>,
    dest: String,
    file_list: List<'static>,
}

impl App {
    pub fn new(mut terminal: DefaultTerminal) -> Self {
        Self {
            list: vec![],
            selected: vec![],
            state: ListState::default(),
            dest: String::new(),
            file_list: List::new(vec![""]),
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.list.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }

            None => 0,
        };

        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.list.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i));
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.list = fs::read_dir("./")
            .unwrap()
            .flatten()
            .map(|entry| entry.path().display().to_string())
            .collect();
        let mut title = "Select files: ".to_string();
        loop {
            terminal.draw(|frame| {
                let file_list = List::new(self.list.clone())
                    .block(
                        Block::default()
                            .title(title.clone())
                            .style(Style::default().fg(Color::White).bg(Color::Black)),
                    )
                    .highlight_style(Style::default().fg(Color::Black).bg(Color::White).italic())
                    .direction(ListDirection::TopToBottom)
                    .highlight_symbol("->")
                    .repeat_highlight_symbol(true)
                    .highlight_spacing(widgets::HighlightSpacing::Always);
                frame.render_stateful_widget(file_list, frame.area(), &mut self.state)
            })?;

            match event::read()? {
                Event::Key(key_event) => match key_event.kind {
                    KeyEventKind::Press => match key_event.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char(' ') => {
                            if !self
                                .selected
                                .contains(&self.list[self.state.selected().unwrap()])
                            {
                                self.selected
                                    .push(self.list[self.state.selected().unwrap()].clone())
                            } else {
                                self.selected
                                    .retain(|x| x != &self.list[self.state.selected().unwrap()]);
                            }
                            let formated = format!("{:?}", self.selected);
                            title = format!("Select files: {}", formated);
                        }
                        KeyCode::Down => self.next(),
                        KeyCode::Up => self.previous(),
                        KeyCode::Enter => {
                            if self.dest.is_empty() && self.dest != " " {
                                title =
                                    "Use <Left> <Right> to choose destination dir: ".to_string();
                                self.dest = " ".to_string();
                            } else {
                                self.dest = self.list[self.state.selected().unwrap()].clone() + "/";
                                for (_, v) in self.selected.iter().enumerate() {
                                    // title = format!("File: {}", v);
                                    let file = Path::new(v);
                                    let dest_name = file.file_name().map(|n| n.to_string_lossy());

                                    let res = fs::rename(
                                        v,
                                        &format!("{}{}", self.dest.clone(), dest_name.unwrap()),
                                    );

                                    if res.is_err() {
                                        title = format!("Error: {}", res.err().unwrap());
                                    }
                                }
                            }
                        }
                        KeyCode::Right => {
                            let temp: String;
                            if !self.list[self.state.selected().unwrap()].contains("./") {
                                temp = format!(
                                    "./{}",
                                    self.list[self.state.selected().unwrap()].clone()
                                );
                            } else {
                                temp = format!(
                                    "{}",
                                    self.list[self.state.selected().unwrap()].clone()
                                );
                            }
                            // title = temp.clone();
                            self.list = fs::read_dir(temp)
                                .unwrap()
                                .flatten()
                                .map(|entry| entry.path().display().to_string())
                                .collect();
                        }
                        KeyCode::Left => {
                            self.list = fs::read_dir(format!("../",))
                                .unwrap()
                                .flatten()
                                .map(|entry| entry.path().display().to_string())
                                .collect();
                        }
                        _ => {}
                    },
                    _ => {}
                },
                Event::Mouse(_) => {}
                Event::Paste(_) => {}
                Event::FocusLost => {}
                Event::FocusGained => {}
                Event::Resize(_, _) => {}
            }
        }
    }
}
