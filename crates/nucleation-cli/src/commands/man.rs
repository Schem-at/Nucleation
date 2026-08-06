//! `nucleation man [--install]` — the manual, from inside the binary.
//!
//! The page is mdoc(7), mandoc-native, embedded at compile time so the
//! binary and its documentation cannot drift apart. On a terminal it opens
//! through the system's own man(1); piped, the raw roff comes out, so
//! `nucleation man | mandoc -a` and friends work.

use std::io::IsTerminal as _;
use std::io::Write as _;

const PAGE: &str = include_str!("../../man/nucleation.1");

pub(crate) fn man_main(mut args: impl Iterator<Item = String>) {
    match args.next().as_deref() {
        Some("--install") => install(),
        Some(_) => crate::usage_and_exit(),
        None => show(),
    }
}

fn show() {
    if std::io::stdout().is_terminal() {
        // Through the real man(1), so the pager, styling and MANPAGER all
        // behave exactly as the user configured them.
        let dir = std::env::temp_dir();
        let path = dir.join("nucleation.1");
        if std::fs::write(&path, PAGE).is_ok() {
            let status = std::process::Command::new("man").arg(&path).status();
            if matches!(status, Ok(s) if s.success()) {
                return;
            }
        }
    }
    // Piped, or no man(1) around: the roff itself.
    let _ = std::io::stdout().write_all(PAGE.as_bytes());
}

fn install() {
    let mut candidates: Vec<std::path::PathBuf> = vec![
        "/opt/homebrew/share/man/man1".into(),
        "/usr/local/share/man/man1".into(),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(std::path::PathBuf::from(home).join(".local/share/man/man1"));
    }
    for dir in candidates {
        if std::fs::create_dir_all(&dir).is_err() {
            continue;
        }
        let target = dir.join("nucleation.1");
        if std::fs::write(&target, PAGE).is_ok() {
            println!("installed {}", target.display());
            if dir.starts_with(std::env::var_os("HOME").unwrap_or_default()) {
                println!(
                    "note: `man nucleation` needs that tree on the man path — \
                     add to your shell rc if it is not:\n  export MANPATH=\"$HOME/.local/share/man:$MANPATH\""
                );
            }
            return;
        }
    }
    eprintln!("no writable man1 directory found — run with enough rights, or copy by hand:");
    eprintln!("  nucleation man > nucleation.1  # then place it on your man path");
    std::process::exit(2);
}
