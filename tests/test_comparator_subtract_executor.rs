/// Test comparator subtract mode bug using TypedCircuitExecutor
/// This matches the browser test setup exactly

#[cfg(feature = "simulation")]
use nucleation::{
    simulation::{
        typed_executor::{
            ExecutionMode, IoMapping, IoType, LayoutFunction, TypedCircuitExecutor, Value,
        },
        MchprsWorld,
    },
    UniversalSchematic,
};

#[cfg(feature = "simulation")]
use std::collections::HashMap;

#[cfg(feature = "simulation")]
#[test]
fn test_comparator_subtract_executor() {
    println!("\n=== Comparator Subtract Mode Bug - Using TypedCircuitExecutor ===\n");

    let mut schematic = UniversalSchematic::new("custom_circuit".to_string());

    // Base layer (Y=0) - matching browser test
    schematic.set_block_str(0, 0, 0, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 0, "minecraft:gray_concrete");
    schematic.set_block_str(0, 0, 1, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 1, "minecraft:gray_concrete");
    schematic.set_block_str(0, 0, 2, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 2, "minecraft:gray_concrete");

    // Logic layer (Y=1) - exact coordinates from browser test
    // [0,1,0] = back input wire (north of comparator)
    // [0,1,1] = comparator facing south
    // [1,1,1] = side input wire (east of comparator)
    // [0,1,2] = output wire (south of comparator)
    schematic.set_block_str(0, 1, 0, "minecraft:redstone_wire");
    schematic.set_block_str(0, 1, 1, "minecraft:comparator[facing=south,mode=subtract]");
    schematic.set_block_str(1, 1, 1, "minecraft:redstone_wire");
    schematic.set_block_str(0, 1, 2, "minecraft:redstone_wire");

    println!("Schematic created with exact browser test layout");
    println!("  [0,1,0] = output wire (io.out)");
    println!("  [0,1,1] = comparator[facing=south,mode=subtract]");
    println!("  [1,1,1] = side input wire (io.side)");
    println!("  [0,1,2] = back input wire (io.back)\n");

    // Create IO mappings matching browser test EXACT coordinates
    let mut inputs = HashMap::new();
    inputs.insert(
        "back".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 2)], // Browser test: io.back at [0,1,2]
        },
    );
    inputs.insert(
        "side".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(1, 1, 1)], // Browser test: io.side at [1,1,1]
        },
    );

    let mut outputs = HashMap::new();
    outputs.insert(
        "out".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 0)], // Browser test: io.out at [0,1,0]
        },
    );

    // Create world and executor
    let world = MchprsWorld::new(schematic.clone()).expect("Failed to create world");
    let mut executor = TypedCircuitExecutor::new(world, inputs, outputs);

    println!("TypedCircuitExecutor created\n");

    // Test 1: back=true, side=false (should output true)
    println!("=== Test 1: back=true, side=false ===");
    let mut input_values = HashMap::new();
    input_values.insert("back".to_string(), Value::Bool(true));
    input_values.insert("side".to_string(), Value::Bool(false));

    let mode = ExecutionMode::FixedTicks { ticks: 50 };
    let result = executor
        .execute(input_values, mode.clone())
        .expect("Execution failed");

    println!("Inputs: back=true, side=false");
    println!("Ticks elapsed: {}", result.ticks_elapsed);

    let output1 = result.outputs.get("out").expect("Missing output");
    println!("Output: {:?}", output1);

    let output1_val = match output1 {
        Value::Bool(val) => *val,
        _ => panic!("Output is not a bool"),
    };

    if output1_val {
        println!("✅ PASS: Output is true (expected true)\n");
    } else {
        println!("❌ FAIL: Output is false (expected true)");
        println!("   This is UNEXPECTED - browser test shows this working!\n");
    }

    // Test 2: back=true, side=true (should output false - THIS IS THE BUG)
    println!("=== Test 2: back=true, side=true ===");
    let mut input_values = HashMap::new();
    input_values.insert("back".to_string(), Value::Bool(true));
    input_values.insert("side".to_string(), Value::Bool(true));

    let result = executor
        .execute(input_values, mode)
        .expect("Execution failed");

    println!("Inputs: back=true, side=true");
    println!("Ticks elapsed: {}", result.ticks_elapsed);

    let output2 = result.outputs.get("out").expect("Missing output");
    println!("Output: {:?}", output2);

    let output2_val = match output2 {
        Value::Bool(val) => *val,
        _ => panic!("Output is not a bool"),
    };

    if !output2_val {
        println!("✅ PASS: Output is false (correct)\n");
    } else {
        println!("❌ BUG CONFIRMED!");
        println!("   Comparator subtract mode: output = max(rear - side, 0)");
        println!("   With rear=15, side=15: output = max(15-15, 0) = 0");
        println!("   Expected: false (power 0)");
        println!("   Actual: true (power > 0)");
        println!("   The comparator is IGNORING the side input!\n");
        panic!("BUG: Comparator ignores side input in subtract mode");
    }
}

#[cfg(feature = "simulation")]
#[test]
fn test_comparator_all_cases() {
    println!("\n=== Comprehensive Comparator Test ===\n");

    let mut schematic = UniversalSchematic::new("comparator_test".to_string());

    // Base layer
    schematic.set_block_str(0, 0, 0, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 0, "minecraft:gray_concrete");
    schematic.set_block_str(0, 0, 1, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 1, "minecraft:gray_concrete");
    schematic.set_block_str(0, 0, 2, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 2, "minecraft:gray_concrete");

    // Logic layer - exact browser test layout
    schematic.set_block_str(0, 1, 0, "minecraft:redstone_wire");
    schematic.set_block_str(0, 1, 1, "minecraft:comparator[facing=south,mode=subtract]");
    schematic.set_block_str(1, 1, 1, "minecraft:redstone_wire");
    schematic.set_block_str(0, 1, 2, "minecraft:redstone_wire");

    // Setup IO - matching browser test exact coordinates
    let mut inputs = HashMap::new();
    inputs.insert(
        "back".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 2)], // io.back at [0,1,2]
        },
    );
    inputs.insert(
        "side".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(1, 1, 1)], // io.side at [1,1,1]
        },
    );

    let mut outputs = HashMap::new();
    outputs.insert(
        "out".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 0)], // io.out at [0,1,0]
        },
    );

    let world = MchprsWorld::new(schematic.clone()).expect("Failed to create world");
    let mut executor = TypedCircuitExecutor::new(world, inputs, outputs);

    // Test all combinations
    let test_cases = vec![
        (false, false, false, "Both OFF → output OFF"),
        (
            true,
            false,
            true,
            "Back ON, side OFF → output ON (inverter works)",
        ),
        (false, true, false, "Back OFF, side ON → output OFF"),
        (
            true,
            true,
            false,
            "Both ON → output OFF (BUG: should be OFF, comparator subtracts)",
        ),
    ];

    let mode = ExecutionMode::FixedTicks { ticks: 50 };
    let mut failures = Vec::new();

    for (back, side, expected, description) in test_cases {
        let mut input_values = HashMap::new();
        input_values.insert("back".to_string(), Value::Bool(back));
        input_values.insert("side".to_string(), Value::Bool(side));

        let result = executor
            .execute(input_values, mode.clone())
            .expect("Execution failed");

        let output = result.outputs.get("out").expect("Missing output");
        let actual = match output {
            Value::Bool(val) => *val,
            _ => panic!("Output is not a bool"),
        };

        let status = if actual == expected { "✅" } else { "❌" };
        println!(
            "{} back={:5}, side={:5} → output={:5} (expected {:5}) | {}",
            status, back, side, actual, expected, description
        );

        if actual != expected {
            failures.push((back, side, expected, actual, description));
        }
    }

    if !failures.is_empty() {
        println!("\n❌ COMPARATOR SUBTRACT MODE BUG CONFIRMED!");
        println!("═══════════════════════════════════════════════");
        for (back, side, expected, actual, desc) in &failures {
            println!(
                "   {} | back={}, side={} → expected={}, got={}",
                desc, back, side, expected, actual
            );
        }
        println!("\nExpected behavior:");
        println!("  Comparator in subtract mode: output = max(rear - side, 0)");
        println!("  With rear=15, side=15: output = max(15-15, 0) = 0 (OFF)");
        println!("\nActual behavior:");
        println!("  Output = 15 (ON) - The side input is being IGNORED!");
        println!("\nThis is an MCHPRS bug in comparator subtract mode implementation.");
        panic!("MCHPRS Bug: Comparator subtract mode ignores side input");
    } else {
        println!("\n✅ All test cases passed!");
    }
}
