//! A payload can exceed the push limit by adhering to neighboring piston bases.
#[allow(dead_code)]
#[path = "../examples/check_community_capture.rs"]
mod checker;

use mc_tick::Pos;

const CAPTURE: &[u8] = include_bytes!("corpus/extender12111/honey_piston_bank.json");

#[test]
fn adjacent_piston_bank_matches_every_vanilla_tick() {
    checker::assert_matches_capture(CAPTURE);
}

#[test]
fn twelve_block_load_moves_but_thirteen_and_twenty_three_do_not() {
    let mut sim = checker::fixture_sim(CAPTURE);
    let power = sim.registry().get("minecraft:redstone_block").unwrap();
    for z in [11, 27, 43, 59] {
        sim.place_block(Pos::new(4, 1, z), power);
    }
    sim.run(24);
    let state = |x, z| sim.registry().descriptor(sim.world().get(Pos::new(x, 1, z))).unwrap();
    // Original bank: twelve honey blocks also adhere to eleven passive bases.
    // Boundary control: twelve honey plus one passive base is already too many.
    for z in [11, 59] {
        assert_eq!(state(3, z), "minecraft:sticky_piston[extended=false,facing=west]");
        assert_eq!(state(2, z), "minecraft:honey_block");
    }
    // Removing the passive bases permits all twelve honey blocks to move.
    for z in 16..28 {
        assert_eq!(state(1, z), "minecraft:honey_block");
    }
    // Keeping the bases but removing the eleven other honey blocks also moves.
    assert_eq!(state(1, 43), "minecraft:honey_block");
    for z in [27, 43] {
        assert_eq!(state(3, z), "minecraft:sticky_piston[extended=true,facing=west]");
    }
}
