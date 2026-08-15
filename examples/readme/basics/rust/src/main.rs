//! Executable source for every Rust snippet in docs/features/basics.md.

use nucleation::UniversalSchematic;
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    // --8<-- [start:beacon]
    use nucleation::UniversalSchematic;
    use std::fs;

    let mut beacon = UniversalSchematic::new("beacon".into());
    for x in -1..=1 {
        for z in -1..=1 {
            beacon.set_block_from_string(x, 0, z, "minecraft:gold_block")?;
        }
    }
    beacon.set_block_from_string(0, 1, 0, "minecraft:beacon")?;
    fs::write("beacon.schem", beacon.to_schematic()?)?;
    // --8<-- [end:beacon]

    assert_eq!(beacon.total_blocks(), 10);
    assert_eq!(beacon.get_tight_dimensions(), (3, 2, 3));

    // --8<-- [start:crafting-nook]
    let mut nook = UniversalSchematic::new("crafting_nook".into());
    for x in 0..5 {
        for z in 0..5 {
            nook.set_block_from_string(x, 0, z, "minecraft:spruce_planks")?;
        }
    }

    let wall_block = |i: i32, y: i32, end_posts: &[i32]| {
        if i == 2 && y == 2 {
            "minecraft:light_blue_stained_glass"
        } else if end_posts.contains(&i) {
            "minecraft:stripped_spruce_log[axis=y]"
        } else {
            "minecraft:oak_planks"
        }
    };

    for y in [1, 2, 3] {
        for x in 0..5 {
            nook.set_block_from_string(x, y, 0, wall_block(x, y, &[0, 4]))?;
        }
        for z in 1..5 {
            nook.set_block_from_string(0, y, z, wall_block(z, y, &[4]))?;
        }
    }

    nook.set_block_from_string(1, 1, 1, "minecraft:crafting_table")?;
    nook.set_block_from_string(3, 1, 1, "minecraft:chest[facing=south]")?;
    nook.set_block_from_string(4, 2, 1, "minecraft:wall_torch[facing=south]")?;
    nook.set_block_from_string(1, 2, 4, "minecraft:wall_torch[facing=east]")?;
    fs::write("crafting-nook.schem", nook.to_schematic()?)?;
    // --8<-- [end:crafting-nook]

    assert_eq!(nook.total_blocks(), 56);

    // --8<-- [start:coordinates]
    let mut build = UniversalSchematic::new("signed_coordinates".into());
    build.set_block_from_string(-8, 64, 12, "minecraft:stone")?;
    build.set_block_from_string(24, 80, -3, "minecraft:glass")?;

    let bounds = build.get_tight_bounds().expect("the build has blocks");
    println!("{:?}", bounds.min); // (-8, 64, -3)
    println!("{:?}", bounds.max); // (24, 80, 12)
    println!("{:?}", build.get_tight_dimensions()); // (33, 17, 16)

    // --8<-- [end:coordinates]

    assert_eq!(bounds.min, (-8, 64, -3));
    assert_eq!(bounds.max, (24, 80, 12));
    assert_eq!(build.get_tight_dimensions(), (33, 17, 16));

    // --8<-- [start:block-states]
    let mut inspect = UniversalSchematic::new("inspect".into());
    inspect.set_block_from_string(1, 1, 1, "minecraft:oak_log[axis=x]")?;
    let state = inspect.get_block(1, 1, 1).expect("the block exists");
    println!("{}", state.get_name()); // minecraft:oak_log
    println!("{state}"); // minecraft:oak_log[axis=x]

    inspect.set_block_from_string(1, 1, 1, "minecraft:air")?; // remove it

    // --8<-- [end:block-states]

    assert_eq!(inspect.total_blocks(), 0);

    // --8<-- [start:contents]
    let mut contents = UniversalSchematic::new("contents".into());
    contents.set_block_from_string(0, 0, 0, "minecraft:barrel{signal=13,item=diamond}")?;
    contents.set_block_from_string(1, 0, 0, "minecraft:chest{items=[diamond*64,emerald*12]}")?;
    contents.set_block_from_string(2, 0, 0, "minecraft:jukebox{record=pigstep}")?;
    contents.set_block_from_string(3, 0, 0, "minecraft:jukebox{signal=13}")?;
    // --8<-- [end:contents]

    assert_eq!(contents.total_blocks(), 4);

    // --8<-- [start:simulation]
    let mut circuit = UniversalSchematic::new("placed_by_engine".into());
    circuit.set_block_from_string(4, 0, 0, "minecraft:redstone_block")?;
    circuit.set_block_from_string(5, 0, 0, "minecraft:redstone_wire{simulate=true}")?;
    println!("{}", circuit.get_block(5, 0, 0).expect("the wire exists"));
    // minecraft:redstone_wire[east=side,north=none,power=15,south=none,west=side]
    // --8<-- [end:simulation]

    assert_eq!(
        circuit
            .get_block(5, 0, 0)
            .expect("the wire exists")
            .to_string(),
        "minecraft:redstone_wire[east=side,north=none,power=15,south=none,west=side]"
    );

    // --8<-- [start:io]
    let bytes = fs::read("beacon.schem")?;
    let mut copy = UniversalSchematic::from_schematic(&bytes)?;
    copy.set_block_from_string(0, 2, 0, "minecraft:glass")?;
    fs::write("beacon-edited.schem", copy.to_schematic()?)?;
    // --8<-- [end:io]

    assert_eq!(copy.total_blocks(), 11);
    println!("Basics Rust examples: OK");
    Ok(())
}
