use std::sync::Arc;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::widgets::{Block, List, ListDirection, ListState, Paragraph};
use ratatui::{DefaultTerminal, prelude::*};

use crate::db::Db;
use crate::terminal::Terminal;
use crate::utils;

pub struct Tui {
    term: Arc<Terminal>,
    db: Arc<Db>,
    cmds: Vec<String>,
    exit: bool,
}

impl Tui {
    pub fn new(term: Arc<Terminal>, db: Arc<Db>) -> Self {
        Tui {
            term,
            db,
            cmds: vec!["test".to_string(), "pnpm install".to_string()],
            exit: false,
        }
    }

    pub async fn run(&mut self, mut term: DefaultTerminal) -> Result<()> {
        let session_id = utils::string_to_md5(&format!("{:?} ", self.term));
        let mut rows = self.db.get_commands(&session_id).await?;

        let mut items = Vec::new();
        while let Some(row) = rows.next().await? {
            let r: String = row.get(4)?;
            items.push(r);
        }

        self.cmds = items;
        while !self.exit {
            term.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
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
