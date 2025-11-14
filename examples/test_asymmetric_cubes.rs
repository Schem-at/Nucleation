use nucleation::SchematicBuilder;

fn main() {
    // Create a 9x10x11 cube filled with red concrete
    let mut red_layers = Vec::new();
    for _ in 0..10 {
        // 10 layers (Y)
        let mut layer = Vec::new();
        for _ in 0..11 {
            // 11 rows (Z)
            layer.push("RRRRRRRRR".to_string()); // 9 columns (X)
        }
        red_layers.push(layer);
    }

    let red_layers_ref: Vec<Vec<&str>> = red_layers
        .iter()
        .map(|v| v.iter().map(|s| s.as_str()).collect())
        .collect();
    let red_layers_slice: Vec<&[&str]> = red_layers_ref.iter().map(|v| v.as_slice()).collect();

    let red_cube = SchematicBuilder::new()
        .name("red_9x10x11")
        .map('R', "minecraft:red_concrete")
        .layers(&red_layers_slice)
        .build()
        .expect("Failed to build red cube");

    println!("Red cube (9x10x11):");
    println!("  Full size: {:?}", red_cube.default_region.size);
    if let Some(tight) = red_cube.default_region.get_tight_bounds() {
        println!("  Tight bounds: min={:?}, max={:?}", tight.min, tight.max);
        println!("  Tight dimensions: {:?}", tight.get_dimensions());
    }
    let block_types = red_cube.count_block_types();
    let red_count = block_types
        .iter()
        .filter(|(block, _)| block.to_string().contains("red_concrete"))
        .map(|(_, count)| count)
        .sum::<usize>();
    println!(
        "  Red concrete blocks: {} (expected: {})",
        red_count,
        9 * 10 * 11
    );

    // Create a 9x10x11 cube filled with blue concrete
    let mut blue_layers = Vec::new();
    for _ in 0..10 {
        // 10 layers (Y)
        let mut layer = Vec::new();
        for _ in 0..11 {
            // 11 rows (Z)
            layer.push("BBBBBBBBB".to_string()); // 9 columns (X)
        }
        blue_layers.push(layer);
    }

    let blue_layers_ref: Vec<Vec<&str>> = blue_layers
        .iter()
        .map(|v| v.iter().map(|s| s.as_str()).collect())
        .collect();
    let blue_layers_slice: Vec<&[&str]> = blue_layers_ref.iter().map(|v| v.as_slice()).collect();

    let blue_cube = SchematicBuilder::new()
        .name("blue_9x10x11")
        .map('B', "minecraft:blue_concrete")
        .layers(&blue_layers_slice)
        .build()
        .expect("Failed to build blue cube");

    println!("\nBlue cube (9x10x11):");
    println!("  Full size: {:?}", blue_cube.default_region.size);
    if let Some(tight) = blue_cube.default_region.get_tight_bounds() {
        println!("  Tight bounds: min={:?}, max={:?}", tight.min, tight.max);
        println!("  Tight dimensions: {:?}", tight.get_dimensions());
    }

    // Stack 4 cubes in a row (X direction)
    let stacked_x = SchematicBuilder::new()
        .name("4_cubes_x")
        .map_schematic('R', red_cube.clone())
        .map_schematic('B', blue_cube.clone())
        .layers(&[
            &["RBRB"], // 4 cubes alternating red/blue
        ])
        .build()
        .expect("Failed to build stacked X");

    println!("\n4 cubes stacked in X direction:");
    println!("  Full size: {:?}", stacked_x.default_region.size);
    if let Some(tight) = stacked_x.default_region.get_tight_bounds() {
        println!("  Tight bounds: min={:?}, max={:?}", tight.min, tight.max);
        println!("  Tight dimensions: {:?}", tight.get_dimensions());
        println!("  Expected X dimension: {} (9×4)", 9 * 4);
    }

    let block_types = stacked_x.count_block_types();
    let red_count = block_types
        .iter()
        .filter(|(block, _)| block.to_string().contains("red_concrete"))
        .map(|(_, count)| count)
        .sum::<usize>();
    let blue_count = block_types
        .iter()
        .filter(|(block, _)| block.to_string().contains("blue_concrete"))
        .map(|(_, count)| count)
        .sum::<usize>();
    println!(
        "  Red blocks: {} (expected: {})",
        red_count,
        9 * 10 * 11 * 2
    );
    println!(
        "  Blue blocks: {} (expected: {})",
        blue_count,
        9 * 10 * 11 * 2
    );
    println!(
        "  Total: {} (expected: {})",
        red_count + blue_count,
        9 * 10 * 11 * 4
    );

    // Stack 3 cubes in Z direction
    let stacked_z = SchematicBuilder::new()
        .name("3_cubes_z")
        .map_schematic('R', red_cube.clone())
        .map_schematic('B', blue_cube.clone())
        .layers(&[&["R", "B", "R"]])
        .build()
        .expect("Failed to build stacked Z");

    println!("\n3 cubes stacked in Z direction:");
    println!("  Full size: {:?}", stacked_z.default_region.size);
    if let Some(tight) = stacked_z.default_region.get_tight_bounds() {
        println!("  Tight bounds: min={:?}, max={:?}", tight.min, tight.max);
        println!("  Tight dimensions: {:?}", tight.get_dimensions());
        println!("  Expected Z dimension: {} (11×3)", 11 * 3);
    }

    // Stack in 2D grid (2x2)
    let grid_2x2 = SchematicBuilder::new()
        .name("2x2_grid")
        .map_schematic('R', red_cube.clone())
        .map_schematic('B', blue_cube.clone())
        .layers(&[&["RB", "BR"]])
        .build()
        .expect("Failed to build 2x2 grid");

    println!("\n2x2 grid:");
    println!("  Full size: {:?}", grid_2x2.default_region.size);
    if let Some(tight) = grid_2x2.default_region.get_tight_bounds() {
        println!("  Tight bounds: min={:?}, max={:?}", tight.min, tight.max);
        println!("  Tight dimensions: {:?}", tight.get_dimensions());
        println!(
            "  Expected dimensions: {}x{}x{} (9×2, 10, 11×2)",
            9 * 2,
            10,
            11 * 2
        );
    }

    let block_types = grid_2x2.count_block_types();
    let total = block_types.values().filter(|&&c| c > 0).sum::<usize>();
    println!("  Total blocks: {} (expected: {})", total, 9 * 10 * 11 * 4);

    println!("\n✅ All asymmetric cube tests complete!");
}
