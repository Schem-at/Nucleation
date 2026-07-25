//! Differential conformance: the engine's trace against the real game's.
//!
//! Everything else in this crate's tests asserts what *we* think should happen.
//! This asserts that what happens matches what Minecraft actually did, on the same
//! structure file, tick for tick.
//!
//! # How a golden is made
//!
//! ```sh
//! tools/gametest/run.sh                         # once, to fetch and build
//! # then, per structure:
//! java -cp work/classes:$CP TraceCapture \
//!     --structure nucleation:piston_qc --out work/qc.json
//! cp work/qc.json crates/mc-tick/tests/traces/piston_qc.json
//! ```
//!
//! The structure lives once, in `tests/corpus/structures/`, and is copied into the
//! oracle's datapack — engine and game read the identical file, which is the only
//! way a diff means anything.
//!
//! # Why this is a hand-registered test rather than a corpus case
//!
//! The corpus runner registers no behaviour, so it cannot simulate. Wiring real
//! Minecraft blocks to behaviours needs a descriptor-to-behaviour registry that
//! does not exist yet — see `ROADMAP.md`. Until it does, conformance tests
//! register the handful of blocks their structure uses, explicitly, here.

use mc_tick::{Pos, Simulation, Structure};
use mc_tick_trace::{Detail, EventKind, Trace, TracePos};

/// Padding round a loaded structure; matches the corpus runner.
const MARGIN: i32 = 4;

fn structure(name: &str) -> Structure {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/structures")
        .join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    Structure::parse(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn golden(name: &str) -> Trace {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/traces")
        .join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    Trace::from_json(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Turn the engine's recorded changes into a trace comparable with a capture.
///
/// The capture observes *between* ticks, so it cannot attribute an event to a
/// phase and tags everything `tick_end`. The engine could say more, but says the
/// same thing here — a richer claim on one side would diff as a difference.
fn engine_trace(sim: &Simulation, name: &str, ticks: u64) -> Trace {
    let mut trace = Trace::new("26.2", name, Detail::Normal);
    for tick in 0..ticks {
        let events: Vec<_> = sim
            .recorded()
            .iter()
            .filter(|c| c.tick == tick)
            .map(|c| mc_tick_trace::TraceEvent {
                phase: "tick_end".to_string(),
                kind: EventKind::BlockChanged {
                    pos: TracePos::new(c.pos.x, c.pos.y, c.pos.z),
                    from: sim.registry().descriptor(c.from).unwrap_or("?").to_string(),
                    to: sim.registry().descriptor(c.to).unwrap_or("?").to_string(),
                },
            })
            .collect();
        if !events.is_empty() {
            trace.ticks.push(mc_tick_trace::TickRecord { tick, events });
        }
    }
    trace
}

/// Load a structure, wire vanilla behaviour to it, settle, and run.
///
/// No hand-registration: `mc_tick::vanilla` turns descriptors into behaviour, which
/// is what makes running an arbitrary schematic possible at all.
fn run_conformance(structure_file: &str, golden_file: &str, label: &str) {
    let structure = structure(structure_file);
    // The goldens are snapshot-derived, so intra-tick order is the capture's scan
    // order rather than the game's causal order. Canonicalising both sides compares
    // *what* changed each tick, which is what such a capture actually knows.
    // An instrumented capture would know the real order and must not be canonicalised.
    let expected = golden(golden_file).canonicalized();

    let mut sim = Simulation::new(structure.bounds(MARGIN));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }

    // A build only contains the states it was saved with; a block needs its
    // counterparts to be able to change at all.
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all(sim.registry_mut(), &mut table);
        *sim.behaviours_mut() = table;
    }

    assert_eq!(
        sim.unknown_report(),
        None,
        "{label}: every block must have behaviour, or this compares vanilla against \
         a partially-simulated world"
    );

    sim.record();
    // Placing a build gives every block a chance to react, exactly as vanilla's
    // onPlace does — which is why a piston notices a quasi-connectivity source that
    // touches it nowhere.
    sim.settle();

    let horizon = expected.ticks.last().map(|t| t.tick + 1).unwrap_or(0);
    sim.run(horizon);

    let actual = engine_trace(&sim, label, horizon).canonicalized();
    if let Some(divergence) = expected.diff(&actual) {
        panic!(
            "{label} diverges from vanilla at {divergence}\n\nexpected (vanilla):\n{}\n\nactual (engine):\n{}",
            expected.to_json().unwrap(),
            actual.to_json().unwrap()
        );
    }
}

#[test]
fn the_golden_traces_are_well_formed_and_from_the_right_version() {
    // A golden captured from another Minecraft version looks authoritative while
    // encoding different rules, which is worse than having none.
    let trace = golden("piston_qc.json");
    assert_eq!(trace.mc_version, "26.2");
    assert_eq!(trace.format_version, mc_tick_trace::FORMAT_VERSION);
    assert!(!trace.ticks.is_empty(), "a golden with no events proves nothing");
}

#[test]
fn piston_qc_matches_vanilla_tick_for_tick() {
    run_conformance("piston_qc.snbt", "piston_qc.json", "nucleation:piston_qc");
}

#[test]
fn slime_adhesion_matches_vanilla_tick_for_tick() {
    run_conformance("slime_drag.snbt", "slime_drag.json", "nucleation:slime_drag");
}
