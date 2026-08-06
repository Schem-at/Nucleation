//! Terminal lifecycle, the event loop, and screen switching.
//!
//! Input is drained a whole queue at a time: a held-down key coalesces into
//! one redraw instead of one frame per keypress, which is the difference
//! between crisp and sluggish.

use std::io;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::crossterm::ExecutableCommand;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::{Frame, Terminal};

use super::browser::BrowserState;
use super::inspector::InspectorState;
use super::tests_screen::TestsState;

/// Which screen has the terminal.
pub(crate) enum Screen {
    Browser(BrowserState),
    Inspector(InspectorState),
    Tests(TestsState),
}

/// What a screen wants after a key. (`q` is handled globally, so no screen
/// currently emits `Quit`; the variant stays for one that needs to.)
#[allow(dead_code)]
pub(crate) enum Transition {
    Stay,
    To(Screen),
    Quit,
}

/// Run the shell until the user quits. Restores the terminal on every exit
/// path, panics included.
pub(crate) fn run(initial: Screen) -> io::Result<()> {
    enable_raw_mode()?;
    // Probe the terminal's image protocol NOW — before the alternate screen
    // and before crossterm's event queue starts competing for stdin. Probing
    // lazily from inside a draw raced the event loop for the terminal's
    // reply and lost, which read as a silently blank 3D view.
    super::inspector::init_picker();
    io::stdout().execute(EnterAlternateScreen)?;
    // Mouse tracking, the same xterm sequences every mouse-aware TUI turns
    // on. Native text selection needs Shift held while this is active.
    io::stdout().execute(ratatui::crossterm::event::EnableMouseCapture)?;
    // Bracketed paste is how a terminal delivers a *dropped file*: the path
    // arrives as one paste event instead of a burst of keystrokes.
    io::stdout().execute(ratatui::crossterm::event::EnableBracketedPaste)?;
    // A panic mid-draw must not leave the user's shell in raw mode — but
    // ONLY an uncaught main-thread panic may touch the terminal. Worker
    // threads panic on purpose (the harness refuses unsimulable structures)
    // and are caught into broken rows; letting the hook restore the screen
    // and print for those painted panic text over the whole UI, once per
    // broken file.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if std::thread::current().name() == Some("main") {
            let _ = disable_raw_mode();
            let _ = io::stdout().execute(ratatui::crossterm::event::DisableBracketedPaste);
            let _ = io::stdout().execute(ratatui::crossterm::event::DisableMouseCapture);
            let _ = io::stdout().execute(LeaveAlternateScreen);
            default_hook(info);
        }
    }));

    let backend = ratatui::backend::CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let result = event_loop(&mut terminal, initial);

    let _ = std::panic::take_hook();
    disable_raw_mode()?;
    io::stdout().execute(ratatui::crossterm::event::DisableBracketedPaste)?;
    io::stdout().execute(ratatui::crossterm::event::DisableMouseCapture)?;
    io::stdout().execute(LeaveAlternateScreen)?;
    result
}

fn event_loop(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    mut screen: Screen,
) -> io::Result<()> {
    // Tab toggles between two *kept* sides: the world (browser/inspector)
    // and the tests screen. Each side parks here while the other is up, so
    // tabbing away and back never rebuilds a screen — and never rescans a
    // corpus — unless the user has moved to a file the parked run does not
    // cover.
    let mut world: Option<Screen> = None;
    let mut tests: Option<TestsState> = None;
    // Where the last press/drag sat, for orbiting (left) and panning
    // (right) by mouse. `Some` only when the press landed on a 3D pane.
    let mut drag_from: Option<(u16, u16)> = None;
    let mut pan_from: Option<(u16, u16)> = None;
    loop {
        // Live screens make progress between keys.
        match &mut screen {
            Screen::Tests(state) => state.drain_events(),
            Screen::Browser(state) => state.drain_events(),
            Screen::Inspector(state) => state.drain_events(),
        }
        // A background file-open that just finished opens the inspector.
        if let Screen::Browser(state) = &mut screen {
            if let Some(report) = state.take_ready() {
                screen = Screen::Inspector(InspectorState::new(report));
            }
        }
        // A swapped image engine or protocol leaves the old escape's pixels
        // in cells ratatui's diff sees as unchanged; only a full repaint
        // evicts the ghost.
        if let Screen::Inspector(state) = &mut screen {
            if state.take_hard_clear() {
                terminal.clear()?;
            }
        }
        // Frames paint atomically where the terminal supports synchronized
        // output (mode 2026) — no half-drawn image under a half-drawn UI.
        let _ = io::stdout().execute(ratatui::crossterm::terminal::BeginSynchronizedUpdate);
        terminal.draw(|frame| draw(frame, &mut screen))?;
        let _ = io::stdout().execute(ratatui::crossterm::terminal::EndSynchronizedUpdate);

        if !event::poll(std::time::Duration::from_millis(50))? {
            continue;
        }
        // Drain the whole queue before the next draw.
        loop {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let (next, quit) = handle_key(screen, key.code, &mut world, &mut tests);
                    screen = next;
                    if quit {
                        return Ok(());
                    }
                }
                Event::Paste(text) => {
                    // A file dropped onto the terminal: the path arrives as a
                    // paste. Open what it names — files in the inspector,
                    // directories in the browser.
                    if let Some(next) = open_dropped_path(&text) {
                        screen = next;
                    }
                }
                Event::Mouse(mouse) => {
                    use ratatui::crossterm::event::{MouseButton, MouseEventKind};
                    match mouse.kind {
                        // The wheel zooms a hovered 3D pane; everywhere else
                        // it scrolls, spelled as the arrows it stands for.
                        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                            let up = matches!(mouse.kind, MouseEventKind::ScrollUp);
                            let mut zoomed = false;
                            if let Screen::Inspector(state) = &mut screen {
                                if state.over_view(mouse.column, mouse.row) {
                                    state.zoom_by(if up { 1.15 } else { 1.0 / 1.15 });
                                    zoomed = true;
                                }
                            }
                            if !zoomed {
                                let key = if up { KeyCode::Up } else { KeyCode::Down };
                                let (next, _) = handle_key(screen, key, &mut world, &mut tests);
                                screen = next;
                            }
                        }
                        MouseEventKind::Down(MouseButton::Left) => {
                            // Header chips first: browse/tests toggle sides
                            // exactly like Tab.
                            if mouse.row == 0 {
                                let on_browse = (12..21).contains(&mouse.column);
                                let on_tests = (21..29).contains(&mouse.column);
                                let is_tests = matches!(&screen, Screen::Tests(_));
                                if (on_browse && is_tests) || (on_tests && !is_tests) {
                                    screen = toggle_sides(screen, &mut world, &mut tests);
                                    continue;
                                }
                            }
                            // A press on a 3D pane arms drag-orbit; anywhere
                            // else it is a click the screen resolves.
                            let arm = matches!(
                                &screen,
                                Screen::Inspector(state)
                                    if state.over_view(mouse.column, mouse.row)
                            );
                            drag_from = arm.then_some((mouse.column, mouse.row));
                            if !arm {
                                match &mut screen {
                                    Screen::Inspector(state) => {
                                        state.click(mouse.column, mouse.row);
                                    }
                                    Screen::Browser(state) => {
                                        if let Transition::To(next) =
                                            state.click(mouse.column, mouse.row)
                                        {
                                            screen = next;
                                        }
                                    }
                                    Screen::Tests(state) => {
                                        // Tests' clicks only Stay or expand;
                                        // an `i`-style transition cannot come
                                        // from a click today.
                                        let _ = state.click(mouse.column, mouse.row);
                                    }
                                }
                            }
                        }
                        MouseEventKind::Down(MouseButton::Right) => {
                            let arm = matches!(
                                &screen,
                                Screen::Inspector(state)
                                    if state.over_view(mouse.column, mouse.row)
                            );
                            pan_from = arm.then_some((mouse.column, mouse.row));
                        }
                        MouseEventKind::Drag(MouseButton::Left) => {
                            if let Some((fx, fy)) = drag_from {
                                let (dx, dy) = (
                                    f32::from(mouse.column) - f32::from(fx),
                                    f32::from(mouse.row) - f32::from(fy),
                                );
                                if let Screen::Inspector(state) = &mut screen {
                                    state.orbit_drag(dx, dy);
                                }
                                drag_from = Some((mouse.column, mouse.row));
                            }
                        }
                        MouseEventKind::Drag(MouseButton::Right) => {
                            if let Some((fx, fy)) = pan_from {
                                let (dx, dy) = (
                                    f32::from(mouse.column) - f32::from(fx),
                                    f32::from(mouse.row) - f32::from(fy),
                                );
                                if let Screen::Inspector(state) = &mut screen {
                                    state.pan_drag(dx, dy);
                                }
                                pan_from = Some((mouse.column, mouse.row));
                            }
                        }
                        MouseEventKind::Up(_) => {
                            drag_from = None;
                            pan_from = None;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            if !event::poll(std::time::Duration::ZERO)? {
                break;
            }
        }
    }
}

/// Resolve a paste as a dropped path: unescape the shell-isms terminals
/// wrap drops in (`\ `, quotes, `file://`), and open what remains if it
/// exists. `None` for pastes that are not paths — a stray paste must not
/// yank the screen away.
pub(crate) fn open_dropped_path(text: &str) -> Option<Screen> {
    let cleaned = text.trim().trim_matches('\'').trim_matches('"');
    let cleaned = cleaned.strip_prefix("file://").unwrap_or(cleaned);
    let cleaned = cleaned.replace("\\ ", " ");
    let path = std::path::PathBuf::from(cleaned.trim());
    if path.is_file() {
        match crate::model::gather(&path) {
            Ok(report) => Some(Screen::Inspector(InspectorState::new(report))),
            Err(_) => None,
        }
    } else if path.is_dir() {
        Some(Screen::Browser(BrowserState::new(path)))
    } else {
        None
    }
}

pub(crate) fn handle_key(
    screen: Screen,
    code: KeyCode,
    world: &mut Option<Screen>,
    tests: &mut Option<TestsState>,
) -> (Screen, bool) {
    // Global keys. While the browser's `/` filter is being typed, characters
    // belong to the filter; everywhere else `q` quits.
    let typing = matches!(&screen, Screen::Browser(state) if state.filtering);
    match code {
        KeyCode::Char('q') if !typing => return (screen, true),
        // Tab swaps sides; Esc on a collapsed tests screen means the same
        // "back to what I was looking at".
        KeyCode::Tab => return (toggle_sides(screen, world, tests), false),
        KeyCode::Esc if matches!(&screen, Screen::Tests(state) if state.expanded.is_none()) => {
            return (toggle_sides(screen, world, tests), false);
        }
        _ => {}
    }
    match screen {
        Screen::Browser(mut state) => match state.on_key(code) {
            Transition::To(next) => (next, false),
            Transition::Quit => (Screen::Browser(state), true),
            Transition::Stay => (Screen::Browser(state), false),
        },
        Screen::Inspector(mut state) => match state.on_key(code) {
            Transition::To(next) => (next, false),
            Transition::Quit => (Screen::Inspector(state), true),
            Transition::Stay => (Screen::Inspector(state), false),
        },
        Screen::Tests(mut state) => match state.on_key(code) {
            // `i` opens the inspector for a row: the run parks whole, so
            // Tab from that inspector returns to it with results intact.
            Transition::To(next) => {
                *tests = Some(state);
                (next, false)
            }
            Transition::Quit => (Screen::Tests(state), true),
            Transition::Stay => (Screen::Tests(state), false),
        },
    }
}

/// The Tab semantics: park the current side, resume the other.
///
/// Leaving the world for tests, the parked run is reused only when it
/// covers what the user is looking at — tabbing from an inspector whose
/// file the run never touched builds a fresh run scoped to *that file*,
/// and tabbing from the browser scopes to *its* directory, never to the
/// process working directory.
fn toggle_sides(
    screen: Screen,
    world: &mut Option<Screen>,
    tests: &mut Option<TestsState>,
) -> Screen {
    match screen {
        Screen::Tests(state) => {
            *tests = Some(state);
            world.take().unwrap_or_else(|| {
                Screen::Browser(BrowserState::new(
                    std::env::current_dir().unwrap_or_else(|_| ".".into()),
                ))
            })
        }
        world_screen => {
            let scope = match &world_screen {
                Screen::Inspector(state) => state.report.path.clone(),
                Screen::Browser(state) => state.dir.clone(),
                Screen::Tests(_) => unreachable!("matched above"),
            };
            let mut next = match tests.take() {
                Some(parked) if parked.covers(&scope) => parked,
                _ => TestsState::scoped(scope.clone()),
            };
            if scope.is_file() {
                next.focus(&scope);
            }
            *world = Some(world_screen);
            Screen::Tests(next)
        }
    }
}

pub(crate) fn draw(frame: &mut Frame, screen: &mut Screen) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    // Header: which screen is lit.
    let active = |on: bool| {
        if on {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    };
    let (is_browser, is_tests) = match screen {
        Screen::Browser(_) => (true, false),
        Screen::Inspector(_) => (true, false),
        Screen::Tests(_) => (false, true),
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                concat!(" nucleation ", env!("CARGO_PKG_VERSION"), "+m9 "),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(" browse ", active(is_browser)),
            Span::raw(" "),
            Span::styled(" tests ", active(is_tests)),
            Span::styled("   (tab switches)", Style::default().fg(Color::DarkGray)),
        ])),
        header,
    );

    match screen {
        Screen::Browser(state) => super::browser::draw(frame, body, state),
        Screen::Inspector(state) => super::inspector::draw(frame, body, state),
        Screen::Tests(state) => super::tests_screen::draw(frame, body, state),
    }

    let hint = match screen {
        Screen::Browser(state) if state.filtering => "type to filter · enter keep · esc clear",
        Screen::Browser(_) => "↑↓/jk move · enter open · h/backspace up · / filter · tab test this dir · q quit",
        Screen::Inspector(_) => "1-4/←→ tabs (clickable) · drag orbits · right-drag pans · wheel zooms · v protocol · esc back",
        Screen::Tests(_) => "↑↓/jk move · enter detail · i inspect file · r rerun · tab/esc back · q quit",
    };
    frame.render_widget(
        Paragraph::new(Line::from(hint)).style(Style::default().fg(Color::DarkGray)),
        footer,
    );
}
