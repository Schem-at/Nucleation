use nucleation::SchematicBuilder;

fn main() {
    // Single full adder
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

    println!("Single full adder:");
    println!("  Full size: {:?}", full_adder.default_region.size);
    if let Some(tight) = full_adder.default_region.get_tight_bounds() {
        println!("  Tight bounds: min={:?}, max={:?}", tight.min, tight.max);
        println!("  Tight dimensions: {:?}", tight.get_dimensions());
    }

    // Stack 4
    let four_adders = SchematicBuilder::new()
        .name("four_bit_adder")
        .map_schematic('A', full_adder)
        .layers(&[&["AAAA"]])
        .build()
        .expect("Failed to build 4-bit adder");

    println!("\n4 full adders stacked:");
    println!("  Full size: {:?}", four_adders.default_region.size);
    if let Some(tight) = four_adders.default_region.get_tight_bounds() {
        println!("  Tight bounds: min={:?}, max={:?}", tight.min, tight.max);
        println!("  Tight dimensions: {:?}", tight.get_dimensions());
        let dims = tight.get_dimensions();
        println!("  Expected X: 40 (10×4), Actual X: {}", dims.0);
        if dims.0 == 40 {
            println!("  ✅ Width is exactly 40 as expected!");
        } else {
            println!("  ❌ Width mismatch!");
        }
    }

    let block_types = four_adders.count_block_types();
    let non_air: usize = block_types
        .iter()
        .filter(|(block, _)| !block.to_string().contains("air"))
        .map(|(_, count)| count)
        .sum();
    println!("  Non-air blocks: {}", non_air);
}
