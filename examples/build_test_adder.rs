//! Build the exact adder from integration_tests.rs
//!
//! Run with: cargo run --example build_test_adder

use nucleation::{litematic, SchematicBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Building adder from integration tests...\n");

    // This is the EXACT code from create_adder_schematic() in integration_tests.rs
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
    .expect("Failed to parse template")
    .build()
    .expect("Failed to build full adder");

    let four_bit_adder = SchematicBuilder::new()
        .name("four_bit_adder")
        .map_schematic('A', full_adder)
        .map('_', "minecraft:air")
        .layers(&[&["AAAA"]])
        .build()
        .expect("Failed to build 4-bit adder");

    // Get dimensions
    let (width, height, depth) = four_bit_adder.default_region.size;
    println!("✅ 4-bit adder built successfully!");
    println!("   Dimensions: {}x{}x{}", width, height, depth);

    // Count blocks
    let block_types = four_bit_adder.count_block_types();
    let total_blocks: usize = block_types.values().sum();
    let non_air: usize = block_types
        .iter()
        .filter(|(block, _)| !block.to_string().contains("air"))
        .map(|(_, count)| count)
        .sum();

    println!("   Total blocks: {}", total_blocks);
    println!("   Non-air blocks: {}", non_air);
    println!("   Unique block types: {}", block_types.len());

    // Save to file
    let output_path = "tests/fixtures/test_adder.litematic";
    let bytes = litematic::to_litematic(&four_bit_adder)?;
    std::fs::write(output_path, bytes)?;

    println!("\n💾 Saved to: {}", output_path);
    println!("   Load this in Minecraft with Litematica!");

    println!("\n📊 Block breakdown (top 15):");
    let mut sorted_blocks: Vec<_> = block_types.iter().collect();
    sorted_blocks.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
    for (block, count) in sorted_blocks.iter().take(15) {
        let block_str = block.to_string();
        if !block_str.contains("air") {
            println!("   {:4}x {}", count, block_str);
        }
    }

    Ok(())
}
