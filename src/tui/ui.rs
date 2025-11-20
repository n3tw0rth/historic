use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::widgets::{Block, Paragraph};
use ratatui::{DefaultTerminal, prelude::*};

pub struct Tui {}

impl Tui {
    pub fn new() -> Self {
        Tui {}
    }

    pub fn run(&self, mut term: DefaultTerminal) -> Result<()> {
        loop {
            term.draw(|frame| self.render(frame))?;
            if matches!(event::read()?, Event::Key(_)) {
                break ();
            }
        }
        Ok(())
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
            Block::bordered().render(layout[1], buf);
        }
    }
}
