//! Load a list of doors through the *same* bridge calls the browser makes and
//! report which blocks the engine still refuses.
//!
//! This deliberately uses only the public `TickSimulation` surface rather than
//! the crate internals: a diagnostic that takes a shortcut past the product
//! path stops answering the question the batch actually asks, which is whether
//! an upload would work.
//!
//! `cargo run --release --example door_batch_load --features bridge,mc-tick -- <dir> <list>`
//! where `<list>` is newline-delimited file names within `<dir>`.
use std::collections::BTreeMap;

use nucleation::bridge::mc_tick::ffi::{TickSettleMode, TickSimulation};
use nucleation::bridge::schematic::ffi::Schematic;

fn main() {
    let dir = std::env::args().nth(1).expect("a directory");
    // The batch is a curated list, not the whole directory: a downloads folder
    // also holds world-sized exports, and those exercise the volume guard
    // rather than anything about doors.
    let list = std::env::args().nth(2).expect("a newline-delimited file list");

    let mut paths: Vec<std::path::PathBuf> = std::fs::read_to_string(&list)
        .unwrap()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| std::path::Path::new(&dir).join(line))
        .collect();
    paths.sort();

    let mut ok = 0usize;
    let mut failed: Vec<(String, String)> = Vec::new();
    let mut culprits: BTreeMap<String, usize> = BTreeMap::new();

    for path in &paths {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let why = match std::fs::read(path) {
            Err(e) => format!("read: {e}"),
            Ok(bytes) => match load(&bytes) {
                Ok(()) => {
                    ok += 1;
                    continue;
                }
                Err(e) => e,
            },
        };
        for word in why.split(|c: char| !(c.is_alphanumeric() || c == ':' || c == '_')) {
            if word.starts_with("minecraft:") {
                *culprits.entry(word.to_string()).or_default() += 1;
            }
        }
        failed.push((name, why));
    }

    println!("loaded {ok}/{}", paths.len());
    for (name, why) in &failed {
        println!("  FAIL {name}: {why}");
    }
    if !culprits.is_empty() {
        println!("\nblocks named in failures:");
        for (block, count) in &culprits {
            println!("  {count:3}x {block}");
        }
    }
}

fn load(bytes: &[u8]) -> Result<(), String> {
    let schematic = Schematic::from_data(bytes).map_err(|e| format!("parse: {e:?}"))?;
    // The enum carries only a category; the sentence naming the offending
    // blocks is what makes a failure actionable, and it lives here.
    TickSimulation::from_schematic(&schematic, TickSettleMode::InWorld, 0, 0, 0, b"")
        .map(|_| ())
        .map_err(|e| {
            let detail = read_out(TickSimulation::last_error_detail);
            if detail.is_empty() { format!("{e:?}") } else { detail }
        })
}

/// Call a bridge function that writes into a `DiplomatWrite` and get a String.
///
/// The bindings hand these buffers in from the host language; from Rust the
/// only public constructors are the C entry points, so this borrows the same
/// growable buffer the generated C bindings use.
fn read_out(f: fn(&mut diplomat_runtime::DiplomatWrite)) -> String {
    unsafe {
        let write = diplomat_runtime::diplomat_buffer_write_create(0);
        f(&mut *write);
        let text = String::from_utf8_lossy((*write).as_bytes()).into_owned();
        diplomat_runtime::diplomat_buffer_write_destroy(write);
        text
    }
}
