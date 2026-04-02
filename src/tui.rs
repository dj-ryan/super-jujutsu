use crate::ansi;
use crate::commands::{self, CommandTree};
use crate::jj;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Terminal, TerminalOptions, Viewport};
use std::io::stdout;
use std::time::{Duration, Instant};

const VIEWPORT_HEIGHT: u16 = 22;
const MAX_SUGGESTIONS: usize = 5;
const DEBOUNCE_MS: u64 = 250;

struct App {
    input: String,
    cursor: usize,
    suggestions: Vec<String>,
    selected_suggestion: usize,
    tree: CommandTree,
    bookmark_cache: Option<Vec<String>>,
    // Highlight state
    highlighted_ids: Vec<String>,
    last_revset: Vec<String>,
    revset_changed_at: Option<Instant>,
}

impl App {
    fn new() -> Self {
        Self {
            input: String::new(),
            cursor: 0,
            suggestions: vec![],
            selected_suggestion: 0,
            tree: commands::build_tree(),
            bookmark_cache: None,
            highlighted_ids: vec![],
            last_revset: vec![],
            revset_changed_at: None,
        }
    }

    fn update_suggestions(&mut self) {
        let tokens = self.tokenize();
        let token_refs: Vec<&str> = tokens.iter().map(|s| s.as_str()).collect();

        let mut completions = self.tree.completions(&token_refs);

        if commands::expects_bookmark_arg(&token_refs) {
            let bookmarks = self.bookmarks();
            let prefix = token_refs.last().copied().unwrap_or("");
            for b in &bookmarks {
                if b.starts_with(prefix) && !completions.contains(b) {
                    completions.push(b.clone());
                }
            }
        }

        completions.sort();
        self.suggestions = completions;
        self.selected_suggestion = 0;

        // Check if revset args changed
        let full_tokens: Vec<String> = self.input.split_whitespace().map(String::from).collect();
        let full_refs: Vec<&str> = full_tokens.iter().map(|s| s.as_str()).collect();
        let new_revsets = commands::extract_revsets(&full_refs);
        if new_revsets != self.last_revset {
            self.last_revset = new_revsets;
            self.revset_changed_at = Some(Instant::now());
        }
    }

    fn try_resolve_revsets(&mut self) {
        if let Some(changed_at) = self.revset_changed_at {
            if changed_at.elapsed() >= Duration::from_millis(DEBOUNCE_MS) {
                self.revset_changed_at = None;
                if self.last_revset.is_empty() {
                    self.highlighted_ids.clear();
                } else {
                    let combined = self.last_revset.join(" | ");
                    self.highlighted_ids = jj::resolve_revset(&combined);
                }
            }
        }
    }

    fn tokenize(&self) -> Vec<String> {
        let trimmed = &self.input[..self.cursor.min(self.input.len())];
        if trimmed.is_empty() {
            return vec![String::new()];
        }
        let mut tokens: Vec<String> = trimmed.split_whitespace().map(String::from).collect();
        if trimmed.ends_with(' ') {
            tokens.push(String::new());
        }
        if tokens.is_empty() {
            tokens.push(String::new());
        }
        tokens
    }

    fn bookmarks(&mut self) -> Vec<String> {
        if self.bookmark_cache.is_none() {
            self.bookmark_cache = Some(jj::bookmark_names());
        }
        self.bookmark_cache.clone().unwrap_or_default()
    }

    fn accept_suggestion(&mut self) {
        if let Some(suggestion) = self.suggestions.get(self.selected_suggestion).cloned() {
            let tokens = self.tokenize();
            let prefix = tokens.last().map(|s| s.as_str()).unwrap_or("");
            let suffix = &suggestion[prefix.len()..];
            self.input.insert_str(self.cursor, suffix);
            self.cursor += suffix.len();
            self.input.insert(self.cursor, ' ');
            self.cursor += 1;
            self.update_suggestions();
        }
    }

    fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += 1;
        self.update_suggestions();
    }

    fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.input.remove(self.cursor);
            self.update_suggestions();
        }
    }

    fn delete(&mut self) {
        if self.cursor < self.input.len() {
            self.input.remove(self.cursor);
            self.update_suggestions();
        }
    }

    fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
        self.update_suggestions();
    }

    fn move_right(&mut self) {
        if self.cursor < self.input.len() {
            self.cursor += 1;
        }
        self.update_suggestions();
    }
}

pub fn run(log_output: &str, status_output: &str) -> Result<Option<String>> {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        eprintln!();
        default_hook(info);
    }));

    enable_raw_mode()?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout());
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(VIEWPORT_HEIGHT),
        },
    )?;

    let mut app = App::new();
    app.update_suggestions();

    let result = event_loop(&mut terminal, &mut app, log_output, status_output);

    disable_raw_mode()?;
    println!();

    result
}

fn event_loop(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    log_output: &str,
    status_output: &str,
) -> Result<Option<String>> {
    loop {
        // Check if debounced revset resolution is due
        app.try_resolve_revsets();

        terminal.draw(|frame| render(frame, app, log_output, status_output))?;

        // Poll with timeout so we can check debounce even without input
        let timeout = if app.revset_changed_at.is_some() {
            Duration::from_millis(50)
        } else {
            Duration::from_millis(500)
        };

        if !event::poll(timeout)? {
            continue;
        }

        match event::read()? {
            Event::Key(key) => match (key.code, key.modifiers) {
                (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    return Ok(None);
                }
                (KeyCode::Enter, _) => {
                    return Ok(Some(app.input.trim().to_string()));
                }
                (KeyCode::Tab, _) => app.accept_suggestion(),
                (KeyCode::Up, _) => {
                    if !app.suggestions.is_empty() {
                        app.selected_suggestion = app
                            .selected_suggestion
                            .checked_sub(1)
                            .unwrap_or(app.suggestions.len() - 1);
                    }
                }
                (KeyCode::Down, _) => {
                    if !app.suggestions.is_empty() {
                        app.selected_suggestion =
                            (app.selected_suggestion + 1) % app.suggestions.len();
                    }
                }
                (KeyCode::Left, _) => app.move_left(),
                (KeyCode::Right, _) => app.move_right(),
                (KeyCode::Backspace, _) => app.backspace(),
                (KeyCode::Delete, _) => app.delete(),
                (KeyCode::Char(c), _) => app.insert_char(c),
                _ => {}
            },
            Event::Resize(_, _) => {
                terminal.autoresize()?;
            }
            _ => {}
        }
    }
}

fn render(frame: &mut ratatui::Frame, app: &App, log_output: &str, status_output: &str) {
    let area = frame.area();

    let suggestion_count = app.suggestions.len().min(MAX_SUGGESTIONS) as u16;
    let rows = Layout::vertical([
        Constraint::Min(5),
        Constraint::Length(suggestion_count.max(1)),
        Constraint::Length(1),
    ])
    .split(area);

    let cols = Layout::horizontal([
        Constraint::Percentage(60),
        Constraint::Percentage(40),
    ])
    .split(rows[0]);

    render_log(frame, cols[0], log_output, &app.highlighted_ids);
    render_status(frame, cols[1], status_output);
    render_suggestions(frame, rows[1], app);
    render_input(frame, rows[2], app);
}

fn line_contains_id(line: &str, ids: &[String]) -> bool {
    if ids.is_empty() {
        return false;
    }
    // Strip ANSI codes for matching
    let plain: String = {
        let mut out = String::with_capacity(line.len());
        let mut in_esc = false;
        for c in line.chars() {
            if in_esc {
                if c.is_ascii_alphabetic() { in_esc = false; }
            } else if c == '\x1b' {
                in_esc = true;
            } else {
                out.push(c);
            }
        }
        out
    };
    ids.iter().any(|id| plain.contains(id))
}

const HIGHLIGHT_BG: Color = Color::Indexed(236); // subtle dark gray bg

fn render_log(frame: &mut ratatui::Frame, area: Rect, output: &str, highlighted_ids: &[String]) {
    let lines: Vec<Line> = output
        .lines()
        .map(|raw| {
            let mut line = ansi::parse_ansi_line(raw);
            if line_contains_id(raw, highlighted_ids) {
                // Apply highlight background + bold marker to all spans
                line.spans = std::iter::once(Span::styled(
                    "▌",
                    Style::default().fg(Color::Yellow),
                ))
                .chain(line.spans.into_iter().map(|mut s| {
                    s.style = s.style.bg(HIGHLIGHT_BG);
                    s
                }))
                .collect();
            }
            line
        })
        .collect();

    let block = Block::default()
        .title(" Log ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_status(frame: &mut ratatui::Frame, area: Rect, output: &str) {
    let lines = if output.trim().is_empty() {
        vec![Line::styled(
            "  (no changes)",
            Style::default().fg(Color::DarkGray),
        )]
    } else {
        ansi::parse_ansi_text(output)
    };
    let block = Block::default()
        .title(" Status ")
        .borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_suggestions(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    if app.suggestions.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "  (no completions)",
                Style::default().fg(Color::DarkGray),
            ))),
            area,
        );
        return;
    }

    let start = app
        .selected_suggestion
        .saturating_sub(MAX_SUGGESTIONS / 2);
    let visible: Vec<Line> = app
        .suggestions
        .iter()
        .enumerate()
        .skip(start)
        .take(MAX_SUGGESTIONS)
        .map(|(i, s)| {
            let style = if i == app.selected_suggestion {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Line::from(Span::styled(format!("  {s}"), style))
        })
        .collect();

    frame.render_widget(Paragraph::new(visible), area);
}

fn render_input(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let prompt = Span::styled(
        "> jj ",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );

    let mut spans = vec![prompt];
    let (before, after) = app.input.split_at(app.cursor.min(app.input.len()));
    spans.push(Span::raw(before));

    let ghost = if !after.is_empty() {
        spans.push(Span::raw(after));
        String::new()
    } else if let Some(suggestion) = app.suggestions.get(app.selected_suggestion) {
        let tokens = app.tokenize();
        let prefix = tokens.last().map(|s| s.as_str()).unwrap_or("");
        if suggestion.starts_with(prefix) && !prefix.is_empty() {
            suggestion[prefix.len()..].to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    if !ghost.is_empty() {
        spans.push(Span::styled(ghost, Style::default().fg(Color::DarkGray)));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);

    let prompt_len = 5u16; // "> jj "
    frame.set_cursor_position((area.x + prompt_len + app.cursor as u16, area.y));
}
