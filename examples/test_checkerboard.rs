use nucleation::SchematicBuilder;

fn main() {
    // Create a 10x1x10 white concrete block
    let white_block = SchematicBuilder::new()
        .name("white_10x10")
        .map('W', "minecraft:white_concrete")
        .layers(&[&[
            "WWWWWWWWWW",
            "WWWWWWWWWW",
            "WWWWWWWWWW",
            "WWWWWWWWWW",
            "WWWWWWWWWW",
            "WWWWWWWWWW",
            "WWWWWWWWWW",
            "WWWWWWWWWW",
            "WWWWWWWWWW",
            "WWWWWWWWWW",
        ]])
        .build()
        .expect("Failed to build white block");

    println!("White block:");
    println!("  Full size: {:?}", white_block.default_region.size);
    if let Some(tight) = white_block.default_region.get_tight_bounds() {
        println!("  Tight bounds: min={:?}, max={:?}", tight.min, tight.max);
        println!("  Tight dimensions: {:?}", tight.get_dimensions());
    }

    // Create a 10x1x10 black concrete block
    let black_block = SchematicBuilder::new()
        .name("black_10x10")
        .map('B', "minecraft:black_concrete")
        .layers(&[&[
            "BBBBBBBBBB",
            "BBBBBBBBBB",
            "BBBBBBBBBB",
            "BBBBBBBBBB",
            "BBBBBBBBBB",
            "BBBBBBBBBB",
            "BBBBBBBBBB",
            "BBBBBBBBBB",
            "BBBBBBBBBB",
            "BBBBBBBBBB",
        ]])
        .build()
        .expect("Failed to build black block");

    println!("\nBlack block:");
    println!("  Full size: {:?}", black_block.default_region.size);
    if let Some(tight) = black_block.default_region.get_tight_bounds() {
        println!("  Tight bounds: min={:?}, max={:?}", tight.min, tight.max);
        println!("  Tight dimensions: {:?}", tight.get_dimensions());
    }

    // Now create a 4x4 checkerboard
    let checkerboard = SchematicBuilder::new()
        .name("checkerboard_4x4")
        .map_schematic('W', white_block)
        .map_schematic('B', black_block)
        .layers(&[&["WBWB", "BWBW", "WBWB", "BWBW"]])
        .build()
        .expect("Failed to build checkerboard");

    println!("\n4x4 Checkerboard:");
    println!("  Full size: {:?}", checkerboard.default_region.size);
    if let Some(tight) = checkerboard.default_region.get_tight_bounds() {
        println!("  Tight bounds: min={:?}, max={:?}", tight.min, tight.max);
        println!("  Tight dimensions: {:?}", tight.get_dimensions());
    }

    let block_types = checkerboard.count_block_types();
    println!("  Block types: {:?}", block_types.len());
    for (block, count) in &block_types {
        if !block.to_string().contains("air") {
            println!("    {}: {}", block, count);
        }
    }

    // Check specific positions
    println!("\n  Checking positions:");
    println!(
        "    (0, 0, 0) = {:?}",
        checkerboard.get_block(0, 0, 0).map(|b| b.name.as_str())
    );
    println!(
        "    (10, 0, 0) = {:?}",
        checkerboard.get_block(10, 0, 0).map(|b| b.name.as_str())
    );
    println!(
        "    (20, 0, 0) = {:?}",
        checkerboard.get_block(20, 0, 0).map(|b| b.name.as_str())
    );
    println!(
        "    (30, 0, 0) = {:?}",
        checkerboard.get_block(30, 0, 0).map(|b| b.name.as_str())
    );

    println!(
        "\n    (0, 0, 10) = {:?}",
        checkerboard.get_block(0, 0, 10).map(|b| b.name.as_str())
    );
    println!(
        "    (10, 0, 10) = {:?}",
        checkerboard.get_block(10, 0, 10).map(|b| b.name.as_str())
    );
    println!(
        "    (20, 0, 10) = {:?}",
        checkerboard.get_block(20, 0, 10).map(|b| b.name.as_str())
    );
    println!(
        "    (30, 0, 10) = {:?}",
        checkerboard.get_block(30, 0, 10).map(|b| b.name.as_str())
    );

    println!("\n✅ Expected: 40x40 with white_concrete and black_concrete");
    println!("   Actual tight bounds should be (40, 1, 40)");
}
