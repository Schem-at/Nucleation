//! Screens render from plain state — provable without a terminal.

use std::path::PathBuf;
use std::time::Duration;

use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

use crate::commands::test::FileOutcome;
use crate::model::{FileReport, TestSummary};
use crate::tui::inspector::{draw as draw_inspector, InspectorState};
use crate::tui::tests_screen::{self, RowState, TestsState};

fn rendered<F: FnOnce(&mut ratatui::Frame)>(f: F) -> String {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("test backend");
    terminal.draw(|frame| f(frame)).expect("draws");
    let buffer = terminal.backend().buffer().clone();
    let mut text = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            text.push_str(buffer.cell((x, y)).map(|c| c.symbol()).unwrap_or(" "));
        }
        text.push('\n');
    }
    text
}

fn sample_report() -> FileReport {
    FileReport {
        path: PathBuf::from("door.litematic"),
        bytes: 999,
        format: "litematic".into(),
        name: Some("door".into()),
        author: None,
        description: None,
        data_version: Some(4325),
        dimensions: (3, 3, 3),
        total_blocks: 9,
        total_volume: 27,
        regions: Vec::new(),
        palette: vec![
            ("minecraft:stone".into(), 6),
            ("minecraft:redstone_lamp[lit=false]".into(), 3),
        ],
        entities: vec!["minecraft:pig @ (0.5, 1.0, 0.5)".into()],
        block_entities: vec!["minecraft:chest @ (1, 1, 1)".into()],
        embedded_test: Some(TestSummary {
            cases: 2,
            names: vec!["a".into()],
            parse_error: None,
            pretty: Some("{\n  \"name\": \"a\"\n}".into()),
        }),
    }
}

#[test]
fn the_inspector_shows_the_overview_and_flips_tabs() {
    let mut state = InspectorState::new(sample_report());
    let text = rendered(|frame| {
        let area = frame.area();
        draw_inspector(frame, area, &state);
    });
    assert!(text.contains("litematic"), "{text}");
    assert!(text.contains("minecraft:stone"), "the palette head renders: {text}");
    assert!(text.contains("2 case(s)"), "the embedded suite is surfaced: {text}");

    state.on_key(KeyCode::Right);
    assert_eq!(state.tab, 1);
    let text = rendered(|frame| {
        let area = frame.area();
        draw_inspector(frame, area, &state);
    });
    assert!(text.contains("minecraft:pig"), "{text}");
    assert!(text.contains("minecraft:chest"), "{text}");
}

#[test]
fn the_browser_lists_dirs_first_and_descends_on_enter() {
    let dir = std::env::temp_dir().join(format!("nucleation-browser-{}", std::process::id()));
    let sub = dir.join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(dir.join("build.litematic"), b"x").unwrap();
    std::fs::write(dir.join("notes.txt"), b"x").unwrap();

    let mut state = crate::tui::browser::BrowserState::new(dir.clone());
    let names: Vec<String> = state
        .entries
        .iter()
        .map(|e| e.path.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert_eq!(names, vec!["sub", "build.litematic", "notes.txt"], "dirs first, then sorted");
    assert!(state.entries[1].supported, "the schematic is highlighted");
    assert!(!state.entries[2].supported);

    assert!(
        matches!(state.on_key(KeyCode::Enter), crate::tui::app::Transition::Stay),
        "entering a dir stays in the browser"
    );
    assert_eq!(state.dir, sub);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn the_tests_screen_reports_rows_and_expands_failures() {
    // Hand-built state: no worker thread, no filesystem.
    let mut state = TestsState::for_test(vec![
        (PathBuf::from("a.litematic"), RowState::Done(FileOutcome::Ran(vec![(
            "a passes".into(),
            4,
            Duration::from_millis(1),
            Ok(()),
        )]))),
        (PathBuf::from("b.litematic"), RowState::Done(FileOutcome::Ran(vec![(
            "b fails".into(),
            4,
            Duration::from_millis(1),
            Err("b: tick 4: expected glass, got stone".into()),
        )]))),
        (PathBuf::from("c.litematic"), RowState::Running),
    ]);

    let text = rendered(|frame| {
        let area = frame.area();
        tests_screen::draw(frame, area, &state);
    });
    assert!(text.contains("✓ a.litematic"), "{text}");
    assert!(text.contains("✗ b.litematic"), "{text}");
    assert!(text.contains("⠋ c.litematic"), "the running row spins: {text}");
    assert!(text.contains("1 pass"), "{text}");
    assert!(text.contains("1 fail"), "{text}");

    // Enter on the failing row exposes the report.
    state.on_key(KeyCode::Down);
    state.on_key(KeyCode::Enter);
    let text = rendered(|frame| {
        let area = frame.area();
        tests_screen::draw(frame, area, &state);
    });
    assert!(text.contains("expected glass, got stone"), "{text}");
}
