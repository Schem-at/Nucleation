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

use mc_tick::behaviour::Inert;
use mc_tick::components::{PowerSource, StatePair};
use mc_tick::piston::{Movability, Piston, Sticky};
use mc_tick::{Dir, Pos, Simulation, StateId, Structure, World};
use mc_tick_trace::{Detail, EventKind, Trace, TracePos};

/// Padding round a loaded structure; matches the corpus runner.
const MARGIN: i32 = 4;

/// Minecraft's real block semantics for the small set a structure uses.
#[derive(Clone)]
struct Vanilla {
    powered: Vec<StateId>,
    immovable: Vec<StateId>,
    slime: Vec<StateId>,
    honey: Vec<StateId>,
}

impl PowerSource for Vanilla {
    fn is_powered(&self, world: &World, pos: Pos, _toward: Dir) -> bool {
        self.powered.contains(&world.get(pos))
    }
    fn is_diode(&self, _world: &World, _pos: Pos) -> bool {
        false
    }
    fn diode_facing(&self, _world: &World, _pos: Pos) -> Option<Dir> {
        None
    }
}

impl Movability for Vanilla {
    fn is_movable(&self, world: &World, pos: Pos) -> bool {
        let s = world.get(pos);
        s != StateId::AIR && !self.immovable.contains(&s)
    }
    fn sticky(&self, world: &World, pos: Pos) -> Option<Sticky> {
        let s = world.get(pos);
        if self.slime.contains(&s) {
            Some(Sticky::Slime)
        } else if self.honey.contains(&s) {
            Some(Sticky::Honey)
        } else {
            None
        }
    }
}

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
    let name = "piston_qc.snbt";
    let structure = structure(name);
    let expected = golden("piston_qc.json");

    let mut sim = Simulation::new(structure.bounds(MARGIN));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }

    // Intern the states this structure needs, then register their behaviour.
    let id = |sim: &mut Simulation, d: &str| sim.registry_mut().intern(d).unwrap();
    let stone = id(&mut sim, "minecraft:stone");
    let redstone_block = id(&mut sim, "minecraft:redstone_block");
    let piston_off = id(&mut sim, "minecraft:piston[extended=false,facing=east]");
    let piston_on = id(&mut sim, "minecraft:piston[extended=true,facing=east]");
    let head = id(&mut sim, "minecraft:piston_head[facing=east,short=false,type=normal]");
    let moving = id(&mut sim, "minecraft:moving_piston[facing=east,type=normal]");

    let model = Vanilla {
        powered: vec![redstone_block],
        immovable: vec![moving, head],
        slime: vec![],
        honey: vec![],
    };

    for (state, name) in [
        (stone, "stone"),
        (redstone_block, "redstone_block"),
        (head, "piston_head"),
        (moving, "moving_piston"),
    ] {
        sim.behaviours_mut().register(state, Box::new(Inert::new(name)));
    }
    for (state, extended) in [(piston_off, false), (piston_on, true)] {
        sim.behaviours_mut().register(
            state,
            Box::new(Piston {
                facing: Dir::East,
                extended,
                sticky: false,
                states: StatePair { off: piston_off, on: piston_on },
                head,
                moving,
                power: model.clone(),
                movability: model.clone(),
            }),
        );
    }

    assert_eq!(
        sim.unknown_report(),
        None,
        "every block in the structure must have behaviour, or the comparison is \
         between vanilla and a partially-simulated world"
    );

    // The capture's clock starts after placement has settled, so ours must too.
    sim.record();
    // Vanilla's redstone updates reach neighbours *and neighbours of neighbours*,
    // which is precisely what lets a quasi-connectivity source — diagonal to the
    // piston, touching it nowhere — ever be noticed. `TickCtx::set` only does the
    // first order, so the second is supplied here explicitly. See ROADMAP.md.
    sim.notify_neighbors(Pos::new(1, 2, 0)); // the redstone block
    sim.notify_neighbors(Pos::new(0, 2, 0)); // the space above the piston

    let horizon = expected.ticks.last().map(|t| t.tick + 1).unwrap_or(0);
    sim.run(horizon);

    let actual = engine_trace(&sim, "nucleation:piston_qc", horizon);

    if let Some(divergence) = expected.diff(&actual) {
        panic!(
            "engine diverges from vanilla at {divergence}\n\n\
             expected (vanilla):\n{}\n\nactual (engine):\n{}",
            expected.to_json().unwrap(),
            actual.to_json().unwrap()
        );
    }
}
