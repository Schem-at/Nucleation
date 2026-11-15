//! Tick-end conformance against fresh, playerless Minecraft 1.21.11 captures.
//!
//! `tools/gametest/src/TraceCapture12111.java` installs states directly into
//! chunk sections, loads native block-entity NBT, then captures before ticking.
//! These are raw world snapshots, not structure-placement or onPlace tests.
//! The oracle records the inclusive relative box [-2..7, -2..5, -2..2].
//! It does not establish intra-tick notification ordering or full BE equality.

use mc_test::{build_sim, SettleMode};
use mc_tick::{Pos, Simulation, Structure};
use serde::Deserialize;
use std::collections::BTreeMap;

type BlockRow = (i32, i32, i32, String);
type InventoryRow = (i32, i32, i32, u8, String, u8);
type ComparatorRow = (i32, i32, i32, u8);

#[derive(Deserialize)]
struct Capture {
    mc_version: String,
    data_version: i32,
    case: String,
    negative_control: bool,
    origin: [i32; 3],
    rng_seed: i64,
    size: [i32; 3],
    settle: String,
    capture_ticks: u64,
    fixture: Vec<FixtureBlock>,
    actions: Vec<Action>,
    initial: Snapshot,
    snapshots: Vec<Snapshot>,
}

#[derive(Deserialize)]
struct FixtureBlock {
    pos: [i32; 3],
    state: String,
    nbt: Option<String>,
}

#[derive(Deserialize)]
struct Action {
    tick: u64,
    #[serde(rename = "type")]
    kind: String,
    pos: [i32; 3],
    state: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Snapshot {
    tick: u64,
    players: u32,
    blocks: Vec<BlockRow>,
    inventories: Vec<InventoryRow>,
    comparator_outputs: Vec<ComparatorRow>,
}

fn pos([x, y, z]: [i32; 3]) -> Pos {
    Pos::new(x, y, z)
}

fn watched([x, y, z]: [i32; 3]) -> bool {
    (-2..=7).contains(&x) && (-2..=5).contains(&y) && (-2..=2).contains(&z)
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap()
}

fn palette_entry(descriptor: &str) -> String {
    let Some((name, properties)) = descriptor.split_once('[') else {
        return format!("{{Name:{}}}", quoted(descriptor));
    };
    let properties = properties.strip_suffix(']').expect("state properties");
    let properties = properties
        .split(',')
        .map(|pair| {
            let (key, value) = pair.split_once('=').expect("state property value");
            format!("{}:{}", quoted(key), quoted(value))
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{Name:{},Properties:{{{properties}}}}}", quoted(name))
}

fn simulation(capture: &Capture) -> Simulation {
    // Feed the actual captured fixture and its NBT through the shared loader.
    // The schema is already native 4671; no datafix or version substitution is
    // appropriate. InWorld was checked in build_sim: it skips BOTH onPlace
    // and the settle pass, while loading inventories/outputs and BE tickers.
    let palette = capture
        .fixture
        .iter()
        .map(|block| palette_entry(&block.state))
        .collect::<Vec<_>>()
        .join(",");
    let blocks = capture
        .fixture
        .iter()
        .enumerate()
        .map(|(index, block)| {
            let [x, y, z] = block.pos;
            let nbt = block
                .nbt
                .as_ref()
                .map(|nbt| format!(",nbt:{nbt}"))
                .unwrap_or_default();
            format!("{{pos:[{x},{y},{z}],state:{index}{nbt}}}")
        })
        .collect::<Vec<_>>()
        .join(",");
    let [sx, sy, sz] = capture.size;
    let structure = Structure::parse(&format!(
        "{{DataVersion:{},size:[{sx},{sy},{sz}],palette:[{palette}],blocks:[{blocks}],entities:[]}}",
        capture.data_version
    ))
    .expect("captured fixture must parse with its original BE NBT");
    assert_eq!(structure.data_version, Some(4671));
    let extra_states = capture
        .actions
        .iter()
        .filter_map(|action| action.state.clone())
        .collect::<Vec<_>>();
    let mut sim = build_sim(
        &structure,
        pos(capture.origin),
        SettleMode::InWorld,
        &extra_states,
        &[],
        structure.data_version,
        &capture.case,
    );
    sim.set_rng_seed(capture.rng_seed);
    sim
}

fn snapshot(sim: &Simulation) -> Snapshot {
    let mut blocks = Vec::new();
    let mut inventories = Vec::new();
    let mut comparator_outputs = Vec::new();
    // Match the oracle's exact watcher and scan order. Coordinates stay local;
    // the absolute Java origin is passed into register_all_at by build_sim.
    for y in -2..=5 {
        for z in -2..=2 {
            for x in -2..=7 {
                let position = Pos::new(x, y, z);
                let state = sim.world().get(position);
                let descriptor = sim.registry().descriptor(state).expect("registered state");
                if descriptor == "minecraft:air" {
                    continue;
                }
                blocks.push((x, y, z, descriptor.to_owned()));
                if let Some(inventory) = sim.inventory(position) {
                    let mut stacks = inventory
                        .stacks
                        .iter()
                        .filter(|stack| stack.count > 0)
                        .collect::<Vec<_>>();
                    stacks.sort_by_key(|stack| stack.slot);
                    for stack in stacks {
                        inventories.push((x, y, z, stack.slot, stack.id.clone(), stack.count));
                    }
                }
                if descriptor.split('[').next() == Some("minecraft:comparator") {
                    // Missing map entries have the same stored output as a
                    // freshly loaded vanilla comparator: zero. Include zero
                    // rows, and only compare currently present comparators.
                    let output = sim.comparator_outputs().get(&position).copied().unwrap_or(0);
                    comparator_outputs.push((x, y, z, output));
                }
            }
        }
    }
    Snapshot {
        tick: sim.tick_count(),
        players: 0,
        blocks,
        inventories,
        comparator_outputs,
    }
}

fn compare(expected: &Snapshot, actual: &Snapshot, label: &str) {
    assert_eq!(expected.players, 0, "{label}: playerless oracle");
    assert_eq!(actual.tick, expected.tick, "{label}: tick boundary");
    assert_eq!(actual.blocks, expected.blocks, "{label} tick {}: blocks", expected.tick);
    assert_eq!(
        actual.inventories, expected.inventories,
        "{label} tick {}: occupied slots (position, slot, item ID, count)", expected.tick
    );
    assert_eq!(
        actual.comparator_outputs, expected.comparator_outputs,
        "{label} tick {}: stored comparator outputs", expected.tick
    );
}

fn item_count(snapshot: &Snapshot, position: [i32; 3]) -> u32 {
    snapshot
        .inventories
        .iter()
        .filter(|(x, y, z, _, _, _)| [*x, *y, *z] == position)
        .map(|(_, _, _, _, _, count)| u32::from(*count))
        .sum()
}

fn block_at(snapshot: &Snapshot, position: [i32; 3]) -> &str {
    snapshot
        .blocks
        .iter()
        .find(|(x, y, z, _)| [*x, *y, *z] == position)
        .map(|(_, _, _, state)| state.as_str())
        .unwrap_or("minecraft:air")
}

fn validate_capture(capture: &Capture, case: &str, negative: bool) {
    assert_eq!(capture.mc_version, "1.21.11");
    assert_eq!(capture.data_version, 4671);
    assert_eq!(capture.case, case);
    assert_eq!(capture.negative_control, negative);
    assert_eq!(capture.origin, [0, 100, 0]);
    assert_eq!(capture.rng_seed, 0);
    assert_eq!(capture.size, [6, 4, 1]);
    assert_eq!(capture.settle, "in_world");
    assert_eq!(capture.initial.tick, 0);
    assert!(capture.capture_ticks >= 20, "capture must reach final quiescence");
    assert_eq!(capture.snapshots.len() as u64, capture.capture_ticks);
    let fixture = capture.fixture.iter().map(|b| (b.pos, b.state.as_str())).collect::<BTreeMap<_, _>>();
    let initial = capture.initial.blocks.iter().map(|(x, y, z, state)| ([*x, *y, *z], state.as_str())).collect::<BTreeMap<_, _>>();
    assert_eq!(fixture.len(), capture.fixture.len(), "duplicate fixture position");
    assert_eq!(initial.len(), capture.initial.blocks.len(), "duplicate captured position");
    assert!(fixture.keys().all(|position| watched(*position)));
    assert_eq!(fixture, initial, "raw placement must preserve every authored state");
    for (index, snapshot) in capture.snapshots.iter().enumerate() {
        assert_eq!(snapshot.tick, index as u64 + 1, "capture must contain every tick");
    }
    for action in &capture.actions {
        assert!((1..=capture.capture_ticks).contains(&action.tick));
    }
    let end = capture.snapshots.last().unwrap();
    if case == "comparator_drain" {
        assert!(capture.actions.is_empty());
        assert_eq!(item_count(&capture.initial, [2, 2, 0]), 3);
        assert_eq!(item_count(&capture.initial, [2, 1, 0]), 0);
        assert_eq!(capture.initial.comparator_outputs, vec![(3, 2, 0, 0)]);
        assert!(capture.initial.inventories.iter().all(|row| row.4 == "minecraft:redstone"));
        if negative {
            for snapshot in &capture.snapshots {
                assert_eq!(item_count(snapshot, [2, 2, 0]), 3, "locked hopper control");
                assert_eq!(item_count(snapshot, [2, 1, 0]), 0, "locked hopper control");
            }
        } else {
            assert_eq!(item_count(end, [2, 2, 0]), 0);
            assert_eq!(item_count(end, [2, 1, 0]), 3);
            assert!(capture.snapshots.iter().any(|s| s.comparator_outputs.iter().any(|row| row.3 > 0)));
            assert_eq!(end.comparator_outputs, vec![(3, 2, 0, 0)]);
        }
    } else {
        assert!(block_at(&capture.initial, [2, 1, 0]).contains("extended=false"));
        let expected_actions = if negative { vec![(1, "notify_neighbors"), (12, "notify_neighbors")] }
            else { vec![(1, "notify_neighbors"), (10, "set_block"), (12, "notify_neighbors")] };
        assert_eq!(capture.actions.iter().map(|a| (a.tick, a.kind.as_str())).collect::<Vec<_>>(), expected_actions);
        for action in &capture.actions {
            if action.kind == "notify_neighbors" {
                assert_eq!(action.pos, [2, 2, 0]);
                assert!(action.state.is_none());
            } else {
                assert_eq!(action.pos, [1, 2, 0]);
                assert_eq!(action.state.as_deref(), Some("minecraft:air"));
            }
        }
        if negative {
            assert!(capture.snapshots.iter().all(|s| block_at(s, [2, 1, 0]).contains("extended=false")));
        } else {
            assert!(capture.snapshots.iter().any(|s| block_at(s, [2, 1, 0]).contains("extended=true")));
            for tick in [10, 11] {
                assert!(block_at(&capture.snapshots[tick - 1], [2, 1, 0]).contains("extended=true"), "QC must wait for notification at tick 12");
            }
            assert!(block_at(end, [2, 1, 0]).contains("extended=false"));
        }
    }
}

fn run_capture(case: &str, negative: bool) {
    let label = format!("{case}{}", if negative { "_negative" } else { "" });
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/vanilla12111_execution_next")
        .join(format!("{label}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("missing vanilla capture {}: {error}", path.display()));
    let capture: Capture = serde_json::from_str(&text).expect("Minecraft 1.21.11 probe JSON");
    validate_capture(&capture, case, negative);
    let mut sim = simulation(&capture);
    compare(&capture.initial, &snapshot(&sim), &label);
    assert!(sim.recorded().is_empty(), "raw loading must not produce callbacks");
    for expected in &capture.snapshots {
        for action in capture.actions.iter().filter(|a| a.tick == expected.tick) {
            match action.kind.as_str() {
                // Probe: Level.updateNeighborsAt(airPos, Blocks.AIR, null).
                // This public API eagerly delivers the same six neighbor
                // callbacks; it does not include shape updates or placement.
                "notify_neighbors" => sim.notify_neighbors(pos(action.pos)),
                // Probe: Level.setBlock(pos, state, 3), at the tick boundary.
                "set_block" => {
                    let state = sim.registry().get(action.state.as_deref().expect("set state"))
                        .expect("action state interned before behavior registration");
                    sim.place_block(pos(action.pos), state);
                }
                other => panic!("unsupported oracle action {other}"),
            }
        }
        sim.step();
        compare(expected, &snapshot(&sim), &label);
        assert!(sim.unknown_report().is_none(), "{label}: unimplemented block");
    }
}

#[test]
fn comparator_drain_matches_fresh_vanilla_capture() {
    run_capture("comparator_drain", false);
}

#[test]
fn locked_hopper_matches_fresh_vanilla_control() {
    run_capture("comparator_drain", true);
}

#[test]
fn piston_quasi_connectivity_matches_fresh_vanilla_capture() {
    run_capture("piston_qc", false);
}

#[test]
fn unpowered_piston_matches_fresh_vanilla_control() {
    run_capture("piston_qc", true);
}
