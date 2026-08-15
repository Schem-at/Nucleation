//! Executable Rust source for docs/features/smart-placement-and-simulation.md.

use std::error::Error;
use std::fs;

use nucleation::UniversalSchematic;

fn main() -> Result<(), Box<dyn Error>> {
    // --8<-- [start:author]
    let mut scene = UniversalSchematic::new("smart_circuit".into());
    scene.fill_cuboid_str((0, 0, 0), (8, 0, 2), "minecraft:smooth_stone");
    scene.set_block_from_string(
        0, 1, 0,
        "minecraft:lever[face=floor,facing=east,powered=false]",
    )?;

    // Rust's native API exposes the same descriptor path one placement at a time.
    for x in 1..=6 {
        scene.set_block_from_string(x, 1, 0, "minecraft:redstone_wire{simulate=true}")?;
    }
    scene.set_block_from_string(
        7, 1, 0,
        "minecraft:redstone_lamp[lit=false]{simulate=true}",
    )?;
    scene.set_block_from_string(
        0, 1, 2,
        "minecraft:barrel[facing=west]{signal=13,item=iron_ingot}",
    )?;
    // --8<-- [end:author]

    // --8<-- [start:tick]
    use mc_tick::embed::SimulationBuilder;
    use mc_tick::pos::{Bounds, Pos};

    let mut builder = SimulationBuilder::new(Bounds::new(Pos::new(0, 0, 0), Pos::new(8, 1, 2)));
    for (position, block) in scene.iter_blocks() {
        builder.set_block(
            Pos::new(position.x, position.y, position.z),
            &block.to_string(),
        );
    }
    let mut tick = builder.build()?;
    tick.use_block(Pos::new(0, 1, 0));
    tick.step();
    tick.step();
    let lamp = tick.registry().descriptor(tick.world().get(Pos::new(7, 1, 0))).unwrap();
    assert_eq!(lamp, "minecraft:redstone_lamp[lit=true]");
    assert_eq!(tick.tick_count(), 2);
    // --8<-- [end:tick]

    assert_eq!(scene.total_blocks(), 36);
    assert_eq!(scene.get_tight_dimensions(), (9, 2, 3));
    assert_eq!(
        scene.get_block(3, 1, 0).unwrap().to_string(),
        "minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]"
    );

    let output = std::env::var("SMART_SIMULATION_OUT")
        .unwrap_or_else(|_| "smart-circuit.schem".to_string());
    fs::write(&output, scene.to_schematic()?)?;
    println!("Smart simulation Rust example: OK ({output})");
    Ok(())
}
