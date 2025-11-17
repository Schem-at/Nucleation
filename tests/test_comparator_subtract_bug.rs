/// Test for comparator subtract mode bug
///
/// Bug report: Comparators in subtract mode are not working correctly:
/// When both inputs are powered, the comparator ignores the side input and just passes through the back input
///
/// This test reproduces the exact issue seen in the browser

#[cfg(feature = "simulation")]
use nucleation::{
    simulation::{BlockPos, SimulationOptions},
    UniversalSchematic,
};

#[cfg(feature = "simulation")]
#[test]
fn test_comparator_subtract_both_inputs_basic() {
    println!("\n=== Test Case 1: Basic two-input comparator (back=ON, side=ON) ===");

    let mut schematic = UniversalSchematic::new("comparator_test".to_string());

    // Base layer (Y=0) - concrete platform
    schematic.set_block_str(0, 0, 0, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 0, "minecraft:gray_concrete");
    schematic.set_block_str(0, 0, 1, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 1, "minecraft:gray_concrete");
    schematic.set_block_str(0, 0, 2, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 2, "minecraft:gray_concrete");

    // Logic layer (Y=1)
    // Comparator at [0,1,1] facing SOUTH means:
    //   - Back input (rear) at [0,1,2] (from the north)
    //   - Side inputs at [1,1,1] (from the east)
    //   - Output at [0,1,0] (to the south)
    schematic.set_block_str(0, 1, 2, "minecraft:redstone_wire"); // Back input wire (north)
    schematic.set_block_str(
        0,
        1,
        1,
        "minecraft:comparator[facing=south,mode=subtract,powered=false]",
    ); // Comparator
    schematic.set_block_str(1, 1, 1, "minecraft:redstone_wire"); // Side input wire (east)
    schematic.set_block_str(0, 1, 0, "minecraft:redstone_wire"); // Output wire (south)

    println!(
        "Schematic built - dimensions: {:?}",
        schematic.get_tight_dimensions()
    );

    // Create simulation with custom IO at the wire positions
    let options = SimulationOptions {
        custom_io: vec![
            BlockPos::new(0, 1, 2), // Back input wire
            BlockPos::new(1, 1, 1), // Side input wire
            BlockPos::new(0, 1, 0), // Output wire
        ],
        ..Default::default()
    };

    let mut world = nucleation::simulation::MchprsWorld::with_options(schematic, options)
        .expect("Failed to create world");

    println!("\n--- Test: back=ON, side=OFF (should output ON) ---");

    // Use set_signal_strength for custom IO nodes
    world.set_signal_strength(BlockPos::new(0, 1, 2), 15); // Back ON
    world.set_signal_strength(BlockPos::new(1, 1, 1), 0); // Side OFF
    world.flush(); // Sync custom IO to world BEFORE ticking
    world.tick(20);
    world.flush();

    println!("After 20 ticks:");
    let back_power1 = world.get_redstone_power(BlockPos::new(0, 1, 2));
    let side_power1 = world.get_redstone_power(BlockPos::new(1, 1, 1));
    let output1 = world.get_redstone_power(BlockPos::new(0, 1, 0));
    println!("  Back wire [0,1,2]: {}", back_power1);
    println!("  Side wire [1,1,1]: {}", side_power1);
    println!("  Output wire [0,1,0]: {}", output1);

    println!("\nBack=15, Side=0 → Output={} (expected 15)", output1);

    assert_eq!(output1, 15, "With back=15, side=0, output should be 15");

    println!("\n--- Test: back=ON, side=ON (should output OFF) ---");
    // Set both to ON
    world.set_signal_strength(BlockPos::new(0, 1, 2), 15); // Back ON
    world.set_signal_strength(BlockPos::new(1, 1, 1), 15); // Side ON
    world.flush(); // Sync custom IO to world BEFORE ticking
    world.tick(20);
    world.flush();

    let back_power2 = world.get_redstone_power(BlockPos::new(0, 1, 2));
    let side_power2 = world.get_redstone_power(BlockPos::new(1, 1, 1));
    let output2 = world.get_redstone_power(BlockPos::new(0, 1, 0));
    println!("  Back wire [0,1,2]: {}", back_power2);
    println!("  Side wire [1,1,1]: {}", side_power2);
    println!("  Output wire [0,1,0]: {}", output2);

    println!("\nBack=15, Side=15 → Output={} (expected 0)", output2);

    if output2 != 0 {
        println!("\n❌ BUG CONFIRMED!");
        println!("   The comparator is ignoring the side input!");
        println!("   Expected: max(15-15, 0) = 0");
        println!("   Actual: {}", output2);
    }

    assert_eq!(
        output2, 0,
        "BUG: With back=15, side=15, output should be 0 but got {}. Comparator is ignoring side input!",
        output2
    );
}

#[cfg(feature = "simulation")]
#[test]
fn test_comparator_subtract_redstone_block_simple() {
    println!("\n=== Test Case 2: Simple comparator with redstone block (no back input) ===");

    let mut schematic = UniversalSchematic::new("comparator_simple".to_string());

    // Base layer (Y=0) - concrete platform
    schematic.set_block_str(0, 0, 0, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 0, "minecraft:gray_concrete");
    schematic.set_block_str(0, 0, 1, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 1, "minecraft:gray_concrete");

    // Logic layer (Y=1)
    schematic.set_block_str(0, 1, 0, "minecraft:redstone_block"); // Constant power source
    schematic.set_block_str(
        0,
        1,
        1,
        "minecraft:comparator[facing=south,mode=subtract,powered=false]",
    ); // Comparator
    schematic.set_block_str(1, 1, 0, "minecraft:redstone_wire"); // Side input wire
    schematic.set_block_str(0, 1, 2, "minecraft:redstone_wire"); // Output wire (would be at Y=1, Z=2 but we'll check comparator output)

    println!(
        "Schematic built - dimensions: {:?}",
        schematic.get_tight_dimensions()
    );

    // Create simulation with custom IO
    let options = SimulationOptions {
        custom_io: vec![
            BlockPos::new(1, 1, 0), // Side input wire
            BlockPos::new(0, 1, 1), // Comparator position
        ],
        ..Default::default()
    };

    let mut world = nucleation::simulation::MchprsWorld::with_options(schematic, options)
        .expect("Failed to create world");

    println!("\n--- Test: redstone block back, side=OFF (should output ON) ---");
    world.set_signal_strength(BlockPos::new(1, 1, 1), 0); // Side OFF
    world.flush(); // Sync custom IO to world BEFORE ticking
    world.tick(10);
    world.flush();

    let output1 = world.get_redstone_power(BlockPos::new(0, 1, 0));
    println!(
        "Block back=15, Side=0 → Output power={} (expected 15)",
        output1
    );

    println!("\n--- Test: redstone block back, side=ON (should output OFF) ---");
    world.set_signal_strength(BlockPos::new(1, 1, 1), 15); // Side ON
    world.flush(); // Sync custom IO to world BEFORE ticking
    world.tick(10);
    world.flush();

    let output2 = world.get_redstone_power(BlockPos::new(0, 1, 0));
    println!(
        "Block back=15, Side=15 → Output power={} (expected 0)",
        output2
    );

    // Check side wire power
    let side_power = world.get_signal_strength(BlockPos::new(1, 1, 1));
    println!("Side wire power: {}", side_power);

    if output2 != 0 {
        println!("\n❌ BUG CONFIRMED!");
        println!("   With redstone block (constant 15) and side=15, output should be 0");
        println!("   Expected: max(15-15, 0) = 0");
        println!("   Actual: {}", output2);
    }

    assert_eq!(
        output2, 0,
        "BUG: With constant back=15, side=15, output should be 0 but got {}",
        output2
    );
}

#[cfg(feature = "simulation")]
#[test]
fn test_comparator_subtract_comprehensive() {
    println!("\n=== Comprehensive comparator subtract mode test ===");

    let mut schematic = UniversalSchematic::new("comparator_comprehensive".to_string());

    // Base layer
    schematic.set_block_str(0, 0, 0, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 0, "minecraft:gray_concrete");
    schematic.set_block_str(0, 0, 1, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 1, "minecraft:gray_concrete");
    schematic.set_block_str(0, 0, 2, "minecraft:gray_concrete");
    schematic.set_block_str(1, 0, 2, "minecraft:gray_concrete");

    // Logic layer (Y=1) - Matching browser test exactly
    // Comparator at [0,1,1] facing SOUTH
    schematic.set_block_str(0, 1, 2, "minecraft:redstone_wire"); // Back input (north)
    schematic.set_block_str(
        0,
        1,
        1,
        "minecraft:comparator[facing=south,mode=subtract,powered=false]",
    );
    schematic.set_block_str(1, 1, 1, "minecraft:redstone_wire"); // Side input (east)
    schematic.set_block_str(0, 1, 0, "minecraft:redstone_wire"); // Output (south)

    let options = SimulationOptions {
        custom_io: vec![
            BlockPos::new(0, 1, 2), // Back input
            BlockPos::new(1, 1, 1), // Side input
            BlockPos::new(0, 1, 0), // Output
        ],
        ..Default::default()
    };

    // Test cases: (back, side, expected)
    let test_cases = vec![
        (0, 0, 0, "Both off"),
        (15, 0, 15, "Back on, side off - should pass through"),
        (0, 15, 0, "Back off, side on - nothing to subtract from"),
        (15, 15, 0, "Both on - should output 0 (BUG CASE)"),
        (15, 5, 10, "Back=15, side=5 - should output 10"),
        (10, 3, 7, "Back=10, side=3 - should output 7"),
        (5, 15, 0, "Back=5, side=15 - should output 0 (side > back)"),
    ];

    let mut failures = Vec::new();

    for (back, side, expected, description) in test_cases {
        let mut world =
            nucleation::simulation::MchprsWorld::with_options(schematic.clone(), options.clone())
                .expect("Failed to create world");

        world.set_signal_strength(BlockPos::new(0, 1, 2), back);
        world.set_signal_strength(BlockPos::new(1, 1, 1), side);
        world.flush(); // Sync custom IO to world BEFORE ticking
        world.tick(10);
        world.flush();

        let output = world.get_signal_strength(BlockPos::new(0, 1, 0));
        let status = if output == expected { "✅" } else { "❌" };

        println!(
            "{} back={:2}, side={:2} → output={:2} (expected {:2}) - {}",
            status, back, side, output, expected, description
        );

        if output != expected {
            failures.push((back, side, expected, output, description));
        }
    }

    if !failures.is_empty() {
        println!("\n❌ FAILURES ({} cases):", failures.len());
        for (back, side, expected, actual, desc) in &failures {
            println!(
                "   {} | back={}, side={} → expected={}, got={}",
                desc, back, side, expected, actual
            );
        }
        panic!("{} test case(s) failed", failures.len());
    } else {
        println!("\n✅ All test cases passed!");
    }
}
