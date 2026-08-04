//! The Tests screen: live per-file status while a worker thread runs suites.
//!
//! Discovery *and* the run happen on the worker — walking a corpus and
//! simulating machines both take real time, and neither may stall a frame.

use std::path::PathBuf;
use std::sync::mpsc;

use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use super::app::{Screen, Transition};
use super::inspector::InspectorState;
use crate::commands::test::{discover, display_path, run_file_caught, FileOutcome, Options};

pub(crate) enum RowState {
    Pending,
    Running,
    Done(FileOutcome),
}

/// Worker → UI messages.
pub(crate) enum UiEvent {
    Discovered(Vec<PathBuf>),
    Started(PathBuf),
    Done(PathBuf, FileOutcome),
}

pub(crate) struct TestsState {
    pub(crate) rows: Vec<(PathBuf, RowState)>,
    pub(crate) cursor: usize,
    /// Row whose detail fills the lower pane.
    pub(crate) expanded: Option<usize>,
    pub(crate) expanded_scroll: u16,
    /// Still walking the corpus (before any rows exist).
    pub(crate) discovering: bool,
    /// Advances every drain; feeds the spinner.
    spinner: usize,
    receiver: Option<mpsc::Receiver<UiEvent>>,
    /// What to discover again on `r`.
    paths: Vec<PathBuf>,
    specs: Option<PathBuf>,
    trace_window: u64,
}

impl TestsState {
    pub(crate) fn discovering_cwd() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new(vec![cwd], None, 2)
    }

    pub(crate) fn new(paths: Vec<PathBuf>, specs: Option<PathBuf>, trace_window: u64) -> Self {
        let mut state = Self {
            rows: Vec::new(),
            cursor: 0,
            expanded: None,
            expanded_scroll: 0,
            discovering: true,
            spinner: 0,
            receiver: None,
            paths,
            specs,
            trace_window,
        };
        state.start();
        state
    }

    fn start(&mut self) {
        self.rows.clear();
        self.cursor = 0;
        self.expanded = None;
        self.discovering = true;
        let paths = self.paths.clone();
        let options = Options {
            filter: String::new(),
            specs: self.specs.clone(),
            json: false,
            trace_window: self.trace_window,
            paths: Vec::new(),
        };
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        std::thread::spawn(move || {
            let files = discover(&paths, "");
            if sender
                .send(UiEvent::Discovered(files.iter().map(|(_, f)| f.clone()).collect()))
                .is_err()
            {
                return;
            }
            for (root, file) in files {
                let _ = sender.send(UiEvent::Started(file.clone()));
                let outcome = run_file_caught(&root, &file, &options);
                // A dropped receiver means the UI moved on; stopping is correct.
                if sender.send(UiEvent::Done(file, outcome)).is_err() {
                    return;
                }
            }
        });
    }

    /// Pull everything the worker produced since the last frame.
    pub(crate) fn drain_events(&mut self) {
        self.spinner = self.spinner.wrapping_add(1);
        let Some(receiver) = &self.receiver else { return };
        let events: Vec<UiEvent> = receiver.try_iter().collect();
        for event in events {
            match event {
                UiEvent::Discovered(files) => {
                    self.discovering = false;
                    self.rows = files.into_iter().map(|f| (f, RowState::Pending)).collect();
                }
                UiEvent::Started(path) => self.set_row(&path, RowState::Running),
                UiEvent::Done(path, outcome) => self.set_row(&path, RowState::Done(outcome)),
            }
        }
    }

    fn set_row(&mut self, path: &PathBuf, state: RowState) {
        if let Some(row) = self.rows.iter_mut().find(|(p, _)| p == path) {
            row.1 = state;
        }
    }

    /// A state with hand-authored rows and no worker — for screen tests.
    #[cfg(test)]
    pub(crate) fn for_test(rows: Vec<(PathBuf, RowState)>) -> Self {
        Self {
            rows,
            cursor: 0,
            expanded: None,
            expanded_scroll: 0,
            discovering: false,
            spinner: 0,
            receiver: None,
            paths: Vec::new(),
            specs: None,
            trace_window: 2,
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyCode) -> Transition {
        match key {
            KeyCode::Esc => {
                if self.expanded.is_some() {
                    self.expanded = None;
                } else {
                    return Transition::To(Screen::Browser(super::browser::BrowserState::new(
                        std::env::current_dir().unwrap_or_else(|_| ".".into()),
                    )));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.expanded.is_some() {
                    self.expanded_scroll = self.expanded_scroll.saturating_sub(1);
                } else {
                    self.cursor = self.cursor.saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.expanded.is_some() {
                    self.expanded_scroll = self.expanded_scroll.saturating_add(1);
                } else {
                    self.cursor = (self.cursor + 1).min(self.rows.len().saturating_sub(1));
                }
            }
            KeyCode::PageUp => self.cursor = self.cursor.saturating_sub(20),
            KeyCode::PageDown => {
                self.cursor = (self.cursor + 20).min(self.rows.len().saturating_sub(1))
            }
            KeyCode::Enter => {
                if self.expanded == Some(self.cursor) {
                    self.expanded = None;
                } else if self.detail_text(self.cursor).is_some() {
                    self.expanded = Some(self.cursor);
                    self.expanded_scroll = 0;
                }
            }
            KeyCode::Char('i') => {
                // Inspect the selected file: schematics gather fine; sidecar
                // descriptors and snbt fall back with a status-free no-op.
                if let Some((path, _)) = self.rows.get(self.cursor) {
                    if let Ok(report) = crate::model::gather(path) {
                        return Transition::To(Screen::Inspector(InspectorState::new(report)));
                    }
                }
            }
            KeyCode::Char('r') => self.start(),
            _ => {}
        }
        Transition::Stay
    }

    /// The expandable detail for a row: per-case verdicts plus failure text.
    pub(crate) fn detail_text(&self, index: usize) -> Option<String> {
        match &self.rows.get(index)?.1 {
            RowState::Done(FileOutcome::Broken(why)) => Some(why.clone()),
            RowState::Done(FileOutcome::Ran(results)) => {
                let mut out = String::new();
                for (name, ticks, wall, outcome) in results {
                    let glyph = if outcome.is_ok() { '✓' } else { '✗' };
                    out.push_str(&format!(
                        "{glyph} {name}  ({ticks} ticks, {}ms)\n",
                        wall.as_millis()
                    ));
                }
                for (_, _, _, outcome) in results {
                    if let Err(report) = outcome {
                        out.push('\n');
                        out.push_str(report);
                        out.push('\n');
                    }
                }
                Some(out)
            }
            _ => None,
        }
    }

    pub(crate) fn counts(&self) -> (usize, usize, usize, usize, usize) {
        let (mut pass, mut fail, mut unported, mut broken, mut pending) = (0, 0, 0, 0, 0);
        for (_, row) in &self.rows {
            match row {
                RowState::Pending | RowState::Running => pending += 1,
                RowState::Done(FileOutcome::Unported) => unported += 1,
                RowState::Done(FileOutcome::Broken(_)) => broken += 1,
                RowState::Done(FileOutcome::Ran(results)) => {
                    for (_, _, _, outcome) in results {
                        if outcome.is_ok() {
                            pass += 1;
                        } else {
                            fail += 1;
                        }
                    }
                }
            }
        }
        (pass, fail, unported, broken, pending)
    }

    pub(crate) fn summary(&self) -> String {
        let (pass, fail, unported, broken, pending) = self.counts();
        format!(
            "{} files · {pass} pass · {fail} fail · {unported} unported · {broken} broken · {pending} to go",
            self.rows.len()
        )
    }

    fn spinner_glyph(&self) -> char {
        const FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        FRAMES[self.spinner % FRAMES.len()]
    }
}

pub(crate) fn draw(frame: &mut Frame, area: Rect, state: &TestsState) {
    let (_, _, _, _, pending) = state.counts();
    let running = pending > 0 || state.discovering;
    let gauge_height = u16::from(running);
    let constraints = if state.expanded.is_some() {
        [Constraint::Length(gauge_height), Constraint::Percentage(45), Constraint::Min(4)]
    } else {
        [Constraint::Length(gauge_height), Constraint::Min(1), Constraint::Length(0)]
    };
    let [gauge_area, list_area, detail_area] = Layout::vertical(constraints).areas(area);

    if running {
        let total = state.rows.len().max(1);
        let done = total - pending.min(total);
        let label = if state.discovering {
            format!("{} discovering…", state.spinner_glyph())
        } else {
            format!("{} {done}/{total}", state.spinner_glyph())
        };
        frame.render_widget(
            Gauge::default()
                .gauge_style(Style::default().fg(Color::Cyan).bg(Color::Black))
                .ratio(done as f64 / total as f64)
                .label(label),
            gauge_area,
        );
    }

    let items: Vec<ListItem> = state
        .rows
        .iter()
        .map(|(path, row)| {
            let (glyph, style) = match row {
                RowState::Pending => ('·', Style::default().fg(Color::DarkGray)),
                RowState::Running => (state.spinner_glyph(), Style::default().fg(Color::Yellow)),
                RowState::Done(FileOutcome::Unported) => {
                    ('∅', Style::default().fg(Color::DarkGray))
                }
                RowState::Done(FileOutcome::Broken(_)) => {
                    ('!', Style::default().fg(Color::Magenta))
                }
                RowState::Done(FileOutcome::Ran(results)) => {
                    if results.iter().all(|(_, _, _, outcome)| outcome.is_ok()) {
                        ('✓', Style::default().fg(Color::Green))
                    } else {
                        ('✗', Style::default().fg(Color::Red))
                    }
                }
            };
            let cases = match row {
                RowState::Done(FileOutcome::Ran(results)) => {
                    let ms: u128 =
                        results.iter().map(|(_, _, wall, _)| wall.as_millis()).sum();
                    format!("  {} case(s) · {ms}ms", results.len())
                }
                _ => String::new(),
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{glyph} "), style),
                Span::raw(display_path(path)),
                Span::styled(cases, Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(" {} ", state.summary())))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    let mut list_state = ListState::default().with_selected(Some(state.cursor));
    frame.render_stateful_widget(list, list_area, &mut list_state);

    if let Some(index) = state.expanded {
        let title = state
            .rows
            .get(index)
            .map(|(p, _)| format!(" {} ", display_path(p)))
            .unwrap_or_default();
        let text = state.detail_text(index).unwrap_or_default();
        frame.render_widget(
            Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title(title))
                .scroll((state.expanded_scroll, 0)),
            detail_area,
        );
    }
}
