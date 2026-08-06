//! `place_block_by_hand`: vanilla's placement order, on one block.
//!
//! A hand-placed block derives its state from the surroundings *before*
//! anything reacts — `getStateForPlacement` plus the `updateShape` chain —
//! and only then does `onPlace` run. The observable that pins the order: a
//! propertyless wire placed next to a redstone block must come out both
//! *connected* (the shape derivation) and *powered* (its `onPlace`), and the
//! power write requires the connected state to already be in place, because
//! `wire_with_power` walks siblings of the state actually in the world.

use mc_tick::{Pos, Simulation};

fn build() -> (Simulation, mc_tick::StateId) {
    let mut sim = Simulation::new(mc_tick::Bounds {
        min: Pos::new(-4, -4, -4),
        max: Pos::new(8, 8, 8),
    });
    let stone = sim.registry_mut().intern("minecraft:smooth_stone").unwrap();
    let block = sim
        .registry_mut()
        .intern("minecraft:redstone_block")
        .unwrap();
    let wire = sim
        .registry_mut()
        .intern("minecraft:redstone_wire")
        .unwrap();
    for x in 0..4 {
        sim.world_mut().set(Pos::new(x, 0, 0), stone);
    }
    sim.world_mut().set(Pos::new(0, 1, 0), block);
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all(sim.registry_mut(), &mut table);
        *sim.behaviours_mut() = table;
    }
    let (solidity, frictions, heights, webs) = mc_tick::vanilla::physics_tables(sim.registry());
    sim.set_physics_tables(solidity, frictions, heights, webs);
    (sim, wire)
}

#[test]
fn a_hand_placed_wire_arrives_connected_and_powered() {
    let (mut sim, wire) = build();
    sim.place_block_by_hand(Pos::new(1, 1, 0), wire);
    sim.run_until_quiescent(50);
    let state = sim.world().get(Pos::new(1, 1, 0));
    let descriptor = sim.registry().descriptor(state).expect("a real state");
    assert!(
        descriptor.contains("power=15"),
        "powered by the block beside it, got {descriptor}"
    );
    assert!(
        descriptor.contains("west=side"),
        "connected toward its source, got {descriptor}"
    );
}

#[test]
fn a_hand_placed_repeater_reads_its_input() {
    // Vanilla's `getStateForPlacement` derives POWERED from the inputs; the
    // engine reaches the same end state through the placement's self-poke and
    // the diode's own scheduled tick.
    // Same world as `build`, but the repeater must be interned before
    // `intern_companions` runs — its powered twin has to exist for the
    // behaviour to register at all, exactly as the real wiring orders it.
    let mut sim = Simulation::new(mc_tick::Bounds {
        min: Pos::new(-4, -4, -4),
        max: Pos::new(8, 8, 8),
    });
    let stone = sim.registry_mut().intern("minecraft:smooth_stone").unwrap();
    let block = sim
        .registry_mut()
        .intern("minecraft:redstone_block")
        .unwrap();
    let repeater = sim
        .registry_mut()
        .intern("minecraft:repeater[delay=1,facing=west,locked=false,powered=false]")
        .unwrap();
    for x in 0..4 {
        sim.world_mut().set(Pos::new(x, 0, 0), stone);
    }
    sim.world_mut().set(Pos::new(0, 1, 0), block);
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all(sim.registry_mut(), &mut table);
        *sim.behaviours_mut() = table;
    }
    let (solidity, frictions, heights, webs) = mc_tick::vanilla::physics_tables(sim.registry());
    sim.set_physics_tables(solidity, frictions, heights, webs);
    sim.place_block_by_hand(Pos::new(1, 1, 0), repeater);
    sim.run_until_quiescent(50);
    let state = sim.world().get(Pos::new(1, 1, 0));
    let descriptor = sim.registry().descriptor(state).expect("a real state");
    assert!(
        descriptor.contains("powered=true"),
        "lit by the block behind it, got {descriptor}"
    );
}

#[test]
fn the_actuator_write_still_leaves_a_wire_as_authored() {
    // The contrast that keeps the two paths honest: `place_block` is a state
    // write, not a placement, and must not run the derivation.
    let (mut sim, wire) = build();
    sim.place_block(Pos::new(1, 1, 0), wire);
    sim.run_until_quiescent(50);
    let state = sim.world().get(Pos::new(1, 1, 0));
    let descriptor = sim.registry().descriptor(state).expect("a real state");
    assert_eq!(
        descriptor, "minecraft:redstone_wire",
        "an actuator write is verbatim"
    );
}
