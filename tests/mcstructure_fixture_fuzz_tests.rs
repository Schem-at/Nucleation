//! Round-trip stability test against real-world `.mcstructure` fixtures.
//!
//! For every Bedrock sample we ship, import it once, export it back, then
//! import the exported bytes. The two `UniversalSchematic` instances should
//! agree on:
//! - total block count
//! - dimensions
//! - the block-name population at every cell that was non-air originally
//!
//! Failures here mean either the importer is losing information or the
//! exporter we just wired up (N-1, N-2) isn't fully symmetric. They're
//! the canary that catches silent data loss in the
//! Bedrock → Universal → Bedrock pipeline.

use nucleation::formats::mcstructure::{from_mcstructure, to_mcstructure};
use std::path::PathBuf;

fn samples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/samples")
}

fn fixtures() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(samples_dir()) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("mcstructure") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// `true` if `name` is one of the air-like blocks. Bedrock export tightens
/// bounds via `to_compact()` which drops air at the schematic edges, so
/// trimming on a round-trip is expected behaviour, not data loss.
fn is_air(name: &str) -> bool {
    name == "minecraft:air"
        || name == "minecraft:cave_air"
        || name == "minecraft:void_air"
        || name == "air"
}

#[test]
fn round_trip_stable_block_count() {
    let fixtures = fixtures();
    if fixtures.is_empty() {
        eprintln!("(no .mcstructure fixtures found — skipping)");
        return;
    }
    let mut failures: Vec<String> = Vec::new();
    for path in &fixtures {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                failures.push(format!("{}: read failed: {}", name, e));
                continue;
            }
        };
        let s1 = match from_mcstructure(&bytes) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("{}: initial import failed: {}", name, e));
                continue;
            }
        };
        let count1 = s1.total_blocks();

        let bytes2 = match to_mcstructure(&s1) {
            Ok(b) => b,
            Err(e) => {
                failures.push(format!("{}: re-export failed: {}", name, e));
                continue;
            }
        };
        let s2 = match from_mcstructure(&bytes2) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("{}: re-import failed: {}", name, e));
                continue;
            }
        };
        let count2 = s2.total_blocks();

        // `total_blocks()` counts non-air. It must be invariant — the
        // exporter tightens bounds (which drops air at edges) but it
        // cannot drop a real block.
        if count1 != count2 {
            failures.push(format!(
                "{}: non-air block count drift {} → {} after round-trip",
                name, count1, count2
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "round-trip stability failed on {} fixture(s):\n  - {}",
        failures.len(),
        failures.join("\n  - ")
    );
}

#[test]
fn round_trip_preserves_block_names() {
    // Stronger check: for every block position with a non-air block in the
    // imported schematic, the same position in the round-tripped schematic
    // must hold a block whose name canonicalises identically.
    let fixtures = fixtures();
    if fixtures.is_empty() {
        eprintln!("(no .mcstructure fixtures found — skipping)");
        return;
    }
    let mut drifted: Vec<String> = Vec::new();
    for path in &fixtures {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let bytes = std::fs::read(path).expect("read fixture");
        let Ok(s1) = from_mcstructure(&bytes) else {
            continue;
        };
        let Ok(bytes2) = to_mcstructure(&s1) else {
            continue;
        };
        let Ok(s2) = from_mcstructure(&bytes2) else {
            continue;
        };

        let (w, h, l) = s1.get_dimensions();
        // Limit how many drifts we report per fixture so test output stays
        // human-readable.
        let mut local: Vec<String> = Vec::new();
        const MAX_REPORT: usize = 5;
        'outer: for x in 0..w {
            for y in 0..h {
                for z in 0..l {
                    let a = s1.get_block(x, y, z).map(|b| b.name.to_string());
                    let b = s2.get_block(x, y, z).map(|b| b.name.to_string());
                    // Ignore air-to-None: the exporter tightens bounds and
                    // legitimately drops trailing air at schematic edges.
                    let drifted_meaningfully = match (&a, &b) {
                        (Some(an), None) => !is_air(an),
                        (None, Some(bn)) => !is_air(bn),
                        (Some(an), Some(bn)) if an != bn => !(is_air(an) && is_air(bn)),
                        _ => false,
                    };
                    if drifted_meaningfully {
                        local.push(format!("  ({},{},{}): {:?} → {:?}", x, y, z, a, b));
                        if local.len() >= MAX_REPORT {
                            break 'outer;
                        }
                    }
                }
            }
        }
        if !local.is_empty() {
            drifted.push(format!("{}:\n{}", name, local.join("\n")));
        }
    }
    assert!(
        drifted.is_empty(),
        "block-name drift on {} fixture(s):\n{}",
        drifted.len(),
        drifted.join("\n")
    );
}
