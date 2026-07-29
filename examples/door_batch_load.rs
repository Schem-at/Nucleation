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
                Ok(rest) => {
                    ok += 1;
                    // Loading is only half the question. A door that changes a
                    // block with nobody touching it was disturbed on the way
                    // in, and every timing measured against it is measured
                    // against a machine that is already moving. Report it here
                    // so a corpus run says which doors are trustworthy.
                    println!(
                        "  {name}: {}",
                        match rest {
                            Rest { changes: 0, quiescent: true } =>
                                "at rest (0 changes, quiescent)".to_string(),
                            Rest { changes, quiescent } => format!(
                                "DISTURBED — {changes} block change(s) untouched, quiescent={quiescent}"
                            ),
                        }
                    );
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

/// What a build did when it was loaded and then left alone.
struct Rest {
    changes: u32,
    quiescent: bool,
}

fn load(bytes: &[u8]) -> Result<Rest, String> {
    let schematic = Schematic::from_data(bytes).map_err(|e| format!("parse: {e:?}"))?;
    // The enum carries only a category; the sentence naming the offending
    // blocks is what makes a failure actionable, and it lives here.
    // `InWorld` is what the product path uses and what a saved door wants.
    // The other modes are selectable so a corpus run can *measure* how much a
    // placement perturbs these builds rather than assuming it doesn't.
    let settle = match std::env::var("SETTLE").as_deref().unwrap_or("in-world") {
        "quiet" => TickSettleMode::Quiet,
        "placement" => TickSettleMode::Placement,
        _ => TickSettleMode::InWorld,
    };
    let mut sim = TickSimulation::from_schematic(&schematic, settle, 0, 0, 0, b"")
        .map_err(|e| {
            let detail = read_out(TickSimulation::last_error_detail);
            if detail.is_empty() { format!("{e:?}") } else { detail }
        })?;
    // Nothing is triggered. `InWorld` places nothing and settles nothing, so a
    // build saved at rest must still be at rest after this — that is the whole
    // definition of the mode, and the only way to know a door was not
    // perturbed on the way in.
    sim.run(REST_TICKS);
    Ok(Rest { changes: sim.changes_count(), quiescent: sim.is_quiescent() })
}

/// Long enough for a door's slowest observer/piston cascade to show itself.
const REST_TICKS: u32 = 200;

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
