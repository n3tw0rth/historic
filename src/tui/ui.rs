use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::{Block, List, ListDirection, ListState, Paragraph};
use ratatui::{DefaultTerminal, prelude::*};

use crate::{Event, EventHandler, Result};

#[derive(Default, PartialEq)]
pub enum Mode {
    Search,
    #[default]
    Normal,
}

#[derive(Default, Clone)]
struct Input {
    pub val: String,
}

impl Input {
    pub fn put(&mut self, char: String) {
        self.val.push_str(&char);
    }

    pub fn delete(&mut self) {
        self.val.truncate(self.val.len() - 1);
    }
}

#[derive(Default)]
pub struct Tui {
    cmds: Vec<String>,
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
                Event::Key(key_event) => self.handle_key_event(key_event),
                Event::Search => self.handle_search(),
                _ => {
                    println!("not a valid event")
                }
            }
        }
        Ok(())
    }

    fn handle_search(&self) {}

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if self.mode == Mode::Search {
            match key_event.code {
                KeyCode::Enter => {
                    self.events
                        .sender
                        .send(Event::Search)
                        .expect("failed to send the search event");
                }
                KeyCode::Backspace => {
                    self.search.delete();
                }
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Char(c) => {
                    self.search.put(c.to_string());
                }
                _ => {}
            }
        } else {
            match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('s') => match self.mode {
                    Mode::Normal => self.mode = Mode::Search,
                    Mode::Search => {}
                },
                _ => {}
            }
        }
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
            .constraints(vec![Constraint::Length(3), Constraint::Min(50)])
            .split(area);

        {
            Paragraph::new(self.search.val.clone())
                .block(Block::bordered())
                .render(layout[0], buf);
        }
        {
            let mut state = ListState::default();
            let list = List::new(self.cmds.clone())
                .block(Block::bordered().title("List"))
                .style(Style::new().white())
                .highlight_style(Style::new().italic())
                .highlight_symbol(">>")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::BottomToTop);

            StatefulWidget::render(list, layout[1], buf, &mut state);
        }
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit();
    }
}
