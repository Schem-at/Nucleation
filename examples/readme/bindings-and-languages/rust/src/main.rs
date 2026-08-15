//! Executable Rust source for docs/features/bindings-and-languages.md.

use std::error::Error;
use std::fs;

use nucleation::UniversalSchematic;

fn main() -> Result<(), Box<dyn Error>> {
    // --8<-- [start:build]
    let mut stack = UniversalSchematic::new("binding_stack".into());
    stack.fill_cuboid_str((-3, 0, -3), (3, 0, 3), "minecraft:polished_deepslate");
    stack.fill_cuboid_str((-2, 1, -2), (2, 1, 2), "minecraft:light_blue_concrete");
    stack.fill_cuboid_str((-1, 2, -1), (1, 2, 1), "minecraft:yellow_concrete");
    stack.set_block_from_string(0, 3, 0, "minecraft:emerald_block")?;

    assert_eq!(stack.total_blocks(), 84);
    assert_eq!(stack.get_tight_dimensions(), (7, 4, 7));
    // --8<-- [end:build]

    let output = std::env::var("BINDINGS_OUT")
        .unwrap_or_else(|_| "binding-stack.schem".into());
    fs::write(&output, stack.to_schematic()?)?;
    println!("Bindings Rust example: OK ({output})");
    Ok(())
}
