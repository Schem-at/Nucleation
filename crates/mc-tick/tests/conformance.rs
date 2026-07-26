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
///
/// For the same reason, same-tick changes at one position are collapsed to their
/// **net** change. A retraction writes `piston_head -> air -> moving_piston` into
/// the head slot within one tick; a snapshot diff can only ever see
/// `piston_head -> moving_piston`, and a no-op round trip not at all.
fn engine_trace(sim: &Simulation, name: &str, ticks: u64) -> Trace {
    let mut trace = Trace::new("26.2", name, Detail::Normal);
    for tick in 0..ticks {
        let mut first_seen: Vec<mc_tick::Pos> = Vec::new();
        let mut net: std::collections::HashMap<mc_tick::Pos, (mc_tick::StateId, mc_tick::StateId)> =
            std::collections::HashMap::new();
        for c in sim.recorded().iter().filter(|c| c.tick == tick) {
            match net.entry(c.pos) {
                std::collections::hash_map::Entry::Vacant(slot) => {
                    slot.insert((c.from, c.to));
                    first_seen.push(c.pos);
                }
                std::collections::hash_map::Entry::Occupied(mut slot) => {
                    slot.get_mut().1 = c.to;
                }
            }
        }
        let events: Vec<_> = first_seen
            .into_iter()
            .filter_map(|pos| {
                let (from, to) = net[&pos];
                if from == to {
                    return None; // a round trip is invisible between ticks
                }
                Some(mc_tick_trace::TraceEvent {
                    phase: "tick_end".to_string(),
                    kind: EventKind::BlockChanged {
                        pos: TracePos::new(pos.x, pos.y, pos.z),
                        from: sim.registry().descriptor(from).unwrap_or("?").to_string(),
                        to: sim.registry().descriptor(to).unwrap_or("?").to_string(),
                    },
                })
            })
            .collect();
        if !events.is_empty() {
            trace.ticks.push(mc_tick_trace::TickRecord { tick, events });
        }
    }
    trace
}

/// An actuation applied at a tick boundary, mirroring the capture tool's flags.
///
/// The capture applies `--break`/`--pulse`/`--use` *between* server ticks; the
/// tick number here is the tick that will run next — the one whose snapshot diff
/// records the action, which is how the golden buckets it.
enum Actuate {
    /// Write a block state, as `--pulse` placing or removing its source does.
    /// Breaking a block is placing `minecraft:air`.
    Place(Pos, &'static str),
    /// Right-click with an empty hand, as `--use` does.
    Use(Pos),
}

/// Load a structure, wire vanilla behaviour to it, settle, actuate, and run.
///
/// No hand-registration: `mc_tick::vanilla` turns descriptors into behaviour, which
/// is what makes running an arbitrary schematic possible at all. `extra_states` are
/// interned before behaviours are bound, so an actuation may introduce a block the
/// structure itself never mentions (a pulse's redstone block, typically).
fn run_conformance_actuated(
    structure_file: &str,
    golden_file: &str,
    label: &str,
    extra_states: &[&str],
    actions: &[(u64, Actuate)],
) {
    run_conformance_bounded(structure_file, golden_file, label, extra_states, actions, None)
}

fn run_conformance_bounded(
    structure_file: &str,
    golden_file: &str,
    label: &str,
    extra_states: &[&str],
    actions: &[(u64, Actuate)],
    ticking: Option<mc_tick::Bounds>,
) {
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
    for descriptor in extra_states {
        sim.registry_mut()
            .intern(descriptor)
            .unwrap_or_else(|e| panic!("{label}: interning {descriptor}: {e:?}"));
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

    if let Some(bounds) = ticking {
        sim.set_ticking_bounds(bounds);
    }

    sim.record();
    // Placing a build gives every block a chance to react, exactly as vanilla's
    // onPlace does — which is why a piston notices a quasi-connectivity source that
    // touches it nowhere, and why every observer pulses once at placement.
    sim.settle();

    let horizon = expected.ticks.last().map(|t| t.tick + 1).unwrap_or(0);
    for tick in 0..horizon {
        for (at, action) in actions {
            if *at != tick {
                continue;
            }
            match action {
                Actuate::Place(pos, descriptor) => {
                    let state = sim
                        .registry()
                        .get(descriptor)
                        .unwrap_or_else(|| panic!("{label}: {descriptor} was not interned"));
                    sim.place_block(*pos, state);
                }
                Actuate::Use(pos) => sim.use_block(*pos),
            }
        }
        sim.step();
    }

    let actual = engine_trace(&sim, label, horizon).canonicalized();
    if let Some(divergence) = expected.diff(&actual) {
        panic!(
            "{label} diverges from vanilla at {divergence}\n\nexpected (vanilla):\n{}\n\nactual (engine):\n{}",
            expected.to_json().unwrap(),
            actual.to_json().unwrap()
        );
    }
}

/// The no-actuation case: settle and run.
fn run_conformance(structure_file: &str, golden_file: &str, label: &str) {
    run_conformance_actuated(structure_file, golden_file, label, &[], &[]);
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

#[test]
fn the_manual_engine_runs_its_placement_cycle_tick_for_tick() {
    // The first real community schematic: a 2-step slimestone flying-machine
    // engine (`tools/gametest/samples/manual_engine.litematic`). Placement pulses
    // its observers, which acts as one trigger: the machine advances two full
    // 9-game-tick steps — pistons, slime adhesion, moved observers re-pulsing —
    // and stops at tick 21. Twenty-one ticks of interlocking behaviour, with no
    // actuation at all.
    //
    // The ticking bounds mirror the capture: its origin sat exactly on a chunk
    // corner, so blocks the machine pushes to x < 0 land in a chunk that is
    // loaded but not block-entity-ticking. Their moving_piston placeholders
    // freeze there — and, being immovable, they are what *stops* the machine
    // after its second step. Without the bounds the engine happily resolves
    // them and the machine takes a third step vanilla never took.
    run_conformance_bounded(
        "manual_engine.snbt",
        "manual_engine_settle.json",
        "nucleation:manual_engine",
        &[],
        &[],
        Some(mc_tick::Bounds::new(Pos::new(0, -4, 0), Pos::new(15, 7, 15))),
    );
}

#[test]
fn clicking_the_manual_engine_advances_it_two_more_steps() {
    // The same engine, placed with room to fly: padded to the east end of the
    // capture's chunk so nothing crosses the frozen chunk border. Placement
    // runs the first two steps (ticks 0-25); the world then goes quiet; a
    // right-click on the note block — at [13,0,2], where the machine's two
    // steps left it — cycles its pitch on tick 30, the observers see the
    // change, and the engine runs a complete second activation through
    // tick 55. Fifty-five ticks, two full activations, one of them started by
    // the player-input path.
    run_conformance_bounded(
        "manual_engine_padded.snbt",
        "manual_engine_click.json",
        "nucleation:manual_engine_padded",
        &[],
        &[(30, Actuate::Use(Pos::new(13, 0, 2)))],
        Some(mc_tick::Bounds::new(Pos::new(0, -4, 0), Pos::new(15, 7, 15))),
    );
}

#[test]
fn a_note_block_follows_neighbour_power_synchronously() {
    // Captured with `--pulse 1,0,0 --pulse-ticks 2`: the powered flag flips on the
    // same tick the source appears and again on the tick it vanishes — NoteBlock
    // has no scheduled delay at all.
    run_conformance_actuated(
        "note_powered.snbt",
        "note_powered.json",
        "nucleation:note_powered",
        &["minecraft:redstone_block"],
        &[
            (0, Actuate::Place(Pos::new(1, 0, 0), "minecraft:redstone_block")),
            (2, Actuate::Place(Pos::new(1, 0, 0), "minecraft:air")),
        ],
    );
}

#[test]
fn clicking_a_note_block_cycles_its_pitch_and_the_observer_sees_it() {
    // Captured with `--use 0,0,0 --use-tick 6`. Three things in one golden:
    // the observer's placement pulse (ticks 1 and 3), the click cycling `note`
    // on its own tick (6), and the observer pulsing one tick after the click
    // (7 and 9) — boundary scheduling, not the in-phase two-tick offset.
    run_conformance_actuated(
        "note_click.snbt",
        "note_click.json",
        "nucleation:note_click",
        &[],
        &[(6, Actuate::Use(Pos::new(0, 0, 0)))],
    );
}
