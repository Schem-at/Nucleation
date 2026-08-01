//! A water source written mid-run must start flowing on its own.
//!
//! `LiquidBlock.onPlace` schedules the fluid's first tick, so vanilla water
//! placed into a world spreads without waiting for a neighbour to change.
//! The engine's `Water` behaviour used to lack that hook: a `place_block`ed
//! source sat inert forever unless something nearby happened to update it,
//! while the same source loaded from a structure (whose settle pass pokes
//! neighbours) flowed fine. This pins the `place_block` route.

use mc_tick::{Pos, Simulation};

#[test]
fn a_placed_water_source_spreads_without_a_neighbour_poke() {
    // A bare 9x9 stone floor, built in place rather than parsed, so nothing
    // about loading or settling is in the frame.
    let mut sim = Simulation::new(mc_tick::Bounds {
        min: Pos::new(-4, -4, -4),
        max: Pos::new(12, 8, 12),
    });
    let stone = sim.registry_mut().intern("minecraft:smooth_stone").unwrap();
    let water = sim.registry_mut().intern("minecraft:water[level=0]").unwrap();
    for x in 0..9 {
        for z in 0..9 {
            sim.world_mut().set(Pos::new(x, 0, z), stone);
        }
    }
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

    sim.place_block(Pos::new(4, 1, 4), water);
    for _ in 0..40 {
        sim.step();
    }

    let spread = sim
        .world()
        .iter_non_air()
        .filter(|(_, state)| {
            sim.registry()
                .descriptor(*state)
                .is_some_and(|d| d.starts_with("minecraft:water"))
        })
        .count();
    assert!(
        spread > 1,
        "a placed source must schedule its own first fluid tick; still {spread} cell(s) after 40 ticks"
    );
}
