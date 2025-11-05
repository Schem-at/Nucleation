#[cfg(feature = "simulation")]
#[cfg(test)]
mod mchprs_state_tests {
    use mchprs_world::World;
    use nucleation::simulation::{BlockPos, MchprsWorld};
    use nucleation::UniversalSchematic; // Needed for get_block methods

    /// Helper to create a simple XOR gate schematic
    fn create_xor_gate() -> UniversalSchematic {
        let mut schematic = UniversalSchematic::new("XOR Gate Test".to_string());

        // Base layer
        for x in 0..3 {
            for z in 0..4 {
                schematic.set_block_str(x, 0, z, "minecraft:gray_concrete");
            }
        }

        // Input levers
        schematic.set_block_str(
            0,
            0,
            3,
            "minecraft:lever[facing=south,powered=false,face=wall]",
        );
        schematic.set_block_str(
            2,
            0,
            3,
            "minecraft:lever[facing=south,powered=false,face=wall]",
        );

        // Redstone wire layer 1
        schematic.set_block_str(0, 1, 1, "minecraft:redstone_wire[power=0]");
        schematic.set_block_str(2, 1, 1, "minecraft:redstone_wire[power=0]");

        // Redstone torches
        schematic.set_block_str(0, 1, 2, "minecraft:redstone_torch[lit=true]");
        schematic.set_block_str(2, 1, 2, "minecraft:redstone_torch[lit=true]");

        // Top layer wire
        schematic.set_block_str(1, 2, 2, "minecraft:redstone_wire[power=0]");

        // Wall torches
        schematic.set_block_str(
            0,
            0,
            0,
            "minecraft:redstone_wall_torch[facing=north,lit=true]",
        );
        schematic.set_block_str(
            1,
            1,
            1,
            "minecraft:redstone_wall_torch[facing=north,lit=true]",
        );
        schematic.set_block_str(
            2,
            0,
            0,
            "minecraft:redstone_wall_torch[facing=north,lit=true]",
        );

        // Output lamp
        schematic.set_block_str(1, 0, 0, "minecraft:redstone_lamp[lit=false]");

        schematic
    }

    #[test]
    fn test_mchprs_block_properties_after_flush() {
        println!("\n=== TEST: MCHPRS Block Properties After Flush ===\n");

        let schematic = create_xor_gate();
        let mut world = MchprsWorld::new(schematic).expect("Failed to create world");

        println!("STEP 1: Initial state (before any interaction)");
        let initial_schematic = world.get_schematic();

        // Check initial lamp state
        if let Some(lamp) = initial_schematic.get_block(1, 0, 0) {
            println!("  Lamp initial: {}", lamp.to_string());
            assert_eq!(lamp.properties.get("lit"), Some(&"false".to_string()));
        }

        println!("\nSTEP 2: Toggle lever at [2,0,3]");
        world.on_use_block(BlockPos::new(2, 0, 3));
        world.flush();

        // Check world's internal view of blocks BEFORE sync
        println!("\nSTEP 3: Check MCHPRS internal state (via World::get_block)");
        let lamp_block = world.get_block(BlockPos::new(1, 0, 0));
        let lever_block = world.get_block(BlockPos::new(2, 0, 3));
        let wire_block = world.get_block(BlockPos::new(2, 1, 1));
        let torch_block = world.get_block(BlockPos::new(2, 1, 2));

        println!(
            "  Lamp (from MCHPRS):   {} -> properties: {:?}",
            lamp_block.get_name(),
            lamp_block.properties()
        );
        println!(
            "  Lever (from MCHPRS):  {} -> properties: {:?}",
            lever_block.get_name(),
            lever_block.properties()
        );
        println!(
            "  Wire (from MCHPRS):   {} -> properties: {:?}",
            wire_block.get_name(),
            wire_block.properties()
        );
        println!(
            "  Torch (from MCHPRS):  {} -> properties: {:?}",
            torch_block.get_name(),
            torch_block.properties()
        );

        println!("\nSTEP 4: Tick simulation 10 times");
        world.tick(10);
        world.flush();

        println!("\nSTEP 5: Check MCHPRS internal state after ticking");
        let lamp_block_after = world.get_block(BlockPos::new(1, 0, 0));
        let wire_block_after = world.get_block(BlockPos::new(2, 1, 1));
        let torch_block_after = world.get_block(BlockPos::new(2, 1, 2));

        println!(
            "  Lamp (from MCHPRS):   {} -> properties: {:?}",
            lamp_block_after.get_name(),
            lamp_block_after.properties()
        );
        println!(
            "  Wire (from MCHPRS):   {} -> properties: {:?}",
            wire_block_after.get_name(),
            wire_block_after.properties()
        );
        println!(
            "  Torch (from MCHPRS):  {} -> properties: {:?}",
            torch_block_after.get_name(),
            torch_block_after.properties()
        );

        // Key question: Did MCHPRS update the properties?
        let lamp_lit = lamp_block_after.properties().get("lit").map(|s| s.as_str()) == Some("true");
        let wire_powered = wire_block_after
            .properties()
            .get("power")
            .map(|s| s.as_str())
            != Some("0");
        let torch_lit = torch_block_after
            .properties()
            .get("lit")
            .map(|s| s.as_str())
            == Some("true");

        println!("\n  Lamp lit property updated: {}", lamp_lit);
        println!("  Wire power property updated: {}", wire_powered);
        println!("  Torch lit property updated: {}", torch_lit);

        println!("\nSTEP 6: Call sync_to_schematic()");
        world.sync_to_schematic();
        let synced = world.get_schematic();

        println!("\nSTEP 7: Check synced schematic properties");
        if let Some(lamp) = synced.get_block(1, 0, 0) {
            println!("  Synced lamp: {}", lamp.to_string());
        }
        if let Some(wire) = synced.get_block(2, 1, 1) {
            println!("  Synced wire: {}", wire.to_string());
        }
        if let Some(torch) = synced.get_block(2, 1, 2) {
            println!("  Synced torch: {}", torch.to_string());
        }

        println!("\n=== KEY FINDINGS ===");
        println!("If MCHPRS properties ARE updated: sync_to_schematic() should work correctly");
        println!("If MCHPRS properties ARE NOT updated: we need to query MCHPRS state differently");
        println!(
            "If lamp IS updated but wire/torch NOT: MCHPRS only updates blocks with state changes"
        );
    }

    #[test]
    fn test_mchprs_block_id_changes() {
        println!("\n=== TEST: MCHPRS Block ID Changes ===\n");

        let schematic = create_xor_gate();
        let mut world = MchprsWorld::new(schematic).expect("Failed to create world");

        println!("Checking if MCHPRS changes block IDs (not just properties)");

        let lamp_id_before = world.get_block_raw(BlockPos::new(1, 0, 0));
        let torch_id_before = world.get_block_raw(BlockPos::new(2, 1, 2));

        println!("  Lamp block ID before:  {}", lamp_id_before);
        println!("  Torch block ID before: {}", torch_id_before);

        // Toggle and tick
        world.on_use_block(BlockPos::new(2, 0, 3));
        world.flush();
        world.tick(10);
        world.flush();

        let lamp_id_after = world.get_block_raw(BlockPos::new(1, 0, 0));
        let torch_id_after = world.get_block_raw(BlockPos::new(2, 1, 2));

        println!("  Lamp block ID after:   {}", lamp_id_after);
        println!("  Torch block ID after:  {}", torch_id_after);

        println!("\n  Lamp ID changed: {}", lamp_id_before != lamp_id_after);
        println!("  Torch ID changed: {}", torch_id_before != torch_id_after);

        println!("\nIf IDs changed: MCHPRS is updating block states correctly");
        println!("If IDs same: MCHPRS is not placing new blocks with updated states");
    }

    #[test]
    fn test_direct_property_comparison() {
        println!("\n=== TEST: Direct Property Comparison ===\n");

        let schematic = create_xor_gate();
        let mut world = MchprsWorld::new(schematic).expect("Failed to create world");

        // Get initial properties from schematic
        let initial_lamp = world.get_schematic().get_block(1, 0, 0).unwrap().clone();
        println!(
            "Initial schematic lamp properties: {:?}",
            initial_lamp.properties
        );

        // Toggle and tick
        world.on_use_block(BlockPos::new(2, 0, 3));
        world.flush();
        world.tick(10);
        world.flush();

        // Get MCHPRS block directly
        let mchprs_lamp = world.get_block(BlockPos::new(1, 0, 0));
        let mchprs_props = mchprs_lamp.properties();
        println!("MCHPRS lamp properties after ticks: {:?}", mchprs_props);
        let mchprs_lit = mchprs_props.get("lit").map(|s| s.to_string());

        // Now sync
        world.sync_to_schematic();
        let synced_lamp = world.get_schematic().get_block(1, 0, 0).unwrap();
        println!(
            "Synced schematic lamp properties: {:?}",
            synced_lamp.properties
        );
        let synced_lit = synced_lamp.properties.get("lit").map(|s| s.clone());

        // Compare
        println!("\nMCHPRS says lit={:?}", mchprs_lit);
        println!("Synced says lit={:?}", synced_lit);
        println!("Match: {}", mchprs_lit == synced_lit);
    }
}
