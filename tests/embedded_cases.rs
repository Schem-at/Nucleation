//! Every schematic under `tests/scenarios/` is a build that carries its own test.
//!
//! A scenario is a *file you drop in a folder*, not Rust somebody compiles: the
//! descriptor lives in a root-level `NucleationTest` compound inside the
//! schematic, so the build and the claims about it travel together and cannot
//! drift apart. Adding one recompiles nothing.
//!
//! The carrier is *any* format the `FormatManager` detects — `.litematic`,
//! `.schem`, whatever imports to a `UniversalSchematic`. The test layer sits on
//! `metadata.embedded_test`, which every importer that preserves the
//! `NucleationTest` tag fills in; the runner never looks at the extension.
//!
//! ```text
//! cargo test --test embedded_cases
//! MC_TICK_CASE=55_3x3 cargo test --test embedded_cases      # just one
//! ```
//!
//! The descriptor and its evaluator are shared verbatim with mc-tick's
//! `*.test.json` driver (the `mc-test` crate) — one assertion vocabulary,
//! two carriers. See `crates/mc-tick/tests/cases/README.md` for the format
//! and `examples/scenario_inspect.rs` for the authoring loop.
//!
//! This driver lives in nucleation rather than in mc-tick because reading a
//! schematic means gzip and NBT, and mc-tick's whole value is that it does
//! not depend on either. The engine stays reachable from here because it is an
//! unconditional dev-dependency, so this runs under a plain `cargo test`.

use std::path::{Path, PathBuf};

use mc_tick::Structure;
use nucleation::formats::gametest::to_gametest_snbt;

use mc_test as scenario;

fn run_file(path: &Path) -> Result<(), String> {
    let file = path.file_name().unwrap().to_string_lossy().to_string();
    let bytes = std::fs::read(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    let schematic = {
        let manager = nucleation::formats::manager::get_manager();
        let manager = manager
            .lock()
            .map_err(|e| format!("{file}: format manager: {e}"))?;
        manager
            .read(&bytes)
            .map_err(|e| format!("{file}: no importer recognises it: {e:?}"))?
    };

    let spec = schematic.metadata.embedded_test.as_deref().ok_or_else(|| {
        format!(
            "{file}: carries no NucleationTest — a build in tests/scenarios/ without a test \
             would pass by having nothing to say. Attach one with \
             `cargo run --example scenario_inspect -- {file} --embed spec.json --write {file}`"
        )
    })?;
    // One descriptor object, an array of them, or a versioned suite — a door
    // has more than one thing to prove, and those are separate runs of the
    // same build, not two copies of a megabyte of blocks.
    let cases = scenario::parse_suite(spec, &file)?;

    let snbt = to_gametest_snbt(&schematic);
    let structure =
        Structure::parse(&snbt).map_err(|e| format!("{file}: the engine refused it: {e:?}"))?;

    let mut failures = Vec::new();
    for case in &cases {
        if case.structure.is_some() {
            return Err(format!(
                "{file}: \"{}\" names a separate structure file, but the carrier *is* the \
                 structure — drop the field",
                case.name
            ));
        }
        // The data version the file states is the authority on `Entity.load`
        // Motion semantics, and it is load-bearing: the record door is 4082,
        // which keeps NaN velocities. Read it at the wrong version and its nan
        // carts load as ordinary carts and the door silently un-glues, so this
        // comes from the file rather than from the descriptor.
        if let Err(report) = scenario::run(&structure, case, schematic.metadata.source_data_version)
        {
            failures.push(report);
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

#[test]
fn every_self_testing_schematic_passes() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/scenarios");
    let filter = std::env::var("MC_TICK_CASE").unwrap_or_default();
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("tests/scenarios must exist")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        // Anything except documentation is expected to be a loadable carrier;
        // an unreadable file is a loud failure, not a skip.
        .filter(|p| p.is_file())
        .filter(|p| {
            !p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with('.'))
        })
        .filter(|p| p.extension().is_none_or(|e| e != "md"))
        .filter(|p| filter.is_empty() || p.to_string_lossy().contains(&filter))
        .collect();
    paths.sort();

    let cases: Vec<(String, _)> = paths
        .into_iter()
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            (name, move || run_file(&path))
        })
        .collect();
    scenario::report("self-testing schematics", cases);
}
