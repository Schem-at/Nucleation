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

fn load(case: &str) -> Simulation {
    let (fixture, settle) = match case {
        c if c.starts_with("hipster") => ("hipster_door", SettleMode::InWorld),
        "door6" => ("door_6x6_inworld", SettleMode::InWorld),
        "flyer" => ("flying_machine_east", SettleMode::Quiet),
        "bb" => ("bb", SettleMode::InWorld),
        "adder_idle" => ("adder32", SettleMode::Placement),
        _ => panic!("unknown case {case}; choose {CASES:?}"),
    };
    let source = match fixture {
        "hipster_door" => include_str!("../tests/corpus/structures/hipster_door.snbt"),
        "door_6x6_inworld" => include_str!("../tests/corpus/structures/door_6x6_inworld.snbt"),
        "flying_machine_east" => include_str!("../tests/corpus/structures/flying_machine_east.snbt"),
        "bb" => include_str!("../tests/corpus/structures/bb.snbt"),
        "adder32" => include_str!("../tests/corpus/structures/adder32.snbt"),
        _ => unreachable!(),
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
    sim.step();
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

fn verify() {
    for &case in CASES.iter().filter(|&&c| c != "hipster_load") {
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
        assert!(sim.unknown_report().is_none());
        println!(
            "{}",
            json!({"case":case,"ticks":ticks,"changes":sim.recorded().len(),
            "updates":sim.recorded_updates().len(),"fingerprint":hash.finalize().to_hex().to_string()})
        );
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).is_some_and(|a| a == "--verify") {
        verify();
        return;
    }
    let case = args.get(1).map(String::as_str).unwrap_or("hipster");
    assert!(CASES.contains(&case));
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
                assert!(sim.unknown_report().is_none());
                if case == "hipster" {
                    assert_eq!(changes, 8984);
                }
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
