/// Exact reproduction of the failing topology from the browser test
///
/// Layout (exact coordinates):
/// ```
/// [0,1,2]=INPUT_A  [0,1,1]  [0,1,0]=TARGET
///                    │
///              [1,1,1]
///                │
///              [2,1,1]
///                │
///              [3,1,1]
///                │
///           [3,1,2]=INPUT_B
/// ```

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
fn test_exact_failing_topology() {
    println!("\n=== EXACT TOPOLOGY TEST: Matching original failing case ===");

    let mut schematic = UniversalSchematic::new("exact_topology".to_string());

    // Base layer
    for z in 0..3 {
        for x in 0..4 {
            schematic.set_block_str(x, 0, z, "minecraft:gray_concrete");
        }
    }

    // Logic layer - exact coordinates from failing case
    schematic.set_block_str(0, 1, 2, "minecraft:redstone_wire"); // INPUT_A (top-left)
    schematic.set_block_str(0, 1, 1, "minecraft:redstone_wire");
    schematic.set_block_str(0, 1, 0, "minecraft:redstone_wire"); // TARGET (bottom-left) - THIS FAILS
    schematic.set_block_str(1, 1, 1, "minecraft:redstone_wire");
    schematic.set_block_str(2, 1, 1, "minecraft:redstone_wire");
    schematic.set_block_str(3, 1, 1, "minecraft:redstone_wire");
    schematic.set_block_str(3, 1, 2, "minecraft:redstone_wire"); // INPUT_B (top-right)

    println!("Exact layout from failing case:");
    println!("  [0,1,2]=INPUT_A  [0,1,1]  [0,1,0]=TARGET");
    println!("                      │");
    println!("                 [1,1,1]");
    println!("                    │");
    println!("                 [2,1,1]");
    println!("                    │");
    println!("                 [3,1,1]");
    println!("                    │");
    println!("              [3,1,2]=INPUT_B");

    // Test WITHOUT output
    println!("\n--- Test: INPUT_A=OFF, INPUT_B=ON, NO output ---");

    let mut inputs = HashMap::new();
    inputs.insert(
        "region_2".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 2)],
        },
    );
    inputs.insert(
        "b".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(3, 1, 2)],
        },
    );

    let outputs = HashMap::new(); // NO outputs - this is when it fails

    let world = MchprsWorld::new(schematic.clone()).expect("Failed to create world");
    let mut executor = TypedCircuitExecutor::new(world, inputs.clone(), outputs);

    let mut input_values = HashMap::new();
    input_values.insert("region_2".to_string(), Value::Bool(false)); // INPUT_A = OFF
    input_values.insert("b".to_string(), Value::Bool(true)); // INPUT_B = ON

    let mode = ExecutionMode::FixedTicks { ticks: 50 };
    executor
        .execute(input_values, mode)
        .expect("Execution failed");

    let world = executor.world();
    let power_a = world.get_signal_strength(BlockPos::new(0, 1, 2));
    let power_0_1_1 = world.get_signal_strength(BlockPos::new(0, 1, 1));
    let power_target = world.get_signal_strength(BlockPos::new(0, 1, 0));
    let power_1_1_1 = world.get_signal_strength(BlockPos::new(1, 1, 1));
    let power_2_1_1 = world.get_signal_strength(BlockPos::new(2, 1, 1));
    let power_3_1_1 = world.get_signal_strength(BlockPos::new(3, 1, 1));
    let power_b = world.get_signal_strength(BlockPos::new(3, 1, 2));

    println!("\nWire powers:");
    println!("  INPUT_A  [0,1,2]: {}", power_a);
    println!("           [0,1,1]: {}", power_0_1_1);
    println!("  TARGET   [0,1,0]: {}", power_target);
    println!("           [1,1,1]: {}", power_1_1_1);
    println!("           [2,1,1]: {}", power_2_1_1);
    println!("           [3,1,1]: {}", power_3_1_1);
    println!("  INPUT_B  [3,1,2]: {}", power_b);

    println!("\nExpected TARGET [0,1,0]: ~10");
    println!("Actual TARGET: {}", power_target);

    if power_target == 0 {
        println!("\n❌ BUG REPRODUCED!");
        println!("   Exact topology from original failing case!");
        println!("   TARGET wire [0,1,0] has power=0 (expected ~10)");
    } else {
        println!("\n✅ This topology works (power={})", power_target);
    }

    assert!(
        power_target > 0,
        "BUG: TARGET wire [0,1,0] should be powered. Got power={}, expected ~10",
        power_target
    );
}
