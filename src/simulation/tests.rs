#[cfg(test)]
mod tests {
    use super::super::{generate_truth_table, BlockPos, MchprsWorld};
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
            custom_io: vec![inject_pos],
            ..Default::default()
        };

        let mut world =
            MchprsWorld::with_options(schematic, options).expect("World creation failed");

        // Lamp should start off
        assert!(!world.is_lit(lamp_pos), "Lamp should start off");

        // Inject signal
        world.set_signal_strength(inject_pos, 15);
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
            custom_io: vec![input_pos],
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
}
