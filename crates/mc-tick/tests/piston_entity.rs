//! Pistons displacing entities, end to end, against the vanilla capture.
//!
//! `crates/mc-tick/src/piston.rs` unit-tests the geometry in isolation. This
//! runs the *same structure the oracle ran* — `tests/corpus/structures/
//! piston_entity.snbt` — through the whole engine, powers the pistons the way
//! the capture did, and compares the entity positions vanilla recorded in
//! `tools/gametest/captures/piston_entity.entities.log`.
//!
//! Four lanes, one sticky piston each facing east, one entity per lane standing
//! in the block the head is about to occupy:
//!
//! ```text
//!   z=1  minecart                  vel (0, 0, 0)
//!   z=3  minecart                  vel (0, 0, NaN)   the nan cart
//!   z=5  small_fireball            vel (0, 0, 0)
//!   z=7  dragon_fireball           vel (0, 0, 0)
//! ```
//!
//! Every entity starts centred at x = 3.5 and vanilla leaves each at a
//! *different* x, because the displacement is the depth of the entity's own
//! hitbox in the arm's sweep plus a fixed 0.01. That is what makes this rig
//! worth running: a wrong hitbox, a wrong overshoot and a wrong sweep are three
//! distinguishable failures.

use mc_tick::{Bounds, Pos, Simulation};

/// The x each entity holds once its piston has finished extending, from the
/// capture. Exact, not approximate: the cart's eighth decimal is the float
/// width of its hitbox, and rounding it away would hide a real disagreement.
const AFTER_EXTENSION: [(&str, f64); 4] = [
    ("minecart", 4.500000009536743),
    ("nan minecart", 4.500000009536743),
    ("small_fireball", 4.16625),
    ("dragon_fireball", 4.51),
];

/// The intermediate x after the *first* of the two half-block steps. Vanilla
/// records these at t2, one tick before it records the finals.
const AFTER_FIRST_STEP: [f64; 4] = [4.000000009536743, 4.000000009536743, 3.66625, 4.01];

/// Lane z for each entity, in the order the capture spawns them.
const LANES: [i32; 4] = [1, 3, 5, 7];

fn build() -> Simulation {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/structures/piston_entity.snbt");
    let text = std::fs::read_to_string(&path).expect("the oracle's own structure file");
    let structure = mc_tick::Structure::parse(&text).expect("must parse");
    let mut sim = Simulation::new(structure.bounds(4));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    // The redstone blocks the capture drops in at t2 are not in the structure,
    // so their state has to exist in the registry before the run.
    sim.registry_mut()
        .intern("minecraft:redstone_block")
        .expect("a real block");
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all(sim.registry_mut(), &mut table);
        *sim.behaviours_mut() = table;
    }
    assert_eq!(
        sim.unknown_report(),
        None,
        "a partially-simulated world proves nothing"
    );
    let (solidity, frictions, heights, webs) = mc_tick::vanilla::physics_tables(sim.registry());
    sim.set_physics_tables(solidity, frictions, heights, webs);
    let (water_kinds, bubble_kinds) = mc_tick::vanilla::fluid_tables(sim.registry());
    sim.set_fluid_tables(water_kinds, bubble_kinds);
    let (rails, conductors) = mc_tick::vanilla::rail_tables(sim.registry());
    sim.set_rail_tables(rails, conductors);
    // The capture's four `--spawn` arguments, in order. `--spawn` writes
    // `Entity.deltaMovement` directly and does not go through `Entity.load`,
    // which is why the NaN is taken verbatim here rather than through
    // `MotionSemantics`.
    sim.spawn_minecart(
        "minecraft:minecart".into(),
        [3.5, 1.0, 1.5],
        [0.0, 0.0, 0.0],
    );
    sim.spawn_minecart(
        "minecraft:minecart".into(),
        [3.5, 1.0, 3.5],
        [0.0, 0.0, f64::NAN],
    );
    sim.spawn_frozen_entity("minecraft:small_fireball".into(), [3.5, 1.0, 5.5])
        .unwrap();
    sim.spawn_frozen_entity("minecraft:dragon_fireball".into(), [3.5, 1.0, 7.5])
        .unwrap();
    let order = structure.placement_order(
        mc_tick::vanilla::is_collision_full_cube,
        mc_tick::vanilla::has_dynamic_shape,
    );
    sim.place_on_place(&order);
    sim.settle_with_order(&order);
    sim
}

/// Every entity's x, in spawn order: two carts, then the two frozen bodies.
fn xs(sim: &Simulation) -> Vec<f64> {
    let mut out: Vec<f64> = sim.minecarts().iter().map(|c| c.pos[0]).collect();
    for lane in [5, 7] {
        let body = sim
            .entity_bodies()
            .iter()
            .find(|b| {
                !b.is_minecart
                    && b.min[2] < f64::from(lane) + 0.5
                    && b.max[2] > f64::from(lane) + 0.5
            })
            .unwrap_or_else(|| panic!("the frozen body in lane z={lane}"));
        out.push((body.min[0] + body.max[0]) / 2.0);
    }
    out
}

fn power(sim: &mut Simulation, on: bool) {
    let state = if on {
        sim.registry_mut()
            .intern("minecraft:redstone_block")
            .unwrap()
    } else {
        mc_tick::StateId::AIR
    };
    for z in LANES {
        sim.place_block(Pos::new(1, 1, z), state);
    }
}

/// The whole capture: four entities, four different answers, exact.
///
/// Asserted as the *sequence of distinct positions* each entity passes
/// through rather than by absolute tick number, because the oracle harness's
/// `--at N` and this engine's `place_block` do not have to agree on which side
/// of tick N a boundary write lands. The physics — two half-block steps, the
/// exact distance of each, and stopping afterwards — is checked exactly.
#[test]
fn a_piston_displaces_every_entity_by_the_depth_of_its_own_hitbox() {
    let mut sim = build();
    let start = xs(&sim);
    assert_eq!(
        start,
        vec![3.5; 4],
        "all four start centred in the head's slot"
    );

    power(&mut sim, true);
    let mut seen: Vec<Vec<f64>> = vec![start];
    for _ in 0..12 {
        sim.step();
        let now = xs(&sim);
        if seen.last() != Some(&now) {
            seen.push(now);
        }
    }

    assert_eq!(
        seen.len(),
        3,
        "vanilla moves them exactly twice — half a block each — and then stops; saw {seen:?}"
    );
    assert_eq!(
        seen[1],
        AFTER_FIRST_STEP.to_vec(),
        "after the first half-block step"
    );
    let finals: Vec<f64> = AFTER_EXTENSION.iter().map(|(_, x)| *x).collect();
    assert_eq!(
        seen[2], finals,
        "after the second, which is where vanilla leaves them"
    );

    // No velocity is imparted. `moveEntityByPiston` calls `entity.move` and
    // never touches `deltaMovement`, so the nan cart keeps its NaN and the
    // others keep their zero — the capture reads (0, 0, 0) for all four.
    let carts = sim.minecarts();
    assert_eq!(
        carts[0].vel,
        [0.0, 0.0, 0.0],
        "an ordinary cart gains nothing"
    );
    assert_eq!(carts[1].vel[0], 0.0);
    assert_eq!(carts[1].vel[1], 0.0);
    assert!(
        carts[1].vel[2].is_nan(),
        "the nan cart is still a nan cart afterwards"
    );

    // Nothing was in a retracting arm's way, so the unmodelled path was never
    // reached — a real answer, not an absence of instrumentation.
    assert!(sim.piston_retract_contacts().is_empty());
}

/// Retraction, the second half of the same capture: by the time the pistons
/// pull back the entities are a block clear of the arm, and vanilla moves
/// nothing for the remaining fifteen ticks.
#[test]
fn retracting_past_an_entity_that_is_already_clear_moves_it_nowhere() {
    let mut sim = build();
    power(&mut sim, true);
    sim.run(12);
    let extended = xs(&sim);
    power(&mut sim, false);
    sim.run(15);
    assert_eq!(
        xs(&sim),
        extended,
        "the arm never reaches them on the way back"
    );
    assert!(
        sim.piston_retract_contacts().is_empty(),
        "and it never even touched them, which is why doing nothing is correct here"
    );
}

/// The nan cart is moved by a piston and by nothing else.
///
/// This is the property the record doors are built on: the cart ignores
/// gravity, its own velocity and every other entity, and a piston arm still
/// shoves it. Left unpowered it must not drift at all.
#[test]
fn a_nan_cart_sits_still_until_a_piston_touches_it() {
    let mut sim = build();
    sim.run(30);
    assert_eq!(xs(&sim), vec![3.5; 4], "nothing moves without a piston");
}
