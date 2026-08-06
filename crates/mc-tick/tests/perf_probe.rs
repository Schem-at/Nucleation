//! Throughput, and what region growth costs.
//!
//! ```sh
//! cargo test -p mc-tick --release --test perf_probe -- --ignored --nocapture
//! ```
//!
//! Run it `--release`. A debug build measures the borrow checker's idea of a
//! tick, not the engine's, and the two differ by more than an order of
//! magnitude.

use std::time::Instant;

use mc_tick::{Pos, Simulation, Structure};

const FLYER: &str = include_str!("corpus/structures/flying_machine_east.snbt");

/// The bundled flying machine, wired exactly as the case harness wires it —
/// block-entity tickers included, without which the pistons never resolve a
/// move and the machine sits there.
fn flyer() -> Simulation {
    let structure = Structure::parse(FLYER).expect("parses");
    // The kick's states have to be declared up front: a descriptor interned
    // after the behaviour table is built gets an id and no behaviour, so
    // placing it is a write nothing reacts to — the machine just sits there.
    // This is what the case runner does with its actions' `state` fields.
    let actuators = [
        "minecraft:redstone_block".to_string(),
        "minecraft:air".to_string(),
    ];
    mc_test::build_sim(
        &structure,
        Pos::new(0, 0, 0),
        mc_test::SettleMode::Quiet,
        &actuators,
        &[],
        None,
        "perf",
    )
}

fn kick(sim: &mut Simulation) {
    let redstone = sim
        .registry_mut()
        .intern("minecraft:redstone_block")
        .expect("interns");
    let air = sim.registry_mut().intern("minecraft:air").expect("interns");
    // The timing the bundled case uses: redstone in at tick 2, out at tick 4.
    // Kicking at tick 0 instead does nothing — the machine has not finished
    // arriving, and the pulse lands on a piston that is not listening yet.
    sim.step();
    sim.step();
    sim.place_block(Pos::new(2, 1, 1), redstone);
    sim.step();
    sim.step();
    sim.place_block(Pos::new(2, 1, 1), air);
}

fn run(label: &str, sim: &mut Simulation, ticks: u64) -> f64 {
    let start = Instant::now();
    for _ in 0..ticks {
        sim.step();
    }
    let secs = start.elapsed().as_secs_f64();
    let rate = ticks as f64 / secs;
    let b = sim.world().bounds();
    println!(
        "  {label:<28} {ticks} ticks in {:>7.1} ms  = {rate:>9.0} tick/s   region {}x{}x{}",
        secs * 1000.0,
        b.size().0,
        b.size().1,
        b.size().2,
    );
    rate
}

#[test]
#[ignore = "diagnostic, run by hand"]
fn growth_costs_little_and_the_engine_is_fast() {
    let ticks = 4000;

    // Grows: the region starts snug around the build and has to enlarge every
    // 16 blocks the machine travels.
    let mut growing = flyer();
    kick(&mut growing);

    // Never grows: the same machine in the same flight, but the region is
    // enlarged up front so no reallocation happens mid-flight. Any difference
    // between the two is what growth costs.
    let mut roomy = flyer();
    roomy
        .registry_and_world_mut()
        .1
        .grow_to_include(Pos::new(600, 8, 8));
    kick(&mut roomy);

    println!(
        "\nflying machine (6 blocks, travels ~{} blocks):",
        ticks / 10
    );
    let a = run("region grows to follow", &mut growing, ticks);
    let b = run("region pre-sized", &mut roomy, ticks);
    println!("  growth overhead: {:+.1}%", (b / a - 1.0) * 100.0);

    // Both must have actually flown, or this measured an engine sitting still.
    let reach = |sim: &Simulation| {
        sim.world()
            .iter_non_air()
            .map(|(p, _)| p.x)
            .max()
            .unwrap_or(i32::MIN)
    };
    println!(
        "  reached x = {} (grown) / {} (pre-sized)",
        reach(&growing),
        reach(&roomy)
    );
    assert!(reach(&growing) > 300, "the growing run never got moving");
    assert_eq!(reach(&growing), reach(&roomy), "growth changed the flight");
}
