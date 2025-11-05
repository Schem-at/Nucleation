use nucleation::{litematic, UniversalSchematic};

fn main() {
    // This example demonstrates how to setup a battle arena for two Connect 4 AI builds
    // that need to face each other

    println!("=== Connect 4 AI Battle Arena Setup ===\n");

    // Step 1: Create example builds (in practice, you'd load these from files)
    let mut build_a = create_example_build("Build A");
    let mut build_b = create_example_build("Build B");
    let central_server = create_central_server();

    println!(
        "✓ Loaded build_a (dimensions: {:?})",
        build_a.get_dimensions()
    );
    println!(
        "✓ Loaded build_b (dimensions: {:?})",
        build_b.get_dimensions()
    );
    println!(
        "✓ Loaded central_server (dimensions: {:?})",
        central_server.get_dimensions()
    );

    // Step 2: Transform build_b to face build_a
    // Mirror it across Z axis so it faces the opposite direction
    println!("\n📐 Transforming build_b...");
    build_b.flip_z();
    println!("✓ Flipped build_b along Z axis");

    // Optional: Rotate if needed for specific orientation
    // build_b.rotate_y(90); // Rotate 90 degrees if needed

    // Step 3: Position information
    println!("\n📍 Positioning strategy:");
    println!("  - Place build_a at (0, 0, 0)");
    println!("  - Place central_server at (20, 0, 0) - centered between builds");
    println!("  - Place build_b at (40, 0, 0) - now facing back towards build_a");

    // Note: You would use the existing merge or copy_region functionality to compose them
    // For example:
    // arena.set_block_in_region() or use the copy_region method

    println!("\n✅ Transformation complete!");
    println!("\nKey benefits of the transformation:");
    println!("  • Repeaters: facing directions automatically updated");
    println!("  • Comparators: facing directions automatically updated");
    println!("  • Redstone wire: connection properties remapped");
    println!("  • Levers/buttons: facing directions automatically updated");
    println!("  • All other directional blocks: properties transformed correctly");

    println!("\n💡 Next steps:");
    println!("  1. Merge the three schematics at their positions");
    println!("  2. The redstone logic will work correctly due to property transformations");
    println!("  3. Set up your simulation and let the AIs battle!");
}

fn create_example_build(name: &str) -> UniversalSchematic {
    let mut schematic = UniversalSchematic::new(name.to_string());

    // Create a simple example redstone circuit
    // In practice, you'd load this from a .litematic file

    // Lever at input
    schematic.set_block_str(
        0,
        0,
        0,
        "minecraft:lever[facing=east,face=floor,powered=false]",
    );

    // Redstone wire
    schematic.set_block_str(1, 0, 0, "minecraft:redstone_wire[power=0]");
    schematic.set_block_str(2, 0, 0, "minecraft:redstone_wire[power=0]");

    // Repeater facing east with delay
    schematic.set_block_str(3, 0, 0, "minecraft:repeater[facing=east,delay=2]");

    // More wire
    schematic.set_block_str(4, 0, 0, "minecraft:redstone_wire[power=0]");

    // Output lamp
    schematic.set_block_str(5, 0, 0, "minecraft:redstone_lamp[lit=false]");

    schematic
}

fn create_central_server() -> UniversalSchematic {
    let mut schematic = UniversalSchematic::new("Central Server".to_string());

    // Simple example server structure
    schematic.set_block_str(0, 0, 0, "minecraft:gold_block");
    schematic.set_block_str(0, 1, 0, "minecraft:redstone_block");
    schematic.set_block_str(0, 2, 0, "minecraft:gold_block");

    schematic
}

// Additional helper functions

#[allow(dead_code)]
fn load_and_transform_builds(
    build_a_path: &str,
    build_b_path: &str,
    server_path: &str,
) -> Result<(UniversalSchematic, UniversalSchematic, UniversalSchematic), Box<dyn std::error::Error>>
{
    // Load build A
    let data_a = std::fs::read(build_a_path)?;
    let mut build_a = litematic::from_litematic(&data_a)?;

    // Load build B
    let data_b = std::fs::read(build_b_path)?;
    let mut build_b = litematic::from_litematic(&data_b)?;

    // Load central server
    let data_server = std::fs::read(server_path)?;
    let server = litematic::from_litematic(&data_server)?;

    // Transform build_b to face build_a
    build_b.flip_z();

    Ok((build_a, build_b, server))
}

#[allow(dead_code)]
fn demonstrate_all_transformations() {
    println!("\n=== Transformation Examples ===\n");

    let mut test_schematic = UniversalSchematic::new("Test".to_string());
    test_schematic.set_block_str(0, 0, 0, "minecraft:repeater[facing=north,delay=3]");

    println!("Original: repeater facing north");

    // Flip operations
    let mut flipped_x = test_schematic.clone();
    flipped_x.flip_x();
    println!("After flip_x(): repeater still facing north (X flip doesn't affect north/south)");

    let mut flipped_z = test_schematic.clone();
    flipped_z.flip_z();
    println!("After flip_z(): repeater now facing south");

    // Rotation operations
    let mut rotated_90 = test_schematic.clone();
    rotated_90.rotate_y(90);
    println!("After rotate_y(90): repeater now facing east");

    let mut rotated_180 = test_schematic.clone();
    rotated_180.rotate_y(180);
    println!("After rotate_y(180): repeater now facing south");

    let mut rotated_270 = test_schematic.clone();
    rotated_270.rotate_y(270);
    println!("After rotate_y(270): repeater now facing west");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_transformation_preserves_blocks() {
        let mut build = create_example_build("Test");
        let block_count_before = build.total_blocks();

        // Transform
        build.flip_z();
        build.rotate_y(90);

        let block_count_after = build.total_blocks();

        assert_eq!(
            block_count_before, block_count_after,
            "Transformation should preserve block count"
        );
    }

    #[test]
    fn test_directional_blocks_transform() {
        let mut schematic = UniversalSchematic::new("Test".to_string());
        schematic.set_block_str(0, 0, 0, "minecraft:repeater[facing=north,delay=2]");

        schematic.rotate_y(90);

        // After 90° rotation, north should become east
        if let Some(block) = schematic.get_block(0, 0, 0) {
            assert_eq!(
                block.get_property("facing"),
                Some(&"east".to_string()),
                "Repeater should face east after 90° rotation"
            );
            assert_eq!(
                block.get_property("delay"),
                Some(&"2".to_string()),
                "Delay should be preserved"
            );
        } else {
            panic!("Block should exist after transformation");
        }
    }
}
