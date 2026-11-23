//! Text report and an interactive view of the same observed game ticks.
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    path::Path,
    time::Duration,
};

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
type Milestones = BTreeMap<usize, String>;

fn milestones(p: &Value, trace: &[Sample]) -> Milestones {
    let mut rows = Milestones::new();
    let mut add = |tick: usize, label: &str| {
        rows.entry(tick)
            .and_modify(|v| {
                v.push_str(" · ");
                v.push_str(label)
            })
            .or_insert_with(|| label.to_owned());
    };
    add(0, "Input");
    let t = &p["aperture"]["timing"];
    if let Some(action) = t["action"].as_str() {
        let opening = action == "open";
        let target = if opening { "open" } else { "closed" };
        if let Some(tick) = t["delay_ticks"].as_u64() {
            add(
                tick as usize,
                if opening {
                    "Starts opening"
                } else {
                    "Starts closing"
                },
            );
        }
        let first = t["aperture_first_ticks"].as_u64();
        let stable = t["aperture_stable_ticks"].as_u64();
        if let Some(first) = first {
            let label = if stable == Some(first) {
                format!("Fully {target}, stays {target}")
            } else {
                format!("First fully {target}")
            };
            add(first as usize, &label);
            if t["lost_target_after_first_arrival"] == true {
                let n = p["aperture"]["cells"].as_u64().unwrap_or(0) as usize;
                if let Some((tick, _)) =
                    trace
                        .iter()
                        .enumerate()
                        .skip(first as usize + 1)
                        .find(|(_, s)| {
                            if opening {
                                s.filled != 0
                            } else {
                                s.moving || s.filled != n
                            }
                        })
                {
                    add(
                        tick,
                        if opening {
                            "Obstructed again"
                        } else {
                            "No longer fully closed"
                        },
                    );
                }
            }
        }
        if let Some(tick) = stable.filter(|v| Some(*v) != first) {
            add(tick as usize, &format!("Stays {target}"));
        }
    }
    if let Some(tick) = p["mechanism_settled_ticks"].as_u64() {
        add(tick as usize, "Mechanism settled");
    }
    rows
}
fn unobserved(p: &Value) -> Option<String> {
    let t = &p["aperture"]["timing"];
    let mut missing = vec![];
    if t.is_null() {
        missing.push("opening/closing direction unknown");
    } else if t["aperture_first_ticks"].is_null() {
        missing.push("full target unobserved");
    } else if t["aperture_stable_ticks"].is_null() {
        missing.push("stable target unobserved");
    }
    if p["mechanism_settled_ticks"].is_null() {
        missing.push("settlement unobserved");
    }
    (!missing.is_empty()).then(|| missing.join(" · "))
}
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
fn phase_color(p: &Value) -> &'static str {
    if action(p) == "Open" {
        "36"
    } else {
        "33"
    }
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
        println!("Door timeline · game ticks after each input");
        let phases = self.phases();
        let rows = self.milestone_rows();
        if width >= 76 {
            let col = rows[0]
                .values()
                .map(|s| s.chars().count())
                .max()
                .unwrap_or(0)
                .max(12)
                + 3;
            println!(
                "\n{:<8}{:<col$}{}",
                "Tick",
                action(&phases[0]),
                action(&phases[1])
            );
            let ticks: BTreeSet<_> = rows.iter().flat_map(|r| r.keys()).collect();
            for tick in ticks {
                let left = rows[0].get(tick).map(String::as_str).unwrap_or("");
                let right = rows[1].get(tick).map(String::as_str).unwrap_or("");
                println!(
                    "{:<8}{}{}",
                    format!("+{tick}"),
                    self.paint(&format!("{left:<col$}"), phase_color(&phases[0])),
                    self.paint(right, phase_color(&phases[1]))
                );
            }
        } else {
            for (p, rows) in phases.iter().zip(&rows) {
                println!("\n{}", self.paint(&action(p), phase_color(p)));
                for (tick, label) in rows {
                    println!("  {:<7}{label}", format!("+{tick}"));
                }
            }
        }
        for p in phases {
            if let Some(note) = unobserved(p) {
                println!("{}: {note}", action(p));
            }
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
        if self.cells(0) == 0 {
            println!("Aperture {}", bounds(self.report));
            super::print_panels(self.report["aperture_candidates"].as_array().unwrap());
        }
        if std::io::IsTerminal::is_terminal(&std::io::stdout()) {
            println!("\nExplore individual ticks with --tui");
        }
    }
    fn milestone_rows(&self) -> Vec<Milestones> {
        self.phases()
            .iter()
            .zip(self.traces)
            .map(|(p, s)| milestones(p, s))
            .collect()
    }
    fn style(&self, c: Color) -> Style {
        if self.color {
            Style::default().fg(c)
        } else {
            Style::default()
        }
    }
    fn draw(&self, f: &mut Frame, state: &Cursor, rows: &Milestones) {
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
            let lines=vec!["Timing definitions","", "Delay   First departure from the starting aperture state.","First   First tick all watched cells reach the target.","Stable  First tick they stay at target through this use.","Settled  Start of the final quiet interval, confirmed one tick later.","","Open means all watched cells are air. Closed means all are filled and piston payloads have landed.","","Timeline columns show the peak sample in each bucket. The cursor reads one exact tick. +0 is immediately before input.","","Tab phase · Left/Right step · Home/End bounds · [/] milestones · Space play/pause · a toggle active/full range · d mechanics details · q quit","","? or Esc to return"];
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
        if !state.details {
            line(f, Span::styled("Tick      Door event", muted).into());
            let current = rows.range(..=state.tick).next_back().map(|(t, _)| *t);
            for (&tick, event) in rows {
                let selected = current == Some(tick);
                line(
                    f,
                    Span::styled(
                        format!(
                            "{} {:<7} {event}",
                            if selected { "▸" } else { " " },
                            format!("+{tick}")
                        ),
                        if selected {
                            accent.add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        },
                    )
                    .into(),
                );
            }
            if let Some(note) = unobserved(p) {
                line(f, note.into());
            }
            line(f, Line::default());
            let next = rows
                .range((state.tick + 1)..)
                .next()
                .map(|(t, _)| format!(" · next event +{t}"))
                .unwrap_or_default();
            line(
                f,
                Span::styled(format!("Cursor +{} gt{next}", state.tick), accent).into(),
            );
            let cols = usize::from(inner.width.saturating_sub(2));
            let column = |t: usize| t * (cols - 1) / end.max(1);
            let mut axis = vec!['─'; cols];
            for &tick in rows.keys() {
                axis[column(tick.min(end))] = '◆';
            }
            axis[column(state.tick)] = '▲';
            line(
                f,
                Span::styled(axis.into_iter().collect::<String>(), accent).into(),
            );
            if area.height >= 21 {
                for text in frequency(self.report) {
                    line(f, text.into());
                }
            }
        } else {
            line(
                f,
                format!(
                    "Input {} · Aperture {}",
                    coords(&self.report["input"]),
                    bounds(self.report)
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
            if let Some(event) = rows.get(&state.tick) {
                line(f, Span::styled(event.as_str(), accent).into());
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
        }
        let keys = if area.width >= 86 {
            "Tab phase  ← → tick  [ ] event  Space play  d details  a range  ? help  q quit"
        } else if area.width >= 60 {
            "Tab phase  ← → tick  [ ] event  d details  ? help  q quit"
        } else {
            "Tab phase  ← → tick  d details  q quit"
        };
        f.render_widget(
            Paragraph::new(Span::styled(keys, emph)),
            Rect::new(inner.x + 1, last, inner.width - 2, 1),
        );
    }
    pub fn explore(&self) -> io::Result<()> {
        let mut terminal = ratatui::try_init().inspect_err(|_| ratatui::restore())?;
        let mut state = Cursor::default();
        let rows = self.milestone_rows();
        let result = (|| {
            loop {
                terminal.draw(|f| self.draw(f, &state, &rows[state.phase]))?;
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
                    KeyCode::Char('d') => state.details = !state.details,
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
                        state.tick = rows[state.phase]
                            .range((state.tick + 1)..)
                            .next()
                            .map(|(t, _)| *t)
                            .unwrap_or(end)
                    }
                    KeyCode::Char('[') => {
                        state.tick = rows[state.phase]
                            .range(..state.tick)
                            .next_back()
                            .map(|(t, _)| *t)
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
    details: bool,
    playing: bool,
}
