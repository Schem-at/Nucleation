//! Reproducible headless circuit timings and ordered differential fingerprints.
//! Usage: bench_circuits CASE [SAMPLES=1] [ITERATIONS=1], or --verify.
use cpu_time::ThreadTime;
use mc_test::{build_sim, SettleMode};
use mc_tick::{Pos, Simulation, StateId, Structure};
use serde_json::json;
use std::time::Instant;

const CASES: &[&str] = &[
    "hipster",
    "hipster_unrecorded",
    "hipster_updates",
    "hipster_timeline",
    "door6",
    "flyer",
    "bb",
    "adder_idle",
    "hipster_load",
];

const EXTERNAL: &[&str] = &["item_sorter", "hopper_transfer_speed", "hopper_storagecart_interaction", "comparator_update_collection", "trapdoor2x3"];
// Kept runnable to reproduce a discovered functional failure; never part of
// the accepted timing set. It closes but leaves quartz in the opening.
const PROBES: &[&str] = &["shutter4"];
fn is_external(case: &str) -> bool { EXTERNAL.contains(&case) || PROBES.contains(&case) }
const EXTRA: &[&str] = &["comparator_drain", "item_into_hopper"];

fn fixture_dir() -> std::path::PathBuf {
    std::env::var_os("MC_TICK_BENCH_FIXTURES").map(Into::into)
        .unwrap_or_else(|| "tests/benchmark-fixtures".into())
}

fn load(case: &str) -> Simulation {
    let (fixture, settle) = match case {
        c if c.starts_with("hipster") => ("hipster_door", SettleMode::InWorld),
        "door6" => ("door_6x6_inworld", SettleMode::InWorld),
        "flyer" => ("flying_machine_east", SettleMode::Quiet),
        "bb" => ("bb", SettleMode::InWorld),
        "adder_idle" => ("adder32", SettleMode::Placement),
        c if EXTRA.contains(&c) => (c, SettleMode::Placement),
        c if is_external(c) => (c, if matches!(c, "shutter4" | "trapdoor2x3") { SettleMode::InWorld } else { SettleMode::Placement }),
        _ => panic!("unknown case {case}; choose {CASES:?}"),
    };
    let external = is_external(case).then(|| std::fs::read_to_string(fixture_dir().join(format!("{case}.snbt"))).expect("prepare external benchmark fixtures first"));
    let source = match fixture {
        "hipster_door" => include_str!("../tests/corpus/structures/hipster_door.snbt"),
        "door_6x6_inworld" => include_str!("../tests/corpus/structures/door_6x6_inworld.snbt"),
        "flying_machine_east" => include_str!("../tests/corpus/structures/flying_machine_east.snbt"),
        "bb" => include_str!("../tests/corpus/structures/bb.snbt"),
        "adder32" => include_str!("../tests/corpus/structures/adder32.snbt"),
        "comparator_drain" => include_str!("../tests/corpus/structures/comparator_drain.snbt"),
        "item_into_hopper" => include_str!("../tests/corpus/structures/item_into_hopper.snbt"),
        _ => external.as_deref().unwrap(),
    };
    let structure = Structure::parse(source).expect("parse fixture");
    let extra = [
        "minecraft:redstone_block".to_owned(),
        "minecraft:air".to_owned(),
    ];
    let mut sim = build_sim(
        &structure,
        Pos::default(),
        settle,
        &extra,
        &[],
        if case.starts_with("hipster") {
            Some(2730)
        } else {
            None
        },
        case,
    );
    sim.set_rng_seed(0);
    assert!(sim.unknown_report().is_none());
    sim
}

fn prepare(case: &str) -> Simulation {
    let mut sim = load(case);
    match case {
        "flyer" => {
            let redstone = sim.registry().get("minecraft:redstone_block").unwrap();
            sim.run(2);
            sim.place_block(Pos::new(2, 1, 1), redstone);
            sim.run(2);
            sim.place_block(Pos::new(2, 1, 1), StateId::AIR);
            sim.run(40);
            assert!(
                sim.world().iter_non_air().any(|(p, _)| p.x > 5),
                "flyer must move"
            );
        }
        "bb" => {
            sim.place_block(Pos::new(31, 7, 13), StateId::AIR);
            sim.run(30);
            assert!(sim.recorded().len() > 100, "BB must be active");
        }
        "adder_idle" => {
            sim.run(200);
            assert!(sim.is_quiescent());
        }
        _ => {}
    }
    if is_external(case) && !matches!(case, "shutter4" | "trapdoor2x3") {
        sim.set_random_ticks(3);
        sim.run(10);
        let rearm: Vec<_> = sim.world().iter_non_air().filter_map(|(pos, id)| {
            let d = sim.registry().descriptor(id)?;
            if d.starts_with("minecraft:test_block") && d.contains("fired=true") {
                sim.registry().get(&d.replace("fired=true", "fired=false")).map(|id| (pos, id))
            } else { None }
        }).collect();
        for (pos, id) in rearm { sim.registry_and_world_mut().1.set(pos, id); }
    }
    // Both verification and timed iterations start from this exact RNG state,
    // including workloads whose setup ticks consumed random numbers.
    sim.set_rng_seed(0);
    sim
}

fn recording(sim: &mut Simulation, case: &str, verify: bool) {
    if case == "hipster_unrecorded" {
        sim.discard_recording();
    } else if case == "hipster_timeline" {
        sim.record_timeline();
    } else {
        sim.record();
    }
    sim.record_updates(verify || case == "hipster_updates");
}

fn ticks(case: &str) -> u64 {
    match case {
        c if c.starts_with("hipster") => 320,
        "door6" => 100,
        "flyer" | "bb" => 200,
        "adder_idle" => 500,
        "comparator_drain" | "item_into_hopper" => 40,
        "shutter4" | "trapdoor2x3" => 240,
        c if is_external(c) => 400,
        _ => unreachable!(),
    }
}

fn step(sim: &mut Simulation, case: &str, tick: u64) {
    if (case.starts_with("hipster") && matches!(tick % 320, 10 | 170))
        || (case == "door6" && tick == 0)
    {
        sim.use_block(if case == "door6" {
            Pos::new(10, 4, 1)
        } else {
            Pos::new(12, 7, 13)
        });
    }
    if tick == 0 {
        let start = match case {
            "item_sorter" => Some(Pos::new(9, 5, 2)),
            "hopper_transfer_speed" => Some(Pos::new(2, 2, 3)),
            "hopper_storagecart_interaction" => Some(Pos::new(6, 2, 5)),
            "comparator_update_collection" => Some(Pos::new(1, 3, 12)),
            _ => None,
        };
        if let Some(pos) = start {
            sim.place_block(pos, sim.registry().get("minecraft:redstone_block").unwrap());
        }
    }
    if matches!(case, "shutter4" | "trapdoor2x3") && matches!(tick % 120, 10 | 70) {
        sim.use_block(if case == "shutter4" { Pos::new(0, 1, 7) } else { Pos::new(0, 3, 7) });
    }
    sim.step();
}

fn validate(sim: &mut Simulation, case: &str) {
    assert!(sim.unknown_report().is_none());
    let accept = match case {
        "item_sorter" => Some(Pos::new(11, 2, 10)),
        "hopper_transfer_speed" => Some(Pos::new(1, 1, 14)),
        "hopper_storagecart_interaction" => Some(Pos::new(5, 1, 25)),
        "comparator_update_collection" => Some(Pos::new(8, 1, 26)),
        _ => None,
    };
    if let Some(pos) = accept {
        assert!(sim.registry().descriptor(sim.world().get(pos)).unwrap().contains("fired=true"), "{case}: upstream accept block did not fire");
        assert!(!sim.recorded_inventory().is_empty(), "{case}: no inventory activity");
    }
    match case {
        "hipster" => assert_eq!(sim.recorded().len(), if sim.tick_count() >= 640 { 17968 } else { 8984 }),
        "comparator_drain" => {
            assert!(sim.inventory(Pos::new(0, 2, 0)).unwrap().stacks.is_empty());
            assert_eq!(sim.inventory(Pos::new(0, 1, 0)).unwrap().stacks.iter().map(|s| s.count as u64).sum::<u64>(), 3);
        }
        "item_into_hopper" => {
            assert!(sim.item_entities().is_empty());
            assert_eq!(sim.inventory(Pos::new(0, 1, 0)).unwrap().stacks.iter().map(|s| s.count as u64).sum::<u64>(), 2);
        }
        "shutter4" | "trapdoor2x3" => {
            assert!(sim.recorded().len() > 100, "{case}: inactive door");
            let cells: Vec<_> = if case == "shutter4" {
                (4..=7).flat_map(|y| (4..=7).map(move |z| Pos::new(4, y, z))).collect()
            } else {
                (2..=4).flat_map(|x| (3..=4).map(move |z| Pos::new(x, 4, z))).collect()
            };
            for tick in [60, 120, 180, 240] {
                for &pos in &cells {
                    let state = sim.recorded().iter().rev().find(|r| r.pos == pos && r.tick < tick)
                        .map_or(StateId::AIR, |r| r.to);
                    let expected = if tick % 120 == 0 { "minecraft:air" }
                        else if case == "shutter4" { "minecraft:quartz_block" } else { "minecraft:orange_wool" };
                    assert_eq!(sim.registry().descriptor(state).unwrap(), expected,
                        "{case}: doorway at {pos:?} after {tick} ticks");
                }
            }
            let mut initial = std::collections::BTreeMap::new();
            for r in sim.recorded() { initial.entry(r.pos).or_insert(r.from); }
            for (pos, state) in initial {
                assert_eq!(sim.world().get(pos), state, "{case}: two cycles must restore every changed block at {pos:?}");
            }
        },
        _ => {}
    }
}

fn hash_tick(hash: &mut blake3::Hasher, sim: &Simulation) {
    hash.update(&sim.tick_count().to_le_bytes());
    for (pos, state) in sim.world().iter_non_air() {
        for v in [pos.x, pos.y, pos.z] {
            hash.update(&v.to_le_bytes());
        }
        let descriptor = sim.registry().descriptor(state).unwrap().as_bytes();
        hash.update(&(descriptor.len() as u64).to_le_bytes());
        hash.update(descriptor);
    }
    hash.update(format!("{:?}{:?}", sim.pending_ticks(), sim.pending_events()).as_bytes());
}

fn verify(cases: &[&str]) {
    for &case in cases.iter().filter(|&&c| c != "hipster_load") {
        let mut sim = prepare(case);
        recording(&mut sim, case, true);
        let ticks = if case.starts_with("hipster") {
            640
        } else {
            ticks(case)
        };
        let mut hash = blake3::Hasher::new();
        for tick in 0..ticks {
            step(&mut sim, case, tick);
            hash_tick(&mut hash, &sim);
        }
        hash.update(
            format!(
                "{:?}{:?}{:?}{:?}",
                sim.recorded(),
                sim.recorded_updates(),
                sim.recorded_inventory(),
                sim.recorded_entities()
            )
            .as_bytes(),
        );
        if case == "hipster_timeline" {
            hash.update(format!("{:?}", sim.recorded_timeline()).as_bytes());
        }
        validate(&mut sim, case);
        println!(
            "{}",
            json!({"case":case,"ticks":ticks,"changes":sim.recorded().len(),
            "updates":sim.recorded_updates().len(),"inventory_changes":sim.recorded_inventory().len(),"entity_changes":sim.recorded_entities().len(),"fingerprint":hash.finalize().to_hex().to_string()})
        );
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).is_some_and(|a| a == "--verify") {
        let cases: Vec<_> = if args.get(2).is_some_and(|a| a == "--expanded") {
            CASES.iter().chain(EXTRA).chain(EXTERNAL).copied().collect()
        } else { CASES.to_vec() };
        verify(&cases);
        return;
    }
    if args.get(1).is_some_and(|a| a == "--verify-case") {
        verify(&[args.get(2).expect("case")]); return;
    }
    let case = args.get(1).map(String::as_str).unwrap_or("hipster");
    assert!(CASES.contains(&case) || EXTRA.contains(&case) || is_external(case));
    let samples: usize = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(1);
    let iterations: usize = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(1);
    assert!(samples > 0 && iterations > 0);
    let loading = case == "hipster_load";
    let mut sim = prepare(case);
    let checkpoint = sim.checkpoint();
    let mut wall = Vec::new();
    let mut cpu = Vec::new();
    let mut changes = 0;
    // One untimed warmup sample, with exactly the measured iteration count.
    for sample in 0..=samples {
        let (mut elapsed, mut consumed) = (0u128, 0u128);
        for _ in 0..iterations {
            if loading {
                let cpu_start = ThreadTime::now();
                let start = Instant::now();
                let loaded = load(case);
                elapsed += start.elapsed().as_nanos();
                consumed += cpu_start.elapsed().as_nanos();
                std::hint::black_box(&loaded);
                drop(loaded); // destruction is outside the load timer
            } else {
                sim.restore(&checkpoint);
                sim.set_rng_seed(0);
                recording(&mut sim, case, false);
                let cpu_start = ThreadTime::now();
                let start = Instant::now();
                for tick in 0..ticks(case) {
                    step(&mut sim, case, tick);
                }
                elapsed += start.elapsed().as_nanos();
                consumed += cpu_start.elapsed().as_nanos();
                changes = sim.recorded().len();
                validate(&mut sim, case);
            }
        }
        if sample > 0 {
            wall.push(elapsed);
            cpu.push(consumed);
        }
    }
    println!(
        "{}",
        json!({"case":case,"ticks":if loading {0} else {ticks(case)},
        "iterations":iterations,"changes":changes,"samples_ns":wall,"samples_cpu_ns":cpu})
    );
}
