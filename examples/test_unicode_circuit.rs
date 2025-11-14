//! Test script for Unicode palette circuit design
//!
//! Run with: cargo run --example test_unicode_circuit

use nucleation::SchematicBuilder;

fn main() -> Result<(), String> {
    println!("🎨 Testing Unicode Palette Circuit...\n");
    println!("✨ Standard palette is loaded by default!\n");

    // Create a half-adder using Unicode characters
    // No need to call .use_standard_palette() - it's automatic!
    let half_adder = SchematicBuilder::new()
        .map('c', "minecraft:gray_concrete") // Override 'c' if needed
        .layers(&[
            // Base layer
            &[
                "cccccccccc",
                "cccccccccc",
                "cccccccccc",
                "cccccccccc",
                "cccccccccc",
                "cccccccccc",
                "cccccccccc",
                "cccccccccc",
                "cccccccccc",
                "cccccccccc",
            ],
            // Logic layer - using Unicode characters from standard palette!
            &[
                "·····│····",
                "·····↑····",
                "··│█←┤█···",
                "·█◀←┬▲▲┐··",
                "──··├──┴←─",
                "·█··↑·····",
                "·▲─←┤█····",
                "·█←┬▲▲┐···",
                "···├──┤···",
                "···│··│···",
            ],
        ])
        .build()?;

    println!("✅ Half-adder built successfully!");

    // Get dimensions from the default region
    let (width, height, depth) = half_adder.default_region.size;
    println!("   Dimensions: {}x{}x{}", width, height, depth);

    // Count blocks
    let block_types = half_adder.count_block_types();
    let total_blocks: usize = block_types.values().sum();
    println!("   Total blocks: {}", total_blocks);
    println!("   Unique block types: {}", block_types.len());

    // Print some sample blocks to verify
    println!("\n🔍 Sample blocks (first 15 non-air):");
    let mut count = 0;
    'outer: for y in 0..height as i32 {
        for z in 0..depth as i32 {
            for x in 0..width as i32 {
                if let Some(block) = half_adder.get_block(x, y, z) {
                    let block_str = block.to_string();
                    if !block_str.contains("air") {
                        println!("   ({:2}, {:2}, {:2}) = {}", x, y, z, block_str);
                        count += 1;
                        if count >= 15 {
                            break 'outer;
                        }
                    }
                }
            }
        }
    }

    // Test with template approach (also uses standard palette by default!)
    println!("\n🧪 Testing template approach...");

    let template_circuit = SchematicBuilder::from_template(
        r#"
# Base layer
█████
█████

# Logic layer
─→─*─
╋╋╋╋╋

[palette]
# No need to define █, ─, →, *, ╋ - they're in the standard palette!
# But you can override them if you want
"#,
    )?
    .build()?;

    println!("✅ Template circuit built successfully!");
    let (w, h, d) = template_circuit.default_region.size;
    println!("   Dimensions: {}x{}x{}", w, h, d);

    // Test simple circuit
    println!("\n🔧 Testing simple repeater chain...");
    let repeater_chain = SchematicBuilder::new()
        .layers(&[
            &["█████"],
            &["─→→→─"], // Wire, 3 repeaters, wire - all from standard palette!
        ])
        .build()?;

    println!("✅ Repeater chain built!");
    let (w, h, d) = repeater_chain.default_region.size;
    println!("   Dimensions: {}x{}x{}", w, h, d);

    // Show what blocks were created
    println!("\n   Blocks in repeater chain:");
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            if let Some(block) = repeater_chain.get_block(x, y, 0) {
                let block_str = block.to_string();
                if !block_str.contains("air") {
                    println!("     ({}, {}, 0) = {}", x, y, block_str);
                }
            }
        }
    }

    println!("\n✨ All tests passed!");
    println!("\n💡 Tip: Standard palette is loaded by default!");
    println!("   Just use Unicode characters directly in your layers.");
    println!("   Override specific characters with .map() if needed.");

    Ok(())
}
