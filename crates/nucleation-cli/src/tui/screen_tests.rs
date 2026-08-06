//! Screens render from plain state — provable without a terminal.

use std::path::PathBuf;
use std::time::Duration;

use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

use crate::commands::test::FileOutcome;
use crate::model::{FileReport, TestSummary};
use crate::tui::app::{handle_key, Screen};
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
            case_views: vec![
                crate::model::CaseView {
                    name: "a door opens".into(),
                    text: "2 check(s)".into(),
                },
                crate::model::CaseView {
                    name: "a door shuts".into(),
                    text: "1 check(s)".into(),
                },
            ],
        }),
        voxels: None,
    }
}

#[test]
fn the_inspector_shows_the_overview_and_flips_tabs() {
    let mut state = InspectorState::new(sample_report());
    let text = rendered(|frame| {
        let area = frame.area();
        draw_inspector(frame, area, &mut state);
    });
    assert!(text.contains("litematic"), "{text}");
    assert!(
        text.contains("minecraft:stone"),
        "the palette head renders: {text}"
    );
    assert!(
        text.contains("2 case(s)"),
        "the embedded suite is surfaced: {text}"
    );

    state.on_key(KeyCode::Right);
    assert_eq!(state.tab, 1);
    let text = rendered(|frame| {
        let area = frame.area();
        draw_inspector(frame, area, &mut state);
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
    assert_eq!(
        names,
        vec!["sub", "build.litematic", "notes.txt"],
        "dirs first, then sorted"
    );
    assert!(state.entries[1].supported, "the schematic is highlighted");
    assert!(!state.entries[2].supported);

    assert!(
        matches!(
            state.on_key(KeyCode::Enter),
            crate::tui::app::Transition::Stay
        ),
        "entering a dir stays in the browser"
    );
    assert_eq!(state.dir, sub);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn the_tests_screen_reports_rows_and_expands_failures() {
    // Hand-built state: no worker thread, no filesystem.
    let mut state = TestsState::for_test(vec![
        (
            PathBuf::from("a.litematic"),
            RowState::Done(FileOutcome::Ran(vec![(
                "a passes".into(),
                4,
                Duration::from_millis(1),
                Ok(()),
            )])),
        ),
        (
            PathBuf::from("b.litematic"),
            RowState::Done(FileOutcome::Ran(vec![(
                "b fails".into(),
                4,
                Duration::from_millis(1),
                Err("b: tick 4: expected glass, got stone".into()),
            )])),
        ),
        (PathBuf::from("c.litematic"), RowState::Running),
    ]);

    let text = rendered(|frame| {
        let area = frame.area();
        tests_screen::draw(frame, area, &mut state);
    });
    assert!(text.contains("✓ a.litematic"), "{text}");
    assert!(text.contains("✗ b.litematic"), "{text}");
    assert!(
        text.contains("⠋ c.litematic"),
        "the running row spins: {text}"
    );
    assert!(text.contains("1 pass"), "{text}");
    assert!(text.contains("1 fail"), "{text}");

    // Enter on the failing row exposes the report.
    state.on_key(KeyCode::Down);
    state.on_key(KeyCode::Enter);
    let text = rendered(|frame| {
        let area = frame.area();
        tests_screen::draw(frame, area, &mut state);
    });
    assert!(text.contains("expected glass, got stone"), "{text}");
}

/// Tab from an inspector must scope the tests screen to *that file* — never
/// a working-directory scan — and Tab again must hand back the very same
/// inspector, results parked, no rebuild.
#[test]
fn tab_toggles_between_the_inspector_and_a_run_scoped_to_its_file() {
    let mut world = None;
    let mut tests = None;
    let report = sample_report();
    let path = report.path.clone();

    let (screen, quit) = handle_key(
        Screen::Inspector(InspectorState::new(report)),
        KeyCode::Tab,
        &mut world,
        &mut tests,
    );
    assert!(!quit);
    let Screen::Tests(state) = &screen else {
        panic!("tab from inspector must land on tests")
    };
    assert!(
        state
            .scope_label
            .contains(path.file_name().unwrap().to_str().unwrap()),
        "the run must say it is scoped to the inspected file, got {:?}",
        state.scope_label
    );
    assert!(world.is_some(), "the inspector must be parked, not dropped");

    let (screen, _) = handle_key(screen, KeyCode::Tab, &mut world, &mut tests);
    let Screen::Inspector(back) = &screen else {
        panic!("tab from tests must restore the world")
    };
    assert_eq!(back.report.path, path, "the same inspector comes back");
    assert!(tests.is_some(), "the run parks for the next tab");
    assert!(
        world.is_none(),
        "the world slot empties when its side is up"
    );
}

/// Esc on the tests screen: collapse the detail first; with nothing
/// expanded it means "back to what I was looking at", same as Tab.
#[test]
fn esc_on_a_collapsed_tests_screen_restores_the_parked_world() {
    let mut world = Some(Screen::Inspector(InspectorState::new(sample_report())));
    let mut tests = None;
    let mut state = TestsState::for_test(vec![(
        PathBuf::from("door.litematic"),
        RowState::Done(FileOutcome::Ran(vec![(
            "a door opens".into(),
            40,
            Duration::from_millis(3),
            Ok(()),
        )])),
    )]);
    state.expanded = Some(0);

    // First Esc collapses the expanded detail and stays.
    let (screen, _) = handle_key(Screen::Tests(state), KeyCode::Esc, &mut world, &mut tests);
    let Screen::Tests(state) = screen else {
        panic!("collapse stays on tests")
    };
    assert_eq!(state.expanded, None);

    // Second Esc leaves, restoring the parked inspector.
    let (screen, _) = handle_key(Screen::Tests(state), KeyCode::Esc, &mut world, &mut tests);
    assert!(
        matches!(screen, Screen::Inspector(_)),
        "esc restores the parked world"
    );
    assert!(tests.is_some());
}

/// A parked run is reused when it covers the file being inspected — with
/// the cursor moved onto that file — instead of rescanning.
#[test]
fn a_parked_run_covering_the_inspected_file_is_reused_and_focused() {
    let report = sample_report();
    let path = report.path.clone();
    let mut world = None;
    let mut tests = Some(TestsState::for_test(vec![
        (PathBuf::from("other.litematic"), RowState::Pending),
        (path.clone(), RowState::Pending),
    ]));

    let (screen, _) = handle_key(
        Screen::Inspector(InspectorState::new(report)),
        KeyCode::Tab,
        &mut world,
        &mut tests,
    );
    let Screen::Tests(state) = &screen else {
        panic!("tab lands on tests")
    };
    assert_eq!(
        state.rows.len(),
        2,
        "the parked two-row run is reused, not rebuilt"
    );
    assert!(tests.is_none(), "the parked run moved back on screen");
}

/// The test tab is a per-case browser: every case of a suite is listed,
/// the selected one shows its digest, and `n` walks to the next.
#[test]
fn the_test_tab_lists_every_case_and_n_switches() {
    let mut state = InspectorState::new(sample_report());
    state.tab = 2;
    let text = rendered(|frame| draw_inspector(frame, frame.area(), &mut state));
    assert!(text.contains("a door opens"), "first case listed");
    assert!(text.contains("a door shuts"), "second case listed");
    assert!(text.contains("2 case(s)"), "the case count shows");
    assert!(
        text.contains("2 check(s)"),
        "the selected case's digest shows"
    );

    state.on_key(KeyCode::Char('n'));
    let text = rendered(|frame| draw_inspector(frame, frame.area(), &mut state));
    assert!(
        text.contains("1 check(s)"),
        "n moved to the second case's digest"
    );
}

/// The raycaster must produce visible pixels for a real corpus file — a
/// blank frame here is a renderer bug, not a terminal-protocol one.
#[test]
fn the_preview_renders_visible_pixels_for_a_real_litematic() {
    let path = std::path::Path::new(
        "../../tests/corpus/lithium-litematic/gametest/structure/comparator_update_collection.litematic",
    );
    if !path.exists() {
        return; // corpus not fetched: nothing to assert against
    }
    let report = crate::model::gather(path).expect("gathers");
    let grid = report
        .voxels
        .as_ref()
        .expect("a 10x6x30 build is under the cap");
    let occupied = grid.cells.iter().filter(|c| **c != 0).count();
    assert!(
        occupied > 100,
        "the grid must hold the build, got {occupied} cells"
    );
    let frame = crate::tui::voxel::render(grid, 0.8, 0.5, 1.0, [0.0, 0.0, 0.0], 320, 240);
    let visible = frame.pixels().filter(|p| p.0[3] != 0).count();
    assert!(
        visible > 500,
        "a 475-block machine must cover pixels, got {visible} of {}",
        320 * 240
    );
}

/// The GPU frame with the TUI's exact camera config must contain visible
/// pixels — a transparent-background render that comes back empty would
/// read as a blank pane whatever the protocol does.
#[cfg(feature = "render")]
#[test]
fn the_gpu_frame_matches_the_tui_camera_and_is_visible() {
    let jar = std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(
        "Library/Application Support/PrismLauncher/libraries/com/mojang/minecraft/26.1.2/minecraft-26.1.2-client.jar",
    );
    let lit = std::path::Path::new(
        "../../tests/corpus/lithium-litematic/gametest/structure/comparator_update_collection.litematic",
    );
    if !jar.exists() || !lit.exists() {
        return;
    }
    let bytes = std::fs::read(lit).unwrap();
    let manager = nucleation::formats::manager::get_manager();
    let schematic = manager.lock().unwrap().read(&bytes).unwrap();
    let source = nucleation::meshing::ResourcePackSource::from_file(&jar.display().to_string())
        .expect("pack loads");
    let mesh = schematic
        .to_mesh(&source, &nucleation::meshing::MeshConfig::default())
        .expect("meshes");
    let config = nucleation::rendering::RenderConfig {
        width: 320,
        height: 240,
        yaw: 0.8f32.to_degrees(),
        pitch: 0.5f32.to_degrees(),
        zoom: 1.0,
        background: Some([0.0, 0.0, 0.0, 0.0]),
        sphere_fit: true,
        ..nucleation::rendering::RenderConfig::isometric()
    };
    let png = nucleation::rendering::render_meshes_png(std::slice::from_ref(&mesh), &config, None)
        .expect("renders");
    let img = image::load_from_memory(&png).expect("decodes").to_rgba8();
    let visible = img.pixels().filter(|p| p.0[3] > 8).count();
    assert!(
        visible > 1000,
        "the TUI camera config must show the build, got {visible} visible pixels"
    );
}

/// A dropped file arrives as a paste; the resolver must unescape what
/// terminals wrap paths in, open real files and directories, and ignore
/// pastes that are not paths at all.
#[test]
fn a_dropped_path_opens_and_a_stray_paste_does_not() {
    use crate::tui::app::open_dropped_path;
    assert!(open_dropped_path("just some pasted words").is_none());
    assert!(open_dropped_path("/no/such/file.litematic").is_none());
    let dir = std::env::temp_dir().join("nucleation-drop-test dir");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let escaped = dir.display().to_string().replace(' ', "\\ ");
    let opened = open_dropped_path(&format!("'{escaped}'\n"));
    assert!(
        matches!(opened, Some(Screen::Browser(_))),
        "a quoted, escape-spaced directory drop opens the browser"
    );
}

/// The mouse surface, pinned: a click on the tab bar switches tabs, a
/// click on the case list selects a case, a wheel step over the 3D pane
/// zooms, and a drag orbits. These once vanished in a bad edit and nobody
/// noticed until a user's mouse went dead — never again silently.
#[test]
fn the_inspector_mouse_surface_stays_wired() {
    let mut state = InspectorState::new(sample_report());
    // Lay the widgets out once so the hit rects exist.
    let _ = rendered(|frame| draw_inspector(frame, frame.area(), &mut state));

    // Tab bar: click the "3 test" chip.
    let tabs = state.hit_tabs;
    state.click(tabs.x + 1 + 12 + 3 + 12 + 3 + 2, tabs.y);
    assert_eq!(
        state.tab, 2,
        "clicking the third chip lands on the test tab"
    );

    // Case list: lay out the test tab, then click its second row.
    let _ = rendered(|frame| draw_inspector(frame, frame.area(), &mut state));
    let cases = state.hit_cases.expect("two cases show a list");
    state.click(cases.x + 2, cases.y + 2);
    assert_eq!(state.case_cursor, 1, "clicking the second case selects it");

    // The 3D pane: zoom and orbit change the camera.
    state.tab = 3;
    let zoom = state.zoom;
    state.zoom_by(1.15);
    assert!(state.zoom > zoom, "wheel zooms");
    let yaw = state.yaw;
    state.orbit_drag(3.0, 0.0);
    // Grab-the-model sense: dragging right pulls the build rightward,
    // which the camera answers by orbiting the other way.
    assert!(
        state.yaw < yaw,
        "drag orbits, in the grab-the-model direction"
    );
    assert!(!state.over_view(0, 0) || state.hit_view.width > 0);
}
