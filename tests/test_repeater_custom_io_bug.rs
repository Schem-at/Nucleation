/// Test the exact bug from user's JavaScript: wires after repeater show power=0
/// Even though output is correct, intermediate wire block states aren't updated
///
/// Layout:
/// ```
/// [0,1,4] INPUT wire (power=15) ✅
///    ↓
/// [0,1,3] REPEATER (powered=true) ✅
///    ↓
/// [0,1,2] wire (power=0 BUG! should be 15) ❌
///    ↓  ↘
/// [0,1,1] [1,1,2] wires (power=0 BUG!) ❌
///         ↓
///     [1,1,1] COMPARATOR
///         ↓
///     [1,1,0] OUTPUT (power=14) ✅
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
fn test_repeater_middle_wires_power_bug() {
    println!("\n=== REPEATER CUSTOM IO BUG TEST ===");
    println!("BUG: Wires between repeater and output show power=0 even though output is correct!");

    let mut schematic = UniversalSchematic::new("repeater_bug".to_string());

    // Base layer - 2x5 concrete (exactly matching user's circuit)
    for z in 0..5 {
        for x in 0..2 {
            schematic.set_block_str(x, 0, z, "minecraft:gray_concrete");
        }
    }

    // Logic layer - exact user layout
    schematic.set_block_str(0, 1, 4, "minecraft:redstone_wire"); // INPUT
    schematic.set_block_str(0, 1, 3, "minecraft:repeater[facing=south,delay=1]");
    schematic.set_block_str(0, 1, 2, "minecraft:redstone_wire"); // MIDDLE (BUG: shows power=0)
    schematic.set_block_str(0, 1, 1, "minecraft:redstone_wire"); // Side wire
    schematic.set_block_str(1, 1, 2, "minecraft:redstone_wire"); // Back input (BUG: shows power=0)
    schematic.set_block_str(1, 1, 1, "minecraft:comparator[facing=south,mode=subtract]");
    schematic.set_block_str(1, 1, 0, "minecraft:redstone_wire"); // OUTPUT

    println!("\nCircuit:");
    println!("  [0,1,4] INPUT → [0,1,3] REPEATER → [0,1,2] ← BUG HERE");
    println!("                                        ↓");
    println!("                                   [1,1,2] ← BUG HERE");
    println!("                                        ↓");
    println!("                                   [1,1,1] COMPARATOR");
    println!("                                        ↓");
    println!("                                   [1,1,0] OUTPUT");

    // Define IO - input at top, output at bottom
    let mut inputs = HashMap::new();
    inputs.insert(
        "a".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(0, 1, 4)], // Input wire at top
        },
    );

    let mut outputs = HashMap::new();
    outputs.insert(
        "out".to_string(),
        IoMapping {
            io_type: IoType::Boolean,
            layout: LayoutFunction::OneToOne,
            positions: vec![(1, 1, 0)], // Output wire at bottom
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

    // Check internal simulation powers via get_signal_strength
    let world = executor.world();
    let power_input = world.get_signal_strength(BlockPos::new(0, 1, 4));
    let power_after_repeater = world.get_signal_strength(BlockPos::new(0, 1, 2));
    let power_back_input = world.get_signal_strength(BlockPos::new(1, 1, 2));
    let power_side_input = world.get_signal_strength(BlockPos::new(0, 1, 1));
    let power_output = world.get_signal_strength(BlockPos::new(1, 1, 0));

    println!("\nInternal simulation powers (get_signal_strength):");
    println!("  INPUT         [0,1,4]: {}", power_input);
    println!("  AFTER_REP     [0,1,2]: {}", power_after_repeater);
    println!("  BACK_INPUT    [1,1,2]: {}", power_back_input);
    println!("  SIDE_INPUT    [0,1,1]: {}", power_side_input);
    println!("  OUTPUT        [1,1,0]: {}", power_output);

    // Now check schematic block states (what JavaScript sees)
    let schematic = executor.sync_and_get_schematic();

    let get_wire_power = |x, y, z| -> u8 {
        schematic
            .get_block(x, y, z)
            .and_then(|b| b.properties.get("power"))
            .and_then(|p| p.parse().ok())
            .unwrap_or(0)
    };

    let power_input_synced = get_wire_power(0, 1, 4);
    let power_after_repeater_synced = get_wire_power(0, 1, 2);
    let power_back_input_synced = get_wire_power(1, 1, 2);
    let power_side_input_synced = get_wire_power(0, 1, 1);
    let power_output_synced = get_wire_power(1, 1, 0);

    println!("\nSchematic block state powers (what JS sees):");
    println!("  INPUT         [0,1,4]: {}", power_input_synced);
    println!("  AFTER_REP     [0,1,2]: {}", power_after_repeater_synced);
    println!("  BACK_INPUT    [1,1,2]: {}", power_back_input_synced);
    println!("  SIDE_INPUT    [0,1,1]: {}", power_side_input_synced);
    println!("  OUTPUT        [1,1,0]: {}", power_output_synced);

    // Get typed output
    let output_value = result.outputs.get("out").unwrap();
    println!("\nTyped output 'out': {:?}", output_value);

    // Check for the bug
    println!("\n=== BUG CHECK ===");

    if power_after_repeater != power_after_repeater_synced {
        println!("❌ BUG: Wire after repeater mismatch!");
        println!(
            "   Internal: {}, Schematic: {}",
            power_after_repeater, power_after_repeater_synced
        );
    }

    if power_back_input != power_back_input_synced {
        println!("❌ BUG: Back input wire mismatch!");
        println!(
            "   Internal: {}, Schematic: {}",
            power_back_input, power_back_input_synced
        );
    }

    // The paradox: output can be correct even if intermediate wires are wrong
    if power_output == power_output_synced && power_after_repeater_synced == 0 {
        println!("❌ PARADOX: Output is correct but intermediate wires show power=0!");
        println!(
            "   This means Redpiler simulation is working but wire block states aren't updating."
        );
    }

    // Assertions
    assert_eq!(
        power_after_repeater, power_after_repeater_synced,
        "BUG: Wire after repeater power mismatch. Internal={}, Synced={}",
        power_after_repeater, power_after_repeater_synced
    );

    assert_eq!(
        power_back_input, power_back_input_synced,
        "BUG: Back input wire power mismatch. Internal={}, Synced={}",
        power_back_input, power_back_input_synced
    );
}
