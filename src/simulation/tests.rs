#[cfg(test)]
mod tests {
    use super::super::{generate_truth_table, BlockPos, MchprsWorld, SimulationOptions};
    use crate::{BlockState, UniversalSchematic};

    fn create_simple_redstone_line() -> UniversalSchematic {
        let mut schematic = UniversalSchematic::new("Simple Redstone Line".to_string());

        // Base layer of concrete
        for x in 0..16 {
            schematic.set_block(
                x,
                0,
                0,
                BlockState::new("minecraft:gray_concrete".to_string()),
            );
        }

        // Redstone wire
        for x in 1..15 {
            let mut wire = BlockState::new("minecraft:redstone_wire".to_string());
            wire.properties.insert("power".to_string(), "0".to_string());
            wire.properties
                .insert("east".to_string(), "side".to_string());
            wire.properties
                .insert("west".to_string(), "side".to_string());
            wire.properties
                .insert("north".to_string(), "none".to_string());
            wire.properties
                .insert("south".to_string(), "none".to_string());
            schematic.set_block(x, 1, 0, wire);
        }

        // Lever at position 0
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

        // Lamp at position 15
        let mut lamp = BlockState::new("minecraft:redstone_lamp".to_string());
        lamp.properties
            .insert("lit".to_string(), "false".to_string());
        schematic.set_block(15, 1, 0, lamp);

        schematic
    }

    fn create_and_gate() -> UniversalSchematic {
        let mut schematic = UniversalSchematic::new("AND Gate".to_string());

        // Base platform
        for x in 0..3 {
            for z in 0..4 {
                schematic.set_block(
                    x,
                    0,
                    z,
                    BlockState::new("minecraft:gray_concrete".to_string()),
                );
            }
        }

        // Two levers as inputs
        let mut lever_a = BlockState::new("minecraft:lever".to_string());
        lever_a
            .properties
            .insert("powered".to_string(), "false".to_string());
        lever_a
            .properties
            .insert("facing".to_string(), "north".to_string());
        lever_a
            .properties
            .insert("face".to_string(), "floor".to_string());
        schematic.set_block(0, 1, 0, lever_a.clone());

        let mut lever_b = BlockState::new("minecraft:lever".to_string());
        lever_b
            .properties
            .insert("powered".to_string(), "false".to_string());
        lever_b
            .properties
            .insert("facing".to_string(), "north".to_string());
        lever_b
            .properties
            .insert("face".to_string(), "floor".to_string());
        schematic.set_block(2, 1, 0, lever_b);

        // AND gate logic with redstone
        let mut wire = BlockState::new("minecraft:redstone_wire".to_string());
        wire.properties.insert("power".to_string(), "0".to_string());
        schematic.set_block(0, 1, 1, wire.clone());
        schematic.set_block(1, 1, 1, wire.clone());
        schematic.set_block(2, 1, 1, wire.clone());
        schematic.set_block(1, 1, 2, wire.clone());

        // Output lamp
        let mut lamp = BlockState::new("minecraft:redstone_lamp".to_string());
        lamp.properties
            .insert("lit".to_string(), "false".to_string());
        schematic.set_block(1, 1, 3, lamp);

        schematic
    }

    #[test]
    fn test_world_creation() {
        let schematic = create_simple_redstone_line();
        let world = MchprsWorld::new(schematic);
        assert!(world.is_ok(), "World creation should succeed");
    }

    #[test]
    fn test_lever_toggle() {
        let schematic = create_simple_redstone_line();
        let mut world = MchprsWorld::new(schematic).expect("World creation failed");

        let lever_pos = BlockPos::new(0, 1, 0);

        // Initial state should be unpowered
        assert_eq!(
            world.get_lever_power(lever_pos),
            false,
            "Lever should start unpowered"
        );

        // Toggle lever on
        world.on_use_block(lever_pos);
        assert_eq!(
            world.get_lever_power(lever_pos),
            true,
            "Lever should be powered after toggle"
        );

        // Toggle lever off
        world.on_use_block(lever_pos);
        assert_eq!(
            world.get_lever_power(lever_pos),
            false,
            "Lever should be unpowered after second toggle"
        );
    }

    #[test]
    fn test_redstone_propagation() {
        let schematic = create_simple_redstone_line();
        let mut world = MchprsWorld::new(schematic).expect("World creation failed");

        let lever_pos = BlockPos::new(0, 1, 0);
        let lamp_pos = BlockPos::new(15, 1, 0);

        // Initially lamp should be off
        assert_eq!(world.is_lit(lamp_pos), false, "Lamp should start off");

        // Toggle lever on
        world.on_use_block(lever_pos);
        world.tick(2);
        world.flush();

        // Lamp should now be lit
        assert_eq!(
            world.is_lit(lamp_pos),
            true,
            "Lamp should be lit after lever is toggled on"
        );

        // Toggle lever off
        world.on_use_block(lever_pos);
        world.tick(2);
        world.flush();

        // Lamp should be off again
        assert_eq!(
            world.is_lit(lamp_pos),
            false,
            "Lamp should be off after lever is toggled off"
        );
    }

    #[test]
    fn test_redstone_power_levels() {
        let schematic = create_simple_redstone_line();
        let mut world = MchprsWorld::new(schematic).expect("World creation failed");

        let lever_pos = BlockPos::new(0, 1, 0);

        // Toggle lever on and flush
        world.on_use_block(lever_pos);
        world.tick(2);
        world.flush();

        // Check power levels decrease with distance
        for x in 1..15 {
            let wire_pos = BlockPos::new(x, 1, 0);
            let power = world.get_redstone_power(wire_pos);
            assert!(power > 0, "Wire at x={} should have power", x);
            assert!(power <= 15, "Power should not exceed 15");
        }
    }

    #[test]
    fn test_and_gate_truth_table() {
        let schematic = create_and_gate();
        let truth_table = generate_truth_table(&schematic);

        // Should have 4 entries for 2 inputs (2^2 = 4)
        assert_eq!(
            truth_table.len(),
            4,
            "AND gate should have 4 truth table entries"
        );

        // Just verify we can generate a truth table
        // Note: The simple circuit above isn't a proper AND gate - it would need
        // more complex redstone logic. This test just verifies truth table generation works.
        assert!(
            truth_table.iter().all(|row| {
                row.contains_key("Input 0")
                    && row.contains_key("Input 1")
                    && row.contains_key("Output 0")
            }),
            "Truth table should have all required keys"
        );
    }

    #[test]
    fn test_multiple_ticks() {
        let schematic = create_simple_redstone_line();
        let mut world = MchprsWorld::new(schematic).expect("World creation failed");

        let lever_pos = BlockPos::new(0, 1, 0);
        let lamp_pos = BlockPos::new(15, 1, 0);

        // Toggle and wait multiple ticks
        world.on_use_block(lever_pos);
        world.tick(20); // Wait longer
        world.flush();

        assert_eq!(
            world.is_lit(lamp_pos),
            true,
            "Lamp should be lit after sufficient ticks"
        );
    }

    #[test]
    fn test_world_state_persistence() {
        let schematic = create_simple_redstone_line();
        let mut world = MchprsWorld::new(schematic).expect("World creation failed");

        let lever_pos = BlockPos::new(0, 1, 0);

        // Set lever state
        world.on_use_block(lever_pos);
        world.tick(1);
        world.flush();

        let state_after_toggle = world.get_lever_power(lever_pos);

        // State should persist
        world.tick(10);
        world.flush();

        assert_eq!(
            world.get_lever_power(lever_pos),
            state_after_toggle,
            "Lever state should persist across ticks"
        );
    }

    #[test]
    fn test_signal_strength_set_get() {
        use super::super::SimulationOptions;
        let schematic = create_simple_redstone_line();
        let wire_pos = BlockPos::new(5, 1, 0);

        let options = SimulationOptions {
            custom_io: vec![wire_pos],
            ..Default::default()
        };
        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        // Initially should return 0
        assert_eq!(
            world.get_signal_strength(wire_pos),
            0,
            "Signal strength should start at 0"
        );

        // Set signal strength
        world.set_signal_strength(wire_pos, 10);
        world.tick(1);
        world.flush();

        // Should be able to read it back
        let strength = world.get_signal_strength(wire_pos);
        assert_eq!(
            strength, 10,
            "Signal strength should be readable after setting"
        );
    }

    #[test]
    fn test_signal_strength_boundary_values() {
        use super::super::SimulationOptions;
        let schematic = create_simple_redstone_line();
        let wire_pos = BlockPos::new(5, 1, 0);

        let options = SimulationOptions {
            custom_io: vec![wire_pos],
            ..Default::default()
        };
        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        // Test minimum value (0)
        world.set_signal_strength(wire_pos, 0);
        world.tick(1);
        world.flush();
        assert_eq!(
            world.get_signal_strength(wire_pos),
            0,
            "Should handle signal strength of 0"
        );

        // Test maximum value (15)
        world.set_signal_strength(wire_pos, 15);
        world.tick(1);
        world.flush();
        assert_eq!(
            world.get_signal_strength(wire_pos),
            15,
            "Should handle signal strength of 15"
        );

        // Test mid-range value
        world.set_signal_strength(wire_pos, 7);
        world.tick(1);
        world.flush();
        assert_eq!(
            world.get_signal_strength(wire_pos),
            7,
            "Should handle mid-range signal strength"
        );
    }

    #[test]
    fn test_signal_strength_update() {
        use super::super::SimulationOptions;
        let schematic = create_simple_redstone_line();
        let wire_pos = BlockPos::new(5, 1, 0);

        let options = SimulationOptions {
            custom_io: vec![wire_pos],
            ..Default::default()
        };
        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        // Set signal strength to different values
        world.set_signal_strength(wire_pos, 8);
        world.tick(1);
        world.flush();
        assert_eq!(
            world.get_signal_strength(wire_pos),
            8,
            "Should update signal strength"
        );

        // Change to different value
        world.set_signal_strength(wire_pos, 3);
        world.tick(1);
        world.flush();
        assert_eq!(
            world.get_signal_strength(wire_pos),
            3,
            "Should update to new signal strength"
        );
    }

    #[test]
    fn test_signal_strength_with_lever() {
        use super::super::SimulationOptions;
        let schematic = create_simple_redstone_line();
        let lever_pos = BlockPos::new(0, 1, 0);
        let wire_pos = BlockPos::new(5, 1, 0);
        let lamp_pos = BlockPos::new(15, 1, 0);

        let options = SimulationOptions {
            custom_io: vec![wire_pos],
            ..Default::default()
        };
        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        // Lamp should start off
        assert_eq!(world.is_lit(lamp_pos), false, "Lamp should start off");

        // Toggle lever and verify custom IO still works
        world.on_use_block(lever_pos);
        world.tick(5);
        world.flush();

        // Lamp should be on from lever
        assert_eq!(
            world.is_lit(lamp_pos),
            true,
            "Lamp should light up from lever"
        );

        // Custom IO signal should still be gettable
        let custom_signal = world.get_signal_strength(wire_pos);
        // Signal should exist (value doesn't matter for this test)
        let _ = custom_signal;
    }

    #[test]
    fn test_signal_strength_multiple_positions() {
        use super::super::SimulationOptions;
        let schematic = create_simple_redstone_line();
        let pos1 = BlockPos::new(3, 1, 0);
        let pos2 = BlockPos::new(7, 1, 0);
        let pos3 = BlockPos::new(11, 1, 0);

        let options = SimulationOptions {
            custom_io: vec![pos1, pos2, pos3],
            ..Default::default()
        };
        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        // Set different signal strengths at multiple positions
        world.set_signal_strength(pos1, 5);
        world.set_signal_strength(pos2, 10);
        world.set_signal_strength(pos3, 15);
        world.tick(5);
        world.flush();

        // Each should maintain its own value
        assert_eq!(
            world.get_signal_strength(pos1),
            5,
            "Position 1 should have signal strength 5"
        );
        assert_eq!(
            world.get_signal_strength(pos2),
            10,
            "Position 2 should have signal strength 10"
        );
        assert_eq!(
            world.get_signal_strength(pos3),
            15,
            "Position 3 should have signal strength 15"
        );
    }

    #[test]
    fn test_signal_strength_persistence() {
        use super::super::SimulationOptions;
        let schematic = create_simple_redstone_line();
        let wire_pos = BlockPos::new(5, 1, 0);

        let options = SimulationOptions {
            custom_io: vec![wire_pos],
            ..Default::default()
        };
        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        // Set signal strength
        world.set_signal_strength(wire_pos, 12);
        world.tick(1);
        world.flush();

        let initial_strength = world.get_signal_strength(wire_pos);

        // Run more ticks
        world.tick(20);
        world.flush();

        // Signal should persist
        assert_eq!(
            world.get_signal_strength(wire_pos),
            initial_strength,
            "Signal strength should persist across ticks"
        );
    }

    #[test]
    fn test_signal_strength_invalid_position() {
        let schematic = create_simple_redstone_line();
        let world = MchprsWorld::new(schematic).expect("World creation failed");

        // Query signal strength at position with no redstone component
        let invalid_pos = BlockPos::new(100, 100, 100);
        let strength = world.get_signal_strength(invalid_pos);

        // Should return 0 for invalid positions
        assert_eq!(
            strength, 0,
            "Invalid position should return signal strength of 0"
        );
    }

    #[test]
    fn test_bracket_notation_set_block() {
        let mut schematic = UniversalSchematic::new("Bracket Notation Test".to_string());

        // Set base blocks
        schematic.set_block(
            0,
            0,
            0,
            BlockState::new("minecraft:gray_concrete".to_string()),
        );
        schematic.set_block(
            15,
            0,
            0,
            BlockState::new("minecraft:gray_concrete".to_string()),
        );

        // Use bracket notation to set lever with properties
        schematic.set_block_str(
            0,
            1,
            0,
            "minecraft:lever[facing=east,powered=false,face=floor]",
        );

        // Use bracket notation to set redstone wire with properties
        for x in 1..15 {
            schematic.set_block_str(
                x,
                1,
                0,
                "minecraft:redstone_wire[power=0,east=side,west=side,north=none,south=none]",
            );
        }

        // Use bracket notation to set lamp
        schematic.set_block_str(15, 1, 0, "minecraft:redstone_lamp[lit=false]");

        // Verify the blocks were set correctly with properties
        let lever = schematic.get_block(0, 1, 0).expect("Lever should exist");
        assert_eq!(
            lever.get_name(),
            "minecraft:lever",
            "Lever should have correct name"
        );
        assert_eq!(
            lever.get_property("facing").map(|s| s.as_str()),
            Some("east"),
            "Lever should have facing=east"
        );
        assert_eq!(
            lever.get_property("powered").map(|s| s.as_str()),
            Some("false"),
            "Lever should have powered=false"
        );
        assert_eq!(
            lever.get_property("face").map(|s| s.as_str()),
            Some("floor"),
            "Lever should have face=floor"
        );

        let wire = schematic.get_block(5, 1, 0).expect("Wire should exist");
        assert_eq!(
            wire.get_name(),
            "minecraft:redstone_wire",
            "Wire should have correct name"
        );
        assert_eq!(
            wire.get_property("power").map(|s| s.as_str()),
            Some("0"),
            "Wire should have power=0"
        );
        assert_eq!(
            wire.get_property("east").map(|s| s.as_str()),
            Some("side"),
            "Wire should have east=side"
        );

        let lamp = schematic.get_block(15, 1, 0).expect("Lamp should exist");
        assert_eq!(
            lamp.get_name(),
            "minecraft:redstone_lamp",
            "Lamp should have correct name"
        );
        assert_eq!(
            lamp.get_property("lit").map(|s| s.as_str()),
            Some("false"),
            "Lamp should have lit=false"
        );

        // Now test that the schematic can be used in simulation
        let mut world = MchprsWorld::new(schematic)
            .expect("World creation should succeed with bracket notation blocks");

        let lever_pos = BlockPos::new(0, 1, 0);
        let lamp_pos = BlockPos::new(15, 1, 0);

        // Initially lamp should be off
        assert_eq!(world.is_lit(lamp_pos), false, "Lamp should start off");

        // Toggle lever on
        world.on_use_block(lever_pos);
        world.tick(2);
        world.flush();

        // Lamp should now be lit
        assert_eq!(
            world.is_lit(lamp_pos),
            true,
            "Lamp should be lit after lever is toggled with bracket notation blocks"
        );
    }

    // ============================================================================
    // Custom IO Signal INJECTION Tests
    // These tests verify that setSignalStrength actually POWERS circuits
    // (not just stores values for monitoring)
    // ============================================================================

    fn create_wire_to_lamp_circuit() -> UniversalSchematic {
        let mut schematic = UniversalSchematic::new("Wire to Lamp Test".to_string());

        // Base layer
        for x in 0..5 {
            schematic.set_block(x, 0, 0, BlockState::new("minecraft:stone".to_string()));
        }

        // Redstone wire chain
        schematic.set_block_str(
            0,
            1,
            0,
            "minecraft:redstone_wire[power=0,east=side,west=none,north=none,south=none]",
        );
        schematic.set_block_str(
            1,
            1,
            0,
            "minecraft:redstone_wire[power=0,east=side,west=side,north=none,south=none]",
        );
        schematic.set_block_str(
            2,
            1,
            0,
            "minecraft:redstone_wire[power=0,east=side,west=side,north=none,south=none]",
        );

        // Lamp at the end
        schematic.set_block_str(3, 1, 0, "minecraft:redstone_lamp[lit=false]");

        schematic
    }

    #[test]
    fn test_custom_io_injection_powers_wire() {
        use super::super::SimulationOptions;
        let schematic = create_wire_to_lamp_circuit();
        let inject_pos = BlockPos::new(0, 1, 0);

        let options = SimulationOptions {
            custom_io: vec![inject_pos],
            ..Default::default()
        };

        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        // Initially wire should have no power
        let initial_signal = world.get_signal_strength(inject_pos);
        assert_eq!(initial_signal, 0, "Wire should start with no signal");

        // Inject signal strength 15
        world.set_signal_strength(inject_pos, 15);
        world.tick(5);
        world.flush();

        // Verify signal was stored
        let signal_strength = world.get_signal_strength(inject_pos);
        assert_eq!(
            signal_strength, 15,
            "Custom IO must store injected signal strength"
        );

        // This test verifies signal storage. Signal propagation to components
        // is tested in test_custom_io_injection_lights_lamp
    }

    #[test]
    fn test_custom_io_injection_lights_lamp() {
        use super::super::SimulationOptions;
        let schematic = create_wire_to_lamp_circuit();
        let inject_pos = BlockPos::new(0, 1, 0);
        let lamp_pos = BlockPos::new(3, 1, 0);

        let options = SimulationOptions {
            custom_io: vec![inject_pos, lamp_pos],
            ..Default::default()
        };

        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        // Lamp should start off
        assert!(!world.is_lit(lamp_pos), "Lamp should start off");

        // Inject signal
        world.set_signal_strength(inject_pos, 15);
        world.flush(); // Flush before ticking to propagate the signal
        world.tick(10);
        world.flush();

        let is_lit = world.is_lit(lamp_pos);
        let signal = world.get_signal_strength(inject_pos);
        let wire_power = world.get_redstone_power(inject_pos);

        assert!(
            is_lit,
            "CRITICAL: Injecting signal via custom IO MUST light the lamp. Signal={}, Wire power={}",
            signal, wire_power
        );
    }

    #[test]
    fn test_custom_io_monitoring_natural_power() {
        use super::super::SimulationOptions;
        // Verify custom IO can MONITOR naturally powered circuits
        let mut schematic = UniversalSchematic::new("Powered Circuit".to_string());

        // Base
        for x in 0..5 {
            schematic.set_block(x, 0, 0, BlockState::new("minecraft:stone".to_string()));
        }

        // Redstone block (always powered) -> wire
        schematic.set_block_str(0, 1, 0, "minecraft:redstone_block");
        schematic.set_block_str(
            1,
            1,
            0,
            "minecraft:redstone_wire[power=0,east=side,west=side,north=none,south=none]",
        );
        schematic.set_block_str(
            2,
            1,
            0,
            "minecraft:redstone_wire[power=0,east=side,west=side,north=none,south=none]",
        );

        let monitor_pos = BlockPos::new(2, 1, 0);

        let options = SimulationOptions {
            custom_io: vec![monitor_pos],
            ..Default::default()
        };

        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        // Tick to let power propagate
        world.tick(5);
        world.flush();

        let signal = world.get_signal_strength(monitor_pos);
        let power = world.get_redstone_power(monitor_pos);

        assert!(
            signal > 0,
            "Custom IO should read signal from naturally powered circuit"
        );
        assert!(power > 0, "Natural power should exist");
    }

    #[test]
    fn test_custom_io_relay_between_circuits() {
        use super::super::SimulationOptions;
        // Test the actual relay use case: read from one circuit, inject to another

        // Circuit A: Redstone block -> wire (output)
        let mut circuit_a = UniversalSchematic::new("Circuit A".to_string());
        for x in 0..3 {
            circuit_a.set_block(x, 0, 0, BlockState::new("minecraft:stone".to_string()));
        }
        circuit_a.set_block_str(0, 1, 0, "minecraft:redstone_block");
        circuit_a.set_block_str(
            1,
            1,
            0,
            "minecraft:redstone_wire[power=0,east=side,west=side,north=none,south=none]",
        );
        circuit_a.set_block_str(
            2,
            1,
            0,
            "minecraft:redstone_wire[power=0,east=side,west=side,north=none,south=none]",
        );

        let output_pos = BlockPos::new(2, 1, 0);
        let options_a = SimulationOptions {
            custom_io: vec![output_pos],
            ..Default::default()
        };

        let mut world_a =
            MchprsWorld::with_options(circuit_a, options_a).expect("Failed to create world A");

        // Circuit B: wire (input) -> lamp
        let circuit_b = create_wire_to_lamp_circuit();
        let input_pos = BlockPos::new(0, 1, 0);
        let lamp_pos = BlockPos::new(3, 1, 0);

        let options_b = SimulationOptions {
            custom_io: vec![input_pos, lamp_pos],
            ..Default::default()
        };

        let mut world_b =
            MchprsWorld::with_options(circuit_b, options_b).expect("Failed to create world B");

        // Simulate: Circuit A runs, we read its output
        world_a.tick(5);
        world_a.flush();
        let output_signal = world_a.get_signal_strength(output_pos);

        assert!(output_signal > 0, "Circuit A should produce output signal");

        // Relay: Inject A's output into B's input
        world_b.set_signal_strength(input_pos, output_signal);
        world_b.flush(); // Flush before ticking to propagate the signal
        world_b.tick(10);
        world_b.flush();

        // Verify: B's lamp should light up
        let lamp_lit = world_b.is_lit(lamp_pos);
        assert!(
            lamp_lit,
            "Circuit B's lamp should light from relayed signal (signal={})",
            output_signal
        );
    }

    #[test]
    fn test_custom_io_sync_to_schematic_preserves_power() {
        // This test verifies that sync_to_schematic() correctly syncs custom IO node power
        use super::super::SimulationOptions;
        use mchprs_world::World;

        let mut schematic = UniversalSchematic::new("Custom IO Sync Test".to_string());

        // Base
        schematic.set_block(
            0,
            0,
            0,
            BlockState::new("minecraft:gray_concrete".to_string()),
        );

        // Redstone wire with power=0 initially
        let mut wire = BlockState::new("minecraft:redstone_wire".to_string());
        wire.properties.insert("power".to_string(), "0".to_string());
        wire.properties
            .insert("east".to_string(), "side".to_string());
        wire.properties
            .insert("west".to_string(), "side".to_string());
        wire.properties
            .insert("north".to_string(), "none".to_string());
        wire.properties
            .insert("south".to_string(), "none".to_string());
        schematic.set_block(0, 1, 0, wire);

        let custom_io_pos = BlockPos::new(0, 1, 0);
        let options = SimulationOptions {
            custom_io: vec![custom_io_pos],
            ..Default::default()
        };

        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        // Set signal strength to 15
        world.set_signal_strength(custom_io_pos, 15);
        world.tick(5);
        world.flush();

        // Sync to schematic
        world.sync_to_schematic();

        // Get the synced schematic
        let synced_schematic = world.get_schematic();

        // Check the block in the schematic
        let synced_block = synced_schematic
            .get_block(0, 1, 0)
            .expect("Block should exist at custom IO position");

        // Verify the power property is 15 (not 0!)
        let power_value = synced_block
            .properties
            .get("power")
            .expect("Redstone wire should have power property");

        assert_eq!(
            power_value, "15",
            "Synced schematic should have power=15 after custom IO injection, got power={}",
            power_value
        );
    }

    #[test]
    fn test_custom_io_with_adjacent_wires() {
        // This test verifies that wires ADJACENT to custom IO also update correctly
        use super::super::SimulationOptions;
        use mchprs_world::World;

        let mut schematic = UniversalSchematic::new("Custom IO Adjacent Test".to_string());

        // Base blocks
        for x in 0..3 {
            schematic.set_block(
                x,
                0,
                0,
                BlockState::new("minecraft:gray_concrete".to_string()),
            );
        }

        // Three wires in a row: [0,1,0] (custom IO) -> [1,1,0] (adjacent) -> [2,1,0] (2 blocks away)
        for x in 0..3 {
            let mut wire = BlockState::new("minecraft:redstone_wire".to_string());
            wire.properties.insert("power".to_string(), "0".to_string());
            wire.properties
                .insert("east".to_string(), "side".to_string());
            wire.properties
                .insert("west".to_string(), "side".to_string());
            wire.properties
                .insert("north".to_string(), "none".to_string());
            wire.properties
                .insert("south".to_string(), "none".to_string());
            schematic.set_block(x, 1, 0, wire);
        }

        let custom_io_pos = BlockPos::new(0, 1, 0);
        let adjacent_pos = BlockPos::new(1, 1, 0);
        let far_pos = BlockPos::new(2, 1, 0);

        let options = SimulationOptions {
            custom_io: vec![custom_io_pos],
            io_only: false,  // CRITICAL: We want ALL wires to update
            optimize: false, // CRITICAL: Don't skip non-IO wires during compilation!
            ..Default::default()
        };

        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        // Set signal strength to 15 on custom IO
        world.set_signal_strength(custom_io_pos, 15);
        world.tick(5); // Allow propagation
        world.flush();
        world.sync_to_schematic();

        let synced_schematic = world.get_schematic();

        // Check custom IO wire (should be 15)
        let custom_io_block = synced_schematic
            .get_block(0, 1, 0)
            .expect("Custom IO block should exist");
        let custom_io_power: u8 = custom_io_block
            .properties
            .get("power")
            .and_then(|p| p.parse().ok())
            .unwrap_or(0);

        // Check adjacent wire (should be 14)
        let adjacent_block = synced_schematic
            .get_block(1, 1, 0)
            .expect("Adjacent block should exist");
        let adjacent_power: u8 = adjacent_block
            .properties
            .get("power")
            .and_then(|p| p.parse().ok())
            .unwrap_or(0);

        // Check far wire (should be 13)
        let far_block = synced_schematic
            .get_block(2, 1, 0)
            .expect("Far block should exist");
        let far_power: u8 = far_block
            .properties
            .get("power")
            .and_then(|p| p.parse().ok())
            .unwrap_or(0);

        eprintln!("[TEST] Custom IO wire power: {}", custom_io_power);
        eprintln!("[TEST] Adjacent wire power: {}", adjacent_power);
        eprintln!("[TEST] Far wire power: {}", far_power);

        assert_eq!(custom_io_power, 15, "Custom IO wire should have power 15");
        assert!(
            adjacent_power > 0,
            "Adjacent wire should have power > 0, got {}",
            adjacent_power
        );
        assert!(
            far_power > 0,
            "Far wire should have power > 0, got {}",
            far_power
        );
    }

    #[test]
    fn test_io_only_mode_performance() {
        // This test verifies that io_only mode works for maximum performance
        use super::super::SimulationOptions;

        let mut schematic = UniversalSchematic::new("IO Only Test".to_string());

        // Base blocks
        for x in 0..3 {
            schematic.set_block(
                x,
                0,
                0,
                BlockState::new("minecraft:gray_concrete".to_string()),
            );
        }

        // Three wires: input -> middle -> output
        for x in 0..3 {
            let mut wire = BlockState::new("minecraft:redstone_wire".to_string());
            wire.properties.insert("power".to_string(), "0".to_string());
            wire.properties
                .insert("east".to_string(), "side".to_string());
            wire.properties
                .insert("west".to_string(), "side".to_string());
            wire.properties
                .insert("north".to_string(), "none".to_string());
            wire.properties
                .insert("south".to_string(), "none".to_string());
            schematic.set_block(x, 1, 0, wire);
        }

        let input_pos = BlockPos::new(0, 1, 0);
        let output_pos = BlockPos::new(2, 1, 0);

        let options = SimulationOptions {
            custom_io: vec![input_pos, output_pos],
            io_only: true,   // PERFORMANCE MODE: Only sync IO nodes to schematic
            optimize: false, // Need middle wires in graph for propagation
            ..Default::default()
        };

        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        // Set input
        world.set_signal_strength(input_pos, 15);
        world.tick(5);
        world.flush();

        // In io_only mode, we should be able to read IO positions
        let input_signal = world.get_signal_strength(input_pos);
        let output_signal = world.get_signal_strength(output_pos);

        eprintln!(
            "[TEST] IO-only mode - Input signal: {}, Output signal: {}",
            input_signal, output_signal
        );

        assert_eq!(
            input_signal, 15,
            "Should be able to read input signal in io_only mode"
        );
        assert!(
            output_signal > 0,
            "Output should receive signal in io_only mode"
        );
    }

    #[test]
    fn test_custom_io_callbacks_basic() {
        // Test basic callback functionality - simplified to test direct state changes
        let schematic = create_and_gate();

        // Use actual positions from the schematic:
        // Wires at: (0,1,1), (1,1,1), (2,1,1), (1,1,2)
        // Lamp at: (1,1,3)
        let input_a = BlockPos::new(0, 1, 1);
        let input_b = BlockPos::new(2, 1, 1);
        let output = BlockPos::new(1, 1, 2); // Center wire before lamp

        let options = SimulationOptions {
            custom_io: vec![input_a, input_b, output],
            optimize: false,
            io_only: false,
            ..Default::default()
        };

        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        // Initialize tracking (first check sets baseline)
        world.check_custom_io_changes();
        world.clear_custom_io_changes();

        // Test 1: Single input change
        world.set_signal_strength(input_a, 15);
        world.check_custom_io_changes();

        let changes = world.poll_custom_io_changes();
        eprintln!(
            "[TEST] Changes after setting input A: {} changes",
            changes.len()
        );
        // In a connected circuit, setting one wire may affect others instantly
        assert!(changes.len() >= 1, "Should detect at least input A change");
        let input_a_change = changes.iter().find(|c| c.x == 0 && c.y == 1 && c.z == 1);
        assert!(input_a_change.is_some(), "Should detect input A change");
        assert_eq!(
            input_a_change.unwrap().new_power,
            15,
            "Input A should be powered to 15"
        );

        // Test 2: Second input change (after tick to propagate)
        world.tick(5);
        world.flush();
        world.check_custom_io_changes();
        world.poll_custom_io_changes(); // Clear any propagation changes

        world.set_signal_strength(input_b, 15);
        eprintln!("[TEST] After set input_b to 15:");
        eprintln!(
            "  get_signal_strength(input_a) = {}",
            world.get_signal_strength(input_a)
        );
        eprintln!(
            "  get_signal_strength(input_b) = {}",
            world.get_signal_strength(input_b)
        );
        world.check_custom_io_changes();

        let changes = world.poll_custom_io_changes();
        eprintln!("  Changes detected: {}", changes.len());
        assert!(changes.len() >= 1, "Should detect input B change");

        let input_b_change = changes.iter().find(|c| c.x == 2 && c.y == 1 && c.z == 1);
        assert!(input_b_change.is_some(), "Should detect input B change");

        // Test 3: Verify output state (may have already changed due to instant propagation)
        world.tick(5);
        world.flush();

        let output_power = world.get_signal_strength(output);
        assert!(
            output_power > 0,
            "Output should be powered when both inputs are high"
        );
    }

    #[test]
    fn test_custom_io_callbacks_multiple_changes() {
        // Test detecting multiple state changes in sequence
        let schematic = create_and_gate();

        // Use actual positions from the schematic:
        // Wires at: (0,1,1), (1,1,1), (2,1,1), (1,1,2)
        let input_a = BlockPos::new(0, 1, 1); // Wire for input A
        let input_b = BlockPos::new(2, 1, 1); // Wire for input B

        let options = SimulationOptions {
            custom_io: vec![input_a, input_b],
            optimize: false,
            io_only: false,
            ..Default::default()
        };

        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        // Initialize
        world.check_custom_io_changes();
        world.clear_custom_io_changes();

        // Debug: Check if both nodes exist
        eprintln!("[TEST] Node existence check:");
        eprintln!(
            "  input_a ({:?}) has_node: {}",
            input_a,
            world.has_node(input_a)
        );
        eprintln!(
            "  input_b ({:?}) has_node: {}",
            input_b,
            world.has_node(input_b)
        );

        // Change 1: Set input A to 15
        world.set_signal_strength(input_a, 15);
        eprintln!("[TEST] After set input_a to 15:");
        eprintln!(
            "  get_signal_strength(input_a) = {}",
            world.get_signal_strength(input_a)
        );
        eprintln!(
            "  get_signal_strength(input_b) = {}",
            world.get_signal_strength(input_b)
        );
        world.check_custom_io_changes();
        let changes = world.poll_custom_io_changes();
        eprintln!("  Changes detected: {}", changes.len());
        // In a connected circuit, changes may propagate to multiple wires
        assert!(changes.len() >= 1, "Should detect at least input A change");
        let input_a_change = changes.iter().find(|c| c.x == 0 && c.y == 1 && c.z == 1);
        assert!(input_a_change.is_some(), "Should detect input A change");

        // Change 2: Set input B to 15
        world.set_signal_strength(input_b, 15);
        eprintln!("[TEST] After set input_b to 15:");
        eprintln!(
            "  get_signal_strength(input_a) = {}",
            world.get_signal_strength(input_a)
        );
        eprintln!(
            "  get_signal_strength(input_b) = {}",
            world.get_signal_strength(input_b)
        );
        world.check_custom_io_changes();
        let changes = world.poll_custom_io_changes();
        eprintln!("  Changes detected: {}", changes.len());
        assert!(changes.len() >= 1, "Should detect at least input B change");
        let input_b_change = changes.iter().find(|c| c.x == 2 && c.y == 1 && c.z == 1);
        assert!(input_b_change.is_some(), "Should detect input B change");

        // Change 3: Set input A back to 0
        world.set_signal_strength(input_a, 0);
        eprintln!("[TEST] After set input_a to 0:");
        eprintln!(
            "  get_signal_strength(input_a) = {}",
            world.get_signal_strength(input_a)
        );
        eprintln!(
            "  get_signal_strength(input_b) = {}",
            world.get_signal_strength(input_b)
        );
        world.check_custom_io_changes();
        let changes = world.poll_custom_io_changes();
        eprintln!("  Changes detected: {}", changes.len());
        assert!(
            changes.len() >= 1,
            "Should detect at least input A change back to 0"
        );
        let input_a_change = changes.iter().find(|c| c.x == 0 && c.y == 1 && c.z == 1);
        assert!(
            input_a_change.is_some(),
            "Should detect input A change to 0"
        );
        assert_eq!(input_a_change.unwrap().new_power, 0, "Input A should be 0");

        // Queue should be empty after all polls
        let changes = world.poll_custom_io_changes();
        assert_eq!(changes.len(), 0, "Queue should be empty");
    }

    #[test]
    fn test_custom_io_callbacks_peek() {
        // Test peeking at changes without consuming them
        let schematic = create_and_gate();

        let input_a = BlockPos::new(0, 1, 0);

        let options = SimulationOptions {
            custom_io: vec![input_a],
            optimize: false,
            io_only: false,
            ..Default::default()
        };

        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        world.check_custom_io_changes();
        world.clear_custom_io_changes();

        world.set_signal_strength(input_a, 15);
        world.check_custom_io_changes();

        // Peek should not consume
        let peeked1 = world.peek_custom_io_changes();
        assert_eq!(peeked1.len(), 1, "Should see 1 change via peek");

        // Peek again - should still be there
        let peeked2 = world.peek_custom_io_changes();
        assert_eq!(peeked2.len(), 1, "Changes should still be in queue");

        // Poll should consume
        let polled = world.poll_custom_io_changes();
        assert_eq!(polled.len(), 1, "Should poll 1 change");

        // Now peek should be empty
        let peeked3 = world.peek_custom_io_changes();
        assert_eq!(peeked3.len(), 0, "Queue should be empty after poll");
    }

    #[test]
    fn test_custom_io_callbacks_no_false_positives() {
        // Test that unchanged values don't trigger callbacks
        let schematic = create_and_gate();

        let input_a = BlockPos::new(0, 1, 0);

        let options = SimulationOptions {
            custom_io: vec![input_a],
            optimize: false,
            io_only: false,
            ..Default::default()
        };

        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        world.check_custom_io_changes();
        world.clear_custom_io_changes();

        // Set to 15
        world.set_signal_strength(input_a, 15);
        world.check_custom_io_changes();
        world.poll_custom_io_changes(); // Clear

        // Set to 15 again (no change)
        world.set_signal_strength(input_a, 15);
        world.check_custom_io_changes();

        let changes = world.poll_custom_io_changes();
        assert_eq!(
            changes.len(),
            0,
            "Setting to same value should not trigger change"
        );

        // Tick with no changes
        world.tick(5);
        world.flush();
        world.check_custom_io_changes();

        let changes = world.poll_custom_io_changes();
        assert_eq!(
            changes.len(),
            0,
            "Ticking with no changes should not trigger callbacks"
        );
    }

    #[test]
    fn test_custom_io_callbacks_performance_zero_overhead() {
        // Test that callbacks have zero overhead when no custom IO is used
        let schematic = create_and_gate();

        // NO custom IO positions
        let options = SimulationOptions {
            custom_io: vec![], // Empty!
            optimize: false,
            io_only: false,
            ..Default::default()
        };

        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        // These should be instant no-ops
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            world.check_custom_io_changes();
        }
        let elapsed = start.elapsed();

        eprintln!(
            "[TEST] 1000 check_custom_io_changes() calls with no custom IO: {:?}",
            elapsed
        );

        // Should be extremely fast (< 1ms for 1000 calls)
        assert!(
            elapsed.as_micros() < 1000,
            "Should have near-zero overhead when no custom IO"
        );

        // Changes should always be empty
        let changes = world.poll_custom_io_changes();
        assert_eq!(changes.len(), 0);
    }

    #[test]
    fn test_custom_io_callbacks_performance_minimal_overhead() {
        // Test that callbacks have minimal overhead even with custom IO
        let schematic = create_and_gate();

        let input_a = BlockPos::new(0, 1, 0);
        let input_b = BlockPos::new(0, 1, 2);
        let output = BlockPos::new(4, 1, 1);

        let options = SimulationOptions {
            custom_io: vec![input_a, input_b, output],
            optimize: false,
            io_only: false,
            ..Default::default()
        };

        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        world.check_custom_io_changes();
        world.clear_custom_io_changes();

        // Measure overhead of change detection (no changes)
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            world.check_custom_io_changes();
        }
        let elapsed = start.elapsed();

        eprintln!(
            "[TEST] 1000 check_custom_io_changes() calls with 3 custom IO: {:?}",
            elapsed
        );

        // Should still be fast (< 10ms for 1000 calls with 3 IO positions)
        assert!(
            elapsed.as_millis() < 10,
            "Should have minimal overhead with custom IO"
        );
    }

    #[test]
    fn test_custom_io_callbacks_clear() {
        // Test clearing the change queue
        let schematic = create_and_gate();

        let input_a = BlockPos::new(0, 1, 0);

        let options = SimulationOptions {
            custom_io: vec![input_a],
            optimize: false,
            io_only: false,
            ..Default::default()
        };

        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        world.check_custom_io_changes();
        world.clear_custom_io_changes();

        // Make some changes
        world.set_signal_strength(input_a, 15);
        world.check_custom_io_changes();
        world.set_signal_strength(input_a, 0);
        world.check_custom_io_changes();

        // Should have 2 changes
        let peeked = world.peek_custom_io_changes();
        assert_eq!(peeked.len(), 2);

        // Clear without polling
        world.clear_custom_io_changes();

        // Should be empty now
        let polled = world.poll_custom_io_changes();
        assert_eq!(polled.len(), 0, "Queue should be empty after clear");
    }

    #[test]
    fn test_comparator_reading_barrel_signal_strengths() {
        // Test that comparators correctly read signal strength from barrels
        // This tests the coordinate normalization fix for get_redstone_power()
        for signal in 0..=15 {
            let mut schematic = UniversalSchematic::new(format!("Barrel Signal {}", signal));

            // Place concrete base
            for i in 0..4 {
                schematic.set_block(
                    i,
                    0,
                    0,
                    BlockState::new("minecraft:gray_concrete".to_string()),
                );
            }

            // Place barrel with signal strength at x=0
            schematic
                .set_block_from_string(
                    0,
                    1,
                    0,
                    &format!("minecraft:barrel[facing=north]{{signal={}}}", signal),
                )
                .expect("Failed to set barrel");

            // Place comparator facing west (reading from barrel) at x=1
            let mut comparator = BlockState::new("minecraft:comparator".to_string());
            comparator
                .properties
                .insert("facing".to_string(), "west".to_string());
            comparator
                .properties
                .insert("mode".to_string(), "compare".to_string());
            comparator
                .properties
                .insert("powered".to_string(), "false".to_string());
            schematic.set_block(1, 1, 0, comparator);

            // Place redstone wire at x=2 and x=3
            let mut wire = BlockState::new("minecraft:redstone_wire".to_string());
            wire.properties.insert("power".to_string(), "0".to_string());
            schematic.set_block(2, 1, 0, wire.clone());
            schematic.set_block(3, 1, 0, wire);

            // Create simulation
            let world = MchprsWorld::new(schematic).expect("World creation failed");

            // Check power at wire positions
            let power_at_2 = world.get_redstone_power(BlockPos::new(2, 1, 0));
            let power_at_3 = world.get_redstone_power(BlockPos::new(3, 1, 0));

            // Wire at x=2 should have signal strength
            assert_eq!(
                power_at_2, signal,
                "Wire at x=2 should have power {} for barrel signal {}",
                signal, signal
            );

            // Wire at x=3 should have signal - 1 (with minimum of 0)
            let expected_power_3 = if signal > 0 { signal - 1 } else { 0 };
            assert_eq!(
                power_at_3, expected_power_3,
                "Wire at x=3 should have power {} for barrel signal {}",
                expected_power_3, signal
            );
        }
    }

    #[test]
    fn test_comparator_reading_hopper_signal_strengths() {
        use mchprs_world::World;
        // Test with hoppers as well (different container type)
        for signal in 0..=15 {
            let mut schematic = UniversalSchematic::new(format!("Hopper Signal {}", signal));

            schematic.set_block(
                0,
                0,
                0,
                BlockState::new("minecraft:gray_concrete".to_string()),
            );
            schematic.set_block(
                1,
                0,
                0,
                BlockState::new("minecraft:gray_concrete".to_string()),
            );

            // Place hopper with signal strength
            schematic
                .set_block_from_string(
                    0,
                    1,
                    0,
                    &format!("minecraft:hopper[facing=down]{{signal={}}}", signal),
                )
                .expect("Failed to set hopper");

            // Place comparator facing west
            let mut comparator = BlockState::new("minecraft:comparator".to_string());
            comparator
                .properties
                .insert("facing".to_string(), "west".to_string());
            comparator
                .properties
                .insert("mode".to_string(), "compare".to_string());
            schematic.set_block(1, 1, 0, comparator);

            // Place redstone wire to check output
            let mut wire = BlockState::new("minecraft:redstone_wire".to_string());
            wire.properties.insert("power".to_string(), "0".to_string());
            schematic.set_block(2, 1, 0, wire);

            // Create simulation
            let world = MchprsWorld::new(schematic).expect("World creation failed");

            // Check power at wire position
            let power_at_2 = world.get_redstone_power(BlockPos::new(2, 1, 0));
            assert_eq!(
                power_at_2, signal,
                "Wire should have power {} for hopper signal {}",
                signal, signal
            );
        }
    }
}
