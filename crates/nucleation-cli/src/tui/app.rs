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
    io::stdout().execute(EnterAlternateScreen)?;
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
            let _ = io::stdout().execute(LeaveAlternateScreen);
            default_hook(info);
        }
    }));

    let backend = ratatui::backend::CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let result = event_loop(&mut terminal, initial);

    let _ = std::panic::take_hook();
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    result
}

fn event_loop(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    mut screen: Screen,
) -> io::Result<()> {
    loop {
        // Live screens make progress between keys.
        match &mut screen {
            Screen::Tests(state) => state.drain_events(),
            Screen::Browser(state) => state.drain_events(),
            Screen::Inspector(_) => {}
        }
        // A background file-open that just finished opens the inspector.
        if let Screen::Browser(state) = &mut screen {
            if let Some(report) = state.take_ready() {
                screen = Screen::Inspector(InspectorState::new(report));
            }
        }
        terminal.draw(|frame| draw(frame, &mut screen))?;

        if !event::poll(std::time::Duration::from_millis(50))? {
            continue;
        }
        // Drain the whole queue before the next draw.
        loop {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let (next, quit) = handle_key(screen, key.code);
                    screen = next;
                    if quit {
                        return Ok(());
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

fn handle_key(screen: Screen, code: KeyCode) -> (Screen, bool) {
    // Global keys. While the browser's `/` filter is being typed, characters
    // belong to the filter; everywhere else `q` quits.
    let typing = matches!(&screen, Screen::Browser(state) if state.filtering);
    match code {
        KeyCode::Char('q') if !typing => return (screen, true),
        KeyCode::Tab => {
            let next = match screen {
                Screen::Browser(_) | Screen::Inspector(_) => {
                    Screen::Tests(TestsState::discovering_cwd())
                }
                Screen::Tests(_) => Screen::Browser(BrowserState::new(
                    std::env::current_dir().unwrap_or_else(|_| ".".into()),
                )),
            };
            return (next, false);
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
            Transition::To(next) => (next, false),
            Transition::Quit => (Screen::Tests(state), true),
            Transition::Stay => (Screen::Tests(state), false),
        },
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
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
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
            Span::styled(" nucleation ", Style::default().fg(Color::Cyan)),
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
        Screen::Browser(_) => "↑↓/jk move · enter open · h/backspace up · / filter · tab tests · q quit",
        Screen::Inspector(_) => "1/2/3 or ←/→ tabs · ↑↓/jk scroll · esc back · q quit",
        Screen::Tests(_) => "↑↓/jk move · enter detail · i inspect file · r rerun · esc back · q quit",
    };
    frame.render_widget(
        Paragraph::new(Line::from(hint)).style(Style::default().fg(Color::DarkGray)),
        footer,
    );
}
