//! Text report and an interactive view of the same observed game ticks.
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use serde_json::Value;
use std::{io, path::Path, time::Duration};

#[derive(Clone, Copy)]
pub(super) struct Sample {
    pub filled: usize,
    pub writes: usize,
    pub pistons: usize,
    pub quiet: bool,
    pub moving: bool,
}

const LEVELS: [char; 9] = ['·', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

fn level(n: usize, max: usize) -> char {
    LEVELS[if n == 0 {
        0
    } else {
        (n * 8).div_ceil(max.max(1)).min(8)
    }]
}

// Bucket maxima retain short activity spikes on narrow displays.
fn spark(samples: &[Sample], width: usize, max: usize, value: impl Fn(&Sample) -> usize) -> String {
    let cols = width.min(samples.len());
    (0..cols)
        .map(|x| {
            let lo = x * samples.len() / cols;
            let hi = (x + 1) * samples.len() / cols;
            level(samples[lo..hi].iter().map(&value).max().unwrap_or(0), max)
        })
        .collect()
}
/// Expand short runs to use the available columns; take the peak when several
/// ticks share a column. The inspector below the plot still uses the exact tick.
fn buckets(samples: &[Sample], cols: usize, value: impl Fn(&Sample) -> usize) -> Vec<usize> {
    (0..cols)
        .map(|x| {
            let lo = x * samples.len() / cols;
            let hi = ((x + 1) * samples.len() / cols).max(lo + 1);
            samples[lo..hi].iter().map(&value).max().unwrap_or(0)
        })
        .collect()
}
fn track(
    label: String,
    values: impl Iterator<Item = char>,
    cursor: usize,
    style: Style,
) -> Line<'static> {
    let mut spans = vec![Span::raw(format!("{label:<9}"))];
    let glyphs: Vec<_> = values.collect();
    spans.push(Span::styled(
        glyphs[..cursor].iter().collect::<String>(),
        style,
    ));
    spans.push(Span::styled(
        glyphs[cursor].to_string(),
        style.add_modifier(Modifier::REVERSED),
    ));
    spans.push(Span::styled(
        glyphs[cursor + 1..].iter().collect::<String>(),
        style,
    ));
    Line::from(spans)
}
fn action(p: &Value) -> String {
    match p["aperture"]["timing"]["action"].as_str() {
        Some("open") => "Open".into(),
        Some("close") => "Close".into(),
        _ => format!("Use {}", p["use"]),
    }
}
fn metric(v: &Value) -> String {
    v.as_u64().map_or_else(|| "—".into(), |n| n.to_string())
}
fn timing_row(p: &Value) -> String {
    let t = &p["aperture"]["timing"];
    format!(
        "{:<7}{:>8}{:>8}{:>8}{:>9}",
        action(p),
        metric(&t["delay_ticks"]),
        metric(&t["aperture_first_ticks"]),
        metric(&t["aperture_stable_ticks"]),
        metric(&p["mechanism_settled_ticks"])
    )
}
const HEADER: &str = "Phase     Delay   First  Stable  Settled";
fn coords(v: &Value) -> String {
    v.as_array()
        .map(|a| a.iter().map(Value::to_string).collect::<Vec<_>>().join(","))
        .unwrap_or_else(|| "—".into())
}
fn bounds(r: &Value) -> String {
    let b = &r["aperture_bounds"];
    if b.is_null() {
        return format!(
            "{} (use --size or --corner)",
            r["aperture_selection"].as_str().unwrap_or("unknown")
        );
    }
    format!("{} to {}", coords(&b[0]), coords(&b[1]))
}
fn frequency(r: &Value) -> Vec<String> {
    let f = &r["frequency"];
    if f.is_null() {
        return vec![];
    }
    let mut out = vec![];
    for (field, name) in [
        ("fastest", "Fastest"),
        ("fastest_evenly_spaced", "Even inputs"),
    ] {
        let v = &f[field];
        out.push(if let Some(hz) = v["cycles_per_second"].as_f64() {
            format!(
                "{name}  {} gt/cycle · {hz:.3} Hz · open {} / close {} gt",
                v["period_ticks"], v["open_dwell_ticks"], v["close_dwell_ticks"]
            )
        } else {
            format!(
                "{name}  No verified rate ({})",
                f["status"].as_str().unwrap_or("unknown")
            )
        });
    }
    out
}

pub(super) struct View<'a> {
    pub path: &'a Path,
    pub report: &'a Value,
    pub traces: &'a [Vec<Sample>],
    pub color: bool,
}
impl View<'_> {
    fn title(&self) -> String {
        self.path
            .file_name()
            .unwrap_or(self.path.as_os_str())
            .to_string_lossy()
            .replace(char::is_control, "�")
    }
    fn phases(&self) -> &[Value] {
        self.report["phases"].as_array().unwrap()
    }
    fn active_end(&self, index: usize) -> usize {
        let p = &self.phases()[index];
        let total = self.traces[index].len() - 1;
        p["mechanism_settled_ticks"]
            .as_u64()
            .map_or(total, |t| (t as usize + 2).min(total))
    }
    fn cells(&self, index: usize) -> usize {
        self.phases()[index]["aperture"]["cells"]
            .as_u64()
            .unwrap_or(0) as usize
    }
    fn paint(&self, s: &str, code: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{s}\x1b[0m")
        } else {
            s.to_owned()
        }
    }
    pub fn print(&self, width: usize) {
        println!("{}", self.paint(&self.title(), "1"));
        println!(
            "Input {}  ·  Aperture {}",
            coords(&self.report["input"]),
            bounds(self.report)
        );
        println!("\n{}", self.paint("Timing · game ticks after input", "1"));
        println!("{HEADER}");
        for p in self.phases() {
            println!(
                "{}",
                self.paint(
                    &timing_row(p),
                    if action(p) == "Open" { "36" } else { "33" }
                )
            )
        }
        let end = (0..self.phases().len())
            .map(|i| self.active_end(i))
            .max()
            .unwrap_or(0);
        let known = self.cells(0) > 0;
        let label = if known {
            "Aperture occupancy"
        } else {
            "Block writes"
        };
        println!("\n{}  +0 … +{end} gt", self.paint(label, "1"));
        let max = if known {
            self.cells(0)
        } else {
            self.traces
                .iter()
                .flatten()
                .map(|s| s.writes)
                .max()
                .unwrap_or(1)
        };
        for (i, p) in self.phases().iter().enumerate() {
            let chart = spark(
                &self.traces[i][..=end],
                width.saturating_sub(9).clamp(8, 72),
                max,
                |s| if known { s.filled } else { s.writes },
            );
            println!(
                "{:<7} {}",
                action(p),
                self.paint(&chart, if action(p) == "Open" { "36" } else { "33" })
            );
        }
        if known {
            println!("        · empty   █ full ({} cells)", self.cells(0));
        }
        if !self.report["frequency"].is_null() {
            println!(
                "\n{}",
                self.paint(
                    &format!(
                        "Frequency · {} consecutive cycles",
                        self.report["frequency"]["cycles_requested"]
                    ),
                    "1"
                )
            );
        }
        for line in frequency(self.report) {
            println!("{line}")
        }
        println!(
            "\nBlock layout restored: {}",
            if self.report["cycle_restores_block_layout"] == true {
                "yes"
            } else {
                "no"
            }
        );
        if std::io::IsTerminal::is_terminal(&std::io::stdout()) {
            println!("Explore individual ticks with --tui");
        }
        if !known {
            super::print_panels(self.report["aperture_candidates"].as_array().unwrap());
        }
    }
    fn style(&self, c: Color) -> Style {
        if self.color {
            Style::default().fg(c)
        } else {
            Style::default()
        }
    }
    fn draw(&self, f: &mut Frame, state: &Cursor) {
        let area = f.area();
        if area.width < 41 || area.height < 16 {
            f.render_widget(
                Paragraph::new("Door timeline · enlarge to 41×16\nq quit")
                    .wrap(Wrap { trim: true }),
                area,
            );
            return;
        }
        let emph = self.style(Color::Cyan).add_modifier(Modifier::BOLD);
        let muted = self.style(Color::Gray);
        let p = &self.phases()[state.phase];
        let name = action(p);
        let accent = self.style(if name == "Open" {
            Color::Cyan
        } else {
            Color::Yellow
        });
        let title = Line::from(vec![
            Span::styled(" Door stats ", emph),
            Span::raw(self.title()),
            Span::raw(" "),
        ]);
        let block = Block::default().borders(Borders::ALL).title(title);
        let inner = block.inner(area);
        f.render_widget(block, area);
        if state.help {
            let lines=vec!["Timing definitions","", "Delay   First departure from the starting aperture state.","First   First tick all watched cells reach the target.","Stable  First tick they stay at target through this use.","Settled  Start of the final quiet interval, confirmed one tick later.","","Open means all watched cells are air. Closed means all are filled and piston payloads have landed.","","Timeline columns show the peak sample in each bucket. The cursor reads one exact tick. +0 is immediately before input.","","Tab phase · Left/Right step · Home/End bounds · [/] milestones · Space play/pause · a toggle active/full range · q quit","","? or Esc to return"];
            f.render_widget(
                Paragraph::new(lines.join("\n")).wrap(Wrap { trim: false }),
                inner,
            );
            return;
        }
        let mut y = inner.y;
        let last = inner.bottom() - 1;
        let mut line = |f: &mut Frame, l: Line<'_>| {
            if y < last {
                f.render_widget(
                    Paragraph::new(l),
                    Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), 1),
                );
                y += 1;
            }
        };
        line(
            f,
            format!(
                "Input {}   Aperture {}",
                coords(&self.report["input"]),
                bounds(self.report)
            )
            .into(),
        );
        line(f, Span::styled(HEADER, muted).into());
        for (i, phase) in self.phases().iter().enumerate() {
            line(
                f,
                Span::styled(
                    timing_row(phase),
                    if state.phase == i {
                        accent.add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                )
                .into(),
            );
        }
        line(f, Line::default());
        let trace = &self.traces[state.phase];
        let end = if state.full {
            trace.len() - 1
        } else {
            self.active_end(state.phase)
        };
        let samples = &trace[..=end];
        line(
            f,
            Span::styled(
                format!("{name} timeline    +0 … +{end} gt"),
                accent.add_modifier(Modifier::BOLD),
            )
            .into(),
        );
        let cols = usize::from(inner.width.saturating_sub(11));
        let x = if cols >= samples.len() {
            (state.tick * cols).div_ceil(samples.len())
        } else {
            ((state.tick + 1) * cols).div_ceil(samples.len()) - 1
        };
        let cells = self.cells(state.phase);
        if cells > 0 {
            let bars = buckets(samples, cols, |s| s.filled);
            let height = if area.height >= 24 { 3 } else { 1 };
            for row in (0..height).rev() {
                let label = if row == height - 1 {
                    format!("{cells} cells")
                } else if row == 0 {
                    "0 cells".into()
                } else {
                    String::new()
                };
                line(
                    f,
                    track(
                        label,
                        bars.iter().map(|&n| {
                            let band = (n * height).saturating_sub(row * cells).min(cells);
                            if band == 0 && row > 0 {
                                ' '
                            } else {
                                level(band, cells)
                            }
                        }),
                        x,
                        accent,
                    ),
                );
            }
        }
        let writes = buckets(samples, cols, |s| s.writes);
        let peak = writes.iter().copied().max().unwrap_or(1);
        line(
            f,
            track(
                "Writes".into(),
                writes.iter().map(|&n| level(n, peak)),
                x,
                self.style(Color::Blue),
            ),
        );
        let mut axis = vec!['─'; cols];
        axis[x] = '▲';
        line(
            f,
            Span::styled(
                format!("         {}", axis.iter().collect::<String>()),
                muted,
            )
            .into(),
        );
        let sample = trace[state.tick];
        let phase_note = if sample.moving {
            " · piston in transit"
        } else {
            ""
        };
        line(
            f,
            Span::styled(
                format!(
                    "+{} gt   {} writes · {} piston events",
                    state.tick, sample.writes, sample.pistons
                ),
                Style::default().add_modifier(Modifier::BOLD),
            )
            .into(),
        );
        let queues = if sample.quiet { "quiet" } else { "busy" };
        line(
            f,
            if cells > 0 {
                format!(
                    "{}/{} cells filled{} · queues {queues}",
                    sample.filled, cells, phase_note
                )
            } else {
                format!("Aperture unknown · queues {queues}")
            }
            .into(),
        );
        let stamps = state.milestones(self);
        let now: Vec<_> = stamps
            .iter()
            .filter(|(_, t)| *t == state.tick)
            .map(|(n, _)| *n)
            .collect();
        if !now.is_empty() {
            line(f, Span::styled(now.join(" / "), accent).into())
        }
        if area.height >= 21 {
            line(
                f,
                format!(
                    "Pistons  {} extend / {} retract    Inventory  {} slot changes",
                    p["piston_extensions"],
                    p["piston_retractions_including_drops"],
                    p["inventory_slot_changes"]
                )
                .into(),
            );
            line(f, Line::default());
            for text in frequency(self.report) {
                line(f, text.into())
            }
            line(
                f,
                format!(
                    "{} observed ticks/use · {} block writes this use",
                    self.report["ticks_per_use"], p["block_writes"]
                )
                .into(),
            );
        }
        let keys = if area.width >= 86 {
            "Tab phase   ← → tick   [ ] milestone   Space play   a range   ? help   q quit"
        } else if area.width >= 60 {
            "Tab phase  ← → tick  [ ] jump  Space play  ? help  q quit"
        } else {
            "Tab phase  ← → tick  ? help  q quit"
        };
        f.render_widget(
            Paragraph::new(Span::styled(keys, emph)),
            Rect::new(inner.x + 1, last, inner.width - 2, 1),
        );
    }
    pub fn explore(&self) -> io::Result<()> {
        let mut terminal = ratatui::try_init().inspect_err(|_| ratatui::restore())?;
        let mut state = Cursor::default();
        let result = (|| {
            loop {
                terminal.draw(|f| self.draw(f, &state))?;
                let end = if state.full {
                    self.traces[state.phase].len() - 1
                } else {
                    self.active_end(state.phase)
                };
                if state.playing && !event::poll(Duration::from_millis(50))? {
                    state.tick = (state.tick + 1).min(end);
                    state.playing = state.tick < end;
                    continue;
                }
                let Event::Key(k) = event::read()? else {
                    continue;
                };
                if k.kind == KeyEventKind::Release {
                    continue;
                }
                let was_playing = state.playing;
                state.playing = false;
                if k.code == KeyCode::Char('q')
                    || (k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL))
                {
                    break;
                }
                if state.help {
                    if matches!(k.code, KeyCode::Esc | KeyCode::Char('?')) {
                        state.help = false;
                    }
                    continue;
                }
                match k.code {
                    KeyCode::Esc => break,
                    KeyCode::Char('?') => state.help = true,
                    KeyCode::Tab | KeyCode::BackTab => {
                        state.phase = 1 - state.phase;
                        state.tick = 0;
                    }
                    KeyCode::Right => state.tick = (state.tick + 1).min(end),
                    KeyCode::Left => state.tick = state.tick.saturating_sub(1),
                    KeyCode::Home => state.tick = 0,
                    KeyCode::End => state.tick = end,
                    KeyCode::Char('a') => {
                        state.full = !state.full;
                        if !state.full {
                            state.tick = state.tick.min(self.active_end(state.phase));
                        }
                    }
                    KeyCode::Char(' ') => {
                        if state.tick == end {
                            state.tick = 0;
                        }
                        state.playing = !was_playing;
                    }
                    KeyCode::Char(']') => {
                        state.tick = state
                            .milestones(self)
                            .iter()
                            .map(|(_, t)| *t)
                            .filter(|&t| t > state.tick)
                            .min()
                            .unwrap_or(end)
                    }
                    KeyCode::Char('[') => {
                        state.tick = state
                            .milestones(self)
                            .iter()
                            .map(|(_, t)| *t)
                            .filter(|&t| t < state.tick)
                            .max()
                            .unwrap_or(0)
                    }
                    _ => {}
                }
            }
            Ok(())
        })();
        let restored = ratatui::try_restore();
        result.and(restored)
    }
}
#[derive(Default)]
struct Cursor {
    phase: usize,
    tick: usize,
    full: bool,
    help: bool,
    playing: bool,
}
impl Cursor {
    fn milestones(&self, view: &View) -> Vec<(&'static str, usize)> {
        let p = &view.phases()[self.phase];
        let t = &p["aperture"]["timing"];
        [
            ("Input", &serde_json::Value::Null),
            ("Response", &t["delay_ticks"]),
            ("First", &t["aperture_first_ticks"]),
            ("Stable", &t["aperture_stable_ticks"]),
            ("Settled", &p["mechanism_settled_ticks"]),
        ]
        .into_iter()
        .filter_map(|(name, v)| {
            if name == "Input" {
                Some((name, 0))
            } else {
                v.as_u64().map(|n| (name, n as usize))
            }
        })
        .collect()
    }
}
