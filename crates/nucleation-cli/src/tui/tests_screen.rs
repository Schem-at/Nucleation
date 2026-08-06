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
    /// Last frame's list area and scroll offset, for resolving clicks.
    pub(crate) hit_rows: Option<(Rect, usize)>,
    /// Where this run points, for the title — a file name, a directory, or
    /// a root count. The screen must say what it is testing: a scan that
    /// looks identical whether it covers one file or the whole tree is how
    /// focus gets lost.
    pub(crate) scope_label: String,
    specs: Option<PathBuf>,
    trace_window: u64,
}

impl TestsState {
    /// A run scoped to one file or directory — what Tab from the inspector
    /// or browser builds, instead of a working-directory-wide scan.
    pub(crate) fn scoped(path: PathBuf) -> Self {
        Self::new(vec![path], None, 2)
    }

    /// Whether `path` is inside what this run covers — the test for reusing
    /// a finished run instead of rescanning when the user tabs back in.
    pub(crate) fn covers(&self, path: &std::path::Path) -> bool {
        self.rows.iter().any(|(p, _)| p == path)
            || self.paths.iter().any(|root| path.starts_with(root))
    }

    /// Put the cursor on `path`'s row, when it has one.
    pub(crate) fn focus(&mut self, path: &std::path::Path) {
        if let Some(index) = self.rows.iter().position(|(p, _)| p == path) {
            self.cursor = index;
        }
    }

    pub(crate) fn new(paths: Vec<PathBuf>, specs: Option<PathBuf>, trace_window: u64) -> Self {
        let scope_label = match paths.as_slice() {
            [one] if one.is_file() => one
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| display_path(one)),
            [one] => display_path(one),
            many => format!("{} roots", many.len()),
        };
        let mut state = Self {
            rows: Vec::new(),
            cursor: 0,
            expanded: None,
            expanded_scroll: 0,
            discovering: true,
            spinner: 0,
            receiver: None,
            paths,
            hit_rows: None,
            scope_label,
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
                .send(UiEvent::Discovered(
                    files.iter().map(|(_, f)| f.clone()).collect(),
                ))
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
        let Some(receiver) = &self.receiver else {
            return;
        };
        let events: Vec<UiEvent> = receiver.try_iter().collect();
        for event in events {
            match event {
                UiEvent::Discovered(files) => {
                    self.discovering = false;
                    self.rows = files.into_iter().map(|f| (f, RowState::Pending)).collect();
                }
                UiEvent::Started(path) => self.set_row(&path, RowState::Running),
                UiEvent::Done(path, outcome) => {
                    self.set_row(&path, RowState::Done(outcome));
                    // A run over exactly one file opens its per-case detail
                    // by itself — that detail *is* what a scoped run is for.
                    if self.rows.len() == 1 && self.expanded.is_none() {
                        self.expanded = Some(0);
                        self.expanded_scroll = 0;
                    }
                }
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
            hit_rows: None,
            scope_label: String::new(),
            specs: None,
            trace_window: 2,
        }
    }

    /// A left click on the run list: select, or expand the already-selected
    /// row's detail (the same thing Enter does).
    pub(crate) fn click(&mut self, x: u16, y: u16) -> Transition {
        let Some((area, offset)) = self.hit_rows else {
            return Transition::Stay;
        };
        let top = area.y + 1;
        if x < area.x || x >= area.x + area.width || y < top || y >= area.y + area.height - 1 {
            return Transition::Stay;
        }
        let row = offset + (y - top) as usize;
        if row >= self.rows.len() {
            return Transition::Stay;
        }
        if self.cursor == row {
            return self.on_key(KeyCode::Enter);
        }
        self.cursor = row;
        Transition::Stay
    }

    pub(crate) fn on_key(&mut self, key: KeyCode) -> Transition {
        match key {
            KeyCode::Esc => {
                // Collapse the detail; leaving the screen entirely is the
                // app's job — it restores whatever the user tabbed in from.
                if self.expanded.is_some() {
                    self.expanded = None;
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

pub(crate) fn draw(frame: &mut Frame, area: Rect, state: &mut TestsState) {
    let (_, _, _, _, pending) = state.counts();
    let running = pending > 0 || state.discovering;
    let gauge_height = u16::from(running);
    let constraints = if state.expanded.is_some() {
        [
            Constraint::Length(gauge_height),
            Constraint::Percentage(45),
            Constraint::Min(4),
        ]
    } else {
        [
            Constraint::Length(gauge_height),
            Constraint::Min(1),
            Constraint::Length(0),
        ]
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
                    let ms: u128 = results.iter().map(|(_, _, wall, _)| wall.as_millis()).sum();
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
        .block(Block::default().borders(Borders::ALL).title(format!(
            " tests · {} · {} ",
            state.scope_label,
            state.summary()
        )))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    let mut list_state = ListState::default().with_selected(Some(state.cursor));
    frame.render_stateful_widget(list, list_area, &mut list_state);
    let rows_visible = list_area.height.saturating_sub(2) as usize;
    let offset = (state.cursor + 1).saturating_sub(rows_visible.max(1));
    state.hit_rows = Some((list_area, offset));

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
