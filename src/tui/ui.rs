use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::{Block, List, ListDirection, ListState, Paragraph};
use ratatui::{DefaultTerminal, prelude::*};

use super::event::{Event, EventHandler};
use crate::Result;

pub struct Tui {
    cmds: Vec<String>,
    exit: bool,
    events: EventHandler,
}

impl Tui {
    pub fn new(events: EventHandler) -> Self {
        Tui {
            cmds: Vec::new(),
            exit: false,
            events,
        }
    }

    pub async fn run(&mut self, mut term: DefaultTerminal, cmds: Vec<String>) -> Result<()> {
        self.cmds = cmds;
        while !self.exit {
            term.draw(|frame| self.render(frame))?;

            match self.events.next().await? {
                Event::Key(key_event) => self.handle_key_event(key_event),
                _ => {
                    println!("not a valid event")
                }
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('s') => {
                // TODO: check the current mode and let the user to search on the records
                todo!()
            }
            _ => {}
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
            Paragraph::new("Search the command")
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
