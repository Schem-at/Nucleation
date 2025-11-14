//! Integration tests with real redstone circuits
//!
//! These tests use actual schematics to verify the TypedCircuitExecutor works
//! with real redstone logic gates and circuits.

use super::*;
use crate::simulation::MchprsWorld;
use crate::{SchematicBuilder, UniversalSchematic};
use std::collections::HashMap;

/// Helper function to create a test executor from a schematic
fn create_executor(
    schematic: UniversalSchematic,
    inputs: HashMap<String, IoMapping>,
    outputs: HashMap<String, IoMapping>,
) -> TypedCircuitExecutor {
    use crate::simulation::SimulationOptions;
    use mchprs_blocks::BlockPos;

    // Collect all IO positions for custom IO registration
    let mut custom_io = Vec::new();
    for mapping in inputs.values() {
        for &(x, y, z) in &mapping.positions {
            custom_io.push(BlockPos::new(x, y, z));
        }
    }
    for mapping in outputs.values() {
        for &(x, y, z) in &mapping.positions {
            custom_io.push(BlockPos::new(x, y, z));
        }
    }

    // Create simulation options with custom IO
    let options = SimulationOptions {
        optimize: true,
        io_only: false,
        custom_io: custom_io.clone(),
    };

    let world =
        MchprsWorld::with_options(schematic, options.clone()).expect("Failed to create world");
    TypedCircuitExecutor::with_options(world, inputs, outputs, options)
}

// ============================================================================
// AND GATE TESTS
// ============================================================================

#[test]
fn test_and_gate_both_false() {
    let schematic = create_and_gate_schematic();
    let (inputs, outputs) = create_and_gate_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    // Test: false AND false = false
    let mut input_values = HashMap::new();
    input_values.insert("a".to_string(), Value::Bool(false));
    input_values.insert("b".to_string(), Value::Bool(false));

    let result = executor
        .execute(input_values, ExecutionMode::FixedTicks { ticks: 20 })
        .expect("Execution failed");

    let output = result.outputs.get("output").expect("Missing output");
    assert_eq!(
        *output,
        Value::Bool(false),
        "false AND false should be false"
    );
}

#[test]
fn test_and_gate_one_true() {
    let schematic = create_and_gate_schematic();
    let (inputs, outputs) = create_and_gate_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    // Test: true AND false = false
    let mut input_values = HashMap::new();
    input_values.insert("a".to_string(), Value::Bool(true));
    input_values.insert("b".to_string(), Value::Bool(false));

    let result = executor
        .execute(input_values, ExecutionMode::FixedTicks { ticks: 20 })
        .expect("Execution failed");

    let output = result.outputs.get("output").expect("Missing output");
    assert_eq!(
        *output,
        Value::Bool(false),
        "true AND false should be false"
    );
}

#[test]
fn test_and_gate_both_true() {
    let schematic = create_and_gate_schematic();
    let (inputs, outputs) = create_and_gate_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    // Test: true AND true = true
    let mut input_values = HashMap::new();
    input_values.insert("a".to_string(), Value::Bool(true));
    input_values.insert("b".to_string(), Value::Bool(true));

    let result = executor
        .execute(input_values, ExecutionMode::FixedTicks { ticks: 20 })
        .expect("Execution failed");

    // Debug: Check intermediate circuit state
    use mchprs_blocks::BlockPos;
    let world = executor.world();
    eprintln!("\n=== AND Gate Circuit State (both inputs true) ===");
    eprintln!(
        "Input A wire (0,1,0): power = {:?}",
        world.get_signal_strength(BlockPos::new(0, 1, 0))
    );
    eprintln!(
        "Wire at (1,1,0): power = {:?}",
        world.get_signal_strength(BlockPos::new(1, 1, 0))
    );
    eprintln!(
        "Input B wire (0,1,2): power = {:?}",
        world.get_signal_strength(BlockPos::new(0, 1, 2))
    );
    eprintln!(
        "Wire at (1,1,2): power = {:?}",
        world.get_signal_strength(BlockPos::new(1, 1, 2))
    );

    let torch1 = world.get_schematic().get_block(2, 2, 0);
    eprintln!(
        "Torch at (2,2,0): {:?}, lit={:?}",
        torch1.map(|b| &b.name),
        torch1.and_then(|b| b.get_property("lit"))
    );

    let torch2 = world.get_schematic().get_block(2, 2, 2);
    eprintln!(
        "Torch at (2,2,2): {:?}, lit={:?}",
        torch2.map(|b| &b.name),
        torch2.and_then(|b| b.get_property("lit"))
    );

    eprintln!(
        "Wire at (2,2,1): power = {:?}",
        world.get_signal_strength(BlockPos::new(2, 2, 1))
    );

    let wall_torch = world.get_schematic().get_block(3, 1, 1);
    eprintln!(
        "Wall torch at (3,1,1): {:?}, lit={:?}",
        wall_torch.map(|b| &b.name),
        wall_torch.and_then(|b| b.get_property("lit"))
    );

    eprintln!(
        "Output wire (4,1,1): power = {:?}",
        world.get_signal_strength(BlockPos::new(4, 1, 1))
    );

    let output = result.outputs.get("output").expect("Missing output");
    eprintln!("Final output value: {:?}\n", output);
    assert_eq!(*output, Value::Bool(true), "true AND true should be true");
}

// ============================================================================
// XOR GATE TESTS
// ============================================================================

#[test]
fn test_xor_gate_both_false() {
    let schematic = create_xor_gate_schematic();
    let (inputs, outputs) = create_xor_gate_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    let mut input_values = HashMap::new();
    input_values.insert("a".to_string(), Value::Bool(false));
    input_values.insert("b".to_string(), Value::Bool(false));

    let result = executor
        .execute(input_values, ExecutionMode::FixedTicks { ticks: 20 })
        .expect("Execution failed");

    let output = result.outputs.get("output").expect("Missing output");
    assert_eq!(
        *output,
        Value::Bool(false),
        "false XOR false should be false"
    );
}

#[test]
fn test_xor_gate_one_true() {
    let schematic = create_xor_gate_schematic();
    let (inputs, outputs) = create_xor_gate_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    // Test: true XOR false = true
    let mut input_values = HashMap::new();
    input_values.insert("a".to_string(), Value::Bool(true));
    input_values.insert("b".to_string(), Value::Bool(false));

    let result = executor
        .execute(input_values, ExecutionMode::FixedTicks { ticks: 20 })
        .expect("Execution failed");

    let output = result.outputs.get("output").expect("Missing output");
    assert_eq!(*output, Value::Bool(true), "true XOR false should be true");
}

#[test]
fn test_xor_gate_both_true() {
    let schematic = create_xor_gate_schematic();
    let (inputs, outputs) = create_xor_gate_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    let mut input_values = HashMap::new();
    input_values.insert("a".to_string(), Value::Bool(true));
    input_values.insert("b".to_string(), Value::Bool(true));

    let result = executor
        .execute(input_values, ExecutionMode::FixedTicks { ticks: 20 })
        .expect("Execution failed");

    let output = result.outputs.get("output").expect("Missing output");
    assert_eq!(*output, Value::Bool(false), "true XOR true should be false");
}

// ============================================================================
// D-LATCH TESTS (Stateful)
// ============================================================================

// ============================================================================
// ADDER TESTS
// ============================================================================

#[test]
#[ignore] // Remove this once the circuit is built
fn test_adder_simple() {
    let schematic = create_adder_schematic();
    let (inputs, outputs) = create_adder_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    // Test: 5 + 3 = 8
    let mut input_values = HashMap::new();
    input_values.insert("a".to_string(), Value::U32(5));
    input_values.insert("b".to_string(), Value::U32(3));

    let result = executor
        .execute(input_values, ExecutionMode::FixedTicks { ticks: 50 })
        .expect("Execution failed");

    let output = result.outputs.get("sum").expect("Missing output");
    assert_eq!(*output, Value::U32(8), "5 + 3 should equal 8");
}

#[test]
#[ignore] // Remove this once the circuit is built
fn test_adder_overflow() {
    let schematic = create_adder_schematic();
    let (inputs, outputs) = create_adder_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    // Test: 15 + 1 = 0 (4-bit overflow)
    let mut input_values = HashMap::new();
    input_values.insert("a".to_string(), Value::U32(15));
    input_values.insert("b".to_string(), Value::U32(1));

    let result = executor
        .execute(input_values, ExecutionMode::FixedTicks { ticks: 50 })
        .expect("Execution failed");

    let output = result.outputs.get("sum").expect("Missing output");
    assert_eq!(
        *output,
        Value::U32(0),
        "15 + 1 should overflow to 0 (4-bit)"
    );
}

// ============================================================================
// CIRCUIT BUILDER HELPERS
// ============================================================================

/// Helper to build circuits from a list of block placements
///
/// Takes a list of (x, y, z, block_string) tuples and builds a schematic
fn build_schematic_from_blocks(blocks: &[(i32, i32, i32, &str)]) -> UniversalSchematic {
    let mut schematic = UniversalSchematic::new("circuit".to_string());

    for &(x, y, z, block_str) in blocks {
        match UniversalSchematic::parse_block_string(block_str) {
            Ok((block_state, _)) => {
                schematic.set_block(x, y, z, block_state);
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse block '{}': {}", block_str, e);
            }
        }
    }

    schematic
}

// ============================================================================
// CIRCUIT BUILDERS - TO BE IMPLEMENTED
// ============================================================================

/// Creates an AND gate schematic
///
/// Layout:
/// - Input A at (0, 1, 0) - redstone wire
/// - Input B at (0, 1, 2) - redstone wire  
/// - Output at (4, 1, 1) - redstone wire
fn create_and_gate_schematic() -> UniversalSchematic {
    let blocks = vec![
        // Gray concrete base
        (0, 0, 0, "minecraft:gray_concrete"),
        (1, 0, 0, "minecraft:gray_concrete"),
        (0, 0, 2, "minecraft:gray_concrete"),
        (1, 0, 2, "minecraft:gray_concrete"),
        (2, 0, 0, "minecraft:gray_concrete"),
        (2, 0, 1, "minecraft:gray_concrete"),
        (2, 0, 2, "minecraft:gray_concrete"),
        (2, 1, 0, "minecraft:gray_concrete"),
        (2, 1, 1, "minecraft:gray_concrete"),
        (2, 1, 2, "minecraft:gray_concrete"),
        (3, 0, 1, "minecraft:gray_concrete"),
        (4, 0, 1, "minecraft:gray_concrete"),
        // Redstone wiring for AND gate logic
        (
            0,
            1,
            0,
            "minecraft:redstone_wire[power=0,north=none,south=none,east=side,west=side]",
        ), // Input A
        (
            1,
            1,
            0,
            "minecraft:redstone_wire[power=0,north=none,south=none,east=side,west=side]",
        ),
        (
            0,
            1,
            2,
            "minecraft:redstone_wire[power=0,north=none,south=none,east=side,west=side]",
        ), // Input B
        (
            1,
            1,
            2,
            "minecraft:redstone_wire[power=0,north=none,south=none,east=side,west=side]",
        ),
        (2, 2, 0, "minecraft:redstone_torch[lit=true]"),
        (2, 2, 2, "minecraft:redstone_torch[lit=true]"),
        (
            2,
            2,
            1,
            "minecraft:redstone_wire[power=15,north=side,south=side,east=none,west=none]",
        ),
        (
            3,
            1,
            1,
            "minecraft:redstone_wall_torch[facing=east,lit=false]",
        ),
        (
            4,
            1,
            1,
            "minecraft:redstone_wire[power=0,north=none,south=none,east=side,west=side]",
        ), // Output
    ];

    build_schematic_from_blocks(&blocks)
}

fn create_and_gate_io() -> (HashMap<String, IoMapping>, HashMap<String, IoMapping>) {
    let mut inputs = HashMap::new();
    let mut outputs = HashMap::new();

    // Input A - single bit at (0, 1, 0)
    inputs.insert(
        "a".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 0)],
        },
    );

    // Input B - single bit at (0, 1, 2)
    inputs.insert(
        "b".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 2)],
        },
    );

    // Output - single bit at (4, 1, 1)
    outputs.insert(
        "output".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(4, 1, 1)],
        },
    );

    (inputs, outputs)
}

/// Creates an XOR gate schematic
///
/// Layout:
/// - Input A at (0, 1, 0) - redstone wire
/// - Input B at (0, 1, 2) - redstone wire  
/// - Output at (5, 1, 1) - redstone wire
fn create_xor_gate_schematic() -> UniversalSchematic {
    let blocks = vec![
        // Gray concrete base
        (0, 0, 0, "minecraft:gray_concrete"),
        (1, 0, 0, "minecraft:gray_concrete"),
        (2, 0, 0, "minecraft:gray_concrete"),
        (0, 0, 2, "minecraft:gray_concrete"),
        (1, 0, 2, "minecraft:gray_concrete"),
        (2, 0, 2, "minecraft:gray_concrete"),
        (3, 0, 0, "minecraft:gray_concrete"),
        (3, 0, 2, "minecraft:gray_concrete"),
        (3, 1, 0, "minecraft:gray_concrete"),
        (3, 1, 2, "minecraft:gray_concrete"),
        (4, 0, 1, "minecraft:gray_concrete"),
        (5, 0, 1, "minecraft:gray_concrete"),
        // Redstone wiring for XOR gate logic
        (
            0,
            1,
            0,
            "minecraft:redstone_wire[power=0,north=none,south=none,east=side,west=side]",
        ), // Input A
        (
            1,
            1,
            0,
            "minecraft:redstone_wire[power=0,north=none,south=none,east=side,west=side]",
        ),
        (2, 1, 0, "minecraft:gray_concrete"),
        (
            0,
            1,
            2,
            "minecraft:redstone_wire[power=0,north=none,south=none,east=side,west=side]",
        ), // Input B
        (
            1,
            1,
            2,
            "minecraft:redstone_wire[power=0,north=none,south=none,east=side,west=side]",
        ),
        (2, 1, 2, "minecraft:gray_concrete"),
        (2, 2, 0, "minecraft:redstone_torch[lit=true]"),
        (2, 2, 2, "minecraft:redstone_torch[lit=true]"),
        (2, 2, 1, "minecraft:gray_concrete"),
        (2, 3, 0, "minecraft:gray_concrete"),
        (
            2,
            3,
            1,
            "minecraft:redstone_wire[power=0,north=side,south=side,east=side,west=side]",
        ),
        (2, 3, 2, "minecraft:gray_concrete"),
        (
            3,
            2,
            0,
            "minecraft:redstone_wire[power=15,north=none,south=side,east=none,west=side]",
        ),
        (
            3,
            2,
            2,
            "minecraft:redstone_wire[power=15,north=side,south=none,east=none,west=side]",
        ),
        (
            4,
            1,
            0,
            "minecraft:redstone_wall_torch[facing=east,lit=false]",
        ),
        (
            4,
            1,
            2,
            "minecraft:redstone_wall_torch[facing=east,lit=false]",
        ),
        (
            5,
            1,
            1,
            "minecraft:redstone_wire[power=0,north=none,south=none,east=side,west=side]",
        ), // Output
        (
            3,
            2,
            1,
            "minecraft:redstone_wall_torch[facing=east,lit=false]",
        ),
        (
            4,
            1,
            1,
            "minecraft:redstone_wire[power=0,north=side,south=side,east=side,west=none]",
        ),
    ];

    build_schematic_from_blocks(&blocks)
}

fn create_xor_gate_io() -> (HashMap<String, IoMapping>, HashMap<String, IoMapping>) {
    let mut inputs = HashMap::new();
    let mut outputs = HashMap::new();

    // Input A - single bit at (0, 1, 0)
    inputs.insert(
        "a".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 0)],
        },
    );

    // Input B - single bit at (0, 1, 2)
    inputs.insert(
        "b".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 2)],
        },
    );

    // Output - single bit at (5, 1, 1)
    outputs.insert(
        "output".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(5, 1, 1)],
        },
    );

    (inputs, outputs)
}

/// Creates a D-latch schematic
///
/// TODO: Implement using SchematicBuilder::from_template()
///
/// Template structure:
/// ```text
/// # Base layer
/// cccccc
/// cccccc
/// cccccc
///
/// # Logic layer
/// d_____
/// _____q
/// e_____
///
/// [palette]
/// c = minecraft:gray_concrete
/// d = minecraft:redstone_wire  # Data input
/// e = minecraft:redstone_wire  # Enable input
/// q = minecraft:redstone_wire  # Output
/// t = minecraft:redstone_torch
/// R = minecraft:repeater[facing=east]
/// _ = minecraft:air
/// ```
fn create_d_latch_schematic() -> UniversalSchematic {
    use crate::SchematicBuilder;

    // D-latch with locking repeater for state holding
    let template = r#"
# Base layer
ccc
ccc
ccc
ccc

# Logic layer
··│
^→⇑
c·⟰
│·│

[palette]
→ = minecraft:repeater[facing=west,delay=1,locked=false,powered=true]
⇑ = minecraft:repeater[facing=south,delay=1,locked=true,powered=false]
⟰ = minecraft:repeater[facing=south,delay=1,locked=false,powered=false]
^ = minecraft:redstone_wall_torch[facing=north,lit=true]
"#;

    SchematicBuilder::from_template(template)
        .expect("Failed to parse D-latch template")
        .build()
        .expect("Failed to build D-latch schematic")
}

fn create_d_latch_io() -> (HashMap<String, IoMapping>, HashMap<String, IoMapping>) {
    let mut inputs = HashMap::new();
    let mut outputs = HashMap::new();

    // D-latch IO positions based on the 3x4 circuit template
    // ··│  (z=0) - wire at x=2 is OUTPUT
    // ^→⇑  (z=1) - torch at x=0, repeaters at x=1,2
    // c·│  (z=2) - concrete at x=0, wire at x=2
    // │·│  (z=3) - wire at x=0 is CLOCK, wire at x=2 is DATA

    // Input Enable (clock) - vertical wire at (0, 1, 3)
    inputs.insert(
        "enable".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 3)],
        },
    );

    // Input D (data) - vertical wire at (2, 1, 3)
    inputs.insert(
        "d".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(2, 1, 3)],
        },
    );

    // Output Q (stored value) - vertical wire at (2, 1, 0)
    outputs.insert(
        "q".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(2, 1, 0)],
        },
    );

    (inputs, outputs)
}

/// Creates a 4-bit adder schematic using compositional design
///
/// Strategy: Build hierarchically using the schematic palette system
/// 1. Use existing AND and XOR gates as building blocks
/// 2. Compose them into a half-adder
/// 3. Compose two half-adders into a full-adder
/// 4. Compose four full-adders into a 4-bit adder
///
/// This demonstrates the power of the compositional approach!
fn create_adder_schematic() -> UniversalSchematic {
    use crate::SchematicBuilder;

    let full_adder = SchematicBuilder::from_template(
        r#"
        # Base layer
        ·····c····
        ·····c····
        ··ccccc···
        ·ccccccc··
        cc··cccccc
        ·c··c·····
        ·ccccc····
        ·cccccc···
        ···cccc···
        ···c··c···
        
        # Logic layer
        ·····│····
        ·····↑····
        ··│█←┤█···
        ·█◀←┬▲▲┐··
        ──··├┴┴┴←─
        ·█··↑·····
        ·▲─←┤█····
        ·█←┬▲▲┐···
        ···├┴┴┤···
        ···│··│···
        "#,
    )
    .expect("Failed to parse full-adder template")
    .build()
    .expect("Failed to build full-adder schematic");

    // Stack 4 full-adders side by side
    // Note: Don't mix single-block characters with schematics - all palette entries
    // in a layer should have matching dimensions for proper tiling
    let four_bit_adder = SchematicBuilder::new()
        .name("four_bit_adder")
        .map_schematic('A', full_adder)
        .layers(&[
            &["AAAA"], // 4 full-adders in a row
        ])
        .build()
        .expect("Failed to build 4-bit adder");

    four_bit_adder
}

fn create_adder_io() -> (HashMap<String, IoMapping>, HashMap<String, IoMapping>) {
    let mut inputs = HashMap::new();
    let mut outputs = HashMap::new();

    // 4-bit input A using Packed4 layout (1 position = 4 bits)
    inputs.insert(
        "a".to_string(),
        IoMapping {
            io_type: IoType::UnsignedInt { bits: 4 },
            layout: LayoutFunction::Packed4,
            positions: vec![(0, 0, 0)],
        },
    );

    // 4-bit input B
    inputs.insert(
        "b".to_string(),
        IoMapping {
            io_type: IoType::UnsignedInt { bits: 4 },
            layout: LayoutFunction::Packed4,
            positions: vec![(0, 0, 2)],
        },
    );

    // 4-bit output sum
    outputs.insert(
        "sum".to_string(),
        IoMapping {
            io_type: IoType::UnsignedInt { bits: 4 },
            layout: LayoutFunction::Packed4,
            positions: vec![(10, 0, 0)],
        },
    );

    (inputs, outputs)
}

// ============================================================================
// NEW FEATURES DEMONSTRATION TESTS
// ============================================================================

#[test]
fn test_io_layout_builder_with_and_gate() {
    // Demonstrate IoLayoutBuilder usage
    let schematic = create_and_gate_schematic();

    let layout = IoLayoutBuilder::new()
        .add_input(
            "a",
            IoType::Boolean,
            LayoutFunction::OneToOne,
            vec![(0, 1, 0)],
        )
        .unwrap()
        .add_input(
            "b",
            IoType::Boolean,
            LayoutFunction::OneToOne,
            vec![(0, 1, 2)],
        )
        .unwrap()
        .add_output(
            "output",
            IoType::Boolean,
            LayoutFunction::OneToOne,
            vec![(4, 1, 1)],
        )
        .unwrap()
        .build();

    // Create executor from layout
    use crate::simulation::SimulationOptions;
    use mchprs_blocks::BlockPos;

    let mut custom_io = Vec::new();
    for mapping in layout.inputs.values() {
        for &(x, y, z) in &mapping.positions {
            custom_io.push(BlockPos::new(x, y, z));
        }
    }
    for mapping in layout.outputs.values() {
        for &(x, y, z) in &mapping.positions {
            custom_io.push(BlockPos::new(x, y, z));
        }
    }

    let options = SimulationOptions {
        optimize: true,
        io_only: false,
        custom_io,
    };

    let world =
        MchprsWorld::with_options(schematic, options.clone()).expect("Failed to create world");
    let mut executor = TypedCircuitExecutor::from_layout_with_options(world, layout, options);

    // Test with true AND true
    let mut inputs = HashMap::new();
    inputs.insert("a".to_string(), Value::Bool(true));
    inputs.insert("b".to_string(), Value::Bool(true));

    let result = executor
        .execute(inputs, ExecutionMode::FixedTicks { ticks: 20 })
        .expect("Execution failed");

    assert_eq!(*result.outputs.get("output").unwrap(), Value::Bool(true));
}

#[test]
fn test_execution_mode_until_condition() {
    // Create a simple circuit that outputs true after some ticks
    let schematic = create_and_gate_schematic();
    let (inputs, outputs) = create_and_gate_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    // Set inputs to true AND true (should eventually output true)
    let mut input_values = HashMap::new();
    input_values.insert("a".to_string(), Value::Bool(true));
    input_values.insert("b".to_string(), Value::Bool(true));

    // Execute until output equals true (with timeout)
    let result = executor
        .execute(
            input_values,
            ExecutionMode::UntilCondition {
                output_name: "output".to_string(),
                condition: OutputCondition::Equals(Value::Bool(true)),
                max_ticks: 100,
                check_interval: 2,
            },
        )
        .expect("Execution failed");

    // Should have met the condition
    assert!(result.condition_met, "Condition should have been met");
    assert_eq!(*result.outputs.get("output").unwrap(), Value::Bool(true));
    assert!(
        result.ticks_elapsed <= 100,
        "Should complete within timeout"
    );
}

#[test]
fn test_execution_mode_until_stable() {
    // Test that the circuit stabilizes
    let schematic = create_and_gate_schematic();
    let (inputs, outputs) = create_and_gate_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    let mut input_values = HashMap::new();
    input_values.insert("a".to_string(), Value::Bool(true));
    input_values.insert("b".to_string(), Value::Bool(false));

    // Execute until stable for 5 ticks
    let result = executor
        .execute(
            input_values,
            ExecutionMode::UntilStable {
                stable_ticks: 5,
                max_ticks: 100,
            },
        )
        .expect("Execution failed");

    // Should have stabilized
    assert!(result.condition_met, "Circuit should have stabilized");
    assert_eq!(*result.outputs.get("output").unwrap(), Value::Bool(false));
}

#[test]
fn test_state_mode_stateful() {
    // Test stateful execution (state preserved between runs)
    let schematic = create_and_gate_schematic();
    let (inputs, outputs) = create_and_gate_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    // Set to stateful mode
    executor.set_state_mode(StateMode::Stateful);

    // First execution
    let mut input_values = HashMap::new();
    input_values.insert("a".to_string(), Value::Bool(true));
    input_values.insert("b".to_string(), Value::Bool(true));

    let result1 = executor
        .execute(
            input_values.clone(),
            ExecutionMode::FixedTicks { ticks: 10 },
        )
        .expect("Execution failed");

    // Second execution (state should be preserved)
    let result2 = executor
        .execute(input_values, ExecutionMode::FixedTicks { ticks: 10 })
        .expect("Execution failed");

    // Both should produce the same result
    assert_eq!(result1.outputs.get("output"), result2.outputs.get("output"));
}

#[test]
fn test_state_mode_manual_reset() {
    // Test manual state control
    let schematic = create_and_gate_schematic();
    let (inputs, outputs) = create_and_gate_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    // Set to manual mode
    executor.set_state_mode(StateMode::Manual);

    // First execution
    let mut input_values = HashMap::new();
    input_values.insert("a".to_string(), Value::Bool(true));
    input_values.insert("b".to_string(), Value::Bool(true));

    let _result1 = executor
        .execute(
            input_values.clone(),
            ExecutionMode::FixedTicks { ticks: 10 },
        )
        .expect("Execution failed");

    // Manually reset
    executor.reset().expect("Reset failed");

    // Second execution (should start fresh)
    let result2 = executor
        .execute(input_values, ExecutionMode::FixedTicks { ticks: 10 })
        .expect("Execution failed");

    // Should produce expected result
    assert_eq!(*result2.outputs.get("output").unwrap(), Value::Bool(true));
}

#[test]
fn test_io_layout_builder_auto_inference() {
    // Test automatic layout inference
    let schematic = create_and_gate_schematic();

    let layout = IoLayoutBuilder::new()
        .add_input_auto("a", IoType::Boolean, vec![(0, 1, 0)])
        .unwrap()
        .add_input_auto("b", IoType::Boolean, vec![(0, 1, 2)])
        .unwrap()
        .add_output_auto("output", IoType::Boolean, vec![(4, 1, 1)])
        .unwrap()
        .build();

    // Verify layouts were inferred correctly
    let input_a = layout.get_input("a").unwrap();
    assert!(matches!(input_a.layout, LayoutFunction::OneToOne));

    let input_b = layout.get_input("b").unwrap();
    assert!(matches!(input_b.layout, LayoutFunction::OneToOne));

    let output = layout.get_output("output").unwrap();
    assert!(matches!(output.layout, LayoutFunction::OneToOne));
}

#[test]
fn test_execution_mode_until_change() {
    // Test UntilChange execution mode
    let schematic = create_and_gate_schematic();
    let (inputs, outputs) = create_and_gate_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    // Start with both inputs false
    let mut input_values = HashMap::new();
    input_values.insert("a".to_string(), Value::Bool(false));
    input_values.insert("b".to_string(), Value::Bool(false));

    // Execute until output changes (with a reasonable timeout)
    let result = executor
        .execute(
            input_values,
            ExecutionMode::UntilChange {
                max_ticks: 100,
                check_interval: 5,
            },
        )
        .expect("Execution failed");

    // The output should have stabilized to false
    // Since inputs are both false, output won't change from initial state
    // So this test verifies timeout behavior
    assert_eq!(*result.outputs.get("output").unwrap(), Value::Bool(false));

    // Now test with changing inputs - set one input to true
    let mut input_values2 = HashMap::new();
    input_values2.insert("a".to_string(), Value::Bool(true));
    input_values2.insert("b".to_string(), Value::Bool(false));

    // Reset to stateless mode to ensure fresh start
    executor.set_state_mode(StateMode::Stateless);

    let result2 = executor
        .execute(
            input_values2,
            ExecutionMode::UntilChange {
                max_ticks: 100,
                check_interval: 5,
            },
        )
        .expect("Execution failed");

    // Output should be false (true AND false = false)
    assert_eq!(*result2.outputs.get("output").unwrap(), Value::Bool(false));
}

#[test]
fn test_execution_mode_until_change_with_state() {
    // Test UntilChange with stateful execution to detect actual changes
    let schematic = create_and_gate_schematic();
    let (inputs, outputs) = create_and_gate_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    // Set to stateful mode
    executor.set_state_mode(StateMode::Stateful);

    // First execution: both inputs true
    let mut input_values1 = HashMap::new();
    input_values1.insert("a".to_string(), Value::Bool(true));
    input_values1.insert("b".to_string(), Value::Bool(true));

    let result1 = executor
        .execute(input_values1, ExecutionMode::FixedTicks { ticks: 20 })
        .expect("Execution failed");

    // Output should be true
    assert_eq!(*result1.outputs.get("output").unwrap(), Value::Bool(true));

    // Second execution: change one input to false
    // This should cause the output to change from true to false
    let mut input_values2 = HashMap::new();
    input_values2.insert("a".to_string(), Value::Bool(true));
    input_values2.insert("b".to_string(), Value::Bool(false));

    let result2 = executor
        .execute(
            input_values2,
            ExecutionMode::UntilChange {
                max_ticks: 100,
                check_interval: 5,
            },
        )
        .expect("Execution failed");

    // Output should have changed to false
    assert_eq!(*result2.outputs.get("output").unwrap(), Value::Bool(false));

    // The change should have been detected
    assert!(
        result2.condition_met,
        "Output change should have been detected"
    );
}

// ============================================================================
// INVERTER TESTS
// ============================================================================

fn create_inverter_schematic() -> UniversalSchematic {
    let template = r#"
# Base layer
c

# Logic layer
│

# Torch layer
^

# Output layer
│

[palette]
^ = minecraft:redstone_torch[lit=true]
"#;

    SchematicBuilder::from_template(template)
        .expect("Failed to parse inverter template")
        .build()
        .expect("Failed to build inverter schematic")
}

fn create_inverter_io() -> (HashMap<String, IoMapping>, HashMap<String, IoMapping>) {
    let mut inputs = HashMap::new();
    let mut outputs = HashMap::new();

    // Input at (0, 1, 0)
    inputs.insert(
        "input".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 0)],
        },
    );

    // Output at (0, 3, 0)
    outputs.insert(
        "output".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 3, 0)],
        },
    );

    (inputs, outputs)
}

#[test]
fn test_inverter_false() {
    // Test inverter with false input -> should output true
    let schematic = create_inverter_schematic();
    let (inputs, outputs) = create_inverter_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    let mut input_values = HashMap::new();
    input_values.insert("input".to_string(), Value::Bool(false));

    let result = executor
        .execute(input_values, ExecutionMode::FixedTicks { ticks: 5 })
        .expect("Execution failed");

    let output = result.outputs.get("output").expect("Missing output");
    assert_eq!(*output, Value::Bool(true), "NOT false should be true");
}

#[test]
fn test_inverter_true() {
    // Test inverter with true input -> should output false
    let schematic = create_inverter_schematic();
    let (inputs, outputs) = create_inverter_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    let mut input_values = HashMap::new();
    input_values.insert("input".to_string(), Value::Bool(true));

    let result = executor
        .execute(input_values, ExecutionMode::FixedTicks { ticks: 5 })
        .expect("Execution failed");

    let output = result.outputs.get("output").expect("Missing output");
    assert_eq!(*output, Value::Bool(false), "NOT true should be false");
}

// ============================================================================
// D-LATCH TESTS
// ============================================================================

#[test]
fn test_d_latch_transparent_mode() {
    // Test D-latch in transparent mode (enable=true)
    // When enable is high, output Q should follow input D
    let schematic = create_d_latch_schematic();
    let (inputs, outputs) = create_d_latch_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    // Test: D=true, Enable=true -> Q should become true
    let mut input_values = HashMap::new();
    input_values.insert("enable".to_string(), Value::Bool(true));
    input_values.insert("d".to_string(), Value::Bool(true));

    let result = executor
        .execute(input_values, ExecutionMode::FixedTicks { ticks: 10 })
        .expect("Execution failed");

    let q = result.outputs.get("q").expect("Missing output q");
    assert_eq!(
        *q,
        Value::Bool(true),
        "Q should be true when D=true and Enable=true"
    );
}

#[test]
fn test_d_latch_latch_mode() {
    // Test D-latch in latch mode (enable goes low)
    // When enable goes low, output Q should hold the last D value
    let schematic = create_d_latch_schematic();
    let (inputs, outputs) = create_d_latch_io();
    let mut executor = create_executor(schematic, inputs, outputs);

    executor.set_state_mode(StateMode::Stateful);

    // Step 1: Set D=true, Enable=true (transparent mode)
    let mut input_values1 = HashMap::new();
    input_values1.insert("enable".to_string(), Value::Bool(true));
    input_values1.insert("d".to_string(), Value::Bool(true));

    let result1 = executor
        .execute(input_values1, ExecutionMode::FixedTicks { ticks: 10 })
        .expect("Execution failed");

    let q1 = result1.outputs.get("q").expect("Missing output q");
    assert_eq!(
        *q1,
        Value::Bool(true),
        "Q should be true after D=true, Enable=true"
    );

    // Step 2: Set Enable=false, D=false (latch mode - Q should stay true)
    let mut input_values2 = HashMap::new();
    input_values2.insert("enable".to_string(), Value::Bool(false));
    input_values2.insert("d".to_string(), Value::Bool(false));

    let result2 = executor
        .execute(input_values2, ExecutionMode::FixedTicks { ticks: 10 })
        .expect("Execution failed");

    let q2 = result2.outputs.get("q").expect("Missing output q");
    assert_eq!(
        *q2,
        Value::Bool(true),
        "Q should hold true even when D=false and Enable=false"
    );
}
