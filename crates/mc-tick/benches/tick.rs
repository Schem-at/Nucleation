//! What a tick costs, on builds people actually run.
//!
//! Six criterion benches existed before this one and every one measured the
//! schematic side — `set_block`, regions, fingerprints. Nothing measured the
//! engine. That was deliberate while vanilla parity was the goal; parity is
//! good enough now that the ordering has inverted.
//!
//! Each bench asserts it did work. A flying-machine benchmark was measured
//! three times during the piston investigation before anyone noticed the
//! machine was not flying — its kick's block states had not been declared, so
//! they interned without behaviours and the redstone block did nothing. The
//! timings were stable, plausible and meaningless.

use std::time::{Duration, Instant};

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use mc_test::SettleMode;
use mc_tick::{Pos, Simulation, Structure};

const BB: &str = include_str!("../tests/corpus/structures/bb.snbt");
const ADDER32: &str = include_str!("../tests/corpus/structures/adder32.snbt");
const DOOR: &str = include_str!("../tests/corpus/structures/door_6x6_inworld.snbt");
const FLYER: &str = include_str!("../tests/corpus/structures/flying_machine_east.snbt");

/// Parse and settle, with the actuator states a build's inputs need declared
/// up front — interning them later yields an id with no behaviour, and the
/// actuation then silently does nothing.
fn sim(snbt: &str, settle: SettleMode) -> Simulation {
    let structure = Structure::parse(snbt).expect("fixture parses");
    let actuators = [
        "minecraft:redstone_block".to_string(),
        "minecraft:air".to_string(),
    ];
    mc_test::build_sim(
        &structure,
        Pos::new(0, 0, 0),
        settle,
        &actuators,
        &[],
        None,
        "bench",
    )
}

fn changes(s: &Simulation) -> usize {
    s.recorded().len()
}

/// Time `step()` in runs of `batch`, restoring a checkpoint between runs.
///
/// A machine that travels must not be measured with one long-lived
/// simulation: a flying machine crosses thousands of blocks over a
/// benchmark's iterations, the region grows to follow it, and the per-tick
/// cost climbs with it. The first run of this suite reported 52 microseconds
/// for a six-block flyer for exactly that reason. Restoring keeps every
/// sample measuring the same world.
fn timed_steps(c: &mut Criterion, name: &str, s: &mut Simulation, batch: u64) {
    let start = s.checkpoint();
    c.bench_function(name, |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            let mut done = 0;
            while done < iters {
                s.restore(&start);
                let n = (iters - done).min(batch);
                let t0 = Instant::now();
                for _ in 0..n {
                    s.step();
                }
                total += t0.elapsed();
                done += n;
            }
            total
        })
    });
}

/// BB: 4,820 blocks, a flying machine mid-flight. The heaviest per-tick case
/// here — pistons resolving structures, blocks landing, observers pulsing.
fn bench_bb(c: &mut Criterion) {
    let mut s = sim(BB, SettleMode::InWorld);
    let air = s.registry_mut().intern("minecraft:air").expect("air");
    s.record();
    s.place_block(Pos::new(31, 7, 13), air); // mine the obsidian: the trigger
                                             // Long enough to be unambiguously running: the first stroke lands around
                                             // t6 and the machine is in full swing by t30.
    for _ in 0..30 {
        s.step();
    }
    assert!(
        changes(&s) > 50,
        "BB did not start: {} changes",
        changes(&s)
    );

    timed_steps(c, "tick/bb", &mut s, 200);
}

/// A settled 32-bit adder: 2,732 blocks that are *quiet*. Big-and-idle is a
/// different cost from big-and-busy, and it is the one dominated by whatever
/// the tick loop does per block rather than per change.
fn bench_adder_idle(c: &mut Criterion) {
    let mut s = sim(ADDER32, SettleMode::Placement);
    s.record();
    for _ in 0..200 {
        s.step();
    }
    assert!(
        s.is_quiescent(),
        "adder had not settled; this would measure work, not idleness"
    );
    timed_steps(c, "tick/adder32_idle", &mut s, 500);
}

/// The same adder from parse to answer. A computational build does its work
/// during placement and the ticks just after it, so this — not `tick/*` — is
/// where its cost lives.
fn bench_adder_solve(c: &mut Criterion) {
    c.bench_function("solve/adder32", |b| {
        b.iter_batched(
            || Structure::parse(ADDER32).expect("parses"),
            |structure| {
                let actuators = [
                    "minecraft:redstone_block".to_string(),
                    "minecraft:air".to_string(),
                ];
                let mut s = mc_test::build_sim(
                    &structure,
                    Pos::new(0, 0, 0),
                    SettleMode::Placement,
                    &actuators,
                    &[],
                    None,
                    "bench",
                );
                while !s.is_quiescent() {
                    s.step();
                }
                s
            },
            BatchSize::SmallInput,
        )
    });

    // Sanity, outside the timing: the sum really is computed.
    let mut s = sim(ADDER32, SettleMode::Placement);
    s.record();
    for _ in 0..400 {
        s.step();
    }
    let bit = |i: i32| {
        s.registry()
            .descriptor(s.world().get(Pos::new(0, 3 + 2 * i, 2)))
            .unwrap_or_default()
            .contains("powered=true")
    };
    let sum: u64 = (0..32).filter(|i| bit(*i)).map(|i| 1u64 << i).sum();
    assert_eq!(
        sum, 0xF0E2_1568,
        "adder computed {sum:#x}, fixture expects DEADBEEF+12345678+1"
    );
}

/// A redstone-dense piston door: observers, clocks, wire, and a structure
/// resolve every stroke.
fn bench_door(c: &mut Criterion) {
    let mut s = sim(DOOR, SettleMode::InWorld);
    s.record();
    // Open it. Without this the door just stands there and the bench measures
    // an idle world — which is what the first run of this suite did, reporting
    // exactly the same 1.15us as the idle adder.
    s.use_block(Pos::new(10, 4, 1));
    for _ in 0..4 {
        s.step();
    }
    assert!(
        changes(&s) > 10,
        "door did not open: {} changes",
        changes(&s)
    );
    timed_steps(c, "tick/door_6x6", &mut s, 100);
}

/// Six blocks in flight. Whatever this costs is the fixed price of a tick,
/// because there is almost nothing in the world to spend it on.
fn bench_flyer(c: &mut Criterion) {
    let mut s = sim(FLYER, SettleMode::Quiet);
    let redstone = s
        .registry_mut()
        .intern("minecraft:redstone_block")
        .expect("interns");
    let air = s.registry_mut().intern("minecraft:air").expect("interns");
    s.record();
    s.step();
    s.step();
    s.place_block(Pos::new(2, 1, 1), redstone);
    s.step();
    s.step();
    s.place_block(Pos::new(2, 1, 1), air);
    for _ in 0..40 {
        s.step();
    }
    let reach = |s: &Simulation| {
        s.world()
            .iter_non_air()
            .map(|(p, _)| p.x)
            .max()
            .unwrap_or(0)
    };
    assert!(
        reach(&s) > 5,
        "flyer never took off; it reached x={}",
        reach(&s)
    );

    timed_steps(c, "tick/flyer", &mut s, 200);
}

/// Opt-in timeline cost, beside the identical run with recording disabled.
///
/// Recorder setup and reset stay outside the timer. Each sample restores the
/// same in-flight checkpoint, so this measures per-tick capture and hashing
/// rather than construction, cloning an accumulated timeline, or world growth.
fn bench_timeline_recording(c: &mut Criterion) {
    fn in_flight() -> Simulation {
        let mut s = sim(FLYER, SettleMode::Quiet);
        let redstone = s
            .registry_mut()
            .intern("minecraft:redstone_block")
            .expect("interns");
        let air = s.registry_mut().intern("minecraft:air").expect("interns");
        s.run(2);
        s.place_block(Pos::new(2, 1, 1), redstone);
        s.run(2);
        s.place_block(Pos::new(2, 1, 1), air);
        s.run(40);
        let reach = s
            .world()
            .iter_non_air()
            .map(|(p, _)| p.x)
            .max()
            .unwrap_or(0);
        assert!(reach > 5, "flyer never took off; it reached x={reach}");
        s
    }

    let mut plain = in_flight();
    let plain_start = plain.checkpoint();
    c.bench_function("recording/flyer_unrecorded", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            let mut done = 0;
            while done < iters {
                plain.restore(&plain_start);
                let n = (iters - done).min(200);
                let t0 = Instant::now();
                plain.run(n);
                total += t0.elapsed();
                done += n;
            }
            total
        })
    });
    assert!(
        plain.recorded_timeline().is_none(),
        "the unrecorded arm accidentally enabled the timeline"
    );

    let mut recorded = in_flight();
    let recorded_start = recorded.checkpoint();
    recorded.record_timeline();
    recorded.step();
    assert!(
        recorded
            .recorded_timeline()
            .is_some_and(|timeline| timeline.frames.len() == 2),
        "the recorded arm did not capture a verification frame"
    );
    c.bench_function("recording/flyer_recorded", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            let mut done = 0;
            while done < iters {
                recorded.restore(&recorded_start);
                recorded.record_timeline();
                let n = (iters - done).min(200);
                let t0 = Instant::now();
                recorded.run(n);
                total += t0.elapsed();
                done += n;
            }
            total
        })
    });
}

/// What a search pays per candidate: parse, place, settle, run, discard.
/// Construction is invisible to every `tick/*` bench above and a genetic
/// search pays it once per machine.
fn bench_batch(c: &mut Criterion) {
    c.bench_function("batch/build_and_run", |b| {
        b.iter_batched(
            || Structure::parse(FLYER).expect("parses"),
            |structure| {
                let mut s = mc_test::build_sim(
                    &structure,
                    Pos::new(0, 0, 0),
                    SettleMode::Quiet,
                    &[],
                    &[],
                    None,
                    "bench",
                );
                for _ in 0..100 {
                    s.step();
                }
                s
            },
            BatchSize::SmallInput,
        )
    });
}

/// Construction alone: parse, place, intern, wire block entities, settle.
/// Split out because `batch/build_and_run` suggested it dwarfs the ticking,
/// and a search pays it once per candidate.
fn bench_construct(c: &mut Criterion) {
    for (name, snbt, settle) in [
        ("construct/flyer", FLYER, SettleMode::Quiet),
        ("construct/adder32", ADDER32, SettleMode::Placement),
        ("construct/bb", BB, SettleMode::InWorld),
        // The same build under all three settle modes. InWorld does no
        // placement pass at all, Quiet runs `onPlace` only, Placement adds the
        // ordered settle — so the gaps between these three are exactly what
        // each pass costs.
        ("construct/adder32_inworld", ADDER32, SettleMode::InWorld),
        ("construct/adder32_quiet", ADDER32, SettleMode::Quiet),
    ] {
        c.bench_function(name, |b| {
            b.iter_batched(
                || Structure::parse(snbt).expect("parses"),
                |structure| {
                    mc_test::build_sim(
                        &structure,
                        Pos::new(0, 0, 0),
                        settle,
                        &[],
                        &[],
                        None,
                        "bench",
                    )
                },
                BatchSize::SmallInput,
            )
        });
    }
    // World allocation + block placement only, without the block-entity,
    // comparator and ticker wiring `build_sim` does afterwards. The gap
    // between this and `construct/adder32_inworld` is that wiring.
    c.bench_function("construct/adder32_placeonly", |b| {
        b.iter_batched(
            || Structure::parse(ADDER32).expect("parses"),
            |structure| {
                let mut sim = Simulation::new(structure.bounds(4));
                {
                    let (registry, world) = sim.registry_and_world_mut();
                    structure.place(world, registry, Pos::new(0, 0, 0));
                }
                sim
            },
            BatchSize::SmallInput,
        )
    });

    // And parsing on its own, to see how much of construction is just text.
    c.bench_function("parse/adder32", |b| {
        b.iter(|| Structure::parse(ADDER32).expect("parses"))
    });
}

criterion_group!(
    benches,
    bench_bb,
    bench_adder_idle,
    bench_adder_solve,
    bench_door,
    bench_flyer,
    bench_timeline_recording,
    bench_batch,
    bench_construct
);
criterion_main!(benches);
