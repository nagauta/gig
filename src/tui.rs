use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::prelude::*;
use ratatui::widgets::*;
use ratatui::{TerminalOptions, Viewport};

use crate::spec::{Completion, CompletionKind};

// ── Fig-inspired palette ─────────────────────────────────────────────
const BG: Color = Color::Rgb(30, 30, 46); // dark card
const SELECTED_BG: Color = Color::Rgb(56, 68, 199); // blue highlight
const TEXT_PRIMARY: Color = Color::Rgb(255, 255, 255); // white
const TEXT_SECONDARY: Color = Color::Rgb(180, 180, 195); // muted text
const TEXT_DIM: Color = Color::Rgb(120, 120, 140); // dim args/desc
const BORDER_COLOR: Color = Color::Rgb(55, 55, 75); // subtle border
const DESC_BG: Color = Color::Rgb(38, 38, 54); // description bar bg
const ICON_COLOR: Color = Color::Rgb(180, 130, 255); // purple icon
const BRANCH_ICON_COLOR: Color = Color::Rgb(130, 220, 130); // green branch icon
const FILE_ICON_COLOR: Color = Color::Rgb(220, 190, 100); // yellow file icon

const MAX_VISIBLE: u16 = 8;

struct App {
    items: Vec<Completion>,
    filtered: Vec<FilteredItem>,
    query: String,
    selected: usize,
    scroll_offset: usize,
    indent: u16,
}

struct FilteredItem {
    index: usize,
    match_positions: Vec<usize>,
}

impl App {
    fn new(items: Vec<Completion>, indent: u16) -> Self {
        let filtered = (0..items.len())
            .map(|i| FilteredItem {
                index: i,
                match_positions: vec![],
            })
            .collect();

        Self {
            items,
            filtered,
            query: String::new(),
            selected: 0,
            scroll_offset: 0,
            indent,
        }
    }

    fn filter(&mut self) {
        if self.query.is_empty() {
            self.filtered = (0..self.items.len())
                .map(|i| FilteredItem {
                    index: i,
                    match_positions: vec![],
                })
                .collect();
        } else {
            let q = self.query.to_lowercase();
            self.filtered = self
                .items
                .iter()
                .enumerate()
                .filter_map(|(i, c)| {
                    fuzzy_match_positions(&c.value.to_lowercase(), &q).map(|positions| {
                        FilteredItem {
                            index: i,
                            match_positions: positions,
                        }
                    })
                })
                .collect();
        }
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
        self.adjust_scroll();
    }

    fn move_up(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = self.filtered.len() - 1;
        }
        self.adjust_scroll();
    }

    fn move_down(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.filtered.len();
        self.adjust_scroll();
    }

    fn adjust_scroll(&mut self) {
        let visible = MAX_VISIBLE as usize;
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + visible {
            self.scroll_offset = self.selected - visible + 1;
        }
    }

    fn selected_value(&self) -> Option<&str> {
        self.filtered
            .get(self.selected)
            .map(|f| self.items[f.index].value.as_str())
    }

    fn selected_description(&self) -> Option<&str> {
        self.filtered
            .get(self.selected)
            .and_then(|f| self.items[f.index].description.as_deref())
    }
}

fn fuzzy_match_positions(haystack: &str, needle: &str) -> Option<Vec<usize>> {
    let mut positions = Vec::with_capacity(needle.len());
    let mut needle_chars = needle.chars();
    let mut current = needle_chars.next()?;

    for (i, ch) in haystack.chars().enumerate() {
        if ch == current {
            positions.push(i);
            match needle_chars.next() {
                Some(nc) => current = nc,
                None => return Some(positions),
            }
        }
    }
    None
}

pub fn run(items: Vec<Completion>, indent: u16) -> io::Result<Option<String>> {
    if items.is_empty() {
        return Ok(None);
    }

    let (term_width, _) = crossterm::terminal::size()?;
    let indent = indent.min(term_width.saturating_sub(20));

    let mut app = App::new(items, indent);

    enable_raw_mode()?;
    // Move cursor to next line so dropdown appears below the input
    crossterm::execute!(io::stderr(), crossterm::cursor::MoveToNextLine(1))?;
    let backend = CrosstermBackend::new(io::stderr());
    let list_height = std::cmp::min(app.items.len() as u16, MAX_VISIBLE);
    let total_height = list_height + 1; // list + description bar
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(total_height),
        },
    )?;

    let result = run_loop(&mut terminal, &mut app);

    let _ = disable_raw_mode();
    let _ = terminal.clear();

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stderr>>,
    app: &mut App,
) -> io::Result<Option<String>> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        let ev = event::read()?;

        if let Event::Key(key) = ev {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Esc => return Ok(None),
                KeyCode::Enter => {
                    return Ok(app.selected_value().map(|s| s.to_string()));
                }
                KeyCode::Tab | KeyCode::Down => app.move_down(),
                KeyCode::Up | KeyCode::BackTab => app.move_up(),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(None);
                }
                KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.move_up()
                }
                KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.move_down()
                }
                KeyCode::Char(c) => {
                    app.query.push(c);
                    app.filter();
                }
                KeyCode::Backspace => {
                    app.query.pop();
                    app.filter();
                }
                _ => {}
            }
        }
    }
}

// ── Rendering ────────────────────────────────────────────────────────

fn draw(f: &mut Frame, app: &App) {
    let area = f.area();

    if app.filtered.is_empty() {
        return;
    }

    let list_height = (app.filtered.len() as u16)
        .min(MAX_VISIBLE)
        .min(area.height.saturating_sub(1));

    let chunks =
        Layout::vertical([Constraint::Length(list_height), Constraint::Length(1)]).split(area);

    render_list(f, app, chunks[0]);
    render_description(f, app, chunks[1]);
}

fn indent_pad(indent: u16) -> Span<'static> {
    Span::raw(" ".repeat(indent as usize))
}

fn render_list(f: &mut Frame, app: &App, area: Rect) {
    let visible_count = area.height as usize;
    let visible_items = app
        .filtered
        .iter()
        .skip(app.scroll_offset)
        .take(visible_count);

    let items: Vec<ListItem> = visible_items
        .enumerate()
        .map(|(i, fi)| {
            let item = &app.items[fi.index];
            let is_selected = i + app.scroll_offset == app.selected;

            let mut spans: Vec<Span> = Vec::new();

            // Left indent
            spans.push(indent_pad(app.indent));

            // Left border
            spans.push(Span::styled("│", Style::default().fg(BORDER_COLOR)));

            // Icon
            let (icon, icon_color) = match item.kind {
                CompletionKind::Branch => (" ᚠ ", BRANCH_ICON_COLOR),
                CompletionKind::File => (" □ ", FILE_ICON_COLOR),
                _ => (" $ ", ICON_COLOR),
            };
            spans.push(Span::styled(icon, Style::default().fg(icon_color).bold()));

            // Value
            let value_style = if is_selected {
                Style::default().fg(TEXT_PRIMARY).bold()
            } else {
                Style::default().fg(TEXT_SECONDARY)
            };

            for (ci, ch) in item.value.chars().enumerate() {
                if fi.match_positions.contains(&ci) {
                    spans.push(Span::styled(
                        ch.to_string(),
                        Style::default().fg(TEXT_PRIMARY).bold(),
                    ));
                } else {
                    spans.push(Span::styled(ch.to_string(), value_style));
                }
            }

            // Description inline
            if let Some(desc) = &item.description {
                spans.push(Span::styled(
                    format!("  {}", desc),
                    Style::default().fg(TEXT_DIM),
                ));
            }

            let bg = if is_selected { SELECTED_BG } else { BG };
            ListItem::new(Line::from(spans)).style(Style::default().bg(bg))
        })
        .collect();

    f.render_widget(List::new(items), area);
}

fn render_description(f: &mut Frame, app: &App, area: Rect) {
    let desc = app.selected_description().unwrap_or("");

    let line = Line::from(vec![
        indent_pad(app.indent),
        Span::styled("│", Style::default().fg(BORDER_COLOR)),
        Span::styled(format!(" {}", desc), Style::default().fg(TEXT_DIM).italic()),
    ]);

    f.render_widget(
        Paragraph::new(line).style(Style::default().bg(DESC_BG)),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_match_basic() {
        assert!(fuzzy_match_positions("commit", "cmt").is_some());
        assert!(fuzzy_match_positions("commit", "xyz").is_none());
        assert!(fuzzy_match_positions("--message", "msg").is_some());
    }

    #[test]
    fn fuzzy_match_positions_correct() {
        let positions = fuzzy_match_positions("commit", "cmt").unwrap();
        assert_eq!(positions, vec![0, 2, 5]);
    }

    #[test]
    fn fuzzy_match_empty_needle() {
        assert!(fuzzy_match_positions("commit", "").is_none());
    }

    #[test]
    fn app_filter_narrows_results() {
        let items = vec![
            Completion {
                value: "commit".to_string(),
                description: Some("Record changes".to_string()),
                kind: CompletionKind::Subcommand,
            },
            Completion {
                value: "clone".to_string(),
                description: Some("Clone a repo".to_string()),
                kind: CompletionKind::Subcommand,
            },
            Completion {
                value: "push".to_string(),
                description: None,
                kind: CompletionKind::Subcommand,
            },
        ];
        let mut app = App::new(items, 0);
        app.query = "c".to_string();
        app.filter();
        assert_eq!(app.filtered.len(), 2);
    }

    #[test]
    fn app_navigation_wraps() {
        let items = vec![
            Completion {
                value: "a".to_string(),
                description: None,
                kind: CompletionKind::Subcommand,
            },
            Completion {
                value: "b".to_string(),
                description: None,
                kind: CompletionKind::Subcommand,
            },
        ];
        let mut app = App::new(items, 0);
        assert_eq!(app.selected, 0);
        app.move_down();
        assert_eq!(app.selected, 1);
        app.move_down();
        assert_eq!(app.selected, 0);
        app.move_up();
        assert_eq!(app.selected, 1);
    }
}
