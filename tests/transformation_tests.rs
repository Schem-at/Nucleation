use nucleation::{BlockState, Region, UniversalSchematic};

#[test]
fn test_flip_x_simple() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 1, 1));
    let stone = BlockState::new("minecraft:stone".to_string());
    let diamond = BlockState::new("minecraft:diamond_block".to_string());

    region.set_block(0, 0, 0, stone.clone());
    region.set_block(2, 0, 0, diamond.clone());

    region.flip_x();

    // After flipping X: block at 0 should be at 2, block at 2 should be at 0
    assert_eq!(
        region.get_block(2, 0, 0).map(|b| &b.name),
        Some(&"minecraft:stone".to_string())
    );
    assert_eq!(
        region.get_block(0, 0, 0).map(|b| &b.name),
        Some(&"minecraft:diamond_block".to_string())
    );
}

#[test]
fn test_flip_y_simple() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (1, 3, 1));
    let stone = BlockState::new("minecraft:stone".to_string());
    let diamond = BlockState::new("minecraft:diamond_block".to_string());

    region.set_block(0, 0, 0, stone.clone());
    region.set_block(0, 2, 0, diamond.clone());

    region.flip_y();

    assert_eq!(
        region.get_block(0, 2, 0).map(|b| &b.name),
        Some(&"minecraft:stone".to_string())
    );
    assert_eq!(
        region.get_block(0, 0, 0).map(|b| &b.name),
        Some(&"minecraft:diamond_block".to_string())
    );
}

#[test]
fn test_flip_z_simple() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (1, 1, 3));
    let stone = BlockState::new("minecraft:stone".to_string());
    let diamond = BlockState::new("minecraft:diamond_block".to_string());

    region.set_block(0, 0, 0, stone.clone());
    region.set_block(0, 0, 2, diamond.clone());

    region.flip_z();

    assert_eq!(
        region.get_block(0, 0, 2).map(|b| &b.name),
        Some(&"minecraft:stone".to_string())
    );
    assert_eq!(
        region.get_block(0, 0, 0).map(|b| &b.name),
        Some(&"minecraft:diamond_block".to_string())
    );
}

#[test]
fn test_flip_x_with_directional_blocks() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 1, 1));

    let mut lever_east = BlockState::new("minecraft:lever".to_string());
    lever_east.set_property("facing".to_string(), "east".to_string());

    let mut lever_west = BlockState::new("minecraft:lever".to_string());
    lever_west.set_property("facing".to_string(), "west".to_string());

    region.set_block(0, 0, 0, lever_east.clone());
    region.set_block(2, 0, 0, lever_west.clone());

    region.flip_x();

    // Check that facing directions were flipped
    let block_at_2 = region.get_block(2, 0, 0).unwrap();
    assert_eq!(block_at_2.get_property("facing"), Some(&"west".to_string()));

    let block_at_0 = region.get_block(0, 0, 0).unwrap();
    assert_eq!(block_at_0.get_property("facing"), Some(&"east".to_string()));
}

#[test]
fn test_flip_z_with_directional_blocks() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (1, 1, 3));

    let mut lever_north = BlockState::new("minecraft:lever".to_string());
    lever_north.set_property("facing".to_string(), "north".to_string());

    let mut lever_south = BlockState::new("minecraft:lever".to_string());
    lever_south.set_property("facing".to_string(), "south".to_string());

    region.set_block(0, 0, 0, lever_north.clone());
    region.set_block(0, 0, 2, lever_south.clone());

    region.flip_z();

    let block_at_2 = region.get_block(0, 0, 2).unwrap();
    assert_eq!(
        block_at_2.get_property("facing"),
        Some(&"south".to_string())
    );

    let block_at_0 = region.get_block(0, 0, 0).unwrap();
    assert_eq!(
        block_at_0.get_property("facing"),
        Some(&"north".to_string())
    );
}

#[test]
fn test_rotate_y_90_simple() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 1, 3));
    let stone = BlockState::new("minecraft:stone".to_string());

    region.set_block(0, 0, 0, stone.clone());
    region.set_block(2, 0, 0, stone.clone());

    region.rotate_y(90);

    // After 90-degree rotation around Y:
    // (0,0,0) -> (0,0,2)
    // (2,0,0) -> (0,0,0)
    assert_eq!(
        region.get_block(0, 0, 2).map(|b| &b.name),
        Some(&"minecraft:stone".to_string())
    );
    assert_eq!(
        region.get_block(0, 0, 0).map(|b| &b.name),
        Some(&"minecraft:stone".to_string())
    );
}

#[test]
fn test_rotate_y_180() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 1, 3));
    let stone = BlockState::new("minecraft:stone".to_string());

    region.set_block(0, 0, 0, stone.clone());

    region.rotate_y(180);

    // 180-degree rotation should put block at opposite corner
    assert_eq!(
        region.get_block(2, 0, 2).map(|b| &b.name),
        Some(&"minecraft:stone".to_string())
    );
}

#[test]
fn test_rotate_y_with_directional_blocks() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 1, 2));

    let mut lever_north = BlockState::new("minecraft:lever".to_string());
    lever_north.set_property("facing".to_string(), "north".to_string());

    region.set_block(0, 0, 0, lever_north.clone());

    region.rotate_y(90);

    // North should become East after 90-degree rotation
    let rotated_block = region.get_block(0, 0, 1).unwrap();
    assert_eq!(
        rotated_block.get_property("facing"),
        Some(&"east".to_string())
    );
}

#[test]
fn test_rotate_y_270() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 1, 3));

    let mut lever_north = BlockState::new("minecraft:lever".to_string());
    lever_north.set_property("facing".to_string(), "north".to_string());

    region.set_block(1, 0, 1, lever_north.clone());

    region.rotate_y(270);

    // North -> West after 270-degree rotation (or -90)
    let rotated_block = region.get_block(1, 0, 1).unwrap();
    assert_eq!(
        rotated_block.get_property("facing"),
        Some(&"west".to_string())
    );
}

#[test]
fn test_multiple_transformations() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 3, 3));
    let stone = BlockState::new("minecraft:stone".to_string());

    region.set_block(0, 0, 0, stone.clone());
    region.set_block(2, 2, 2, stone.clone());

    // Apply multiple transformations
    region.flip_x();
    region.rotate_y(90);

    // Should still have 2 non-air blocks
    assert_eq!(region.count_non_air_blocks(), 2);
}

#[test]
fn test_transformation_preserves_volume() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (5, 5, 5));
    let stone = BlockState::new("minecraft:stone".to_string());

    // Fill region with stone
    for x in 0..5 {
        for y in 0..5 {
            for z in 0..5 {
                region.set_block(x, y, z, stone.clone());
            }
        }
    }

    let initial_count = region.count_non_air_blocks();

    region.flip_x();
    assert_eq!(region.count_non_air_blocks(), initial_count);

    region.rotate_y(90);
    assert_eq!(region.count_non_air_blocks(), initial_count);
}

#[test]
fn test_flip_with_axis_property() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 1, 1));

    let mut log_x = BlockState::new("minecraft:oak_log".to_string());
    log_x.set_property("axis".to_string(), "x".to_string());

    region.set_block(1, 0, 0, log_x.clone());

    region.flip_x();

    // X-axis flip shouldn't change x-axis blocks
    let flipped_block = region.get_block(1, 0, 0).unwrap();
    assert_eq!(flipped_block.get_property("axis"), Some(&"x".to_string()));
}

#[test]
fn test_rotate_with_axis_property() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 3, 3));

    let mut log_x = BlockState::new("minecraft:oak_log".to_string());
    log_x.set_property("axis".to_string(), "x".to_string());

    region.set_block(1, 1, 1, log_x.clone());

    region.rotate_y(90);

    // X-axis should become Z-axis after 90-degree Y rotation
    let rotated_block = region.get_block(1, 1, 1).unwrap();
    assert_eq!(rotated_block.get_property("axis"), Some(&"z".to_string()));
}

#[test]
fn test_flip_empty_region() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 3, 3));

    region.flip_x();
    region.flip_y();
    region.flip_z();

    // Should still be empty
    assert_eq!(region.count_non_air_blocks(), 0);
}

#[test]
fn test_rotate_empty_region() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 3, 3));

    region.rotate_y(90);
    region.rotate_y(180);
    region.rotate_y(270);

    // Should still be empty
    assert_eq!(region.count_non_air_blocks(), 0);
}

#[test]
fn test_universal_schematic_flip() {
    let mut schematic = UniversalSchematic::new("Test".to_string());
    let stone = BlockState::new("minecraft:stone".to_string());

    schematic.set_block(0, 0, 0, stone.clone());
    schematic.set_block(5, 0, 0, stone.clone());

    schematic.default_region.flip_x();

    assert_eq!(schematic.default_region.count_non_air_blocks(), 2);
}

#[test]
fn test_rotation_property_standing_sign() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 3, 3));

    let mut sign = BlockState::new("minecraft:standing_sign".to_string());
    sign.set_property("rotation".to_string(), "0".to_string());

    region.set_block(1, 0, 1, sign.clone());

    region.rotate_y(90);

    // Rotation 0 + 90 degrees (4 increments in 0-15 scale) = 4
    let rotated_block = region.get_block(1, 0, 1).unwrap();
    assert_eq!(
        rotated_block.get_property("rotation"),
        Some(&"4".to_string())
    );
}

#[test]
fn test_flip_y_directional() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (1, 3, 1));

    let mut lever_up = BlockState::new("minecraft:lever".to_string());
    lever_up.set_property("facing".to_string(), "up".to_string());

    let mut lever_down = BlockState::new("minecraft:lever".to_string());
    lever_down.set_property("facing".to_string(), "down".to_string());

    region.set_block(0, 0, 0, lever_up.clone());
    region.set_block(0, 2, 0, lever_down.clone());

    region.flip_y();

    let block_at_2 = region.get_block(0, 2, 0).unwrap();
    assert_eq!(block_at_2.get_property("facing"), Some(&"down".to_string()));

    let block_at_0 = region.get_block(0, 0, 0).unwrap();
    assert_eq!(block_at_0.get_property("facing"), Some(&"up".to_string()));
}

#[test]
fn test_rotate_x_simple() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (1, 3, 3));
    let stone = BlockState::new("minecraft:stone".to_string());

    region.set_block(0, 0, 0, stone.clone());

    region.rotate_x(90);

    // Y and Z swap with rotation
    assert_eq!(region.count_non_air_blocks(), 1);
}

#[test]
fn test_rotate_z_simple() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 3, 1));
    let stone = BlockState::new("minecraft:stone".to_string());

    region.set_block(0, 0, 0, stone.clone());

    region.rotate_z(90);

    // X and Y swap with rotation
    assert_eq!(region.count_non_air_blocks(), 1);
}

#[test]
fn test_redstone_circuit_flip() {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (5, 1, 1));

    // Create a simple redstone circuit: lever -> wire -> lamp
    let mut lever = BlockState::new("minecraft:lever".to_string());
    lever.set_property("facing".to_string(), "east".to_string());
    lever.set_property("powered".to_string(), "false".to_string());

    let wire = BlockState::new("minecraft:redstone_wire".to_string());

    let mut lamp = BlockState::new("minecraft:redstone_lamp".to_string());
    lamp.set_property("lit".to_string(), "false".to_string());

    region.set_block(0, 0, 0, lever.clone());
    region.set_block(1, 0, 0, wire.clone());
    region.set_block(2, 0, 0, wire.clone());
    region.set_block(3, 0, 0, wire.clone());
    region.set_block(4, 0, 0, lamp.clone());

    region.flip_x();

    // All blocks should still be present
    assert_eq!(region.count_non_air_blocks(), 5);

    // Lever should have flipped facing
    let lever_block = region.get_block(4, 0, 0).unwrap();
    assert_eq!(
        lever_block.get_property("facing"),
        Some(&"west".to_string())
    );
}
