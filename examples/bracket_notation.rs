// Example demonstrating bracket notation support for setting blocks
//
// Bracket notation allows you to specify block properties inline:
// "minecraft:lever[facing=east,powered=false,face=floor]"
//
// This is especially useful for redstone circuits where properties matter.

use nucleation::{BlockState, UniversalSchematic};

fn main() {
    let mut schematic = UniversalSchematic::new("Bracket Notation Example".to_string());

    // Method 1: Using BlockState with properties (verbose but explicit)
    let mut lever = BlockState::new("minecraft:lever".to_string());
    lever
        .properties
        .insert("facing".to_string(), "east".to_string());
    lever
        .properties
        .insert("powered".to_string(), "false".to_string());
    lever
        .properties
        .insert("face".to_string(), "floor".to_string());
    schematic.set_block(0, 1, 0, lever);

    // Method 2: Using bracket notation (concise and readable)
    schematic.set_block_str(
        1,
        1,
        0,
        "minecraft:lever[facing=west,powered=true,face=floor]",
    );

    // Method 3: Simple blocks without properties (still works as before)
    schematic.set_block_str(0, 0, 0, "minecraft:gray_concrete");

    // Method 4: Complex redstone wire with multiple properties
    schematic.set_block_str(
        5,
        1,
        0,
        "minecraft:redstone_wire[power=15,east=side,west=side,north=none,south=up]",
    );

    // Method 5: Block with NBT data (using braces)
    schematic.set_block_str(10, 1, 0, "minecraft:barrel{signal=10}");

    // Verify the blocks were set correctly
    println!("Lever at (0,1,0): {:?}", schematic.get_block(0, 1, 0));
    println!("Lever at (1,1,0): {:?}", schematic.get_block(1, 1, 0));
    println!("Concrete at (0,0,0): {:?}", schematic.get_block(0, 0, 0));
    println!(
        "Redstone wire at (5,1,0): {:?}",
        schematic.get_block(5, 1, 0)
    );
    println!("Barrel at (10,1,0): {:?}", schematic.get_block(10, 1, 0));

    // All three methods produce the same result!
    // Use whichever is most convenient for your use case.
}
