//! Pipe plumbing: `-` means stdin or stdout, ffmpeg-style.
//!
//! Two rules keep the CLI a well-behaved pipeline stage: payload bytes are
//! the only thing ever written to stdout when stdout is the destination,
//! and every human-facing status line goes to stderr in that case — the
//! same split ffmpeg uses, so `nucleation convert a.schem --to litematic - \
//! | next-tool` carries a litematic and nothing else.

use std::io::{Read, Write};
use std::path::Path;

/// Whether a CLI path argument means "the pipe".
pub(crate) fn is_stdio(path: &Path) -> bool {
    path.as_os_str() == "-"
}

/// The input bytes: stdin to EOF for `-`, the file otherwise.
pub(crate) fn read_input(path: &Path) -> Result<Vec<u8>, String> {
    if is_stdio(path) {
        let mut bytes = Vec::new();
        std::io::stdin()
            .read_to_end(&mut bytes)
            .map_err(|e| format!("reading stdin: {e}"))?;
        if bytes.is_empty() {
            return Err("stdin was empty — pipe a schematic in, or name a file".to_string());
        }
        Ok(bytes)
    } else {
        std::fs::read(path).map_err(|e| format!("reading {}: {e}", path.display()))
    }
}

/// Write payload bytes: stdout for `-`, the file otherwise.
pub(crate) fn write_output(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if is_stdio(path) {
        let mut out = std::io::stdout().lock();
        out.write_all(bytes)
            .map_err(|e| format!("writing stdout: {e}"))?;
        out.flush().map_err(|e| format!("flushing stdout: {e}"))
    } else {
        std::fs::write(path, bytes).map_err(|e| format!("writing {}: {e}", path.display()))
    }
}

/// The status line: stderr when the payload owns stdout, stdout otherwise.
pub(crate) fn status(line: &str, payload_on_stdout: bool) {
    if payload_on_stdout {
        eprintln!("{line}");
    } else {
        println!("{line}");
    }
}

/// Newline-separated paths from stdin — `find … | nucleation test -`.
pub(crate) fn paths_from_stdin() -> Result<Vec<std::path::PathBuf>, String> {
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|e| format!("reading the path list from stdin: {e}"))?;
    let paths: Vec<std::path::PathBuf> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(std::path::PathBuf::from)
        .collect();
    if paths.is_empty() {
        return Err("stdin held no paths — pipe one per line, as `find` prints them".to_string());
    }
    Ok(paths)
}

/// A display name for reports when the input was the pipe.
pub(crate) fn display_name(path: &Path) -> String {
    if is_stdio(path) {
        "<stdin>".to_string()
    } else {
        path.display().to_string()
    }
}

/// The output-side twin: `-` is stdout there.
pub(crate) fn display_out(path: &Path) -> String {
    if is_stdio(path) {
        "<stdout>".to_string()
    } else {
        path.display().to_string()
    }
}
