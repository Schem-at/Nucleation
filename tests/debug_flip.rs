use nucleation::{BlockState, Region};

#[test]
fn debug_flip_positions() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (6, 1, 1));

    // Place blocks with facing properties at known positions
    let mut block_east = BlockState::new("minecraft:lever".to_string());
    block_east.set_property("facing".to_string(), "east".to_string());

    let mut block_west = BlockState::new("minecraft:lever".to_string());
    block_west.set_property("facing".to_string(), "west".to_string());

    region.set_block(1, 0, 0, block_east.clone());
    region.set_block(4, 0, 0, block_west.clone());

    println!("Before flip:");
    println!(
        "  Block at (1,0,0): {:?}",
        region.get_block(1, 0, 0).map(|b| b.get_property("facing"))
    );
    println!(
        "  Block at (4,0,0): {:?}",
        region.get_block(4, 0, 0).map(|b| b.get_property("facing"))
    );

    region.flip_x();

    println!("\nAfter X flip:");
    println!(
        "  Region bounding box: min={:?}, max={:?}",
        region.get_bounding_box().min,
        region.get_bounding_box().max
    );

    // Check all positions
    for x in 0..6 {
        if let Some(block) = region.get_block(x, 0, 0) {
            if block.name != "minecraft:air" {
                println!(
                    "  Block at ({},0,0): name={}, facing={:?}",
                    x,
                    block.name,
                    block.get_property("facing")
                );
            }
        }
    }
}
