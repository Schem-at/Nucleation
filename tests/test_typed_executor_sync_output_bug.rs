/// Test to reproduce TypedCircuitExecutor not syncing output IO positions
///
/// Bug Description:
/// When using TypedCircuitExecutor.sync_and_get_schematic(), the output wire power levels
/// at IO positions are not being updated correctly. The executor returns the correct
/// output values (e.g., {out: true}), but after syncing, the output wire blocks
/// still show power=0.

#[cfg(feature = "simulation")]
#[test]
fn test_typed_executor_sync_output_io_position() {
    use nucleation::schematic_builder::SchematicBuilder;
    use nucleation::simulation::typed_executor::insign_io::create_executor_from_insign;
    use nucleation::simulation::typed_executor::{ExecutionMode, Value};
    use nucleation::UniversalSchematic;
    use std::collections::HashMap;

    println!("=== Testing TypedCircuitExecutor Output Sync Bug ===");

    // Build a simple NOT gate circuit using SchematicBuilder
    let template = "# Base layer\nc\nc\nc\n\n# Logic layer\n│\n▲\n│\n";

    let mut builder =
        SchematicBuilder::from_template(template).expect("Failed to create builder from template");

    let schematic = builder.build().expect("Failed to build schematic");

    println!("Built NOT gate circuit:");
    println!("  Dimensions: {:?}", schematic.get_dimensions());

    // Add Insign IO annotations
    let mut annotated = schematic.clone();

    // Input sign at [0, 2, 2]
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
        "{\"text\":\"#io.in:data_type=\\\"bool\\\"\"}".to_string(),
    );
    input_nbt.insert("Text4".to_string(), "{\"text\":\"\"}".to_string());

    annotated
        .set_block_with_nbt(0, 2, 2, "minecraft:oak_sign[rotation=0]", input_nbt)
        .unwrap();

    // Output sign at [0, 2, 0]
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
        "{\"text\":\"#io.out:data_type=\\\"bool\\\"\"}".to_string(),
    );
    output_nbt.insert("Text4".to_string(), "{\"text\":\"\"}".to_string());

    annotated
        .set_block_with_nbt(0, 2, 0, "minecraft:oak_sign[rotation=0]", output_nbt)
        .unwrap();

    println!("Added Insign annotations:");
    println!("  Input: io.in at [0, 2, 2] (wire at [0, 1, 2])");
    println!("  Output: io.out at [0, 2, 0] (wire at [0, 1, 0])");

    // Check initial block states
    println!("=== Initial Block States ===");
    print_block_state(&annotated, 0, 1, 0, "Output wire");
    print_block_state(&annotated, 0, 1, 1, "Repeater");
    print_block_state(&annotated, 0, 1, 2, "Input wire");

    // Create TypedCircuitExecutor
    let mut executor =
        create_executor_from_insign(&annotated).expect("Failed to create executor from Insign");

    println!("Created TypedCircuitExecutor");

    // Execute with input=true
    let mut inputs = HashMap::new();
    inputs.insert("in".to_string(), Value::Bool(true));

    println!("=== Executing with input=true ===");
    let result = executor
        .execute(inputs, ExecutionMode::FixedTicks { ticks: 50 })
        .expect("Execution failed");

    println!("Execution completed:");
    println!("  Ticks elapsed: {}", result.ticks_elapsed);
    println!("  Outputs: {:?}", result.outputs);

    // Check output value
    let output_value = result.outputs.get("out").expect("Output 'out' not found");

    assert!(
        matches!(output_value, Value::Bool(true)),
        "Expected output to be true, got {:?}",
        output_value
    );

    println!("✓ Executor correctly computed output=true");

    // Sync to schematic
    println!("=== Calling sync_and_get_schematic() ===");
    let synced_schematic = executor.sync_and_get_schematic();

    println!("sync_and_get_schematic() completed");

    // Check block states AFTER sync
    println!("=== Block States AFTER Sync ===");
    let output_wire_before = get_block_state(&annotated, 0, 1, 0);
    let output_wire_after = get_block_state(synced_schematic, 0, 1, 0);

    print_block_state(synced_schematic, 0, 1, 0, "Output wire");
    print_block_state(synced_schematic, 0, 1, 1, "Repeater");
    print_block_state(synced_schematic, 0, 1, 2, "Input wire");

    // Extract power level from output wire
    let output_power = extract_power_level(&output_wire_after);

    println!("=== Verification ===");
    println!("Output wire BEFORE sync: {}", output_wire_before);
    println!("Output wire AFTER sync:  {}", output_wire_after);
    println!("Extracted power level: {}", output_power);

    // THIS IS THE BUG: The output wire should have power=15, but it has power=0
    if output_power == 0 {
        println!("❌ BUG CONFIRMED: Output wire still has power=0 after sync!");
        println!("   Expected: power=15 (since output=true)");
        println!("   Actual:   power=0");
        println!("This confirms that TypedCircuitExecutor.sync_and_get_schematic()");
        println!("is NOT updating the block states at IO output positions.");
        panic!("TypedCircuitExecutor sync bug: output IO positions not updated");
    } else {
        println!("✓ Output wire correctly has power={}", output_power);
    }
}

// Helper function to print block state
fn print_block_state(
    schematic: &nucleation::UniversalSchematic,
    x: i32,
    y: i32,
    z: i32,
    label: &str,
) {
    let block_string = get_block_state(schematic, x, y, z);
    println!("  [{},{},{}] {}: {}", x, y, z, label, block_string);
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
