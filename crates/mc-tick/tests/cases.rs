//! Folder-driven, black-box scenario tests.
//!
//! Every `*.test.json` under `tests/cases/` is one case: a structure file, a
//! list of player actions, and end-state checks at named ticks. Adding a case
//! is adding files — nothing recompiles. The checks are deliberately blind to
//! *how* the engine got there (no traces, no event order): a faster redstone
//! backend that still opens and resets the door passes unchanged.
//!
//! Run one case while iterating: `MC_TICK_CASE=vault cargo test -p mc-tick --test cases`
//!
//! The descriptor and its evaluator live in `tests/support/scenario.rs`, shared
//! with nucleation's `.litematic` driver — one vocabulary, two carriers. This
//! file is only the *carrier*: JSON descriptor plus SNBT structure, side by
//! side on disk. See `tests/cases/README.md` for the format.

use std::path::{Path, PathBuf};

use mc_tick::Structure;

#[path = "support/scenario.rs"]
mod scenario;

fn run_case(case_path: &Path) -> Result<(), String> {
    let text = std::fs::read_to_string(case_path)
        .map_err(|e| format!("reading {}: {e}", case_path.display()))?;
    let case: scenario::Case = serde_json::from_str(&text)
        .map_err(|e| format!("parsing {}: {e}", case_path.display()))?;
    let label = &case.name;

    let structure_path = match &case.structure {
        Some(rel) => case_path.parent().unwrap().join(rel),
        None => case_path.with_extension("").with_extension("snbt"),
    };
    let snbt = std::fs::read_to_string(&structure_path)
        .map_err(|e| format!("{label}: reading {}: {e}", structure_path.display()))?;
    let structure =
        Structure::parse(&snbt).map_err(|e| format!("{label}: parsing structure: {e:?}"))?;

    // An SNBT carrier has no save version of its own to speak for; these are
    // captures of contraptions, not of a world's entities.
    scenario::run(&structure, &case, None)
}

#[test]
fn every_bundled_case_passes() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/cases");
    let filter = std::env::var("MC_TICK_CASE").unwrap_or_default();
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("tests/cases must exist")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.file_name().is_some_and(|n| n.to_string_lossy().ends_with(".test.json")))
        .filter(|p| filter.is_empty() || p.to_string_lossy().contains(&filter))
        .collect();
    paths.sort();

    let cases: Vec<(String, _)> = paths
        .into_iter()
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            (name, move || run_case(&path))
        })
        .collect();
    scenario::report("cases", cases);
}
