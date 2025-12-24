use nucleation::{BlockState, UniversalSchematic};

/// Test for the bounds bug where blocks at the maximum boundary of a region
/// were not being included in chunk retrieval due to an off-by-one error.
///
/// The bug occurred because:
/// - BoundingBox.max is INCLUSIVE (blocks exist at max coordinates)
/// - Rust ranges (start..end) are EXCLUSIVE on the end
/// - So blocks at region_bbox.max.z were being skipped
#[test]
fn test_chunk_blocks_indices_includes_max_boundary() {
    println!("\n==========================================");
    println!("Test: Chunk Blocks Indices Includes Max Boundary");
    println!("==========================================");

    let mut schematic = UniversalSchematic::new("BoundsTest".to_string());

    // Create blocks in all 6 directions to test all boundaries
    // Specifically test +z direction which was failing
    let test_limit = 5;

    // Place one block at origin
    schematic.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));

    // Then place blocks in each direction (excluding origin which we already placed)
    for i in 1..test_limit {
        // +x direction
        schematic.set_block(
            i,
            0,
            0,
            BlockState::new("minecraft:red_concrete".to_string()),
        );
        // -x direction
        schematic.set_block(
            -i,
            0,
            0,
            BlockState::new("minecraft:pink_concrete".to_string()),
        );
        // +y direction
        schematic.set_block(
            0,
            i,
            0,
            BlockState::new("minecraft:green_concrete".to_string()),
        );
        // -y direction
        schematic.set_block(
            0,
            -i,
            0,
            BlockState::new("minecraft:lime_concrete".to_string()),
        );
        // +z direction (this was the failing case)
        schematic.set_block(
            0,
            0,
            i,
            BlockState::new("minecraft:blue_concrete".to_string()),
        );
        // -z direction
        schematic.set_block(
            0,
            0,
            -i,
            BlockState::new("minecraft:cyan_concrete".to_string()),
        );
    }

    // Get tight bounds to understand the region
    let tight_bounds = schematic
        .get_tight_bounds()
        .expect("Should have tight bounds");
    println!(
        "Tight bounds: min={:?}, max={:?}",
        tight_bounds.min, tight_bounds.max
    );

    // The region should span from -(test_limit-1) to +(test_limit-1) in each direction
    assert_eq!(
        tight_bounds.min,
        (-(test_limit - 1), -(test_limit - 1), -(test_limit - 1))
    );
    assert_eq!(
        tight_bounds.max,
        (test_limit - 1, test_limit - 1, test_limit - 1)
    );

    // Test retrieving chunks that include the maximum boundaries
    // Use chunk size that covers the entire region
    let chunk_width = (tight_bounds.max.0 - tight_bounds.min.0 + 1) as i32;
    let chunk_height = (tight_bounds.max.1 - tight_bounds.min.1 + 1) as i32;
    let chunk_length = (tight_bounds.max.2 - tight_bounds.min.2 + 1) as i32;

    println!(
        "Chunk dimensions: {}x{}x{}",
        chunk_width, chunk_height, chunk_length
    );

    // Get all blocks in the region using get_chunk_blocks_indices
    let blocks = schematic.get_chunk_blocks_indices(
        tight_bounds.min.0,
        tight_bounds.min.1,
        tight_bounds.min.2,
        chunk_width,
        chunk_height,
        chunk_length,
    );

    println!("Retrieved {} blocks", blocks.len());

    // Verify all expected blocks are present
    // 1 block at origin + (test_limit-1) blocks in each of 6 directions = 1 + 6*(test_limit-1)
    let expected_count: usize = (1 + 6 * (test_limit - 1)) as usize;
    assert_eq!(
        blocks.len(),
        expected_count,
        "Should retrieve all {} blocks (1 at origin + {} blocks in each of 6 directions)",
        expected_count,
        test_limit - 1
    );

    // Specifically verify blocks at maximum boundaries exist
    let mut found_max_x = false;
    let mut found_max_y = false;
    let mut found_max_z = false;
    let mut found_min_x = false;
    let mut found_min_y = false;
    let mut found_min_z = false;

    for (pos, _palette_idx) in &blocks {
        if pos.x == tight_bounds.max.0 {
            found_max_x = true;
        }
        if pos.y == tight_bounds.max.1 {
            found_max_y = true;
        }
        if pos.z == tight_bounds.max.2 {
            found_max_z = true;
        }
        if pos.x == tight_bounds.min.0 {
            found_min_x = true;
        }
        if pos.y == tight_bounds.min.1 {
            found_min_y = true;
        }
        if pos.z == tight_bounds.min.2 {
            found_min_z = true;
        }
    }

    assert!(
        found_max_x,
        "Should find block at max X boundary ({})",
        tight_bounds.max.0
    );
    assert!(
        found_max_y,
        "Should find block at max Y boundary ({})",
        tight_bounds.max.1
    );
    assert!(
        found_max_z,
        "Should find block at max Z boundary ({}) - THIS WAS THE BUG!",
        tight_bounds.max.2
    );
    assert!(
        found_min_x,
        "Should find block at min X boundary ({})",
        tight_bounds.min.0
    );
    assert!(
        found_min_y,
        "Should find block at min Y boundary ({})",
        tight_bounds.min.1
    );
    assert!(
        found_min_z,
        "Should find block at min Z boundary ({})",
        tight_bounds.min.2
    );

    println!("✅ All boundary blocks found correctly!");
}

/// Test with a smaller region to verify the fix works for edge cases
#[test]
fn test_chunk_blocks_indices_small_region() {
    println!("\n==========================================");
    println!("Test: Small Region Boundary Test");
    println!("==========================================");

    let mut schematic = UniversalSchematic::new("SmallBoundsTest".to_string());

    // Create just 2 blocks: one at origin, one at +z=1
    schematic.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));
    schematic.set_block(0, 0, 1, BlockState::new("minecraft:dirt".to_string()));

    let tight_bounds = schematic
        .get_tight_bounds()
        .expect("Should have tight bounds");
    println!(
        "Tight bounds: min={:?}, max={:?}",
        tight_bounds.min, tight_bounds.max
    );

    // Get blocks for a chunk that exactly covers the region
    let blocks = schematic.get_chunk_blocks_indices(
        tight_bounds.min.0,
        tight_bounds.min.1,
        tight_bounds.min.2,
        1, // width
        1, // height
        2, // length (from z=0 to z=1)
    );

    println!("Retrieved {} blocks", blocks.len());
    assert_eq!(blocks.len(), 2, "Should retrieve both blocks");

    // Verify the block at z=1 (max boundary) is included
    let found_z1 = blocks.iter().any(|(pos, _)| pos.z == 1);
    assert!(found_z1, "Should find block at z=1 (max boundary)");

    println!("✅ Small region test passed!");
}

/// Test with blocks at negative coordinates
#[test]
fn test_chunk_blocks_indices_negative_coords() {
    println!("\n==========================================");
    println!("Test: Negative Coordinates Boundary Test");
    println!("==========================================");

    let mut schematic = UniversalSchematic::new("NegativeBoundsTest".to_string());

    // Create blocks from z=-2 to z=0
    for z in -2..=0 {
        schematic.set_block(0, 0, z, BlockState::new("minecraft:stone".to_string()));
    }

    let tight_bounds = schematic
        .get_tight_bounds()
        .expect("Should have tight bounds");
    println!(
        "Tight bounds: min={:?}, max={:?}",
        tight_bounds.min, tight_bounds.max
    );

    // Get blocks for a chunk covering the region
    let blocks = schematic.get_chunk_blocks_indices(
        tight_bounds.min.0,
        tight_bounds.min.1,
        tight_bounds.min.2,
        1, // width
        1, // height
        3, // length (from z=-2 to z=0, inclusive)
    );

    println!("Retrieved {} blocks", blocks.len());
    assert_eq!(blocks.len(), 3, "Should retrieve all 3 blocks");

    // Verify block at z=0 (max boundary) is included
    let found_z0 = blocks.iter().any(|(pos, _)| pos.z == 0);
    assert!(found_z0, "Should find block at z=0 (max boundary)");

    // Verify block at z=-2 (min boundary) is included
    let found_z_minus2 = blocks.iter().any(|(pos, _)| pos.z == -2);
    assert!(found_z_minus2, "Should find block at z=-2 (min boundary)");

    println!("✅ Negative coordinates test passed!");
}
