//! The record 3x3 door's fireball-onto-a-plate gadget, against the vanilla
//! capture — and the collision clip on a piston's sweep that makes it work.
//!
//! Runs `tests/corpus/structures/piston_plate_clip.snbt`, which is the oracle's
//! own structure, with the capture's own fifteen spawns and its own `--at`
//! schedule, and compares against
//! `tools/gametest/captures/piston_plate_clip.entities.log` (positions) and
//! `tests/traces/piston_plate_clip.json` (the plates' `power`). Both are
//! committed: `tools/gametest/work/` is gitignored, so the trace is copied here
//! rather than left where the capture wrote it.
//!
//! Every lane is the same five blocks in a row, with a small fireball embedded
//! in the piston's head slot:
//!
//! ```text
//!   (3,1,z) quartz   (4,1,z) sticky piston   (5,1,z) sticky piston, facing west
//!                    (4,2,z) quartz          (5,2,z) light weighted plate
//! ```
//!
//! and the door's own numbers are lane `z=1`: the extension throws the fireball
//! `0.6875` west and the retraction brings it **all the way back**, to its exact
//! start, flush against the piston base — pressing the plate on the way. What
//! makes the return stop there rather than a third of a block past is an
//! ordinary block collision against the cell the piston is moving a block into,
//! which is [`mc_tick`]'s `Simulation::blocks_in_flight`. Without it the
//! fireball ends `0.3225` east of home, settles inside the plate's touch box,
//! and the plate latches on forever.
//!
//! The lanes, and what each is for:
//!
//! ```text
//!   z=1   the replica: extend, then retract.        plate presses and releases
//!   z=3   extend only — never retracts.             plate never presses
//!   z=5   starts extended; retract.                 the +0.02 second step
//!   z=7   the fireball already a block west
//!   z=9   the fireball at the floor, y=1.0
//!   z=11  overlapping the piston base by 0.05625
//!   z=13  0.10625 into the piston base: untouched
//!   z=15  no fireball at all                        plate never presses
//!   z=17  nothing to push: the sweep is NOT clipped
//!   z=19  nothing to pull: no second retract step
//!   z=21  starts 0.25625 east: clipped to 0.43375
//!   z=23  starts 0.04375 west
//!   z=25  starts 0.03125 east
//!   z=27  flush inside the base: untouched
//!   z=29  extend, retract eighteen ticks later
//!   z=31  the floor lane, extended then retracted
//! ```

use mc_tick::{Pos, Simulation};

/// The capture's `--spawn` list, in order. Lane `z=15` is absent on purpose.
const SPAWNS: &[(f64, f64, f64)] = &[
    (4.84375, 1.875, 1.5),
    (4.84375, 1.875, 3.5),
    (4.84375, 1.875, 5.5),
    (4.15625, 1.875, 7.5),
    (4.84375, 1.0, 9.5),
    (4.9, 1.875, 11.5),
    (4.95, 1.875, 13.5),
    (4.84375, 1.875, 17.5),
    (4.84375, 1.875, 19.5),
    (5.1, 1.875, 21.5),
    (4.8, 1.875, 23.5),
    (4.875, 1.875, 25.5),
    (5.0, 1.875, 27.5),
    (4.84375, 1.875, 29.5),
    (4.84375, 1.0, 31.5),
];

/// The capture's `--at TICK:6,1,Z:STATE` arguments, in the capture's own order.
const SCHEDULE: &[(u64, i32, bool)] = &[
    (2, 1, true),
    (8, 1, false),
    (2, 3, true),
    (6, 5, false),
    (6, 7, false),
    (6, 9, false),
    (6, 11, false),
    (6, 13, false),
    (6, 15, false),
    (2, 17, true),
    (8, 17, false),
    (6, 19, false),
    (2, 21, true),
    (8, 21, false),
    (6, 23, false),
    (6, 25, false),
    (6, 27, false),
    (2, 29, true),
    (20, 29, false),
    (2, 31, true),
    (8, 31, false),
];

/// Vanilla's answer for every lane: the fireball's successive **distinct** `x`
/// values, and the plate's successive **distinct** `power` values.
///
/// Compared as sequences rather than by tick number for the reason
/// `piston_entity.rs` gives: the oracle's `--at N` and this engine's
/// `place_block` need not agree on which side of tick `N` a boundary write
/// lands, and they do not — every event here is exactly one tick later in the
/// engine. The physics is what is asserted, and it is asserted exactly: these
/// are the capture's own decimals.
type Lane = (i32, &'static [f64], &'static [u8]);
const LANES: &[Lane] = &[
    (
        1,
        &[4.84375, 4.33375, 4.15625, 4.66625, 4.84375],
        &[0, 1, 0],
    ),
    (3, &[4.84375, 4.33375, 4.15625], &[0]),
    (
        5,
        &[
            4.84375, 4.82375, 4.84375, 4.33375, 4.15625, 4.66625, 4.84375, 4.33375, 4.15625,
            4.66625, 4.84375, 4.33375, 4.15625,
        ],
        &[0, 1, 0, 1, 0, 1],
    ),
    (
        7,
        &[
            4.15625, 4.66625, 4.84375, 4.33375, 4.15625, 4.66625, 4.84375, 4.33375, 4.15625,
            4.66625, 4.84375, 4.33375,
        ],
        &[0, 1, 0, 1, 0, 1],
    ),
    (9, &[4.84375, 4.82375, 4.84375], &[0]),
    (
        11,
        &[
            4.9, 4.82375, 4.84375, 4.33375, 4.15625, 4.66625, 4.84375, 4.33375, 4.15625, 4.66625,
            4.84375, 4.33375, 4.15625,
        ],
        &[0, 1, 0, 1, 0, 1],
    ),
    (13, &[4.95], &[0, 1]),
    (17, &[4.84375, 4.33375, 3.83375], &[0]),
    (19, &[4.84375, 4.82375, 4.33375, 3.83375], &[0, 1, 0]),
    (
        21,
        &[5.1, 4.59, 4.15625, 4.66625, 4.84375],
        &[0, 1, 0, 1, 0],
    ),
    (
        23,
        &[
            4.8, 4.82375, 4.84375, 4.33375, 4.15625, 4.66625, 4.84375, 4.33375, 4.15625, 4.66625,
            4.84375, 4.33375, 4.15625,
        ],
        &[0, 1, 0, 1, 0, 1],
    ),
    (
        25,
        &[
            4.875, 4.82375, 4.84375, 4.33375, 4.15625, 4.66625, 4.84375, 4.33375, 4.15625, 4.66625,
            4.84375, 4.33375, 4.15625,
        ],
        &[0, 1, 0, 1, 0, 1],
    ),
    (27, &[5.0], &[0, 1]),
    (
        29,
        &[4.84375, 4.33375, 4.15625, 4.66625, 4.84375],
        &[0, 1, 0],
    ),
    (31, &[4.84375, 4.33375, 4.15625, 4.66625, 4.84375], &[0]),
];

/// The lane with no fireball in it. Its plate must never fire, and it is here
/// because a rig that powered every plate for some unrelated reason would pass
/// the fourteen lanes above.
const EMPTY_LANE: i32 = 15;

/// How far the capture ran. The engine is one tick behind it, so it runs one
/// tick longer; several lanes oscillate forever and would diverge past this.
const CAPTURE_TICKS: u64 = 32;

fn build() -> Simulation {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/structures/piston_plate_clip.snbt");
    let text = std::fs::read_to_string(&path).expect("the oracle's own structure file");
    let structure = mc_tick::Structure::parse(&text).expect("must parse");
    let mut sim = Simulation::new(structure.bounds(4));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    // The redstone blocks the capture drops in are not in the structure, so
    // their state has to exist in the registry before the run.
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
    for (x, y, z) in SPAWNS {
        sim.spawn_frozen_entity("minecraft:small_fireball".into(), [*x, *y, *z])
            .expect("a measured hitbox");
    }
    let order = structure.placement_order(
        mc_tick::vanilla::is_collision_full_cube,
        mc_tick::vanilla::has_dynamic_shape,
    );
    sim.place_on_place(&order);
    sim.settle_with_order(&order);
    sim
}

/// A lane's fireball `x`, by the z it is standing in.
fn lane_x(sim: &Simulation, z: i32) -> Option<f64> {
    let centre = f64::from(z) + 0.5;
    sim.entity_bodies()
        .iter()
        .find(|b| !b.is_minecart && b.min[2] < centre && b.max[2] > centre)
        .map(|b| (b.min[0] + b.max[0]) / 2.0)
}

/// A lane's plate `power`, read off the block state — the one channel that
/// survives every entity filter, and the only one that can see inside a tick.
fn plate_power(sim: &Simulation, z: i32) -> u8 {
    let state = sim.world().get(Pos::new(5, 2, z));
    let descriptor = sim
        .registry()
        .descriptor(state)
        .unwrap_or_else(|| panic!("no descriptor at (5,2,{z})"));
    assert!(
        descriptor.contains("pressure_plate"),
        "lane z={z} lost its plate: (5,2,{z}) is {descriptor}"
    );
    descriptor
        .split("power=")
        .nth(1)
        .and_then(|rest| rest.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|digits| digits.parse().ok())
        .unwrap_or_else(|| panic!("no power= in {descriptor}"))
}

/// Every lane's distinct `x` sequence and distinct `power` sequence, in order.
type Observed = (Vec<Vec<f64>>, Vec<Vec<u8>>, Vec<u8>);
fn run() -> Observed {
    let mut sim = build();
    let redstone = sim
        .registry_mut()
        .intern("minecraft:redstone_block")
        .unwrap();
    let mut xs: Vec<Vec<f64>> = LANES.iter().map(|_| Vec::new()).collect();
    let mut powers: Vec<Vec<u8>> = LANES.iter().map(|_| Vec::new()).collect();
    let mut empty: Vec<u8> = Vec::new();
    for tick in 0..=CAPTURE_TICKS {
        for &(at, z, on) in SCHEDULE {
            if at == tick {
                let state = if on { redstone } else { mc_tick::StateId::AIR };
                sim.place_block(Pos::new(6, 1, z), state);
            }
        }
        for (index, (z, _, _)) in LANES.iter().enumerate() {
            let x = lane_x(&sim, *z).unwrap_or_else(|| panic!("lane z={z} lost its fireball"));
            if xs[index].last() != Some(&x) {
                xs[index].push(x);
            }
            let power = plate_power(&sim, *z);
            if powers[index].last() != Some(&power) {
                powers[index].push(power);
            }
        }
        let power = plate_power(&sim, EMPTY_LANE);
        if empty.last() != Some(&power) {
            empty.push(power);
        }
        sim.step();
    }
    (xs, powers, empty)
}

/// The whole capture: fifteen lanes, every position and every plate transition.
#[test]
fn every_lane_of_the_capture_agrees_with_vanilla() {
    let (xs, powers, empty) = run();
    for (index, (z, expected_x, expected_power)) in LANES.iter().enumerate() {
        assert_eq!(
            &xs[index], expected_x,
            "lane z={z}: the fireball's positions disagree with the capture"
        );
        assert_eq!(
            &powers[index], expected_power,
            "lane z={z}: the plate's power disagrees with the capture"
        );
    }
    assert_eq!(
        empty,
        vec![0],
        "lane z={EMPTY_LANE} has no fireball, so its plate must never fire"
    );
}

/// The door's own gadget, named: the retraction returns the fireball to its
/// exact start and the plate fires.
///
/// This is the assertion the door was blocked on. Vanilla clips the second
/// retraction step from the `0.3425` the pulled quartz's sweep asks for down to
/// `0.02`, the room left to the piston's own square at `x = 5.0`. Uncipped, the
/// fireball ends at `5.16625` — `0.3225` east of home and inside the plate's
/// touch box, where the plate latches and never lets go.
#[test]
fn the_replica_returns_the_fireball_flush_against_the_piston_base() {
    let (xs, powers, _) = run();
    let lane = LANES.iter().position(|(z, _, _)| *z == 1).unwrap();
    assert_eq!(
        xs[lane],
        vec![4.84375, 4.33375, 4.15625, 4.66625, 4.84375],
        "west 0.51 then 0.1775, then east 0.51 then 0.1775 — net zero"
    );
    let start = xs[lane].first().unwrap();
    let end = xs[lane].last().unwrap();
    assert_eq!(
        end, start,
        "its east face ends flush on x = 5.0, exactly where it started"
    );
    assert_eq!(
        powers[lane],
        vec![0, 1, 0],
        "and the plate presses on the way, then releases"
    );
}

/// The clip is a distance to a surface, not a fixed amount.
///
/// Lane `z=21` starts the same fireball `0.25625` further east, so the same
/// second step has `0.43375` of room to the same line rather than `0.1775` —
/// and vanilla gives it exactly that. A hard-coded shortening cannot produce
/// both numbers.
#[test]
fn the_clip_is_the_room_left_to_the_surface() {
    let (xs, _, _) = run();
    /// Half a small fireball, so `x` becomes the box's west face.
    const HALF: f64 = 0.15625;
    let replica = LANES.iter().position(|(z, _, _)| *z == 1).unwrap();
    let east = LANES.iter().position(|(z, _, _)| *z == 21).unwrap();
    // Two lanes, two different second steps — 0.1775 and 0.43375 — and both
    // land the west face on exactly the same line. A fixed shortening cannot do
    // that, and neither can a rule that stops at a distance from the entity.
    assert_ne!(
        xs[replica][1], xs[east][1],
        "the two lanes must reach the clip from different places"
    );
    assert_eq!(
        xs[replica][2] - HALF,
        4.0,
        "clipped flush against the arriving block"
    );
    assert_eq!(
        xs[east][2] - HALF,
        4.0,
        "and so is a lane that started 0.25625 further east"
    );
    // Retraction, the other half. Lane z=5 retracts twice from two different
    // places — 0.02 out of the head's own eject, and 0.1775 out of a full
    // stroke — and lands the *east* face on the piston's own square both times.
    let twice = LANES.iter().position(|(z, _, _)| *z == 5).unwrap();
    assert_eq!(xs[twice][1], 4.82375, "the eject leaves it 0.02 short");
    assert_eq!(
        xs[twice][2] + HALF,
        5.0,
        "so the pulled block's sweep is clipped to 0.02"
    );
    assert_eq!(
        xs[twice][5], 4.66625,
        "and a full stroke leaves it 0.1775 short"
    );
    assert_eq!(
        xs[twice][6] + HALF,
        5.0,
        "clipped to 0.1775, against the same face"
    );
    assert_eq!(
        xs[replica][4] + HALF,
        5.0,
        "which is where the door's own fireball ends"
    );
}

/// The negative controls, from the same rig: with nothing in the way there is
/// nothing to clip.
///
/// * `z=17` has no block to push, so the extension's second step is the full
///   `0.5` and the fireball ends a block west — which is also why its
///   retraction cannot reach it and its plate never fires.
/// * `z=19` has no block to *pull*, so the retraction has only the head's own
///   eject and stops at `4.82375` instead of being carried on to `4.84375`.
/// * `z=3` never retracts at all.
#[test]
fn with_nothing_in_the_way_the_sweep_is_not_clipped() {
    let (xs, powers, _) = run();
    let nopush = LANES.iter().position(|(z, _, _)| *z == 17).unwrap();
    assert_eq!(
        xs[nopush],
        vec![4.84375, 4.33375, 3.83375],
        "0.51 then a full 0.5"
    );
    assert_eq!(
        powers[nopush],
        vec![0],
        "and it ends out of the plate's reach"
    );

    let nopull = LANES.iter().position(|(z, _, _)| *z == 19).unwrap();
    assert_eq!(
        xs[nopull][1], 4.82375,
        "the head's eject alone, and no sweep behind it"
    );

    let extend_only = LANES.iter().position(|(z, _, _)| *z == 3).unwrap();
    assert_eq!(
        powers[extend_only],
        vec![0],
        "a piston that never comes back never presses it"
    );
}

/// The same rig with bodies wide enough to make *when* the clip starts matter.
///
/// From `tools/gametest/captures/piston_clip_sizes.entities.log`. Every lane
/// above is a 0.3125 fireball that is a whole block clear of the arriving block
/// when its first step is taken, so the first step never binds and "solid all
/// along" fits as well as "solid only on the second step". A dragon fireball is
/// 1.0 wide, so in the replica lane its box is `[4.0, 5.0]` — leading face
/// already flush on the line the small fireball is clipped to — and the two
/// rules disagree: solid-all-along pins it, and vanilla moves it `0.51`.
mod wide_bodies {
    use super::*;

    /// `(z, kind, spawn, vanilla's distinct x, does the engine agree?)`, in the
    /// capture's own spawn order — a piston walks its entities in the order they
    /// were made, so the order is part of the rig.
    type Wide = (i32, &'static str, [f64; 3], &'static [f64], bool);
    const WIDE: &[Wide] = &[
        // The replica: 0.51 free, then 0.49 — clipped against x = 3.0, the cell
        // the pushed *quartz* is arriving in, because the cell the pushed piston
        // arrives in is one the box already overlaps by then.
        (
            1,
            "minecraft:dragon_fireball",
            [4.5, 1.875, 1.5],
            &[4.5, 3.99, 3.5],
            true,
        ),
        // Starts extended and retracts: the wide-body round trip. The outward
        // quarter is the head's drag clipped against the retracting base's
        // 12/16 slab (`retracting_base_box`), and the return quarter is
        // `fixEntityWithinPistonBase` (`base_fix_displacement`) — the pair of
        // calls this file's tripwire held open until both were wired in.
        (
            5,
            "minecraft:dragon_fireball",
            [4.5, 1.875, 5.5],
            &[4.5, 4.75, 4.5, 3.99, 3.4800000000000004],
            true,
        ),
        // Nothing to push, so nothing to clip: 0.51 then a full 0.5.
        (
            17,
            "minecraft:dragon_fireball",
            [4.5, 1.875, 17.5],
            &[4.5, 3.99, 3.49],
            true,
        ),
        // The same round trip with a 0.98 x 0.7 cart, whose `0.98F` float slop
        // survives both quarters.
        (
            9,
            "minecraft:furnace_minecart",
            [4.51, 1.0, 9.5],
            &[4.51, 4.759999990463257, 4.509999990463257],
            true,
        ),
        // A 0.98 cart starts 0.02 clear of the line and is not clipped either.
        (
            29,
            "minecraft:furnace_minecart",
            [4.51, 1.0, 29.5],
            &[4.51, 4.0, 3.499999990463257],
            true,
        ),
    ];

    /// The capture's own `--at` schedule for these five lanes.
    const SCHEDULE: &[(u64, i32, bool)] = &[
        (2, 1, true),
        (8, 1, false),
        (6, 5, false),
        (6, 9, false),
        (2, 17, true),
        (8, 17, false),
        (2, 29, true),
        (14, 29, false),
    ];

    fn run() -> Vec<Vec<f64>> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/corpus/structures/piston_plate_clip.snbt");
        let text = std::fs::read_to_string(&path).expect("the oracle's own structure file");
        let structure = mc_tick::Structure::parse(&text).expect("must parse");
        let mut sim = Simulation::new(structure.bounds(4));
        {
            let (registry, world) = sim.registry_and_world_mut();
            structure.place(world, registry, Pos::new(0, 0, 0));
        }
        sim.registry_mut()
            .intern("minecraft:redstone_block")
            .expect("a real block");
        mc_tick::intern_companions(sim.registry_mut());
        {
            let mut table = std::mem::take(sim.behaviours_mut());
            mc_tick::register_all(sim.registry_mut(), &mut table);
            *sim.behaviours_mut() = table;
        }
        let (solidity, frictions, heights, webs) = mc_tick::vanilla::physics_tables(sim.registry());
        sim.set_physics_tables(solidity, frictions, heights, webs);
        let (water_kinds, bubble_kinds) = mc_tick::vanilla::fluid_tables(sim.registry());
        sim.set_fluid_tables(water_kinds, bubble_kinds);
        let (rails, conductors) = mc_tick::vanilla::rail_tables(sim.registry());
        sim.set_rail_tables(rails, conductors);
        // Both the dragon fireballs and the carts, in the capture's spawn order,
        // because a piston walks its entities in the order they were made.
        for (_, kind, at, _, _) in WIDE {
            if kind.ends_with("minecart") {
                sim.spawn_minecart((*kind).into(), *at, [0.0, 0.0, 0.0]);
            } else {
                sim.spawn_frozen_entity((*kind).into(), *at)
                    .expect("a measured hitbox");
            }
        }
        let order = structure.placement_order(
            mc_tick::vanilla::is_collision_full_cube,
            mc_tick::vanilla::has_dynamic_shape,
        );
        sim.place_on_place(&order);
        sim.settle_with_order(&order);

        let redstone = sim
            .registry_mut()
            .intern("minecraft:redstone_block")
            .unwrap();
        let mut xs: Vec<Vec<f64>> = WIDE.iter().map(|_| Vec::new()).collect();
        for tick in 0..=24u64 {
            for &(at, z, on) in SCHEDULE {
                if at == tick {
                    let state = if on { redstone } else { mc_tick::StateId::AIR };
                    sim.place_block(Pos::new(6, 1, z), state);
                }
            }
            for (index, (z, _, _, _, _)) in WIDE.iter().enumerate() {
                let centre = f64::from(*z) + 0.5;
                let x = sim
                    .entity_bodies()
                    .iter()
                    .find(|b| b.min[2] < centre && b.max[2] > centre)
                    .map(|b| (b.min[0] + b.max[0]) / 2.0)
                    .unwrap_or_else(|| panic!("lane z={z} lost its body"));
                if xs[index].last() != Some(&x) {
                    xs[index].push(x);
                }
            }
            sim.step();
        }
        xs
    }

    /// A moving block is transparent on the first of the two steps.
    ///
    /// This is the assertion that a `blocks_in_flight` box present on both steps
    /// fails: the dragon fireball in lane `z=1` starts flush against the cell
    /// the pushed sticky piston arrives in, so it would not move at all.
    #[test]
    fn the_first_step_of_a_stroke_is_not_clipped() {
        let xs = run();
        for (index, (z, kind, _, expected, agrees)) in WIDE.iter().enumerate() {
            if *agrees {
                assert_eq!(&xs[index], expected, "lane z={z} ({kind})");
            }
        }
        let replica = 0;
        let nopush = 2;
        // Named, because it is the whole point: 0.51 taken with the leading face
        // already on the line, and only the *second* step shortened.
        assert_eq!(
            xs[replica][0] - xs[replica][1],
            0.5099999999999998,
            "the first step is the full 0.51"
        );
        assert_eq!(
            xs[replica][1] - xs[replica][2],
            0.49000000000000021,
            "and the second is clipped to 0.49"
        );
        assert_eq!(
            xs[nopush][1] - xs[nopush][2],
            0.5,
            "with nothing to push, a full 0.5"
        );
    }

    /// A **disagreement**, recorded rather than dropped: retraction's law has no
    /// answer for a body wider than the piston arm.
    ///
    /// Lanes `z=5` (dragon fireball) and `z=9` (furnace minecart) of the same
    /// capture start extended and retract. Vanilla shoves each `+0.25` east and
    /// then `-0.25` back — a round trip ending exactly where it began. The engine
    /// shoves `+0.49`, and the cart never comes back at all: a 1.0-wide box
    /// straddles the piston arm's 4/16 column instead of lying inside it, so
    /// `inside_eject_displacement`'s cross-axis gate declines, and the pulled
    /// block's own sweep answers instead.
    ///
    /// This is **not** the clip. Both lanes are retractions of a body that never
    /// reaches a surface, and no obstacle the clip supplies is in either answer.
    /// It is recorded as a disagreement, with both sets of numbers, so that
    /// fixing retraction's law fails this test rather than passing silently — and
    /// so that a *third* answer is caught too.
    ///
    /// The narrow lanes hide it: a 0.3125 fireball fits inside the arm column, so
    /// every lane of `piston_plate_clip` proper takes the other branch.
    ///
    /// # The mechanism, now confirmed against these numbers
    ///
    /// `+0.25` then `-0.25` is not one constant twice. It is **two different
    /// surfaces on the two steps**, and the capture identifies both to the last
    /// bit — which is why two body widths agree on it:
    ///
    /// | | dragon fireball, 1.0 wide | furnace cart, 0.98F wide |
    /// |---|---|---|
    /// | east face at rest | `5.0` | `5.000000009536743` |
    /// | after step one | **`5.25`** | **`5.25`** |
    /// | after step two | `5.0` | `5.0` |
    ///
    /// Step one drags the body *inward*, toward the retracting piston at
    /// `(5,1,z)`, and clips it against [`mc_tick::piston::retracting_base_box`] —
    /// the extended base's own collision box, `[5.25, 6.0]`, the block minus the
    /// `4/16` slot the arm sits in. The body's leading face comes to rest flush on
    /// the slot face. Step two clips against the base's **landed** state, the
    /// retracted full cube `[5.0, 6.0]`, and pushes it back out to `5.0`.
    ///
    /// So [`mc_tick::piston::PISTON_BASE_SLOT`] is vindicated as *geometry*: a
    /// 0.98-wide and a 1.0-wide body, starting 9.5e-9 apart, both land on `5.25`
    /// exactly, which no constant displacement explains. Both surfaces are now
    /// wired in — the base slab clips the drag leg and
    /// `base_fix_displacement` runs after every retracting source's shove — so
    /// the tripwire test that pinned the engine's old `+0.49` answer is gone
    /// and both lanes sit in the agreeing set above, checked by
    /// `every_wide_lane_agrees`.
    #[test]
    fn every_wide_lane_agrees() {
        let xs = run();
        for (index, (z, kind, _, vanilla, agrees)) in WIDE.iter().enumerate() {
            assert!(*agrees, "lane z={z} ({kind}) is still marked disagreeing");
            assert_eq!(
                &xs[index], vanilla,
                "lane z={z} ({kind}): positions diverge from the capture"
            );
        }
        // The shove that used to be wrong, named: a quarter block out and a
        // quarter block back.
        let five = WIDE.iter().position(|(z, _, _, _, _)| *z == 5).unwrap();
        assert_eq!(WIDE[five].3[1] - WIDE[five].3[0], 0.25, "the outward shove");
        assert_eq!(WIDE[five].3[2] - WIDE[five].3[1], -0.25, "and the return");
    }
}
