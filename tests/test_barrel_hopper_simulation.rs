use nucleation::UniversalSchematic;

#[cfg(feature = "simulation")]
use nucleation::simulation::MchprsWorld;

#[test]
#[cfg(feature = "simulation")]
fn test_barrel_hopper_simulation() {
    let mut schematic = UniversalSchematic::new("test".to_string());

    // Test blocks from the Python script
    let test_blocks = vec!["minecraft:barrel[facing=north]", "minecraft:hopper"];

    // Helper to set a block with support
    fn set_block_supported(
        schematic: &mut UniversalSchematic,
        block: &str,
        pos: (i32, i32, i32),
        support: &str,
    ) {
        schematic.set_block_str(pos.0, pos.1, pos.2, block);
        schematic.set_block_str(pos.0, pos.1 - 1, pos.2, support);
    }

    // Helper to place a block with signal
    fn place_block_with_signal(
        schematic: &mut UniversalSchematic,
        block: &str,
        signal: i32,
        pos: (i32, i32, i32),
    ) {
        // Just place the block with facing, NBT handling is complex and not the focus of this test
        let block_str = format!("{}[facing=north]", block);
        set_block_supported(schematic, &block_str, pos, "minecraft:gray_concrete");
        set_block_supported(
            schematic,
            "minecraft:comparator[facing=north]",
            (pos.0, pos.1, pos.2 + 1),
            "minecraft:gray_concrete",
        );
        set_block_supported(
            schematic,
            "minecraft:redstone_wire[power=0,east=none,west=none,north=side,south=side]",
            (pos.0, pos.1, pos.2 + 2),
            "minecraft:gray_concrete",
        );
    }

    // Place blocks in a grid pattern
    for signal_strength in 0..16 {
        for (index, block) in test_blocks.iter().enumerate() {
            place_block_with_signal(
                &mut schematic,
                block,
                signal_strength,
                (signal_strength * 2, 1, index as i32 * 4),
            );
        }
    }

    // Debug: Print bounding box info
    let bbox = schematic.get_bounding_box();
    println!("Bounding box: min={:?}, max={:?}", bbox.min, bbox.max);
    println!("Dimensions: {:?}", bbox.get_dimensions());

    // This should not panic
    let simulation = MchprsWorld::new(schematic);

    match simulation {
        Ok(mut sim) => {
            sim.tick(1);
            sim.flush();
            sim.sync_to_schematic();
            println!("Simulation completed successfully");
        }
        Err(e) => {
            panic!("Simulation creation failed: {}", e);
        }
    }
}

#[test]
#[cfg(feature = "simulation")]
fn test_minimal_barrel_simulation() {
    let mut schematic = UniversalSchematic::new("minimal".to_string());

    // Minimal test case: single barrel with support
    schematic.set_block_str(0, 1, 0, "minecraft:barrel[facing=north]");
    schematic.set_block_str(0, 0, 0, "minecraft:gray_concrete");

    let bbox = schematic.get_bounding_box();
    println!(
        "Minimal test - Bounding box: min={:?}, max={:?}",
        bbox.min, bbox.max
    );
    println!("Dimensions: {:?}", bbox.get_dimensions());

    // BUG IDENTIFIED:
    // The schematic stores blocks at Y=0 and Y=1, but Minecraft's default Y coordinate
    // system in UniversalSchematic starts at Y=-64 (the new world height minimum from 1.18+).
    // When MCHPRS tries to access these blocks, it attempts to cast negative Y values
    // to u32, which wraps around to huge positive numbers (e.g., -64 -> 4294967232).
    // This causes an index out of bounds panic in the chunk storage.
    //
    // The fix requires normalizing all coordinates relative to the schematic's minimum
    // coordinates before passing them to MCHPRS's chunk storage.

    let simulation = MchprsWorld::new(schematic);

    match simulation {
        Ok(mut sim) => {
            sim.tick(1);
            println!("Minimal simulation completed successfully");
        }
        Err(e) => {
            panic!("Minimal simulation creation failed: {}", e);
        }
    }
}
