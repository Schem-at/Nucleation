/// Test for bug where two custom IO inputs without an output node causes power loss
///
/// Bug report: When a circuit has:
/// - Two custom IO inputs
/// - No custom IO outputs
/// The redstone wire power doesn't propagate correctly.
///
/// Adding an output node "fixes" the bug, which suggests a compilation issue in MCHPRS.
///
/// Circuit layout:
/// ```
/// [0,1,2] [1,1,1] [2,1,1] [3,1,1] [3,1,2]
///    │       │       │       │       │
///   IN1  ━━━━┼━━━━━━┼━━━━━━┼━━━━━  IN2
///    │       │       │       │       │
///  (OFF)  [1,1,1]  [2,1,1] [3,1,1] (ON)
///    │                               
/// [0,1,1]                            
///    │                               
/// [0,1,0]  <- This should be powered (but isn't without output node)
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
fn test_two_inputs_no_output_power_propagation() {
    println!("\n=== Bug Test: Two Inputs, No Output - Power Propagation ===\n");

    // Build the exact schematic from the bug report
    let mut schematic = UniversalSchematic::new("two_inputs_no_output".to_string());

    // Base layer (Y=0) - 4x3 concrete platform
    for z in 0..3 {
        for x in 0..4 {
            schematic.set_block_str(x, 0, z, "minecraft:gray_concrete");
        }
    }

    // Logic layer (Y=1) - Wire chain from right to left
    // Right side: input b at [3,1,2], wire chain at [3,1,1], [2,1,1], [1,1,1]
    schematic.set_block_str(3, 1, 2, "minecraft:redstone_wire"); // Input b (will be ON)
    schematic.set_block_str(3, 1, 1, "minecraft:redstone_wire");
    schematic.set_block_str(2, 1, 1, "minecraft:redstone_wire");
    schematic.set_block_str(1, 1, 1, "minecraft:redstone_wire");

    // Left side: connects to the wire chain
    schematic.set_block_str(0, 1, 1, "minecraft:redstone_wire");
    schematic.set_block_str(0, 1, 0, "minecraft:redstone_wire"); // This should be powered!

    // Left top: input region_2 at [0,1,2] (will be OFF)
    schematic.set_block_str(0, 1, 2, "minecraft:redstone_wire"); // Input region_2 (will be OFF)

    println!("Schematic layout:");
    println!("  [0,1,2] = Input region_2 (OFF)");
    println!("  [0,1,1] = Wire");
    println!("  [0,1,0] = Wire (should be powered)");
    println!("  [1,1,1] = Wire");
    println!("  [2,1,1] = Wire");
    println!("  [3,1,1] = Wire");
    println!("  [3,1,2] = Input b (ON)\n");

    // Test WITHOUT output node
    println!("=== Test 1: Two inputs, NO output node ===\n");

    let mut inputs = HashMap::new();
    inputs.insert(
        "b".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(3, 1, 2)],
        },
    );
    inputs.insert(
        "region_2".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 2)],
        },
    );

    // NO output nodes
    let outputs = HashMap::new();

    let world = MchprsWorld::new(schematic.clone()).expect("Failed to create world");
    let mut executor = TypedCircuitExecutor::new(world, inputs.clone(), outputs);

    let mut input_values = HashMap::new();
    input_values.insert("b".to_string(), Value::Bool(true));
    input_values.insert("region_2".to_string(), Value::Bool(false));

    let mode = ExecutionMode::FixedTicks { ticks: 50 };
    let result = executor
        .execute(input_values, mode.clone())
        .expect("Execution failed");

    println!("Execution completed in {} ticks", result.ticks_elapsed);

    // Get the actual wire powers
    let world = executor.world();
    let power_b = world.get_signal_strength(BlockPos::new(3, 1, 2));
    let power_region_2 = world.get_signal_strength(BlockPos::new(0, 1, 2));
    let power_3_1_1 = world.get_signal_strength(BlockPos::new(3, 1, 1));
    let power_2_1_1 = world.get_signal_strength(BlockPos::new(2, 1, 1));
    let power_1_1_1 = world.get_signal_strength(BlockPos::new(1, 1, 1));
    let power_0_1_1 = world.get_signal_strength(BlockPos::new(0, 1, 1));
    let power_0_1_0 = world.get_signal_strength(BlockPos::new(0, 1, 0));

    println!("\nWire powers WITHOUT output node:");
    println!("  [3,1,2] b (input):         power={}", power_b);
    println!("  [3,1,1]:                   power={}", power_3_1_1);
    println!("  [2,1,1]:                   power={}", power_2_1_1);
    println!("  [1,1,1]:                   power={}", power_1_1_1);
    println!("  [0,1,1]:                   power={}", power_0_1_1);
    println!("  [0,1,0] (should be ~10):   power={}", power_0_1_0);
    println!("  [0,1,2] region_2 (input):  power={}", power_region_2);

    let power_without_output = power_0_1_0;

    if power_0_1_0 == 0 {
        println!("\n❌ BUG CONFIRMED: [0,1,0] has power=0 (expected ~10)");
        println!("   The wire chain is not propagating power correctly!");
    }

    // Test WITH output node
    println!("\n=== Test 2: Two inputs, WITH output node ===\n");

    let mut outputs = HashMap::new();
    outputs.insert(
        "region_3".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 0)], // Output at the problematic position
        },
    );

    let world = MchprsWorld::new(schematic.clone()).expect("Failed to create world");
    let mut executor = TypedCircuitExecutor::new(world, inputs, outputs);

    let mut input_values = HashMap::new();
    input_values.insert("b".to_string(), Value::Bool(true));
    input_values.insert("region_2".to_string(), Value::Bool(false));

    let result = executor
        .execute(input_values, mode)
        .expect("Execution failed");

    println!("Execution completed in {} ticks", result.ticks_elapsed);

    // Get wire powers
    let world = executor.world();
    let power_b = world.get_signal_strength(BlockPos::new(3, 1, 2));
    let power_region_2 = world.get_signal_strength(BlockPos::new(0, 1, 2));
    let power_3_1_1 = world.get_signal_strength(BlockPos::new(3, 1, 1));
    let power_2_1_1 = world.get_signal_strength(BlockPos::new(2, 1, 1));
    let power_1_1_1 = world.get_signal_strength(BlockPos::new(1, 1, 1));
    let power_0_1_1 = world.get_signal_strength(BlockPos::new(0, 1, 1));
    let power_0_1_0 = world.get_signal_strength(BlockPos::new(0, 1, 0));

    println!("\nWire powers WITH output node:");
    println!("  [3,1,2] b (input):         power={}", power_b);
    println!("  [3,1,1]:                   power={}", power_3_1_1);
    println!("  [2,1,1]:                   power={}", power_2_1_1);
    println!("  [1,1,1]:                   power={}", power_1_1_1);
    println!("  [0,1,1]:                   power={}", power_0_1_1);
    println!("  [0,1,0] (should be ~10):   power={}", power_0_1_0);
    println!("  [0,1,2] region_2 (input):  power={}", power_region_2);

    // Get the output value
    let output = result.outputs.get("region_3").expect("Missing output");
    println!("\nOutput region_3: {:?}", output);

    if power_0_1_0 > 0 {
        println!(
            "\n✅ With output node: [0,1,0] has power={} (correct)",
            power_0_1_0
        );
    }

    // The bug: Without output node, power should still propagate
    // This assertion should fail when the bug is present
    assert!(
        power_without_output > 0,
        "BUG: Wire at [0,1,0] should be powered even WITHOUT an output node. \
         Without output: power={}, With output: power={}. \
         Power chain: [3,1,2](15) → [3,1,1] → [2,1,1] → [1,1,1] → [0,1,1] → [0,1,0]",
        power_without_output,
        power_0_1_0
    );
}

#[cfg(feature = "simulation")]
#[test]
fn test_single_input_no_output_works() {
    println!("\n=== Control Test: Single Input, No Output ===\n");

    let mut schematic = UniversalSchematic::new("single_input_no_output".to_string());

    // Base layer
    for z in 0..3 {
        for x in 0..4 {
            schematic.set_block_str(x, 0, z, "minecraft:gray_concrete");
        }
    }

    // Simple wire chain with only ONE input
    schematic.set_block_str(3, 1, 2, "minecraft:redstone_wire"); // Input (ON)
    schematic.set_block_str(3, 1, 1, "minecraft:redstone_wire");
    schematic.set_block_str(2, 1, 1, "minecraft:redstone_wire");
    schematic.set_block_str(1, 1, 1, "minecraft:redstone_wire");
    schematic.set_block_str(0, 1, 1, "minecraft:redstone_wire");
    schematic.set_block_str(0, 1, 0, "minecraft:redstone_wire");

    let mut inputs = HashMap::new();
    inputs.insert(
        "b".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(3, 1, 2)],
        },
    );

    // NO output nodes
    let outputs = HashMap::new();

    let world = MchprsWorld::new(schematic).expect("Failed to create world");
    let mut executor = TypedCircuitExecutor::new(world, inputs, outputs);

    let mut input_values = HashMap::new();
    input_values.insert("b".to_string(), Value::Bool(true));

    let mode = ExecutionMode::FixedTicks { ticks: 50 };
    executor
        .execute(input_values, mode)
        .expect("Execution failed");

    let world = executor.world();
    let power_0_1_0 = world.get_signal_strength(BlockPos::new(0, 1, 0));

    println!("Wire power at [0,1,0]: {}", power_0_1_0);

    if power_0_1_0 > 0 {
        println!("✅ Single input works correctly (power={})", power_0_1_0);
    } else {
        println!("❌ Even single input fails! (power=0)");
    }

    assert!(
        power_0_1_0 > 0,
        "Control test: Single input should work. Power at [0,1,0] = {}",
        power_0_1_0
    );
}
