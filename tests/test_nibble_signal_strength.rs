/// Test nibble/signal_strength data type for Packed4 encoding
///
/// This test verifies that the "nibble", "signal", and "signal_strength" data types
/// work correctly with TypedCircuitExecutor, allowing 4-bit values (0-15) to be
/// encoded on a single wire using Packed4 layout.

#[cfg(feature = "simulation")]
#[test]
fn test_nibble_data_type_signal_strength() {
    use nucleation::schematic_builder::SchematicBuilder;
    use nucleation::simulation::typed_executor::insign_io::create_executor_from_insign;
    use nucleation::simulation::typed_executor::{ExecutionMode, Value};
    use std::collections::HashMap;

    println!("=== Testing Nibble/Signal Strength Data Type ===");

    // Build a simple NOT gate circuit: input -> repeater -> output
    let template = "# Base layer\nc\nc\nc\n\n# Logic layer\n│\n▲\n│\n";

    let builder =
        SchematicBuilder::from_template(template).expect("Failed to create builder from template");

    let schematic = builder.build().expect("Failed to build schematic");

    println!("Built NOT gate circuit:");
    println!("  Dimensions: {:?}", schematic.get_dimensions());

    // Add Insign IO annotations with nibble data type
    let mut annotated = schematic.clone();

    // Input sign at [0, 2, 2] - references wire below at [0, 1, 2]
    let mut input_nbt = HashMap::new();
    input_nbt.insert(
        "Text1".to_string(),
        "{\"text\":\"@io.in=rc([0,-1,0],[0,-1,0])\"}".to_string(),
    );
    input_nbt.insert(
        "Text2".to_string(),
        "{\"text\":\"#io.in:type=\\\"input\\\"\"}".to_string(),
    );
    input_nbt.insert(
        "Text3".to_string(),
        "{\"text\":\"#io.in:data_type=\\\"nibble\\\"\"}".to_string(),
    );
    input_nbt.insert("Text4".to_string(), "{\"text\":\"\"}".to_string());

    annotated
        .set_block_with_nbt(0, 2, 2, "minecraft:oak_sign[rotation=0]", input_nbt)
        .unwrap();

    // Output sign at [0, 2, 0] - references wire below at [0, 1, 0]
    let mut output_nbt = HashMap::new();
    output_nbt.insert(
        "Text1".to_string(),
        "{\"text\":\"@io.out=rc([0,-1,0],[0,-1,0])\"}".to_string(),
    );
    output_nbt.insert(
        "Text2".to_string(),
        "{\"text\":\"#io.out:type=\\\"output\\\"\"}".to_string(),
    );
    output_nbt.insert(
        "Text3".to_string(),
        "{\"text\":\"#io.out:data_type=\\\"signal_strength\\\"\"}".to_string(),
    );
    output_nbt.insert("Text4".to_string(), "{\"text\":\"\"}".to_string());

    annotated
        .set_block_with_nbt(0, 2, 0, "minecraft:oak_sign[rotation=0]", output_nbt)
        .unwrap();

    println!("Added Insign annotations:");
    println!("  Input: io.in (nibble) at sign [0, 2, 2], wire at [0, 1, 2]");
    println!("  Output: io.out (signal_strength) at sign [0, 2, 0], wire at [0, 1, 0]");

    // Create TypedCircuitExecutor
    let mut executor =
        create_executor_from_insign(&annotated).expect("Failed to create executor from Insign");

    println!("✓ Created TypedCircuitExecutor with nibble data type");

    // Test different signal strength values
    let test_values = vec![0, 7, 15, 3, 12];

    for test_value in test_values {
        println!("\n=== Testing signal strength: {} ===", test_value);

        // Execute with input value
        let mut inputs = HashMap::new();
        inputs.insert("in".to_string(), Value::U32(test_value));

        let result = executor
            .execute(inputs, ExecutionMode::FixedTicks { ticks: 20 })
            .expect("Execution failed");

        println!("Execution completed:");
        println!("  Ticks elapsed: {}", result.ticks_elapsed);
        println!("  Outputs: {:?}", result.outputs);

        // Check output value
        let output_value = result.outputs.get("out").expect("Output 'out' not found");

        if let Value::U32(output_val) = output_value {
            assert_eq!(
                *output_val, test_value,
                "Expected output to be {}, got {}",
                test_value, output_val
            );
            println!("✓ Output correctly matches input: {}", output_val);
        } else {
            panic!("Expected U32 output, got {:?}", output_value);
        }

        // Sync to schematic
        let synced_schematic = executor.sync_and_get_schematic();

        // Check wire power levels
        let input_wire = get_block_state(synced_schematic, 0, 1, 2);
        let output_wire = get_block_state(synced_schematic, 0, 1, 0);

        println!("Block states after sync:");
        println!("  Input wire:  {}", input_wire);
        println!("  Output wire: {}", output_wire);

        let output_power = extract_power_level(&output_wire);

        // For passthrough, output power should match input
        assert_eq!(
            output_power, test_value as u8,
            "Expected output wire power={}, got power={}",
            test_value, output_power
        );

        println!("✓ Output wire power level correct: {}", output_power);
    }

    println!("\n=== All Signal Strength Tests Passed ===");
}

// Helper function to get block state
fn get_block_state(schematic: &nucleation::UniversalSchematic, x: i32, y: i32, z: i32) -> String {
    if let Some(block) = schematic.get_block(x, y, z) {
        block.to_string()
    } else {
        "air".to_string()
    }
}

// Helper function to extract power level from redstone wire block string
fn extract_power_level(block_string: &str) -> u8 {
    if !block_string.contains("redstone_wire") {
        return 0;
    }

    // Extract power=N from the block string
    if let Some(start) = block_string.find("power=") {
        let power_str = &block_string[start + 6..];
        if let Some(end) = power_str.find(|c: char| !c.is_ascii_digit()) {
            if let Ok(power) = power_str[..end].parse::<u8>() {
                return power;
            }
        }
    }

    0
}
