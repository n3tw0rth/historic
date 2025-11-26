use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{DefaultTerminal, prelude::*};
use ratatui::{
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, List, ListDirection, ListState, Padding, Paragraph},
};
use rust_fuzzy_search::fuzzy_search;

use crate::{Event, EventHandler, Result, tui::input::Input};

#[derive(Default, PartialEq)]
pub enum Mode {
    #[default]
    Insert,
    Normal,
}

#[derive(Default)]
pub struct Tui {
    cmds: Vec<String>,
    filtered_cmds: Vec<String>,
    exit: bool,
    events: EventHandler,
    mode: Mode,
    search: Input,
}

impl Tui {
    pub fn new() -> Self {
        Tui::default()
    }

    pub async fn run(&mut self, mut term: DefaultTerminal, cmds: Vec<String>) -> Result<()> {
        self.cmds = cmds;
        while !self.exit {
            term.draw(|frame| self.render(frame))?;

            match self.events.next().await? {
                Event::Key(key_event) => self.handle_key_event(key_event)?,
                Event::Search(s) => self.handle_search(s)?,
                _ => {
                    println!("not a valid event")
                }
            }
        }
        Ok(())
    }

    fn handle_search(&mut self, s: String) -> Result<()> {
        let res = fuzzy_search(
            &s,
            &self.cmds.iter().map(String::as_ref).collect::<Vec<&str>>(),
        )
        .iter()
        .map(|i| i.0.to_string())
        .collect::<Vec<String>>();

        self.filtered_cmds = res;
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        if self.mode == Mode::Insert {
            match key_event.code {
                KeyCode::Backspace => {
                    self.search.delete();
                }
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Char(c) => {
                    self.search.put(c.to_string());
                    self.events
                        .sender
                        .send(Event::Search(self.search.to_string()))
                        .expect("failed to send the search event");
                }
                _ => {}
            }
        } else {
            match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('i') => match self.mode {
                    Mode::Normal => self.mode = Mode::Insert,
                    Mode::Insert => {}
                },
                _ => {}
            }
        }

        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn render(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &Tui {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min(50),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(area);

        {
            let mut state = ListState::default();
            let list = List::new(self.filtered_cmds.clone())
                .block(Block::bordered().title("List"))
                .style(Style::new().white())
                .highlight_style(Style::new().italic())
                .highlight_symbol(">>")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::BottomToTop);

            StatefulWidget::render(list, layout[0], buf, &mut state);
        }

        {
            let s = &self.search.to_string();
            let display = if s.is_empty() { "Search..." } else { s };
            Paragraph::new(display)
                .block(Block::bordered().padding(Padding::left(1)))
                .render(layout[1], buf);
        }

        {
            let seperator = " | ".gray();
            let lines = if self.mode.eq(&Mode::Normal) {
                vec![
                    "Exit: ".blue(),
                    "q".into(),
                    seperator,
                    "Search: ".blue(),
                    "i".into(),
                ]
            } else {
                vec!["Cancel: ".blue(), "Esc".into()]
            };

            let help_text = Line::from(lines);

            Paragraph::new(help_text)
                .alignment(Alignment::Center)
                .render(layout[2], buf);
        }
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit();
    }
}
