#[cfg(feature = "simulation")]
#[cfg(test)]
mod long_wire_test {
    use mchprs_world::World;
    use nucleation::simulation::{BlockPos, MchprsWorld};
    use nucleation::UniversalSchematic; // Needed for get_block methods

    #[test]
    fn test_long_wire_updates() {
        println!("\n=== Testing 17-block circuit: lever -> 15 wires -> lamp ===\n");

        // Create a simple schematic: lever at (0,0,0), 15 wires from (1,0,0) to (15,0,0), lamp at (16,0,0)
        let mut schematic = UniversalSchematic::new("long_wire".to_string());

        // Place lever at origin (unpowered)
        schematic.set_block_str(
            0,
            0,
            0,
            "minecraft:lever[facing=east,powered=false,face=floor]",
        );

        // Place 15 redstone wires
        for x in 1..=15 {
            schematic.set_block_str(x, 0, 0, "minecraft:redstone_wire[power=0]");
        }

        // Place lamp at end (unlit)
        schematic.set_block_str(16, 0, 0, "minecraft:redstone_lamp[lit=false]");

        println!("Initial schematic created:");
        println!("  [0,0,0]  lever[powered=false]");
        println!("  [1,0,0] to [15,0,0] redstone_wire[power=0]");
        println!("  [16,0,0] lamp[lit=false]");

        // Initialize simulation
        let mut world = MchprsWorld::new(schematic).expect("Failed to create world");
        world.flush();

        println!("\n--- STEP 1: Initial state ---");
        let lever_powered = world.get_lever_power(BlockPos::new(0, 0, 0));
        let lamp_lit = world.is_lit(BlockPos::new(16, 0, 0));
        println!("  Lever powered: {}", lever_powered);
        println!("  Lamp lit: {}", lamp_lit);

        // Toggle lever
        println!("\n--- STEP 2: Toggle lever at [0,0,0] ---");
        world.on_use_block(BlockPos::new(0, 0, 0));
        world.flush();

        let lever_powered = world.get_lever_power(BlockPos::new(0, 0, 0));
        println!("  Lever powered after toggle: {}", lever_powered);

        // Tick simulation
        println!("\n--- STEP 3: Tick simulation 20 times ---");
        world.tick(20);
        world.flush();

        // Check MCHPRS state for ALL wires
        println!("\n--- STEP 4: Check MCHPRS block properties ---");

        let lever_block = world.get_block(BlockPos::new(0, 0, 0));
        let lever_props = lever_block.properties();
        println!("  [0,0,0] lever powered={:?}", lever_props.get("powered"));

        for x in 1..=15 {
            let wire_block = world.get_block(BlockPos::new(x, 0, 0));
            let wire_props = wire_block.properties();
            let power = wire_props.get("power").map(|s| s.as_str()).unwrap_or("?");
            println!("  [{},0,0] wire power={}", x, power);
        }

        let lamp_block = world.get_block(BlockPos::new(16, 0, 0));
        let lamp_props = lamp_block.properties();
        println!("  [16,0,0] lamp lit={:?}", lamp_props.get("lit"));

        // Sync to schematic
        println!("\n--- STEP 5: Sync to schematic ---");
        world.sync_to_schematic();

        // Check synced schematic
        println!("\n--- STEP 6: Check synced schematic ---");
        let schematic = world.get_schematic();

        let lever = schematic.get_block(0, 0, 0).unwrap();
        println!("  [0,0,0] {}", format_block_with_props(&lever));

        for x in 1..=15 {
            let wire = schematic.get_block(x, 0, 0).unwrap();
            let power = wire
                .properties
                .get("power")
                .map(|s| s.as_str())
                .unwrap_or("?");
            println!("  [{},0,0] wire power={}", x, power);
        }

        let lamp = schematic.get_block(16, 0, 0).unwrap();
        println!("  [16,0,0] {}", format_block_with_props(&lamp));

        // Verify results
        println!("\n--- VERIFICATION ---");
        let lever = schematic.get_block(0, 0, 0).unwrap();
        assert_eq!(
            lever.properties.get("powered").map(|s| s.as_str()),
            Some("true"),
            "Lever should be powered"
        );

        let lamp = schematic.get_block(16, 0, 0).unwrap();
        assert_eq!(
            lamp.properties.get("lit").map(|s| s.as_str()),
            Some("true"),
            "Lamp should be lit"
        );

        // Check that at least SOME wires have non-zero power
        let mut wires_with_power = 0;
        for x in 1..=15 {
            let wire = schematic.get_block(x, 0, 0).unwrap();
            if let Some(power) = wire.properties.get("power") {
                if power != "0" {
                    wires_with_power += 1;
                }
            }
        }

        println!("\n✓ Lever: powered=true");
        println!("✓ Lamp: lit=true");
        println!("✓ Wires with power > 0: {}/15", wires_with_power);

        if wires_with_power == 0 {
            println!("\n⚠️  WARNING: NO WIRES UPDATED!");
            println!("This means MCHPRS is not updating wire properties,");
            println!("even though the lamp works correctly.");
            panic!(
                "Wires did not update - MCHPRS may not update intermediate component properties"
            );
        } else if wires_with_power < 15 {
            println!("\n⚠️  WARNING: Only {}/15 wires updated", wires_with_power);
            println!("Some wires may be too far from the power source (15 block limit)");
        } else {
            println!("\n✓ SUCCESS: All wires updated correctly");
        }
    }

    fn format_block_with_props(block: &nucleation::BlockState) -> String {
        if block.properties.is_empty() {
            block.name.clone()
        } else {
            let props: Vec<String> = block
                .properties
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("{}[{}]", block.name, props.join(","))
        }
    }

    #[test]
    fn test_invalid_initial_state() {
        println!("\n=== Testing circuit with INVALID initial redstone states ===\n");
        println!("This mimics your XOR gate scenario where initial states are wrong");

        // Create a circuit with WRONG initial properties
        let mut schematic = UniversalSchematic::new("invalid_initial".to_string());

        // Lever OFF but wire shows power=15 (INVALID!)
        schematic.set_block_str(
            0,
            0,
            0,
            "minecraft:lever[facing=east,powered=false,face=floor]",
        );
        schematic.set_block_str(1, 0, 0, "minecraft:redstone_wire[power=15]"); // WRONG!
        schematic.set_block_str(2, 0, 0, "minecraft:redstone_wire[power=14]"); // WRONG!
        schematic.set_block_str(3, 0, 0, "minecraft:redstone_lamp[lit=true]"); // WRONG!

        println!("Initial schematic (INVALID states):");
        println!("  [0,0,0] lever[powered=false]");
        println!("  [1,0,0] wire[power=15] <-- WRONG!");
        println!("  [2,0,0] wire[power=14] <-- WRONG!");
        println!("  [3,0,0] lamp[lit=true] <-- WRONG!");

        // Initialize simulation
        let mut world = MchprsWorld::new(schematic).expect("Failed to create world");

        println!("\n--- After init + flush (before ticks) ---");
        world.flush();

        let wire1 = world.get_block(BlockPos::new(1, 0, 0));
        let wire2 = world.get_block(BlockPos::new(2, 0, 0));
        let lamp = world.get_block(BlockPos::new(3, 0, 0));

        println!(
            "  Wire [1,0,0]: power={:?}",
            wire1.properties().get("power")
        );
        println!(
            "  Wire [2,0,0]: power={:?}",
            wire2.properties().get("power")
        );
        println!("  Lamp [3,0,0]: lit={:?}", lamp.properties().get("lit"));

        // Tick to let MCHPRS correct the invalid states
        println!("\n--- Tick 10 times to let MCHPRS correct ---");
        world.tick(10);
        world.flush();

        let wire1 = world.get_block(BlockPos::new(1, 0, 0));
        let wire2 = world.get_block(BlockPos::new(2, 0, 0));
        let lamp = world.get_block(BlockPos::new(3, 0, 0));

        println!(
            "  Wire [1,0,0]: power={:?}",
            wire1.properties().get("power")
        );
        println!(
            "  Wire [2,0,0]: power={:?}",
            wire2.properties().get("power")
        );
        println!("  Lamp [3,0,0]: lit={:?}", lamp.properties().get("lit"));

        // Sync
        println!("\n--- After sync ---");
        world.sync_to_schematic();
        let synced = world.get_schematic();

        let wire1 = synced.get_block(1, 0, 0).unwrap();
        let wire2 = synced.get_block(2, 0, 0).unwrap();
        let lamp = synced.get_block(3, 0, 0).unwrap();

        println!(
            "  Wire [1,0,0]: power={}",
            wire1.properties.get("power").unwrap()
        );
        println!(
            "  Wire [2,0,0]: power={}",
            wire2.properties.get("power").unwrap()
        );
        println!(
            "  Lamp [3,0,0]: lit={}",
            lamp.properties.get("lit").unwrap()
        );

        println!("\n=== RESULT ===");
        if wire1.properties.get("power") == Some(&"0".to_string())
            && wire2.properties.get("power") == Some(&"0".to_string())
            && lamp.properties.get("lit") == Some(&"false".to_string())
        {
            println!("✓ MCHPRS correctly fixed the invalid states!");
        } else {
            println!("⚠️  MCHPRS did NOT correct the invalid states!");
            println!("This might explain why your XOR gate doesn't work.");
        }
    }
}
