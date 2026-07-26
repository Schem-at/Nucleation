//! Engine throughput on a real contraption.
//!
//!     cargo run -p mc-tick --release --example bench_manual_engine
//!
//! Two numbers matter for the timing product:
//! - **active** ticks: the manual engine mid-activation — pistons resolving,
//!   observers pulsing, slime dragging. One iteration = click + one full
//!   activation from a reset world.
//! - **quiescent** ticks: nothing scheduled. This is what a simulation pays
//!   while waiting between actuations, and it should be near-free.
use mc_tick::{Pos, Simulation, Structure};
use std::time::Instant;

fn main() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/corpus/structures/manual_engine_padded.snbt"
    );
    let text = std::fs::read_to_string(path).expect("read structure");
    let structure = Structure::parse(&text).expect("parse structure");

    let mut sim = Simulation::new(structure.bounds(4));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all(sim.registry_mut(), &mut table);
        *sim.behaviours_mut() = table;
    }
    assert_eq!(sim.unknown_report(), None);
    sim.mark_initial();

    // One warm-up activation, so allocation growth is out of the measurement.
    sim.use_block(Pos::new(15, 0, 2));
    sim.run(35);

    let iterations = 2_000u64;
    let ticks_per = 35u64;
    let start = Instant::now();
    for _ in 0..iterations {
        sim.reset();
        sim.use_block(Pos::new(15, 0, 2));
        sim.run(ticks_per);
    }
    let active = start.elapsed();
    let active_ticks = iterations * ticks_per;
    let per_active_tick = active.as_secs_f64() / active_ticks as f64;

    sim.reset();
    let idle_ticks = 10_000_000u64;
    let start = Instant::now();
    sim.run(idle_ticks);
    let idle = start.elapsed();
    let per_idle_tick = idle.as_secs_f64() / idle_ticks as f64;

    println!("manual engine (16x3x3, one activation = click + 26 active ticks):");
    println!(
        "  active:    {:>8.2} µs/tick  ({:>10.0} ticks/s, {:>7.0}x real time, {:>6.0} activations/s)",
        per_active_tick * 1e6,
        1.0 / per_active_tick,
        1.0 / per_active_tick / 20.0,
        iterations as f64 / active.as_secs_f64(),
    );
    println!(
        "  quiescent: {:>8.2} ns/tick  ({:>10.0} ticks/s, {:>7.0}x real time)",
        per_idle_tick * 1e9,
        1.0 / per_idle_tick,
        1.0 / per_idle_tick / 20.0,
    );
}
