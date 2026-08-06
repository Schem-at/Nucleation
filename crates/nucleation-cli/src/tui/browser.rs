//! The file browser: filterable listing, schematics highlighted, Enter opens.
//!
//! Opening a file gathers its report on a worker thread — a multi-megabyte
//! litematic must never freeze the frame loop.

use std::path::PathBuf;
use std::sync::mpsc;

use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Row, Table, TableState};
use ratatui::Frame;

use super::app::Transition;
use crate::model::FileReport;

/// One row of the listing.
pub(crate) struct Entry {
    pub(crate) path: PathBuf,
    pub(crate) is_dir: bool,
    pub(crate) bytes: u64,
    /// A schematic by extension — highlighted, and Enter inspects it.
    pub(crate) supported: bool,
}

pub(crate) struct BrowserState {
    pub(crate) dir: PathBuf,
    pub(crate) entries: Vec<Entry>,
    /// Cursor over the *filtered* view.
    pub(crate) cursor: usize,
    pub(crate) filter: String,
    /// `/` was pressed: characters edit the filter.
    pub(crate) filtering: bool,
    /// The last load error, shown in the block title until the next action.
    pub(crate) status: Option<String>,
    /// A gather in flight for this path, with its result channel.
    loading: Option<(PathBuf, mpsc::Receiver<Result<FileReport, String>>)>,
    /// A finished gather waiting for the app loop to open the inspector.
    ready: Option<FileReport>,
    /// Last frame's table area and scroll offset, for resolving clicks.
    pub(crate) hit_rows: Option<(Rect, usize)>,
}

/// Extensions worth highlighting. Detection at open time is content-based;
/// this is only the visual cue.
const SUPPORTED: &[&str] = &[
    "litematic",
    "schem",
    "schematic",
    "snbt",
    "nbt",
    "mcstructure",
];

impl BrowserState {
    pub(crate) fn new(dir: PathBuf) -> Self {
        let mut state = Self {
            dir,
            entries: Vec::new(),
            cursor: 0,
            filter: String::new(),
            hit_rows: None,
            filtering: false,
            status: None,
            loading: None,
            ready: None,
        };
        state.reload();
        state
    }

    pub(crate) fn reload(&mut self) {
        self.entries.clear();
        self.cursor = 0;
        self.filter.clear();
        self.filtering = false;
        let Ok(read) = std::fs::read_dir(&self.dir) else {
            self.status = Some(format!("cannot read {}", self.dir.display()));
            return;
        };
        let mut entries: Vec<Entry> = read
            .filter_map(|e| e.ok())
            .map(|e| {
                let path = e.path();
                let is_dir = path.is_dir();
                let bytes = if is_dir {
                    0
                } else {
                    e.metadata().map(|m| m.len()).unwrap_or(0)
                };
                let supported = !is_dir
                    && path
                        .extension()
                        .and_then(|e| e.to_str())
                        .is_some_and(|ext| SUPPORTED.contains(&ext.to_lowercase().as_str()));
                Entry {
                    path,
                    is_dir,
                    bytes,
                    supported,
                }
            })
            .filter(|e| {
                !e.path
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with('.'))
            })
            .collect();
        entries.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| b.supported.cmp(&a.supported))
                .then_with(|| a.path.cmp(&b.path))
        });
        self.entries = entries;
    }

    /// A left click on the file table: first click selects, a click on the
    /// already-selected row opens it — the two-step a mouse user expects.
    pub(crate) fn click(&mut self, x: u16, y: u16) -> super::app::Transition {
        let Some((area, offset)) = self.hit_rows else {
            return super::app::Transition::Stay;
        };
        // Rows start under the border and the header line.
        let top = area.y + 2;
        if x < area.x || x >= area.x + area.width || y < top || y >= area.y + area.height - 1 {
            return super::app::Transition::Stay;
        }
        let row = offset + (y - top) as usize;
        if row >= self.visible().len() {
            return super::app::Transition::Stay;
        }
        if self.cursor == row {
            return self.on_key(ratatui::crossterm::event::KeyCode::Enter);
        }
        self.cursor = row;
        super::app::Transition::Stay
    }

    /// Put the cursor on the entry named `name` in the visible view — how
    /// the inspector's Esc lands back on the file it was showing instead of
    /// on row zero.
    pub(crate) fn select(&mut self, name: &std::ffi::OsStr) {
        let target = self
            .visible()
            .into_iter()
            .position(|i| self.entries[i].path.file_name() == Some(name));
        if let Some(row) = target {
            self.cursor = row;
        }
    }

    /// Indices of entries the filter keeps, in display order.
    pub(crate) fn visible(&self) -> Vec<usize> {
        let needle = self.filter.to_lowercase();
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                needle.is_empty()
                    || e.path
                        .file_name()
                        .is_some_and(|n| n.to_string_lossy().to_lowercase().contains(&needle))
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Poll a pending file-open. A finished gather parks in `ready`; the app
    /// loop collects it with [`take_ready`](Self::take_ready) and switches to
    /// the inspector — `drain_events` itself cannot change screens.
    pub(crate) fn drain_events(&mut self) {
        let done = match &self.loading {
            Some((_, receiver)) => receiver.try_recv().ok(),
            None => None,
        };
        if let Some(result) = done {
            self.loading = None;
            match result {
                Ok(report) => self.ready = Some(report),
                Err(e) => self.status = Some(e),
            }
        }
    }

    /// The finished report of a background open, if one just landed.
    pub(crate) fn take_ready(&mut self) -> Option<FileReport> {
        self.ready.take()
    }

    /// Handle one key. Filter mode consumes characters; otherwise navigation.
    pub(crate) fn on_key(&mut self, key: KeyCode) -> Transition {
        if self.filtering {
            match key {
                KeyCode::Esc => {
                    self.filter.clear();
                    self.filtering = false;
                }
                KeyCode::Enter => self.filtering = false,
                KeyCode::Backspace => {
                    self.filter.pop();
                }
                KeyCode::Char(c) => {
                    self.filter.push(c);
                    self.cursor = 0;
                }
                _ => {}
            }
            return Transition::Stay;
        }
        let visible = self.visible();
        match key {
            KeyCode::Char('/') => self.filtering = true,
            KeyCode::Up | KeyCode::Char('k') => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => {
                self.cursor = (self.cursor + 1).min(visible.len().saturating_sub(1))
            }
            KeyCode::PageUp => self.cursor = self.cursor.saturating_sub(20),
            KeyCode::PageDown => {
                self.cursor = (self.cursor + 20).min(visible.len().saturating_sub(1))
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = visible.len().saturating_sub(1),
            KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => self.ascend(),
            KeyCode::Esc => {
                if !self.filter.is_empty() {
                    self.filter.clear();
                    self.cursor = 0;
                } else {
                    self.ascend();
                }
            }
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                let Some(&index) = visible.get(self.cursor) else {
                    return Transition::Stay;
                };
                let entry = &self.entries[index];
                if entry.is_dir {
                    self.dir = entry.path.clone();
                    self.reload();
                } else if self.loading.is_none() {
                    // Gather on a worker: big files must not freeze the frame.
                    let (sender, receiver) = mpsc::channel();
                    let path = entry.path.clone();
                    self.loading = Some((path.clone(), receiver));
                    self.status = None;
                    std::thread::spawn(move || {
                        let _ = sender.send(crate::model::gather(&path));
                    });
                }
            }
            _ => {}
        }
        Transition::Stay
    }

    fn ascend(&mut self) {
        if let Some(parent) = self.dir.parent() {
            self.dir = parent.to_path_buf();
            self.reload();
        }
    }
}

pub(crate) fn draw(frame: &mut Frame, area: Rect, state: &mut BrowserState) {
    let visible = state.visible();
    let mut title = format!(" {} ", state.dir.display());
    if state.filtering || !state.filter.is_empty() {
        title.push_str(&format!("/{}▏ ", state.filter));
    }
    if let Some((path, _)) = &state.loading {
        title.push_str(&format!("loading {}… ", path.display()));
    } else if let Some(status) = &state.status {
        if !status.is_empty() {
            title.push_str(&format!("— {status} "));
        }
    }

    let rows: Vec<Row> = visible
        .iter()
        .map(|&index| {
            let entry = &state.entries[index];
            let name = entry
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| entry.path.display().to_string());
            let (name, kind, size, style) = if entry.is_dir {
                (
                    format!("{name}/"),
                    "dir".to_string(),
                    String::new(),
                    Style::default().fg(Color::Blue),
                )
            } else {
                let kind = entry
                    .path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();
                let style = if entry.supported {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                (name, kind, human_bytes(entry.bytes), style)
            };
            Row::new(vec![name, kind, size]).style(style)
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Min(24),
            Constraint::Length(10),
            Constraint::Length(9),
        ],
    )
    .header(Row::new(vec!["name", "kind", "size"]).style(Style::default().fg(Color::DarkGray)))
    .block(Block::default().borders(Borders::ALL).title(title))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    let mut table_state = TableState::default().with_selected(Some(state.cursor));
    frame.render_stateful_widget(table, area, &mut table_state);
    // A fresh TableState scrolls just enough to show the selection: the
    // offset a click must subtract is exactly that formula.
    let rows_visible = area.height.saturating_sub(3) as usize; // borders + header
    let offset = (state.cursor + 1).saturating_sub(rows_visible.max(1));
    state.hit_rows = Some((area, offset));

    if visible.is_empty() {
        let message = if state.filter.is_empty() {
            "empty directory".to_string()
        } else {
            format!("nothing matches /{}", state.filter)
        };
        let inner = Rect {
            x: area.x + 2,
            y: area.y + 2,
            width: area.width.saturating_sub(4),
            height: 1,
        };
        frame.render_widget(
            ratatui::widgets::Paragraph::new(Line::from(message))
                .style(Style::default().fg(Color::DarkGray)),
            inner,
        );
    }
}

pub(crate) fn human_bytes(bytes: u64) -> String {
    match bytes {
        0..=1023 => format!("{bytes} B"),
        1024..=1048575 => format!("{:.1} KB", bytes as f64 / 1024.0),
        _ => format!("{:.1} MB", bytes as f64 / 1048576.0),
    }
}
