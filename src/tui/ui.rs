use std::fs::File;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal as RatTerminal, prelude::*};
use ratatui::{
    layout::Position,
    widgets::{List, ListDirection, ListItem, ListState, Paragraph},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::tui::matcher::{self, Match};
use crate::{Event, EventHandler, Result, tui::input::Input};

/// Used for the prompt, the selection marker and matched characters.
const ACCENT: Color = Color::Cyan;
/// Used for hints, counters and placeholders.
const MUTED: Color = Color::DarkGray;

const PROMPT: &str = "❯ ";
const MARKER: &str = "▌ ";
/// Columns reserved in front of every row for the prompt / selection marker.
const GUTTER: u16 = 2;

/// TUI contains to modes
/// Insert - Focus on the search to find the right command
/// Normal - Focus on the stateful list to select the right command
#[derive(Default, Debug, PartialEq)]
pub enum Mode {
    #[default]
    Insert,
    Normal,
}

/// TUI component of the application
#[derive(Default, Debug)]
pub struct Tui {
    cmds: Vec<String>,
    /// Commands matching the current search, best first.
    matches: Vec<Match>,
    exit: bool,
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
        term: &mut RatTerminal<CrosstermBackend<File>>,
        cmds: Vec<String>,
    ) -> Result<Option<String>> {
        self.set_commands(cmds);
        let mut events = EventHandler::new();

        while !self.exit {
            term.draw(|frame| self.render(frame))?;

            // Anything other than a key (e.g. a resize) only needs a redraw.
            if let Event::Key(key_event) = events.next().await? {
                self.handle_key_event(key_event);
            }
        }
        Ok(self.selection.clone())
    }

    fn set_commands(&mut self, cmds: Vec<String>) {
        self.cmds = cmds;
        self.refilter();
    }

    /// Re-run the search and move the selection back to the best match.
    fn refilter(&mut self) {
        self.matches = matcher::filter(self.search.value(), &self.cmds);
        let selected = (!self.matches.is_empty()).then_some(0);
        self.list_state = ListState::default().with_selected(selected);
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        // Keys that work the same way in both modes.
        match key.code {
            KeyCode::Char('c') if ctrl => return self.exit(),
            KeyCode::Enter => return self.select(),
            KeyCode::Up => return self.move_up(),
            KeyCode::Down => return self.move_down(),
            KeyCode::Char('p' | 'k') if ctrl => return self.move_up(),
            KeyCode::Char('n' | 'j') if ctrl => return self.move_down(),
            _ => {}
        }

        match self.mode {
            Mode::Insert => match key.code {
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Backspace => {
                    self.search.delete();
                    self.refilter();
                }
                KeyCode::Char('u') if ctrl => {
                    self.search.clear();
                    self.refilter();
                }
                KeyCode::Char('w') if ctrl => {
                    self.search.delete_word();
                    self.refilter();
                }
                KeyCode::Char(c) if !ctrl && !key.modifiers.contains(KeyModifiers::ALT) => {
                    self.search.put(c.to_string());
                    self.refilter();
                }
                _ => {}
            },
            Mode::Normal => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.exit(),
                KeyCode::Char('i' | '/') => self.mode = Mode::Insert,
                KeyCode::Char('k') => self.move_up(),
                KeyCode::Char('j') => self.move_down(),
                _ => {}
            },
        }
    }

    /// The list is drawn bottom to top, so moving up means a higher index.
    fn move_up(&mut self) {
        if let Some(i) = self.list_state.selected() {
            let last = self.matches.len().saturating_sub(1);
            self.list_state.select(Some((i + 1).min(last)));
        }
    }

    fn move_down(&mut self) {
        if let Some(i) = self.list_state.selected() {
            self.list_state.select(Some(i.saturating_sub(1)));
        }
    }

    fn select(&mut self) {
        self.selection = self
            .list_state
            .selected()
            .and_then(|i| self.matches.get(i))
            .map(|m| self.cmds[m.index].clone());

        if self.selection.is_some() {
            self.exit();
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area().inner(Margin::new(1, 0));
        let [_, list_area, _, prompt_area, hints_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);

        self.render_list(frame, list_area);
        self.render_prompt(frame, prompt_area);
        self.render_hints(frame, hints_area);
    }

    fn render_list(&mut self, frame: &mut Frame, area: Rect) {
        if self.matches.is_empty() {
            let message = if self.cmds.is_empty() {
                Line::from(vec![
                    "No commands saved yet. Add one with ".fg(MUTED),
                    "historic add <command>".into(),
                ])
            } else {
                Line::from("No matches".fg(MUTED))
            };
            let row = Rect {
                x: area.x + GUTTER,
                y: area.bottom().saturating_sub(1),
                width: area.width.saturating_sub(GUTTER),
                height: area.height.min(1),
            };
            frame.render_widget(Paragraph::new(message), row);
            return;
        }

        let selected = self.list_state.selected();
        let width = area.width.saturating_sub(GUTTER) as usize;
        let items: Vec<ListItem> = self
            .matches
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let marker = if selected == Some(i) {
                    Span::styled(MARKER, Style::new().fg(ACCENT))
                } else {
                    Span::raw(" ".repeat(GUTTER as usize))
                };
                let mut spans = vec![marker];
                spans.extend(highlight(&self.cmds[m.index], &m.positions, width));
                ListItem::new(Line::from(spans))
            })
            .collect();

        let list = List::new(items)
            .direction(ListDirection::BottomToTop)
            .highlight_style(Style::new().bold());

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_prompt(&self, frame: &mut Frame, area: Rect) {
        let insert = self.mode == Mode::Insert;
        let count = format!("{}/{}", self.matches.len(), self.cmds.len());
        let [input_area, count_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(count.len() as u16)])
                .spacing(1)
                .areas(area);

        let prompt_style = if insert {
            Style::new().fg(ACCENT).bold()
        } else {
            Style::new().fg(MUTED)
        };
        let query = if self.search.is_empty() {
            "Search commands…".fg(MUTED)
        } else {
            Span::raw(self.search.value())
        };
        let line = Line::from(vec![Span::styled(PROMPT, prompt_style), query]);

        frame.render_widget(Paragraph::new(line), input_area);
        frame.render_widget(
            Paragraph::new(count.fg(MUTED)).alignment(Alignment::Right),
            count_area,
        );

        if insert {
            let x = input_area.x + GUTTER + self.search.value().width() as u16;
            let x = x.min(input_area.right().saturating_sub(1));
            frame.set_cursor_position(Position::new(x, input_area.y));
        }
    }

    fn render_hints(&self, frame: &mut Frame, area: Rect) {
        let hints: &[(&str, &str)] = match self.mode {
            Mode::Insert => &[
                ("enter", "select"),
                ("↑↓", "move"),
                ("esc", "browse"),
                ("ctrl-c", "quit"),
            ],
            Mode::Normal => &[
                ("enter", "select"),
                ("j/k", "move"),
                ("i", "search"),
                ("q", "quit"),
            ],
        };

        let mut spans = vec![Span::raw(" ".repeat(GUTTER as usize))];
        for (i, (key, action)) in hints.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw("   "));
            }
            spans.push(Span::styled(*key, Style::new().fg(MUTED).bold()));
            spans.push(Span::styled(format!(" {action}"), Style::new().fg(MUTED)));
        }

        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }
}

/// Lay `text` out on one line at most `width` columns wide, emphasising the
/// characters at `positions` and ending with `…` if it had to be cut short.
fn highlight(text: &str, positions: &[usize], width: usize) -> Vec<Span<'static>> {
    let text: String = text.chars().map(printable).collect();
    let truncated = text.width() > width;
    let budget = if truncated {
        width.saturating_sub(1)
    } else {
        width
    };

    let mut spans = Vec::new();
    let mut chunk = String::new();
    let mut chunk_matched = false;
    let mut used = 0;
    let mut positions = positions.iter().peekable();

    for (i, c) in text.chars().enumerate() {
        used += c.width().unwrap_or(0);
        if used > budget {
            break;
        }
        let matched = positions.next_if_eq(&&i).is_some();
        if matched != chunk_matched && !chunk.is_empty() {
            spans.push(styled_chunk(std::mem::take(&mut chunk), chunk_matched));
        }
        chunk_matched = matched;
        chunk.push(c);
    }
    if !chunk.is_empty() {
        spans.push(styled_chunk(chunk, chunk_matched));
    }
    if truncated {
        spans.push("…".fg(MUTED));
    }
    spans
}

fn styled_chunk(chunk: String, matched: bool) -> Span<'static> {
    if matched {
        Span::styled(chunk, Style::new().fg(ACCENT).bold())
    } else {
        Span::raw(chunk)
    }
}

/// Swap control characters for something that occupies a single cell, so
/// multi-line commands stay on one row and match positions still line up.
fn printable(c: char) -> char {
    match c {
        '\n' => '↵',
        c if c.is_control() => ' ',
        c => c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    fn tui_with(cmds: &[&str]) -> Tui {
        let mut tui = Tui::new();
        tui.set_commands(cmds.iter().map(|s| s.to_string()).collect());
        tui
    }

    fn press(tui: &mut Tui, code: KeyCode) {
        tui.handle_key_event(KeyEvent::new(code, KeyModifiers::NONE));
    }

    fn ctrl(tui: &mut Tui, c: char) {
        tui.handle_key_event(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL));
    }

    fn type_str(tui: &mut Tui, s: &str) {
        s.chars().for_each(|c| press(tui, KeyCode::Char(c)));
    }

    fn draw(tui: &mut Tui, width: u16, height: u16) -> Vec<String> {
        let mut terminal = RatTerminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| tui.render(frame)).unwrap();
        let buffer = terminal.backend().buffer();
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn test_initialization() {
        let tui = Tui::new();
        assert!(!tui.exit, "TUI should not start in exit state");

        assert!(tui.selection.is_none())
    }

    #[test]
    fn test_enter_picks_first_command_while_typing() {
        let mut tui = tui_with(&["git status", "cargo build"]);
        press(&mut tui, KeyCode::Enter);
        assert_eq!(tui.selection.as_deref(), Some("git status"));
        assert!(tui.exit);
    }

    #[test]
    fn test_search_then_enter_picks_best_match() {
        let mut tui = tui_with(&["git status", "cargo build", "cargo test"]);
        type_str(&mut tui, "ctest");
        press(&mut tui, KeyCode::Enter);
        assert_eq!(tui.selection.as_deref(), Some("cargo test"));
    }

    #[test]
    fn test_backspace_updates_results() {
        let mut tui = tui_with(&["git status", "cargo build"]);
        type_str(&mut tui, "cx");
        assert!(tui.matches.is_empty());
        press(&mut tui, KeyCode::Backspace);
        assert_eq!(tui.matches.len(), 1);
    }

    #[test]
    fn test_navigation_is_clamped() {
        let mut tui = tui_with(&["a", "b", "c"]);
        press(&mut tui, KeyCode::Down);
        assert_eq!(tui.list_state.selected(), Some(0));
        (0..5).for_each(|_| press(&mut tui, KeyCode::Up));
        assert_eq!(tui.list_state.selected(), Some(2));
        ctrl(&mut tui, 'n');
        press(&mut tui, KeyCode::Enter);
        assert_eq!(tui.selection.as_deref(), Some("b"));
    }

    #[test]
    fn test_normal_mode_keys() {
        let mut tui = tui_with(&["a", "b"]);
        press(&mut tui, KeyCode::Esc);
        assert_eq!(tui.mode, Mode::Normal);
        press(&mut tui, KeyCode::Char('k'));
        assert_eq!(tui.list_state.selected(), Some(1));
        press(&mut tui, KeyCode::Char('j'));
        assert_eq!(tui.list_state.selected(), Some(0));
        press(&mut tui, KeyCode::Char('i'));
        assert_eq!(tui.mode, Mode::Insert);
        press(&mut tui, KeyCode::Esc);
        press(&mut tui, KeyCode::Char('q'));
        assert!(tui.exit);
        assert!(tui.selection.is_none());
    }

    #[test]
    fn test_ctrl_c_quits_without_typing() {
        let mut tui = tui_with(&["a"]);
        ctrl(&mut tui, 'c');
        assert!(tui.exit);
        assert!(tui.search.is_empty());
        assert!(tui.selection.is_none());
    }

    #[test]
    fn test_enter_without_matches_does_nothing() {
        let mut tui = tui_with(&["a"]);
        type_str(&mut tui, "zzz");
        press(&mut tui, KeyCode::Enter);
        assert!(!tui.exit);
    }

    #[test]
    fn test_render_has_no_borders() {
        let mut tui = tui_with(&["git status", "cargo build"]);
        type_str(&mut tui, "git");
        let lines = draw(&mut tui, 40, 8);
        let screen = lines.join("\n");
        assert!(!screen.contains(['│', '─', '┌', '┐', '└', '┘']));
        assert!(lines[6].starts_with(" ❯ git "));
        assert!(lines[6].ends_with("1/2"));
        assert_eq!(lines[5], "");
        assert_eq!(lines[4], " ▌ git status");
    }

    #[test]
    fn test_render_empty_state() {
        let mut tui = tui_with(&[]);
        let lines = draw(&mut tui, 70, 6);
        assert!(lines[2].contains("No commands saved yet"));
    }

    #[test]
    fn test_highlight_truncates_with_ellipsis() {
        let spans = highlight("cargo build --release", &[0], 10);
        let text: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "cargo bui…");
        assert_eq!(spans[0].content, "c");
        assert_eq!(spans[0].style.fg, Some(ACCENT));
    }
}
