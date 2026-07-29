//! A minecart standing on another entity, against the vanilla captures.
//!
//! This is the record doors' *"the minecart rests on the villager's head"*: a
//! mob's hitbox used as scaffolding to hold a cart at an exact height. Four
//! captures pin it, and the point of running them here rather than trusting a
//! reading of `Entity.canBeCollidedWith` is that the obvious reading is wrong
//! twice over — an **armor stand is a `LivingEntity` and a cart falls straight
//! through it**, and a **boat is not living and holds a cart up**.
//!
//! ```text
//!   blaze_ride_ai   a cart dropped from y=3 onto five different things
//!   cart_body       the same, plus armor stand / boat / zombie / cart-on-cart,
//!                   plus the sideways-offset negative control, plus the
//!                   grounded-drag probe
//!   cart_body2      a cart driven *sideways* into each, and a cart dropped
//!                   onto a seated passenger
//!   cart_body4      ghast fireball and a real item entity — both transparent
//! ```
//!
//! Every number below is copied out of a `.entities.log` in
//! `tools/gametest/captures/`, to the last digit. The eighth decimals are the
//! measurement: `2.799999952316284` is `1.0 + (1.8f as f64)`, and rounding it
//! would hide a hitbox read as a decimal rather than a float.

use mc_tick::{Pos, Simulation};

/// The free-fall reference: `blaze_ride_ai` lane z=27.5, a cart dropped from
/// y = 3.0 over bare floor with nothing under it. Ten ticks to the ground.
///
/// It is the negative control for every "falls through" row — those lanes are
/// asserted *equal to this sequence*, so a test that passed by never moving
/// anything would fail here first.
const FREE_FALL: [f64; 11] = [
    2.9599999999999937,
    2.882000000476836,
    2.7679000018596582,
    2.619505004533522,
    2.438529758842705,
    2.22660327759381,
    1.9852731229337195,
    1.716009478883521,
    1.4202090202457072,
    1.0991985880660025,
    1.0,
];

/// `blaze_ride_ai` lane z=10.5: the same cart, over a blaze. Two ticks, and it
/// stops on the blaze's exact top, 1.0 + 1.8f.
const ONTO_BLAZE: [f64; 11] = [
    2.9599999999999937,
    2.882000000476836,
    2.799999952316284,
    2.799999952316284,
    2.799999952316284,
    2.799999952316284,
    2.799999952316284,
    2.799999952316284,
    2.799999952316284,
    2.799999952316284,
    2.799999952316284,
];

/// `blaze_ride_ai` lane z=13.5: over a villager, which is taller, so it stops a
/// tick sooner and a fifth of a block higher — 1.0 + 1.95.
const ONTO_VILLAGER: [f64; 11] = [
    2.9599999999999937,
    2.950000047683716,
    2.950000047683716,
    2.950000047683716,
    2.950000047683716,
    2.950000047683716,
    2.950000047683716,
    2.950000047683716,
    2.950000047683716,
    2.950000047683716,
    2.950000047683716,
];

/// Compare a mid-fall trajectory to the capture.
///
/// **Rest positions are asserted exactly**; only the ticks in flight go through
/// this, and only for one reason: the oracle drops its carts at y ≈ 103 (its
/// origin is `BlockPos{0, 100, 0}`) and this engine drops them at y ≈ 3, so the
/// two accumulate different rounding on the way down. The largest disagreement
/// across the eleven ticks below is **1.5e-14** — sixty-odd ulps at that
/// magnitude, and nine orders of magnitude inside the `1.0e-6` the cart
/// conformance goldens already run at. It is not a difference in the physics:
/// every landing value, which is set by the body's top rather than by the sum,
/// matches to the last bit.
#[track_caller]
fn matches_capture(seen: &[f64], capture: &[f64], what: &str) {
    assert_eq!(seen.len(), capture.len(), "{what}");
    for (tick, (got, want)) in seen.iter().zip(capture).enumerate() {
        assert!(
            (got - want).abs() < 1.0e-9,
            "{what}: tick {tick} was {got}, vanilla had {want}"
        );
    }
}

fn world(name: &str) -> (Simulation, mc_tick::Structure) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/structures")
        .join(name);
    let text = std::fs::read_to_string(&path).expect("the oracle's own structure file");
    let structure = mc_tick::Structure::parse(&text).expect("must parse");
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
    assert_eq!(sim.unknown_report(), None, "a partially-simulated world proves nothing");
    let (solidity, frictions, heights, webs) = mc_tick::vanilla::physics_tables(sim.registry());
    sim.set_physics_tables(solidity, frictions, heights, webs);
    let (water_kinds, bubble_kinds) = mc_tick::vanilla::fluid_tables(sim.registry());
    sim.set_fluid_tables(water_kinds, bubble_kinds);
    let (rails, conductors) = mc_tick::vanilla::rail_tables(sim.registry());
    sim.set_rail_tables(rails, conductors);
    (sim, structure)
}

fn settle(sim: &mut Simulation, structure: &mc_tick::Structure) {
    let order = structure.placement_order(
        mc_tick::vanilla::is_collision_full_cube,
        mc_tick::vanilla::has_dynamic_shape,
    );
    sim.place_on_place(&order);
    sim.settle_with_order(&order);
}

/// The y of every cart, in spawn order, over `n` ticks.
fn fall(sim: &mut Simulation, n: usize) -> Vec<Vec<f64>> {
    let mut out = vec![Vec::new(); sim.minecarts().len()];
    for _ in 0..n {
        sim.step();
        for (lane, cart) in sim.minecarts().iter().enumerate() {
            out[lane].push(cart.pos[1]);
        }
    }
    out
}

/// The whole `blaze_ride_ai` support table, tick for tick.
///
/// A living body is a hard obstacle and a projectile is not — and the two
/// projectile lanes are checked against [`FREE_FALL`] rather than against a
/// constant, so "falls through" means *identical to having nothing there*, not
/// merely "ends up low".
#[test]
fn a_cart_rests_on_a_living_body_and_falls_through_a_projectile() {
    let (mut sim, structure) = world("blaze_ride.snbt");
    sim.spawn_frozen_entity("minecraft:blaze".into(), [2.5, 1.0, 10.5]).unwrap();
    sim.spawn_frozen_entity("minecraft:villager".into(), [2.5, 1.0, 13.5]).unwrap();
    sim.spawn_frozen_entity("minecraft:small_fireball".into(), [2.5, 1.0, 16.5]).unwrap();
    sim.spawn_frozen_entity("minecraft:dragon_fireball".into(), [2.5, 1.0, 19.5]).unwrap();
    // Spawn order fixes the lane order the assertions below read.
    for z in [10.5, 13.5, 16.5, 19.5, 27.5] {
        sim.spawn_minecart("minecraft:minecart".into(), [2.5, 3.0, z], [0.0; 3]);
    }
    settle(&mut sim, &structure);

    let seen = fall(&mut sim, FREE_FALL.len());
    matches_capture(&seen[0], &ONTO_BLAZE, "over a blaze");
    matches_capture(&seen[1], &ONTO_VILLAGER, "over a villager");
    matches_capture(&seen[4], &FREE_FALL, "the control, over nothing");
    // Where each one comes to *rest* is exact — that number is the body's top,
    // not a sum of eleven multiplications, so there is nothing for the origin
    // to perturb.
    assert_eq!(seen[0][10], 2.799999952316284, "the blaze's exact float top");
    assert_eq!(seen[1][10], 2.950000047683716, "the villager's");
    assert_eq!(seen[2], seen[4], "a small fireball is not there at all");
    assert_eq!(seen[3], seen[4], "nor is a dragon fireball, one block tall though it is");

    // And the bodies feel nothing back. Vanilla's blaze and villager hold
    // (2.5, 1.0, z) to the last digit for all thirty ticks of the capture,
    // whether AI is on or off, while a cart sits on their heads.
    for body in sim.entity_bodies().iter().filter(|b| !b.is_minecart) {
        let centre = [(body.min[0] + body.max[0]) / 2.0, body.min[1]];
        assert_eq!(centre, [2.5, 1.0], "{} was moved by the cart on it", body.kind);
    }
}

/// A cart parks on a *passenger's* head, not just on a standing body.
///
/// `cart_body2` lane z=40.5: a NaN cart at y = 1.0 with a blaze riding it, and
/// a cart dropped from y = 4.0. Vanilla leaves it at `2.987499952316284` —
/// 1.0 + the 0.1875 seat + the blaze's 1.8f — which is a different number from
/// every other row in this file and cannot be reached by resting on the cart.
#[test]
fn a_cart_rests_on_a_seated_passenger() {
    let (mut sim, structure) = world("cart_body2.snbt");
    let vehicle =
        sim.spawn_minecart("minecraft:minecart".into(), [2.5, 1.0, 40.5], [0.0, 0.0, f64::NAN]);
    sim.spawn_authored_rider(
        vehicle,
        &mc_tick::structure::SpawnedEntity::Blaze(mc_tick::structure::SpawnedBlaze {
            pos: [0.0; 3],
            motion: [0.0, -0.0784, 0.0],
        }),
    )
    .expect("a blaze on a plain minecart is measured");
    sim.spawn_minecart("minecraft:furnace_minecart".into(), [2.5, 4.0, 40.5], [0.0; 3]);
    settle(&mut sim, &structure);

    for _ in 0..20 {
        sim.step();
    }
    let carts = sim.minecarts();
    assert_eq!(carts[1].pos[1], 2.987499952316284, "resting on the rider's head");
    // The vehicle is still a nan cart, and still where it was: nothing about
    // carrying a rider or being stood on makes it finite.
    assert_eq!(carts[0].pos, [2.5, 1.0, 40.5]);
    assert!(carts[0].vel[2].is_nan());
}

/// The support is positional: half a block to the side and the cart falls.
///
/// `cart_body` lane z=43.5 is exactly this — a blaze at x = 2.5 and a cart at
/// x = 4.0, whose boxes miss in x by 0.71 — and vanilla drops that cart to the
/// floor on the same tick as the lane with nothing in it at all.
#[test]
fn a_cart_beside_a_body_rather_than_over_it_falls() {
    let (mut sim, structure) = world("cart_body.snbt");
    sim.spawn_frozen_entity("minecraft:blaze".into(), [2.5, 1.0, 43.5]).unwrap();
    sim.spawn_minecart("minecraft:minecart".into(), [4.0, 3.0, 43.5], [0.0; 3]);
    // The positive control, same body, cart directly over it.
    sim.spawn_frozen_entity("minecraft:blaze".into(), [2.5, 1.0, 31.5]).unwrap();
    sim.spawn_minecart("minecraft:minecart".into(), [2.5, 3.0, 31.5], [0.0; 3]);
    settle(&mut sim, &structure);

    let seen = fall(&mut sim, FREE_FALL.len());
    matches_capture(&seen[0], &FREE_FALL, "0.71 clear of the blaze in x");
    matches_capture(&seen[1], &ONTO_BLAZE, "and the same cart over it does stop");
    assert_eq!(seen[0][10], 1.0, "on the floor");
    assert_eq!(seen[1][10], 2.799999952316284, "on the blaze");
}

/// A cart resting on a body is **on the ground**, exactly as on stone.
///
/// `cart_body` lanes z=37.5 and z=40.5 are the same cart with the same
/// x-velocity, one over a blaze and one over bare floor. Off a rail,
/// `comeOffTrack` halves the horizontal velocity when `onGround` and multiplies
/// by 0.95f when airborne, so the two are told apart by a factor of ten within
/// two ticks — and vanilla gives the blaze lane the *grounded* number.
///
/// Both x sequences are asserted, so the test cannot pass by making everything
/// grounded: the control's slower start is the airborne branch still running
/// while it is still falling.
#[test]
fn a_cart_resting_on_a_body_is_on_the_ground() {
    const ON_BLAZE_X: [f64; 8] = [
        2.6,
        2.694999998807907,
        2.7852499965429307,
        2.8303749954104425,
        2.8529374948441983,
        2.864218744561076,
        2.869859369419515,
        2.8726796818487346,
    ];
    const ON_STONE_X: [f64; 8] = [
        2.6,
        2.694999998807907,
        2.7852499965429307,
        2.870987493315339,
        2.9524381142270566,
        3.029816203122221,
        3.1033253866502086,
        3.173159110125499,
    ];

    let (mut sim, structure) = world("cart_body.snbt");
    sim.spawn_frozen_entity("minecraft:blaze".into(), [2.5, 1.0, 37.5]).unwrap();
    sim.spawn_minecart("minecraft:minecart".into(), [2.5, 3.0, 37.5], [0.1, 0.0, 0.0]);
    sim.spawn_minecart("minecraft:minecart".into(), [2.5, 3.0, 40.5], [0.1, 0.0, 0.0]);
    settle(&mut sim, &structure);

    let mut on_blaze = Vec::new();
    let mut on_stone = Vec::new();
    for _ in 0..ON_BLAZE_X.len() {
        sim.step();
        on_blaze.push(sim.minecarts()[0].pos[0]);
        on_stone.push(sim.minecarts()[1].pos[0]);
    }
    assert_eq!(on_blaze, ON_BLAZE_X.to_vec(), "grounded on the blaze from tick 3");
    assert_eq!(on_stone, ON_STONE_X.to_vec(), "still falling, so still airborne");
    assert_eq!(sim.minecarts()[0].pos[1], 2.799999952316284);
    assert!(sim.minecarts()[0].on_ground, "and it says so");
}

/// A body is a **full box**, not a ledge — it stops a cart sideways too.
///
/// `cart_body2` drives a cart east along a rail into a blaze centred at
/// x = 6.5 and it comes to rest at `5.709999978542328`, whose east face is
/// `6.199999988079071` — the blaze's west face to the last bit. The same lane
/// with a dragon fireball instead reproduces the empty control exactly.
///
/// The cart here is a plain minecart where the capture used a furnace one, and
/// the assertion is the resting *face* rather than the trajectory, because a
/// furnace cart's approach is governed by two constants this engine does not
/// model yet (see the note in `docs/entity-abuse-in-record-doors.md`). Where it
/// stops is geometry and is not one of them.
#[test]
fn a_body_stops_a_cart_sideways_at_its_own_face() {
    let (mut sim, structure) = world("cart_body2.snbt");
    sim.spawn_frozen_entity("minecraft:blaze".into(), [6.5, 1.0, 1.5]).unwrap();
    sim.spawn_frozen_entity("minecraft:dragon_fireball".into(), [6.5, 1.0, 4.5]).unwrap();
    sim.spawn_minecart("minecraft:minecart".into(), [1.5, 1.0625, 1.5], [0.3, 0.0, 0.0]);
    sim.spawn_minecart("minecraft:minecart".into(), [1.5, 1.0625, 4.5], [0.3, 0.0, 0.0]);
    sim.spawn_minecart("minecraft:minecart".into(), [1.5, 1.0625, 7.5], [0.3, 0.0, 0.0]);
    settle(&mut sim, &structure);

    for _ in 0..60 {
        sim.step();
    }
    let carts = sim.minecarts();
    assert_eq!(
        carts[0].pos[0] + mc_tick::minecart::CART_HALF_WIDTH,
        6.199999988079071,
        "parked with its east face on the blaze's west face"
    );
    assert_eq!(
        carts[0].pos[0], 5.709999978542328,
        "which is the x vanilla leaves the cart at in cart_body2"
    );
    assert_eq!(
        carts[1].pos[0], carts[2].pos[0],
        "a dragon fireball in the way is indistinguishable from an empty rail"
    );
    assert!(carts[2].pos[0] > 6.5, "and the control really did run past x=6.5");
}

/// Cart on cart, vertically. `cart_body` lane z=22.5 drops one plain minecart
/// onto another and it stops at `1.699999988079071` — 1.0 plus the float
/// height 0.7f, not the decimal 0.7.
#[test]
fn a_cart_rests_on_a_cart() {
    let (mut sim, structure) = world("cart_body.snbt");
    sim.spawn_minecart("minecraft:minecart".into(), [2.5, 1.0, 22.5], [0.0; 3]);
    sim.spawn_minecart("minecraft:minecart".into(), [2.5, 3.0, 22.5], [0.0; 3]);
    settle(&mut sim, &structure);

    for _ in 0..20 {
        sim.step();
    }
    assert_eq!(sim.minecarts()[1].pos[1], 1.699999988079071);
    assert_eq!(sim.minecarts()[0].pos[1], 1.0, "the one underneath is not pressed down");
}
