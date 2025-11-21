use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::widgets::{Block, List, ListDirection, ListState, Paragraph};
use ratatui::{DefaultTerminal, prelude::*};

pub struct Tui {
    cmds: Vec<String>,
    exit: bool,
}

impl Tui {
    pub fn new() -> Self {
        Tui {
            cmds: Vec::new(),
            exit: false,
        }
    }

    pub fn run(&mut self, mut term: DefaultTerminal, cmds: Vec<String>) -> Result<()> {
        self.cmds = cmds;
        while !self.exit {
            term.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()? {
            self.handle_key_event(key);
        }
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
