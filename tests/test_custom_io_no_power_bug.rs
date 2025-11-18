/// Test custom IO circuit with repeater - wires show power=0 even when they should be powered
///
/// Layout from user's failing case:
/// ```
/// # Base layer
/// cc
/// cc
/// cc
/// cc
/// cc
/// # Logic layer
/// ·│
/// ┌▲
/// ├┘
/// ↑·
/// │·
/// ```
///
/// Circuit flow:
/// [0,1,4]=INPUT (should be 15)
///    ↓
/// [0,1,3]=REPEATER (shows powered=true, good!)
///    ↓
/// [0,1,2]=WIRE (shows power=0, BUG!)
///    ↓
/// [1,1,2]=WIRE (back input, shows power=0, BUG!)
///    ↓
/// [1,1,1]=COMPARATOR
///    ↓
/// [1,1,0]=OUTPUT (shows power=0, BUG!)

#[cfg(feature = "simulation")]
use nucleation::{
    simulation::{
        typed_executor::{
            ExecutionMode, IoMapping, IoType, LayoutFunction, TypedCircuitExecutor, Value,
        },
        BlockPos, MchprsWorld,
    },
    UniversalSchematic,
};

#[cfg(feature = "simulation")]
use std::collections::HashMap;

#[cfg(feature = "simulation")]
#[test]
fn test_custom_io_with_repeater_no_power() {
    println!("\n=== CUSTOM IO + REPEATER BUG TEST ===");
    println!("Circuit: Input → Repeater → Wires → Comparator → Output");
    println!("BUG: All wires show power=0 even though repeater is powered!");

    let mut schematic = UniversalSchematic::new("repeater_circuit".to_string());

    // Base layer - 2x5 concrete
    for z in 0..5 {
        for x in 0..2 {
            schematic.set_block_str(x, 0, z, "minecraft:gray_concrete");
        }
    }

    // Logic layer - exact layout from user's circuit
    // Row 0 (z=0): ·│
    schematic.set_block_str(1, 1, 0, "minecraft:redstone_wire"); // OUTPUT

    // Row 1 (z=1): ┌▲
    schematic.set_block_str(0, 1, 1, "minecraft:redstone_wire"); // Side input to comparator
    schematic.set_block_str(1, 1, 1, "minecraft:comparator[facing=south,mode=subtract]");

    // Row 2 (z=2): ├┘
    schematic.set_block_str(0, 1, 2, "minecraft:redstone_wire");
    schematic.set_block_str(1, 1, 2, "minecraft:redstone_wire"); // Back input to comparator

    // Row 3 (z=3): ↑·
    schematic.set_block_str(0, 1, 3, "minecraft:repeater[facing=south,delay=1]");

    // Row 4 (z=4): │·
    schematic.set_block_str(0, 1, 4, "minecraft:redstone_wire"); // INPUT

    println!("\nCircuit layout:");
    println!("  [0,1,4] INPUT (wire)");
    println!("     ↓");
    println!("  [0,1,3] REPEATER facing south");
    println!("     ↓");
    println!("  [0,1,2] WIRE");
    println!("     ↓  ↘");
    println!("  [0,1,1] [1,1,2] (side & back)");
    println!("     ↓       ↓");
    println!("  [1,1,1] COMPARATOR");
    println!("     ↓");
    println!("  [1,1,0] OUTPUT (wire)");

    // Define IO
    let mut inputs = HashMap::new();
    inputs.insert(
        "a".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 4)], // Input at top
        },
    );

    let mut outputs = HashMap::new();
    outputs.insert(
        "out".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(1, 1, 0)], // Output at bottom
        },
    );

    let world = MchprsWorld::new(schematic).expect("Failed to create world");
    let mut executor = TypedCircuitExecutor::new(world, inputs, outputs);

    // Test with input ON
    println!("\n--- Test: Input a=ON ---");
    let mut input_values = HashMap::new();
    input_values.insert("a".to_string(), Value::Bool(true));

    let mode = ExecutionMode::FixedTicks { ticks: 50 };
    let result = executor
        .execute(input_values, mode)
        .expect("Execution failed");

    let world = executor.world();

    // Check all wire and component powers
    let power_input = world.get_signal_strength(BlockPos::new(0, 1, 4));
    let power_after_repeater = world.get_signal_strength(BlockPos::new(0, 1, 2));
    let power_comparator_back = world.get_signal_strength(BlockPos::new(1, 1, 2));
    let power_comparator_side = world.get_signal_strength(BlockPos::new(0, 1, 1));
    let power_output = world.get_signal_strength(BlockPos::new(1, 1, 0));

    println!("\nWire powers:");
    println!("  INPUT      [0,1,4]: {}", power_input);
    println!("  REPEATER   [0,1,3]: (component, can't check wire power)");
    println!("  AFTER_REP  [0,1,2]: {}", power_after_repeater);
    println!("  COMP_BACK  [1,1,2]: {}", power_comparator_back);
    println!("  COMP_SIDE  [0,1,1]: {}", power_comparator_side);
    println!("  OUTPUT     [1,1,0]: {}", power_output);

    // Get the typed output
    let output_value = result.outputs.get("out").unwrap();
    println!("\nTyped output 'out': {:?}", output_value);

    // Check for the bug
    let mut bugs_found = Vec::new();

    if power_input == 0 {
        bugs_found.push("INPUT wire has power=0 (should be 15)");
    }

    if power_after_repeater == 0 {
        bugs_found.push("Wire after repeater has power=0 (should be 15)");
    }

    if power_comparator_back == 0 {
        bugs_found.push("Comparator back input has power=0 (should be ~14)");
    }

    if power_comparator_side == 0 {
        bugs_found.push("Comparator side input has power=0 (should be ~13)");
    }

    if !bugs_found.is_empty() {
        println!("\n❌ BUGS CONFIRMED:");
        for bug in &bugs_found {
            println!("   - {}", bug);
        }
        println!("\n   This is the custom IO wire propagation bug!");
        println!("   Components (like repeaters) can't update custom IO wires.");
    }

    // The input wire should have power
    assert!(
        power_input > 0,
        "BUG: Input wire should have power > 0. Got {}",
        power_input
    );

    // The wire after the repeater should have power
    assert!(
        power_after_repeater > 0,
        "BUG: Wire after repeater should have power > 0. Got {}",
        power_after_repeater
    );
}

#[cfg(feature = "simulation")]
#[test]
fn test_custom_io_sync_wire_powers() {
    println!("\n=== TEST: Sync wire powers to schematic ===");

    let mut schematic = UniversalSchematic::new("sync_test".to_string());

    // Simple circuit: INPUT → WIRE → OUTPUT
    schematic.set_block_str(0, 0, 0, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 0, "minecraft:gray_concrete");
    schematic.set_block_str(2, 0, 0, "minecraft:gray_concrete");

    schematic.set_block_str(0, 1, 0, "minecraft:redstone_wire"); // INPUT
    schematic.set_block_str(1, 1, 0, "minecraft:redstone_wire"); // MIDDLE
    schematic.set_block_str(2, 1, 0, "minecraft:redstone_wire"); // OUTPUT

    let mut inputs = HashMap::new();
    inputs.insert(
        "in".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 0)],
        },
    );

    let mut outputs = HashMap::new();
    outputs.insert(
        "out".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(2, 1, 0)],
        },
    );

    let world = MchprsWorld::new(schematic).expect("Failed to create world");
    let mut executor = TypedCircuitExecutor::new(world, inputs, outputs);

    let mut input_values = HashMap::new();
    input_values.insert("in".to_string(), Value::Bool(true));

    let mode = ExecutionMode::FixedTicks { ticks: 20 };
    executor
        .execute(input_values, mode)
        .expect("Execution failed");

    // Check powers via get_signal_strength
    let world = executor.world();
    let power_in = world.get_signal_strength(BlockPos::new(0, 1, 0));
    let power_mid = world.get_signal_strength(BlockPos::new(1, 1, 0));
    let power_out = world.get_signal_strength(BlockPos::new(2, 1, 0));

    println!("Powers via get_signal_strength:");
    println!("  IN  [0,1,0]: {}", power_in);
    println!("  MID [1,1,0]: {}", power_mid);
    println!("  OUT [2,1,0]: {}", power_out);

    // Now sync and check schematic block states
    let schematic = executor.sync_and_get_schematic();

    // Get block states from schematic
    let block_in = schematic
        .get_block(0, 1, 0)
        .expect("Input block should exist");
    let block_mid = schematic
        .get_block(1, 1, 0)
        .expect("Middle block should exist");
    let block_out = schematic
        .get_block(2, 1, 0)
        .expect("Output block should exist");

    let power_in_synced: u8 = block_in
        .properties
        .get("power")
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);
    let power_mid_synced: u8 = block_mid
        .properties
        .get("power")
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);
    let power_out_synced: u8 = block_out
        .properties
        .get("power")
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);

    println!("\nPowers from synced schematic:");
    println!("  IN  [0,1,0]: {}", power_in_synced);
    println!("  MID [1,1,0]: {}", power_mid_synced);
    println!("  OUT [2,1,0]: {}", power_out_synced);

    // Check if sync matches get_signal_strength
    assert_eq!(power_in, power_in_synced, "Input power mismatch after sync");
    assert_eq!(
        power_mid, power_mid_synced,
        "Middle power mismatch after sync"
    );
    assert_eq!(
        power_out, power_out_synced,
        "Output power mismatch after sync"
    );

    println!("\n✅ Sync test passed - schematic powers match get_signal_strength");
}
