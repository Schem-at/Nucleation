/// Test comparator with diagonal wire feeding both back and side inputs
///
/// Layout (matching the user's failing case):
/// ```
/// # Base layer
/// ·c
/// cc
/// cc
/// # Logic layer
/// ·│
/// ┌▲
/// ├┴
/// ```
///
/// Input at [0,1,2] with power 15
/// -> [1,1,2] with power 14 (back input)
/// -> [0,1,1] with power 14 (side input)
/// Comparator at [1,1,1] in subtract mode should output: max(14-14, 0) = 0
/// But it outputs 14 instead (BUG!)

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
fn test_comparator_diagonal_equal_inputs() {
    println!("\n=== COMPARATOR DIAGONAL BUG TEST ===");
    println!("Circuit: Input wire diagonally connects to both back and side of comparator");

    let mut schematic = UniversalSchematic::new("diagonal_comparator".to_string());

    // Base layer - exactly matching the user's layout
    // ·c
    // cc
    // cc
    schematic.set_block_str(1, 0, 0, "minecraft:gray_concrete");
    schematic.set_block_str(0, 0, 1, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 1, "minecraft:gray_concrete");
    schematic.set_block_str(0, 0, 2, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 2, "minecraft:gray_concrete");

    // Logic layer - exactly matching the user's layout
    // ·│
    // ┌▲
    // ├┴
    schematic.set_block_str(1, 1, 0, "minecraft:redstone_wire"); // Output (│)
    schematic.set_block_str(0, 1, 1, "minecraft:redstone_wire"); // Side wire (┌)
    schematic.set_block_str(1, 1, 1, "minecraft:comparator[facing=south,mode=subtract]"); // Comparator (▲)
    schematic.set_block_str(0, 1, 2, "minecraft:redstone_wire"); // Input (├)
    schematic.set_block_str(1, 1, 2, "minecraft:redstone_wire"); // Back wire (┴)

    println!("\nLayout:");
    println!("  [0,1,2]=INPUT(15) ┐");
    println!("                    ├─ [1,1,2] (back=14)");
    println!("                    │");
    println!("                    └─ [0,1,1] (side=14)");
    println!("                           │");
    println!("                       [1,1,1]=COMPARATOR");
    println!("                           │");
    println!("                       [1,1,0]=OUTPUT");

    // Define IO
    let mut inputs = HashMap::new();
    inputs.insert(
        "a".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 2)], // Input at top-left
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

    // Check all wire powers
    let power_input = world.get_signal_strength(BlockPos::new(0, 1, 2));
    let power_back = world.get_signal_strength(BlockPos::new(1, 1, 2));
    let power_side = world.get_signal_strength(BlockPos::new(0, 1, 1));
    let power_output = world.get_signal_strength(BlockPos::new(1, 1, 0));

    println!("\nWire powers:");
    println!("  INPUT  [0,1,2]: {}", power_input);
    println!("  BACK   [1,1,2]: {}", power_back);
    println!("  SIDE   [0,1,1]: {}", power_side);
    println!("  OUTPUT [1,1,0]: {}", power_output);

    println!("\nComparator calculation:");
    println!("  back={}, side={}", power_back, power_side);
    println!(
        "  Expected: max({} - {}, 0) = {}",
        power_back,
        power_side,
        power_back.saturating_sub(power_side)
    );
    println!("  Actual output: {}", power_output);

    // Get the typed output
    let output_value = result.outputs.get("out").unwrap();
    println!("\nTyped output 'out': {:?}", output_value);

    // The bug: when back=14 and side=14, output should be 0 (OFF)
    if power_back == power_side && power_output > 0 {
        println!("\n❌ BUG CONFIRMED!");
        println!(
            "   Comparator has equal inputs (back={}, side={})",
            power_back, power_side
        );
        println!("   Output should be 0 but is {}", power_output);
        println!("   Comparator is not subtracting the side input!");
    }

    assert_eq!(power_back, 14, "Back input should be 14");
    assert_eq!(power_side, 14, "Side input should be 14");
    assert_eq!(
        power_output, 0,
        "BUG: Comparator output should be 0 when back=side=14. Got {}",
        power_output
    );
}

#[cfg(feature = "simulation")]
#[test]
fn test_comparator_diagonal_control() {
    println!("\n=== CONTROL: Comparator with back > side ===");

    let mut schematic = UniversalSchematic::new("diagonal_control".to_string());

    // Same layout but we'll use two separate inputs
    schematic.set_block_str(1, 0, 0, "minecraft:gray_concrete");
    schematic.set_block_str(0, 0, 1, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 1, "minecraft:gray_concrete");
    schematic.set_block_str(0, 0, 2, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 2, "minecraft:gray_concrete");

    schematic.set_block_str(1, 1, 0, "minecraft:redstone_wire");
    schematic.set_block_str(0, 1, 1, "minecraft:redstone_wire");
    schematic.set_block_str(1, 1, 1, "minecraft:comparator[facing=south,mode=subtract]");
    schematic.set_block_str(0, 1, 2, "minecraft:redstone_wire");
    schematic.set_block_str(1, 1, 2, "minecraft:redstone_wire");

    let mut inputs = HashMap::new();
    inputs.insert(
        "back_input".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(1, 1, 2)],
        },
    );
    inputs.insert(
        "side_input".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 1)],
        },
    );

    let mut outputs = HashMap::new();
    outputs.insert(
        "out".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(1, 1, 0)],
        },
    );

    let world = MchprsWorld::new(schematic).expect("Failed to create world");
    let mut executor = TypedCircuitExecutor::new(world, inputs, outputs);

    // Test: back=ON (15), side=OFF (0) → should output ON
    println!("\n--- Test: back=ON, side=OFF ---");
    let mut input_values = HashMap::new();
    input_values.insert("back_input".to_string(), Value::Bool(true));
    input_values.insert("side_input".to_string(), Value::Bool(false));

    let mode = ExecutionMode::FixedTicks { ticks: 50 };
    let result = executor
        .execute(input_values, mode)
        .expect("Execution failed");

    let world = executor.world();
    let power_output = world.get_signal_strength(BlockPos::new(1, 1, 0));

    println!("Output power: {} (expected > 0)", power_output);

    let output_value = result.outputs.get("out").unwrap();
    println!("Typed output: {:?}", output_value);

    assert!(power_output > 0, "Control: back > side should output ON");
    println!("✅ Control test passed");
}
