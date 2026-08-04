//! The inspector: one file's [`FileReport`] across three tabs.

use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table, Tabs};
use ratatui::Frame;

use super::app::{Screen, Transition};
use super::browser::{human_bytes, BrowserState};
use crate::model::FileReport;

pub(crate) struct InspectorState {
    pub(crate) report: FileReport,
    /// 0 = Overview, 1 = Entities, 2 = Test.
    pub(crate) tab: usize,
    pub(crate) scroll: [u16; 3],
}

impl InspectorState {
    pub(crate) fn new(report: FileReport) -> Self {
        Self { report, tab: 0, scroll: [0, 0, 0] }
    }

    pub(crate) fn on_key(&mut self, key: KeyCode) -> Transition {
        match key {
            KeyCode::Esc => {
                let dir = self
                    .report
                    .path
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| ".".into());
                return Transition::To(Screen::Browser(BrowserState::new(dir)));
            }
            KeyCode::Char('1') => self.tab = 0,
            KeyCode::Char('2') => self.tab = 1,
            KeyCode::Char('3') => self.tab = 2,
            KeyCode::Left => self.tab = self.tab.saturating_sub(1),
            KeyCode::Right => self.tab = (self.tab + 1).min(2),
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll[self.tab] = self.scroll[self.tab].saturating_sub(1)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll[self.tab] = self.scroll[self.tab].saturating_add(1)
            }
            KeyCode::PageUp => self.scroll[self.tab] = self.scroll[self.tab].saturating_sub(20),
            KeyCode::PageDown => self.scroll[self.tab] = self.scroll[self.tab].saturating_add(20),
            KeyCode::Home => self.scroll[self.tab] = 0,
            _ => {}
        }
        Transition::Stay
    }
}

pub(crate) fn draw(frame: &mut Frame, area: Rect, state: &InspectorState) {
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
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {} · {}", report.format, human_bytes(report.bytes)),
                Style::default().fg(Color::DarkGray),
            ),
        ])),
        title_bar,
    );
    let tabs = Tabs::new(vec!["1 overview", "2 entities", "3 test"])
        .select(state.tab)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    frame.render_widget(tabs, tab_bar);

    match state.tab {
        0 => draw_overview(frame, body, state),
        1 => draw_entities(frame, body, state),
        _ => draw_test(frame, body, state),
    }
}

fn draw_overview(frame: &mut Frame, area: Rect, state: &InspectorState) {
    let report = &state.report;
    let [meta_area, palette_area] =
        Layout::horizontal([Constraint::Percentage(42), Constraint::Percentage(58)]).areas(area);

    let label = Style::default().fg(Color::DarkGray);
    let (w, h, l) = report.dimensions;
    let mut lines: Vec<Line> = vec![
        Line::from(vec![Span::styled("size      ", label), Span::raw(format!("{w} × {h} × {l}"))]),
        Line::from(vec![
            Span::styled("blocks    ", label),
            Span::raw(format!("{} of {} volume", report.total_blocks, report.total_volume)),
        ]),
    ];
    if let Some(name) = &report.name {
        lines.push(Line::from(vec![Span::styled("name      ", label), Span::raw(name.clone())]));
    }
    if let Some(author) = &report.author {
        lines.push(Line::from(vec![Span::styled("author    ", label), Span::raw(author.clone())]));
    }
    if let Some(description) = &report.description {
        lines.push(Line::from(vec![
            Span::styled("descr     ", label),
            Span::raw(description.clone()),
        ]));
    }
    if let Some(dv) = report.data_version {
        lines.push(Line::from(vec![Span::styled("data ver  ", label), Span::raw(dv.to_string())]));
    }
    let test_line = match &report.embedded_test {
        Some(test) if test.parse_error.is_some() => {
            Span::styled("embedded, unreadable — see tab 3", Style::default().fg(Color::Red))
        }
        Some(test) => Span::styled(
            format!("{} case(s) — see tab 3", test.cases),
            Style::default().fg(Color::Green),
        ),
        None => Span::styled("none", label),
    };
    lines.push(Line::from(vec![Span::styled("test      ", label), test_line]));
    lines.push(Line::from(vec![
        Span::styled("entities  ", label),
        Span::raw(format!(
            "{} mobile · {} block entities",
            report.entities.len(),
            report.block_entities.len()
        )),
    ]));
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
        [Constraint::Length(8), Constraint::Length(13), Constraint::Min(10)],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" palette ({} states, ↑↓ scrolls) ", report.palette.len())),
    );
    frame.render_widget(table, palette_area);
}

fn draw_entities(frame: &mut Frame, area: Rect, state: &InspectorState) {
    let report = &state.report;
    let [entities_area, block_entities_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
    let scroll = state.scroll[1];

    let entity_lines: Vec<Line> = report.entities.iter().map(|e| Line::from(e.clone())).collect();
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

    let be_lines: Vec<Line> =
        report.block_entities.iter().map(|e| Line::from(e.clone())).collect();
    frame.render_widget(
        Paragraph::new(be_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" block entities ({}) ", report.block_entities.len())),
            )
            .scroll((scroll, 0)),
        block_entities_area,
    );
}

fn draw_test(frame: &mut Frame, area: Rect, state: &InspectorState) {
    let report = &state.report;
    let (title, text) = match &report.embedded_test {
        None => (
            " embedded test ".to_string(),
            "no NucleationTest tag in this file\n\nattach one with:\n  cargo run --example scenario_inspect -- <file> --embed spec.json --write <file>".to_string(),
        ),
        Some(test) => match (&test.parse_error, &test.pretty) {
            (Some(error), _) => (" embedded test — UNREADABLE ".to_string(), error.clone()),
            (None, Some(pretty)) => {
                (format!(" embedded test — {} case(s) ", test.cases), pretty.clone())
            }
            (None, None) => {
                (format!(" embedded test — {} case(s) ", test.cases), test.names.join("\n"))
            }
        },
    };
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(title))
            .scroll((state.scroll[2], 0)),
        area,
    );
}
