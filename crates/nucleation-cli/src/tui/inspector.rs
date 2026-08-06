//! The inspector: one file's [`FileReport`] across three tabs.

use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Row, Table, Tabs};
use ratatui::Frame;

use super::app::{Screen, Transition};
use super::browser::{human_bytes, BrowserState};
use crate::model::FileReport;

pub(crate) struct InspectorState {
    pub(crate) report: FileReport,
    /// Which case the test tab is showing, for suites with several.
    pub(crate) case_cursor: usize,
    /// 3D view orbit — yaw/pitch in radians, zoom 1.0 = fit.
    pub(crate) yaw: f32,
    pub(crate) pitch: f32,
    pub(crate) zoom: f32,
    /// Right-drag pan, as a **world-space** offset of the orbit target.
    /// Converted from screen cells at drag time against the camera basis of
    /// that moment — stored in world units so a panned view stays put while
    /// the camera orbits, instead of swinging with it.
    pub(crate) pan: [f32; 3],
    /// The encoded frame for the current (orbit, area) — kept until either
    /// changes so a redraw is a no-op, not a re-render.
    pub(crate) view: Option<(ViewKey, ratatui_image::protocol::StatefulProtocol)>,
    /// The meshed GPU pipeline's state, when this build was compiled with
    /// the `render` feature and a resource pack is discoverable.
    #[cfg(feature = "render")]
    pub(crate) gpu: GpuMesh,
    /// A live protocol override (`v` cycles it). Terminals sometimes claim
    /// a protocol they do not render — the probe cannot see the screen, but
    /// the user can, so the user gets the knob.
    pub(crate) protocol: Option<ratatui_image::picker::ProtocolType>,
    /// 0 = Overview, 1 = Entities, 2 = Test, 3 = View.
    pub(crate) tab: usize,
    pub(crate) scroll: [u16; 3],
    /// Where the tab bar, the test tab's case list, and the 3D panes landed
    /// last frame — what a mouse click is resolved against.
    pub(crate) hit_tabs: Rect,
    pub(crate) hit_cases: Option<Rect>,
    pub(crate) hit_view: Rect,
    /// Set when the image engine or protocol swapped: the next frame needs
    /// a full terminal clear to evict the previous image's ghost.
    pub(crate) hard_clear: bool,
}

impl InspectorState {
    pub(crate) fn new(report: FileReport) -> Self {
        #[cfg(feature = "render")]
        let gpu = GpuMesh::start(&report.path);
        Self {
            report,
            case_cursor: 0,
            yaw: 0.8,
            pitch: 0.5,
            zoom: 1.0,
            pan: [0.0, 0.0, 0.0],
            view: None,
            #[cfg(feature = "render")]
            gpu,
            protocol: None,
            tab: 0,
            scroll: [0, 0, 0],
            hit_tabs: Rect::default(),
            hit_cases: None,
            hit_view: Rect::default(),
            hard_clear: false,
        }
    }

    /// One-shot read of the hard-clear request.
    pub(crate) fn take_hard_clear(&mut self) -> bool {
        std::mem::take(&mut self.hard_clear)
    }

    /// Poll the background mesher; invalidate the frame cache when the mesh
    /// arrives so the next draw upgrades to the GPU path.
    #[cfg(feature = "render")]
    pub(crate) fn drain_events(&mut self) {
        if self.gpu.poll() {
            self.view = None;
            self.hard_clear = true;
        }
    }

    #[cfg(not(feature = "render"))]
    pub(crate) fn drain_events(&mut self) {}

    pub(crate) fn on_key(&mut self, key: KeyCode) -> Transition {
        match key {
            KeyCode::Esc => {
                let dir = self
                    .report
                    .path
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| ".".into());
                let mut browser = BrowserState::new(dir);
                // Land on the file we were inspecting, not on row zero.
                if let Some(name) = self.report.path.file_name() {
                    browser.select(name);
                }
                return Transition::To(Screen::Browser(browser));
            }
            KeyCode::Char('1') => self.tab = 0,
            KeyCode::Char('2') => self.tab = 1,
            KeyCode::Char('3') => self.tab = 2,
            KeyCode::Char('4') => self.tab = 3,
            // On the view tab the arrows orbit; everywhere else they page
            // between tabs and scroll.
            KeyCode::Left if self.tab == 3 => self.orbit(-0.2, 0.0),
            KeyCode::Right if self.tab == 3 => self.orbit(0.2, 0.0),
            KeyCode::Up if self.tab == 3 => self.orbit(0.0, 0.12),
            KeyCode::Down if self.tab == 3 => self.orbit(0.0, -0.12),
            KeyCode::Char('+') | KeyCode::Char('=') if self.tab == 3 => {
                self.zoom = (self.zoom * 1.2).min(16.0);
                self.view = None;
            }
            KeyCode::Char('-') if self.tab == 3 => {
                self.zoom = (self.zoom / 1.2).max(0.2);
                self.view = None;
            }
            KeyCode::Char('0') if self.tab == 3 => {
                (self.yaw, self.pitch, self.zoom) = (0.8, 0.5, 1.0);
                self.pan = [0.0, 0.0, 0.0];
                self.view = None;
            }
            // Cycle the image protocol: the probe's answer is a claim, the
            // eye is the test. One keypress reaches half-blocks, which
            // render everywhere.
            KeyCode::Char('v') if self.tab == 3 => {
                let current = self.protocol.unwrap_or_else(|| picker().protocol_type());
                self.protocol = Some(current.next());
                self.view = None;
                self.hard_clear = true;
            }
            KeyCode::Left => self.tab = self.tab.saturating_sub(1),
            KeyCode::Right => self.tab = (self.tab + 1).min(3),
            // On the test tab, n/p walk the suite's cases.
            KeyCode::Char('n') if self.tab == 2 => {
                let cases = self.case_count();
                if cases > 0 {
                    self.case_cursor = (self.case_cursor + 1) % cases;
                    self.scroll[2] = 0;
                }
            }
            KeyCode::Char('p') if self.tab == 2 => {
                let cases = self.case_count();
                if cases > 0 {
                    self.case_cursor = (self.case_cursor + cases - 1) % cases;
                    self.scroll[2] = 0;
                }
            }
            KeyCode::Up | KeyCode::Char('k') if self.tab < 3 => {
                self.scroll[self.tab] = self.scroll[self.tab].saturating_sub(1)
            }
            KeyCode::Down | KeyCode::Char('j') if self.tab < 3 => {
                self.scroll[self.tab] = self.scroll[self.tab].saturating_add(1)
            }
            KeyCode::PageUp if self.tab < 3 => {
                self.scroll[self.tab] = self.scroll[self.tab].saturating_sub(20)
            }
            KeyCode::PageDown if self.tab < 3 => {
                self.scroll[self.tab] = self.scroll[self.tab].saturating_add(20)
            }
            KeyCode::Home if self.tab < 3 => self.scroll[self.tab] = 0,
            _ => {}
        }
        Transition::Stay
    }

    fn case_count(&self) -> usize {
        self.report
            .embedded_test
            .as_ref()
            .map_or(0, |t| t.case_views.len())
    }

    /// Whether a mouse position sits on a 3D pane — the gate for drag-orbit
    /// and wheel-zoom.
    pub(crate) fn over_view(&self, x: u16, y: u16) -> bool {
        let r = self.hit_view;
        r.width > 0 && x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height
    }

    /// A mouse drag on the view pane: cells travelled → orbit radians.
    /// Grab-the-model sense: dragging right pulls the build rightward.
    pub(crate) fn orbit_drag(&mut self, dx: f32, dy: f32) {
        self.orbit(-dx * 0.03, -dy * 0.05);
    }

    /// A right-drag pan: cells travelled, converted to a world offset using
    /// the camera basis *now* and frozen — the build follows the cursor, and
    /// a later orbit revolves around wherever it was left.
    pub(crate) fn pan_drag(&mut self, dx_cells: f32, dy_cells: f32) {
        let cols = f32::from(self.hit_view.width.max(1));
        let rows = f32::from(self.hit_view.height.max(1));
        let radius = self
            .report
            .voxels
            .as_ref()
            .map(|grid| {
                let (dx, dy, dz) = grid.dims;
                ((dx * dx + dy * dy + dz * dz) as f32).sqrt() / 2.0
            })
            .unwrap_or(16.0);
        let span = 2.2 * radius / self.zoom.max(0.05);
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let forward = [cy * cp, -sp, sy * cp];
        let right = [-sy, 0.0, cy];
        let up = [
            right[1] * forward[2] - right[2] * forward[1],
            right[2] * forward[0] - right[0] * forward[2],
            right[0] * forward[1] - right[1] * forward[0],
        ];
        let step_x = -dx_cells / cols * span;
        let step_y = dy_cells / rows * span;
        for axis in 0..3 {
            self.pan[axis] += right[axis] * step_x + up[axis] * step_y;
        }
        self.view = None;
    }

    /// A wheel step over the view pane.
    pub(crate) fn zoom_by(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(0.2, 16.0);
        self.view = None;
    }

    /// A left click, resolved against last frame's layout: the tab bar
    /// switches tabs, the test tab's case list selects a case.
    pub(crate) fn click(&mut self, x: u16, y: u16) {
        let tabs = self.hit_tabs;
        if y == tabs.y && x >= tabs.x {
            // The Tabs widget lays titles out as ` title ` around a one-cell
            // divider; walk the same arithmetic.
            let titles = ["1 overview", "2 entities", "3 test", "4 view"];
            let mut cursor = tabs.x + 1;
            for (index, title) in titles.iter().enumerate() {
                let end = cursor + title.len() as u16 + 1;
                if x >= cursor && x < end {
                    self.tab = index;
                    return;
                }
                cursor = end + 2;
            }
            return;
        }
        if let Some(cases) = self.hit_cases {
            if self.tab == 2
                && x >= cases.x
                && x < cases.x + cases.width
                && y > cases.y
                && y < cases.y + cases.height.saturating_sub(1)
            {
                let row = (y - cases.y - 1) as usize;
                if row < self.case_count() {
                    self.case_cursor = row;
                    self.scroll[2] = 0;
                }
            }
        }
    }

    fn orbit(&mut self, dyaw: f32, dpitch: f32) {
        self.yaw += dyaw;
        self.pitch = (self.pitch + dpitch).clamp(-1.45, 1.45);
        self.view = None;
    }
}

/// What the cached frame was rendered for: quantised orbit plus the pane
/// size in cells. Any difference forces a re-render.
#[derive(PartialEq, Clone, Copy)]
pub(crate) struct ViewKey {
    yaw_q: i32,
    pitch_q: i32,
    zoom_q: i32,
    pan_qx: i32,
    pan_qy: i32,
    pan_qz: i32,
    cols: u16,
    rows: u16,
    /// Which engine drew the cached frame — a mesh arriving mid-session
    /// must invalidate raycast frames.
    gpu: bool,
    /// Which protocol encoded it — `v` re-encodes through the next one.
    proto: u8,
}

/// The terminal's image protocol, probed once per process: Kitty, iTerm2 or
/// Sixel when the terminal answers, unicode half-blocks (the "ascii art"
/// fallback) when it does not. `NUCLEATION_TUI_PROTOCOL` (halfblocks /
/// sixel / kitty / iterm2) overrides the probe for terminals that lie.
static PICKER: std::sync::OnceLock<ratatui_image::picker::Picker> = std::sync::OnceLock::new();

/// Probe now — called by the app before the event loop starts consuming
/// stdin, because the probe reads the terminal's reply from stdin.
pub(crate) fn init_picker() {
    PICKER.get_or_init(|| {
        let mut picker = ratatui_image::picker::Picker::from_query_stdio()
            .unwrap_or_else(|_| ratatui_image::picker::Picker::from_fontsize((8, 16)));
        // Some terminals answer the pixel-size query with nonsense (VS Code
        // reports numbers that put a "font" at several dozen pixels), which
        // both blows the render budget and shrinks the drawn area. Clamp to
        // plausible glyph metrics; `Resize::Scale` covers any residual
        // mismatch by filling the pane regardless.
        let (fw, fh) = picker.font_size();
        if !(4..=24).contains(&fw) || !(8..=48).contains(&fh) {
            let protocol = picker.protocol_type();
            picker = ratatui_image::picker::Picker::from_fontsize((8, 16));
            picker.set_protocol_type(protocol);
        }
        // JetBrains' JediTerm (IDE terminals) answers protocol queries it
        // does not fully render inside alternate-screen apps; half-blocks
        // are the honest default there. `v` or the env var reach the rest.
        if std::env::var("TERMINAL_EMULATOR").is_ok_and(|t| t.contains("JetBrains")) {
            picker.set_protocol_type(ratatui_image::picker::ProtocolType::Halfblocks);
        }
        if let Ok(want) = std::env::var("NUCLEATION_TUI_PROTOCOL") {
            use ratatui_image::picker::ProtocolType;
            let forced = match want.to_lowercase().as_str() {
                "halfblocks" | "blocks" | "ascii" => Some(ProtocolType::Halfblocks),
                "sixel" => Some(ProtocolType::Sixel),
                "kitty" => Some(ProtocolType::Kitty),
                "iterm2" | "iterm" => Some(ProtocolType::Iterm2),
                _ => None,
            };
            if let Some(forced) = forced {
                picker.set_protocol_type(forced);
            }
        }
        picker
    });
}

/// The meshed GPU pipeline: mesh once on a worker (a pack load plus
/// meshing can take seconds), then render frames on demand. Every failure
/// falls back to the raycaster rather than blanking the pane.
#[cfg(feature = "render")]
pub(crate) enum GpuMesh {
    /// No pack found (or the file failed to load) — raycast only.
    Absent,
    /// The worker is meshing; the receiver delivers exactly one message.
    Meshing(std::sync::mpsc::Receiver<Result<nucleation::meshing::MeshOutput, String>>),
    /// Meshed and ready to render.
    Ready(Box<nucleation::meshing::MeshOutput>),
    /// Meshing or a GPU frame failed — the reason shows in the title.
    Failed(String),
}

#[cfg(feature = "render")]
impl GpuMesh {
    fn start(path: &std::path::Path) -> Self {
        let Some(pack) = crate::commands::pack::discover_pack() else {
            return Self::Absent;
        };
        let path = path.to_path_buf();
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = (|| -> Result<nucleation::meshing::MeshOutput, String> {
                let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
                let manager = nucleation::formats::manager::get_manager();
                let schematic = {
                    let manager = manager.lock().map_err(|e| e.to_string())?;
                    manager.read(&bytes).map_err(|e| format!("{e:?}"))?
                };
                let source =
                    nucleation::meshing::ResourcePackSource::from_file(&pack.display().to_string())
                        .map_err(|e| format!("pack: {e:?}"))?;
                schematic
                    .to_mesh(&source, &nucleation::meshing::MeshConfig::default())
                    .map_err(|e| format!("meshing: {e:?}"))
            })();
            let _ = sender.send(result);
        });
        Self::Meshing(receiver)
    }

    /// Returns `true` when the state changed (a frame cache must drop).
    fn poll(&mut self) -> bool {
        let Self::Meshing(receiver) = self else {
            return false;
        };
        match receiver.try_recv() {
            Ok(Ok(mesh)) => {
                *self = Self::Ready(Box::new(mesh));
                true
            }
            Ok(Err(why)) => {
                *self = Self::Failed(why);
                true
            }
            Err(_) => false,
        }
    }
}

fn picker() -> &'static ratatui_image::picker::Picker {
    // A non-TTY consumer (tests, the text fallback) never called init;
    // half-blocks render everywhere.
    PICKER.get_or_init(|| ratatui_image::picker::Picker::from_fontsize((8, 16)))
}

pub(crate) fn draw(frame: &mut Frame, area: Rect, state: &mut InspectorState) {
    let [title_bar, tab_bar, body] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .areas(area);

    let report = &state.report;
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                report.path.display().to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {} · {}", report.format, human_bytes(report.bytes)),
                Style::default().fg(Color::DarkGray),
            ),
        ])),
        title_bar,
    );
    let tabs = Tabs::new(vec!["1 overview", "2 entities", "3 test", "4 view"])
        .select(state.tab)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, tab_bar);
    state.hit_tabs = tab_bar;
    state.hit_cases = None;
    state.hit_view = Rect::default();

    match state.tab {
        0 => draw_overview(frame, body, state),
        1 => draw_entities(frame, body, state),
        2 => draw_test(frame, body, state),
        _ => draw_view(frame, body, state),
    }
}

fn draw_overview(frame: &mut Frame, area: Rect, state: &mut InspectorState) {
    let report = &state.report;
    let [meta_area, right_area] =
        Layout::horizontal([Constraint::Percentage(42), Constraint::Percentage(58)]).areas(area);
    // The build itself belongs on the front page: palette above, a live 3D
    // preview below (tab 4 is the full-pane version of the same camera).
    let [palette_area, preview_area] = if state.report.voxels.is_some() {
        Layout::vertical([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(right_area)
    } else {
        Layout::vertical([Constraint::Percentage(100), Constraint::Percentage(0)]).areas(right_area)
    };

    let label = Style::default().fg(Color::DarkGray);
    let (w, h, l) = report.dimensions;
    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("size      ", label),
            Span::raw(format!("{w} × {h} × {l}")),
        ]),
        Line::from(vec![
            Span::styled("blocks    ", label),
            Span::raw(format!(
                "{} of {} volume",
                report.total_blocks, report.total_volume
            )),
        ]),
    ];
    if let Some(name) = &report.name {
        lines.push(Line::from(vec![
            Span::styled("name      ", label),
            Span::raw(name.clone()),
        ]));
    }
    if let Some(author) = &report.author {
        lines.push(Line::from(vec![
            Span::styled("author    ", label),
            Span::raw(author.clone()),
        ]));
    }
    if let Some(description) = &report.description {
        lines.push(Line::from(vec![
            Span::styled("descr     ", label),
            Span::raw(description.clone()),
        ]));
    }
    if let Some(dv) = report.data_version {
        lines.push(Line::from(vec![
            Span::styled("data ver  ", label),
            Span::raw(dv.to_string()),
        ]));
    }
    let test_line = match &report.embedded_test {
        Some(test) if test.parse_error.is_some() => Span::styled(
            "embedded, unreadable — see tab 3",
            Style::default().fg(Color::Red),
        ),
        Some(test) => Span::styled(
            format!("{} case(s) — see tab 3", test.cases),
            Style::default().fg(Color::Green),
        ),
        None => Span::styled("none", label),
    };
    lines.push(Line::from(vec![
        Span::styled("test      ", label),
        test_line,
    ]));
    lines.push(Line::from(vec![
        Span::styled("entities  ", label),
        Span::raw(format!(
            "{} mobile · {} block entities",
            report.entities.len(),
            report.block_entities.len()
        )),
    ]));
    // Composition at a glance: how full the box is, how varied the build is,
    // and what kinds of machinery it carries.
    if report.total_volume > 0 {
        lines.push(Line::from(vec![
            Span::styled("density   ", label),
            Span::raw(format!(
                "{:.1}% of the bounding box · {} distinct state(s)",
                report.total_blocks as f64 * 100.0 / report.total_volume as f64,
                report.palette.len()
            )),
        ]));
    }
    let kind_counts = |items: &[String]| -> Vec<(String, usize)> {
        let mut counts: Vec<(String, usize)> = Vec::new();
        for item in items {
            let kind = item.split(" @").next().unwrap_or(item).to_string();
            match counts.iter_mut().find(|(k, _)| *k == kind) {
                Some((_, n)) => *n += 1,
                None => counts.push((kind, 1)),
            }
        }
        counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        counts
    };
    let summarise = |counts: &[(String, usize)]| -> String {
        let mut shown: Vec<String> = counts
            .iter()
            .take(4)
            .map(|(kind, n)| {
                let short = kind.strip_prefix("minecraft:").unwrap_or(kind);
                if *n > 1 {
                    format!("{short}×{n}")
                } else {
                    short.to_string()
                }
            })
            .collect();
        if counts.len() > 4 {
            shown.push(format!("+{} more", counts.len() - 4));
        }
        shown.join(" · ")
    };
    let machinery = kind_counts(&report.block_entities);
    if !machinery.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("machinery ", label),
            Span::raw(summarise(&machinery)),
        ]));
    }
    let mobs = kind_counts(&report.entities);
    if !mobs.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("mobile    ", label),
            Span::raw(summarise(&mobs)),
        ]));
    }
    if report.regions.len() > 1 {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("regions ({})", report.regions.len()),
            label,
        )));
        for region in &report.regions {
            let (w, h, l) = region.dimensions;
            lines.push(Line::from(format!(
                "  {}  {w}×{h}×{l}  {} blocks",
                region.name, region.blocks
            )));
        }
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" overview "))
            .scroll((state.scroll[0], 0)),
        meta_area,
    );

    // Palette with proportional bars: the build's composition at a glance.
    let max = report.palette.first().map(|(_, n)| *n).unwrap_or(1).max(1);
    let rows: Vec<Row> = report
        .palette
        .iter()
        .skip(state.scroll[0] as usize)
        .map(|(descriptor, count)| {
            let bar_len = ((count * 12) / max).max(1);
            Row::new(vec![
                Span::styled(format!("{count:>7}"), Style::default().fg(Color::DarkGray)),
                Span::styled("▇".repeat(bar_len), Style::default().fg(Color::Cyan)),
                Span::raw(descriptor.clone()),
            ])
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(13),
            Constraint::Min(10),
        ],
    )
    .block(Block::default().borders(Borders::ALL).title(format!(
        " palette ({} states, ↑↓ scrolls) ",
        report.palette.len()
    )));
    frame.render_widget(table, palette_area);

    if state.report.voxels.is_some() {
        let preview = Block::default()
            .borders(Borders::ALL)
            .title(" preview · tab 4 for the full view ");
        let inner = preview.inner(preview_area);
        frame.render_widget(preview, preview_area);
        render_view_pane(frame, inner, state);
    }
}

fn draw_entities(frame: &mut Frame, area: Rect, state: &InspectorState) {
    let report = &state.report;
    let [entities_area, block_entities_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
    let scroll = state.scroll[1];

    let entity_lines: Vec<Line> = report
        .entities
        .iter()
        .map(|e| Line::from(e.clone()))
        .collect();
    frame.render_widget(
        Paragraph::new(entity_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" entities ({}) ", report.entities.len())),
            )
            .scroll((scroll, 0)),
        entities_area,
    );

    let be_lines: Vec<Line> = report
        .block_entities
        .iter()
        .map(|e| Line::from(e.clone()))
        .collect();
    frame.render_widget(
        Paragraph::new(be_lines)
            .block(Block::default().borders(Borders::ALL).title(format!(
                " block entities ({}) ",
                report.block_entities.len()
            )))
            .scroll((scroll, 0)),
        block_entities_area,
    );
}

fn draw_test(frame: &mut Frame, area: Rect, state: &mut InspectorState) {
    let report = &state.report;
    let Some(test) = &report.embedded_test else {
        frame.render_widget(
            Paragraph::new(
                "no NucleationTest tag in this file\n\nattach one with:\n  nucleation port, or embed a spec at save time",
            )
            .block(Block::default().borders(Borders::ALL).title(" embedded test ")),
            area,
        );
        return;
    };
    if let Some(error) = &test.parse_error {
        frame.render_widget(
            Paragraph::new(error.clone())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" embedded test — UNREADABLE "),
                )
                .scroll((state.scroll[2], 0)),
            area,
        );
        return;
    }
    let cases = &test.case_views;
    if cases.is_empty() {
        frame.render_widget(
            Paragraph::new(test.pretty.clone().unwrap_or_else(|| test.names.join("\n")))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" embedded test "),
                )
                .scroll((state.scroll[2], 0)),
            area,
        );
        return;
    }
    let cursor = state.case_cursor.min(cases.len() - 1);
    let body = if cases.len() > 1 {
        // A suite: case list on the left, the selected case on the right.
        let [list_area, detail_area] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
                .areas(area);
        let items: Vec<ListItem> = cases
            .iter()
            .map(|case| ListItem::new(case.name.clone()))
            .collect();
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} case(s) · n/p to switch ", cases.len())),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        let mut list_state = ListState::default().with_selected(Some(cursor));
        frame.render_stateful_widget(list, list_area, &mut list_state);
        state.hit_cases = Some(list_area);
        detail_area
    } else {
        area
    };
    let case = &cases[cursor];
    frame.render_widget(
        Paragraph::new(case.text.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ", case.name)),
            )
            .scroll((state.scroll[2], 0)),
        body,
    );
}

/// The 3D preview: orbit with the arrows, `+`/`-` zoom, `0` reset. Rendered
/// through whichever image protocol the terminal speaks; unicode
/// half-blocks when it speaks none.
fn draw_view(frame: &mut Frame, area: Rect, state: &mut InspectorState) {
    let Some(grid) = &state.report.voxels else {
        frame.render_widget(
            Paragraph::new(
                "no preview grid for this file\n\nthe build is over the preview cell cap \
                 (or empty); everything else in the inspector still works",
            )
            .block(Block::default().borders(Borders::ALL).title(" 3d view ")),
            area,
        );
        return;
    };
    let engine = {
        #[cfg(feature = "render")]
        {
            match &state.gpu {
                GpuMesh::Ready(_) => "gpu·meshed".to_string(),
                GpuMesh::Meshing(_) => "meshing… (raycast meanwhile)".to_string(),
                GpuMesh::Failed(why) => {
                    format!(
                        "raycast (gpu failed: {})",
                        why.chars().take(40).collect::<String>()
                    )
                }
                GpuMesh::Absent => "raycast (no pack — set NUCLEATION_PACK)".to_string(),
            }
        }
        #[cfg(not(feature = "render"))]
        "raycast (build with --features render for the gpu path)".to_string()
    };
    let block = Block::default().borders(Borders::ALL).title(format!(
        " 3d view · {}×{}×{} · {engine} · {:?} (v cycles) · ←→↑↓ orbit · +/- zoom · 0 reset ",
        grid.dims.0,
        grid.dims.1,
        grid.dims.2,
        state.protocol.unwrap_or_else(|| picker().protocol_type())
    ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }
    render_view_pane(frame, inner, state);
}

/// The shared preview body: cache per (orbit, pane), raycast a frame,
/// display through the probed protocol. The overview's corner preview and
/// the full view tab both come here, so they can never disagree.
fn render_view_pane(frame: &mut Frame, inner: Rect, state: &mut InspectorState) {
    if inner.width == 0 || inner.height == 0 {
        return;
    }
    state.hit_view = inner;
    let gpu_ready = {
        #[cfg(feature = "render")]
        {
            matches!(state.gpu, GpuMesh::Ready(_))
        }
        #[cfg(not(feature = "render"))]
        false
    };
    if state.report.voxels.is_none() && !gpu_ready {
        return;
    }
    let active_protocol = state.protocol.unwrap_or_else(|| picker().protocol_type());
    let key = ViewKey {
        yaw_q: (state.yaw * 100.0) as i32,
        pitch_q: (state.pitch * 100.0) as i32,
        zoom_q: (state.zoom * 100.0) as i32,
        pan_qx: (state.pan[0] * 100.0) as i32,
        pan_qy: (state.pan[1] * 100.0) as i32,
        pan_qz: (state.pan[2] * 100.0) as i32,
        cols: inner.width,
        rows: inner.height,
        gpu: gpu_ready,
        proto: active_protocol as u8,
    };
    let stale = !matches!(&state.view, Some((have, _)) if *have == key);
    if stale {
        // Render at exactly the pane's pixel budget — `Resize::Fit` never
        // upscales, so an image smaller than the pane parks in a corner.
        // Half-blocks resolve 1×2 pixels per cell; pixel protocols get the
        // full cell size, capped so a huge display stays interactive.
        let halfblocks = matches!(
            active_protocol,
            ratatui_image::picker::ProtocolType::Halfblocks
        );
        let (width, height) = if halfblocks {
            (u32::from(inner.width), u32::from(inner.height) * 2)
        } else {
            let font = picker().font_size();
            let width = u32::from(inner.width) * u32::from(font.0);
            let height = u32::from(inner.height) * u32::from(font.1);
            let scale = (1200.0 / width.max(height) as f32).min(1.0);
            (
                ((width as f32 * scale) as u32),
                ((height as f32 * scale) as u32),
            )
        };
        let width = width.max(64);
        let height = height.max(48);
        let mut frame_image: Option<image::DynamicImage> = None;
        #[cfg(feature = "render")]
        if let GpuMesh::Ready(mesh) = &state.gpu {
            // The pan is already a world offset (frozen at drag time), so
            // the orbit target is simply the mesh centre plus it — the same
            // addition the raycaster performs.
            let (bmin, bmax) =
                nucleation::rendering::camera::merged_bounds(std::slice::from_ref(mesh));
            let target = [
                (bmin[0] + bmax[0]) / 2.0 + state.pan[0],
                (bmin[1] + bmax[1]) / 2.0 + state.pan[1],
                (bmin[2] + bmax[2]) / 2.0 + state.pan[2],
            ];
            // The engine's own orbit is degrees-based; ours is radians.
            let config = nucleation::rendering::RenderConfig {
                width,
                height,
                yaw: state.yaw.to_degrees(),
                pitch: state.pitch.to_degrees(),
                zoom: state.zoom,
                target: Some(target),
                background: Some([0.0, 0.0, 0.0, 0.0]),
                // Silhouette fit, not the rotation sphere: sphere fit kept a
                // constant orbit distance at the price of the build floating
                // small in a sea of margin.
                sphere_fit: false,
                // Perspective (RenderConfig::default), not the isometric
                // ortho: flat projection reads as a skewed diagram.
                ..nucleation::rendering::RenderConfig::default()
            };
            match nucleation::rendering::render_meshes_png(
                std::slice::from_ref(mesh),
                &config,
                None,
            )
            .map_err(|e| format!("{e:?}"))
            .and_then(|png| image::load_from_memory(&png).map_err(|e| e.to_string()))
            {
                Ok(rendered) => frame_image = Some(rendered),
                Err(why) => state.gpu = GpuMesh::Failed(why),
            }
        }
        if frame_image.is_none() {
            if let Some(grid) = &state.report.voxels {
                frame_image = Some(image::DynamicImage::ImageRgba8(super::voxel::render(
                    grid,
                    state.yaw,
                    state.pitch,
                    state.zoom,
                    state.pan,
                    width,
                    height,
                )));
            }
        }
        if let Some(frame_image) = frame_image {
            let mut chosen = picker().clone();
            chosen.set_protocol_type(active_protocol);
            let protocol = chosen.new_resize_protocol(frame_image);
            state.view = Some((key, protocol));
        }
    }
    if let Some((_, protocol)) = &mut state.view {
        // Scrub the cells first: swapping protocols mid-session (the mesh
        // arriving, or `v`) must not leave the previous image's residue
        // under the new escape sequences.
        frame.render_widget(ratatui::widgets::Clear, inner);
        // `Scale`, not the default `Fit`: Fit refuses to upscale, so any
        // terminal that misreports its font size (VS Code answers pixel
        // queries badly) parked the frame small in a corner. We render at
        // the pane's aspect already, so scaling to fill distorts nothing.
        frame.render_stateful_widget(
            ratatui_image::StatefulImage::<ratatui_image::protocol::StatefulProtocol>::default()
                .resize(ratatui_image::Resize::Scale(None)),
            inner,
            protocol,
        );
    }
}
