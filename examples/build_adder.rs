//! Build the adder schematic from integration tests
//!
//! Run with: cargo run --example build_adder

use nucleation::{litematic, SchematicBuilder};

fn create_and_gate_schematic() -> nucleation::UniversalSchematic {
    let blocks = vec![
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
        (
            0,
            1,
            0,
            "minecraft:redstone_wire[power=0,north=none,south=none,east=side,west=side]",
        ),
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
        ),
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
        ),
    ];

    let mut schematic = nucleation::UniversalSchematic::new("and_gate".to_string());
    for (x, y, z, block_str) in blocks {
        match nucleation::UniversalSchematic::parse_block_string(block_str) {
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

fn create_xor_gate_schematic() -> nucleation::UniversalSchematic {
    let blocks = vec![
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
        (
            0,
            1,
            0,
            "minecraft:redstone_wire[power=0,north=none,south=none,east=side,west=side]",
        ),
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
        ),
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
        ),
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

    let mut schematic = nucleation::UniversalSchematic::new("xor_gate".to_string());
    for (x, y, z, block_str) in blocks {
        match nucleation::UniversalSchematic::parse_block_string(block_str) {
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

fn create_adder_schematic() -> nucleation::UniversalSchematic {
    // Level 1: Get basic gates (already implemented)
    let and_gate = create_and_gate_schematic();
    let xor_gate = create_xor_gate_schematic();

    // Level 2: Build half-adder from XOR (sum) and AND (carry)
    // Half-adder: Sum = A XOR B, Carry = A AND B
    let half_adder = SchematicBuilder::new()
        .name("half_adder")
        .map_schematic('X', xor_gate.clone()) // XOR for sum
        .map_schematic('A', and_gate.clone()) // AND for carry
        .map('_', "minecraft:air")
        .layers(&[
            &["X_A"], // XOR and AND side by side with spacing
        ])
        .build()
        .expect("Failed to build half-adder");

    // Level 3: Build full-adder from two half-adders and an OR gate
    // For now, we'll use a simplified version with just the half-adders
    let full_adder = SchematicBuilder::new()
        .name("full_adder")
        .map_schematic('H', half_adder)
        .map_schematic('A', and_gate.clone()) // Additional AND for carry
        .map('_', "minecraft:air")
        .layers(&[
            &["H_H_A"], // Two half-adders and an AND gate
        ])
        .build()
        .expect("Failed to build full-adder");

    // Level 4: Build 4-bit adder from four full-adders
    SchematicBuilder::new()
        .name("4bit_adder")
        .map_schematic('F', full_adder)
        .map(
            'w',
            "minecraft:redstone_wire[power=0,east=side,west=side,north=none,south=none]",
        )
        .layers(&[
            &["FwFwFwF"], // 4 full-adders with wiring
        ])
        .build()
        .expect("Failed to build 4-bit adder")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Building 4-bit adder schematic...\n");

    // Build the adder
    let adder = create_adder_schematic();

    // Get dimensions
    let (width, height, depth) = adder.default_region.size;
    println!("✅ Adder built successfully!");
    println!("   Name: {}", adder.default_region.name);
    println!("   Dimensions: {}x{}x{}", width, height, depth);

    // Count blocks
    let block_types = adder.count_block_types();
    let total_blocks: usize = block_types.values().sum();
    println!("   Total blocks: {}", total_blocks);
    println!("   Unique block types: {}", block_types.len());

    // Save to file
    let output_path = "4bit_adder.litematic";
    let bytes = litematic::to_litematic(&adder)?;
    std::fs::write(output_path, bytes)?;

    println!("\n💾 Saved to: {}", output_path);
    println!("   You can now load this in Minecraft with Litematica mod!");

    println!("\n📊 Block breakdown:");
    let mut sorted_blocks: Vec<_> = block_types.iter().collect();
    sorted_blocks.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
    for (block, count) in sorted_blocks.iter().take(10) {
        println!("   {:4}x {}", count, block);
    }

    println!("\n🎯 Structure:");
    println!("   Level 1: AND gate + XOR gate");
    println!("   Level 2: Half-adder (XOR + AND)");
    println!("   Level 3: Full-adder (2x Half-adder + AND)");
    println!("   Level 4: 4-bit adder (4x Full-adder + wiring)");

    Ok(())
}
