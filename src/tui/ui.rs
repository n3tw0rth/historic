use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{DefaultTerminal, prelude::*};
use ratatui::{
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, List, ListDirection, ListState, Padding, Paragraph},
};
use rust_fuzzy_search::fuzzy_search_threshold;
use tracing::{debug, instrument};

use crate::{Event, EventHandler, Result, tui::input::Input};

#[derive(Default, Debug, PartialEq)]
pub enum Mode {
    #[default]
    Insert,
    Normal,
}

#[derive(Default, Debug)]
pub struct Tui {
    cmds: Vec<String>,
    filtered_cmds: Vec<String>,
    exit: bool,
    events: EventHandler,
    mode: Mode,
    search: Input,
    list_state: ListState,
    selection: Option<String>,
}

impl Tui {
    pub fn new() -> Self {
        Tui::default()
    }

    pub async fn run(
        &mut self,
        mut term: DefaultTerminal,
        cmds: Vec<String>,
    ) -> Result<Option<String>> {
        self.cmds = cmds;
        while !self.exit && !self.selection.is_some() {
            term.draw(|frame| self.render(frame))?;

            match self.events.next().await? {
                Event::Key(key_event) => self.handle_key_event(key_event)?,
                Event::Search(s) => self.handle_search(s)?,
                _ => {
                    println!("not a valid event")
                }
            }
        }
        Ok(self.selection.clone())
    }

    #[instrument(fields(s=s),skip(self))]
    fn handle_search(&mut self, s: String) -> Result<()> {
        debug!("searching");
        if s.len() > 0 {
            let threshold: f32 = 0.1f32;
            let res = fuzzy_search_threshold(
                &s,
                &self.cmds.iter().map(String::as_ref).collect::<Vec<&str>>(),
                threshold,
            )
            .iter()
            .map(|i| i.0.to_string())
            .collect::<Vec<String>>();

            self.filtered_cmds = res;
        } else {
            self.filtered_cmds = [].to_vec()
        }
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
                KeyCode::Char('j') => {
                    self.list_state.select_previous();
                }
                KeyCode::Char('k') => {
                    self.list_state.select_next();
                }
                KeyCode::Enter => {
                    let selected_index = self.list_state.selected().unwrap_or(0);
                    self.selection = if self.search.to_string().is_empty() {
                        self.cmds.get(selected_index).cloned()
                    } else {
                        self.filtered_cmds.get(selected_index).cloned()
                    };

                    self.exit();
                }
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
            let mut state = self.list_state.clone();
            let results = if self.search.to_string().is_empty() {
                self.cmds.clone()
            } else {
                self.filtered_cmds.clone()
            };

            let list = List::new(results)
                .block(Block::bordered().title("List"))
                .style(Style::new().white())
                .highlight_style(Style::new().italic())
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
            let lines = if self.mode.eq(&Mode::Normal) {
                vec![
                    "Exit: ".blue(),
                    "q".into(),
                    " | ".gray(),
                    "Up: ".blue(),
                    "k".into(),
                    " | ".gray(),
                    "Down: ".blue(),
                    "j".into(),
                    " | ".gray(),
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
