use nucleation::block_entity::BlockEntity;
use nucleation::schematic;
use nucleation::{BlockState, Region, UniversalSchematic};
use std::fs;

#[test]
fn test_basic_dimension_expansion_bug() {
    println!("\n========================================");
    println!("Test 1: Basic Dimension Expansion Bug");
    println!("========================================");

    let mut schematic = UniversalSchematic::new("Test".to_string());

    // Set first block
    let block1 = BlockState::new("minecraft:barrel".to_string())
        .with_property("facing".to_string(), "north".to_string());
    schematic.set_block(0, 1, 0, block1);
    let dims1 = schematic.get_dimensions();
    println!("After first block at (0, 1, 0): {:?}", dims1);

    // Set second block at negative Y - this triggers expansion
    let block2 = BlockState::new("minecraft:barrel".to_string())
        .with_property("facing".to_string(), "north".to_string());
    schematic.set_block(0, -1, 0, block2);
    let dims2 = schematic.get_dimensions();
    println!("After second block at (0, -1, 0): {:?}", dims2);

    // Expected: should be around (1, 3, 1) - from y=-1 to y=1
    // Actual: Will be something like (1, 67, 1) due to aggressive expansion

    println!("Expected dimensions: approximately (1, 3, 1)");
    println!("Actual dimensions show aggressive pre-allocation!");

    // Check tight bounds
    if let Some(tight_bounds) = schematic.get_tight_bounds() {
        let tight_dims = tight_bounds.get_dimensions();
        println!("Tight bounds (actual content): {:?}", tight_dims);
        assert_eq!(tight_dims, (1, 3, 1), "Tight bounds should be (1, 3, 1)");
    }

    // The allocated dimensions should be much larger
    assert!(
        dims2.1 > 10,
        "Allocated dimensions should be large due to expansion"
    );

    // Now export to schematic and check what dimensions are exported
    let schematic_bytes = schematic::to_schematic(&schematic).expect("Failed to export");

    // Reload it
    let reloaded = schematic::from_schematic(&schematic_bytes).expect("Failed to reload");
    let reloaded_dims = reloaded.get_dimensions();
    println!("Reloaded schematic dimensions: {:?}", reloaded_dims);

    // BUG: The reloaded schematic will have the large expanded dimensions,
    // not the tight bounds! This causes blocks to appear high in the sky when pasted.
    println!("\nBUG: Exported schematic has expanded dimensions, not tight bounds!");
}

#[test]
fn test_block_entity_positions_after_expansion() {
    println!("\n=============================================");
    println!("Test 2: Block Entity Positions After Export");
    println!("=============================================");

    let mut schematic = UniversalSchematic::new("Test".to_string());

    // Set block with block entity at (0, 1, 0)
    let block1 = BlockState::new("minecraft:barrel".to_string())
        .with_property("facing".to_string(), "north".to_string());
    schematic.set_block(0, 1, 0, block1.clone());

    let be1 = BlockEntity::new("minecraft:barrel".to_string(), (0, 1, 0));
    schematic.add_block_entity(be1);

    println!(
        "Dimensions after first block: {:?}",
        schematic.get_dimensions()
    );

    // Set block at negative Y - triggers expansion
    let block2 = BlockState::new("minecraft:barrel".to_string())
        .with_property("facing".to_string(), "north".to_string());
    schematic.set_block(0, -1, 0, block2.clone());

    let be2 = BlockEntity::new("minecraft:barrel".to_string(), (0, -1, 0));
    schematic.add_block_entity(be2);

    println!(
        "Dimensions after second block: {:?}",
        schematic.get_dimensions()
    );

    // Check block entities before export
    let entities_before = schematic.get_block_entities_as_list();
    println!("\nBlock entities before export:");
    for be in &entities_before {
        println!("  {} at {:?}", be.id, be.position);
    }

    // Export and reload
    let schematic_bytes = schematic::to_schematic(&schematic).expect("Failed to export");
    let reloaded = schematic::from_schematic(&schematic_bytes).expect("Failed to reload");

    // Check block entities after reload
    let entities_after = reloaded.get_block_entities_as_list();
    println!("\nBlock entities after export/reload:");
    for be in &entities_after {
        println!("  {} at {:?}", be.id, be.position);
    }

    // Verify positions are preserved
    assert_eq!(
        entities_before.len(),
        entities_after.len(),
        "Block entity count should be preserved"
    );

    // Check that the block entities can be retrieved at their original positions
    let block_at_0_1_0 = reloaded.get_block(0, 1, 0);
    let block_at_0_neg1_0 = reloaded.get_block(0, -1, 0);

    println!("\nBlock retrieval after reload:");
    println!(
        "  Block at (0, 1, 0): {:?}",
        block_at_0_1_0.map(|b| &b.name)
    );
    println!(
        "  Block at (0, -1, 0): {:?}",
        block_at_0_neg1_0.map(|b| &b.name)
    );

    // BUG: Block entities might be at wrong positions due to region merge issues
}

#[test]
fn test_tight_bounds_vs_allocated_bounds() {
    println!("\n============================================");
    println!("Test 3: Tight Bounds vs Allocated Bounds");
    println!("============================================");

    let mut schematic = UniversalSchematic::new("Test".to_string());

    // Place just two blocks close together
    schematic.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));
    schematic.set_block(1, 0, 0, BlockState::new("minecraft:stone".to_string()));

    let allocated_dims = schematic.get_dimensions();
    let tight_dims = schematic.get_tight_dimensions();

    println!("Allocated dimensions: {:?}", allocated_dims);
    println!("Tight dimensions: {:?}", tight_dims);

    // Tight dimensions should be exactly (2, 1, 1)
    assert_eq!(
        tight_dims,
        (2, 1, 1),
        "Tight dimensions should be (2, 1, 1)"
    );

    // Allocated dimensions will be larger due to initial allocation
    println!("\nAllocated dimensions are larger due to Region::new() initial size");

    // Now trigger expansion with a far block
    schematic.set_block(
        10,
        10,
        10,
        BlockState::new("minecraft:diamond_block".to_string()),
    );

    let allocated_dims_after = schematic.get_dimensions();
    let tight_dims_after = schematic.get_tight_dimensions();

    println!("\nAfter placing block at (10, 10, 10):");
    println!("Allocated dimensions: {:?}", allocated_dims_after);
    println!("Tight dimensions: {:?}", tight_dims_after);

    // Tight dimensions should be (11, 11, 11) - from (0,0,0) to (10,10,10) inclusive
    assert_eq!(
        tight_dims_after,
        (11, 11, 11),
        "Tight dimensions should be (11, 11, 11)"
    );

    // Allocated dimensions should be much larger
    assert!(
        allocated_dims_after.0 >= 64,
        "Allocated X should be at least 64 due to expansion"
    );
    assert!(
        allocated_dims_after.1 >= 64,
        "Allocated Y should be at least 64 due to expansion"
    );
    assert!(
        allocated_dims_after.2 >= 64,
        "Allocated Z should be at least 64 due to expansion"
    );

    println!("\nBUG: get_dimensions() returns allocated size, not actual content size!");
    println!("Should potentially default to tight_dimensions for user-facing API");
}

#[test]
fn test_export_uses_allocated_not_tight_bounds() {
    println!("\n=================================================");
    println!("Test 4: Export Uses Allocated Bounds, Not Tight");
    println!("=================================================");

    let mut schematic = UniversalSchematic::new("Test".to_string());

    // Create a small 2x2x2 structure
    for x in 0..=1 {
        for y in 0..=1 {
            for z in 0..=1 {
                schematic.set_block(x, y, z, BlockState::new("minecraft:stone".to_string()));
            }
        }
    }

    println!("Created 2x2x2 structure");

    // Trigger expansion by placing a far block, then removing it
    schematic.set_block(50, 50, 50, BlockState::new("minecraft:air".to_string()));

    let allocated_before = schematic.get_dimensions();
    let tight_before = schematic.get_tight_dimensions();

    println!("Before export:");
    println!("  Allocated: {:?}", allocated_before);
    println!("  Tight: {:?}", tight_before);

    // Export and reload
    let bytes = schematic::to_schematic(&schematic).expect("Failed to export");
    let reloaded = schematic::from_schematic(&bytes).expect("Failed to reload");

    let dims_after_reload = reloaded.get_dimensions();

    println!("\nAfter export/reload:");
    println!("  Dimensions: {:?}", dims_after_reload);

    // The reloaded schematic will have large dimensions, not tight (2, 2, 2)
    println!("\nBUG: Exported schematic includes all allocated space, not just actual content!");
    println!("When pasted in-game, this causes massive empty space and wrong positioning");
}

#[test]
fn test_negative_coordinate_expansion_issue() {
    println!("\n============================================");
    println!("Test 5: Negative Coordinate Expansion");
    println!("============================================");

    let mut region = Region::new("Test".to_string(), (0, 0, 0), (1, 1, 1));

    println!(
        "Initial region: pos={:?}, size={:?}",
        region.position, region.size
    );

    // Set block at positive Y
    region.set_block(0, 5, 0, BlockState::new("minecraft:stone".to_string()));
    println!(
        "After block at (0, 5, 0): pos={:?}, size={:?}",
        region.position, region.size
    );

    // Set block at negative Y - this triggers expansion
    region.set_block(0, -5, 0, BlockState::new("minecraft:dirt".to_string()));
    println!(
        "After block at (0, -5, 0): pos={:?}, size={:?}",
        region.position, region.size
    );

    // Check tight bounds
    if let Some(tight_bounds) = region.get_tight_bounds() {
        println!(
            "Tight bounds: min={:?}, max={:?}",
            tight_bounds.min, tight_bounds.max
        );
        println!("Tight dimensions: {:?}", tight_bounds.get_dimensions());
    }

    // Verify blocks are accessible
    let block_pos = region.get_block(0, 5, 0);
    let block_neg = region.get_block(0, -5, 0);

    println!("\nBlock retrieval:");
    println!("  At (0, 5, 0): {:?}", block_pos.map(|b| &b.name));
    println!("  At (0, -5, 0): {:?}", block_neg.map(|b| &b.name));

    assert!(block_pos.is_some(), "Block at (0, 5, 0) should exist");
    assert!(block_neg.is_some(), "Block at (0, -5, 0) should exist");

    // The actual content spans from -5 to 5 in Y (11 blocks height)
    // But allocated size is much larger
    let allocated_dims = region.get_dimensions();
    let tight_dims = region.get_tight_dimensions();

    println!("\nDimension comparison:");
    println!("  Allocated: {:?}", allocated_dims);
    println!("  Tight: {:?}", tight_dims);

    assert_eq!(
        tight_dims.1, 11,
        "Tight Y dimension should be 11 (from -5 to 5 inclusive)"
    );
    assert!(
        allocated_dims.1 > 20,
        "Allocated Y should be much larger due to expansion strategy"
    );
}

#[test]
fn test_get_merged_region_preserves_block_entities() {
    println!("\n===================================================");
    println!("Test 6: Merged Region Block Entity Preservation");
    println!("===================================================");

    let mut schematic = UniversalSchematic::new("Test".to_string());

    // Add blocks with block entities at various positions
    let positions = vec![(0, 0, 0), (5, 5, 5), (-3, 2, 4)];

    for &(x, y, z) in &positions {
        let block = BlockState::new("minecraft:chest".to_string());
        schematic.set_block(x, y, z, block);

        let be = BlockEntity::new("minecraft:chest".to_string(), (x, y, z));
        schematic.add_block_entity(be);
    }

    println!(
        "Added {} block entities at positions: {:?}",
        positions.len(),
        positions
    );

    // Get dimensions before merge
    let dims_before = schematic.get_dimensions();
    let bbox_before = schematic.get_bounding_box();
    println!(
        "Before merge: dims={:?}, bbox=({:?} to {:?})",
        dims_before, bbox_before.min, bbox_before.max
    );

    // Get merged region (this is what gets exported)
    let merged = schematic.get_merged_region();

    println!("\nMerged region:");
    println!("  Position: {:?}", merged.position);
    println!("  Size: {:?}", merged.size);
    println!("  Block entities: {}", merged.block_entities.len());

    // Check that all block entities are present in merged region
    for &(x, y, z) in &positions {
        let be = merged.block_entities.get(&(x, y, z));
        println!(
            "  Block entity at {:?}: {}",
            (x, y, z),
            if be.is_some() {
                "✓ Found"
            } else {
                "✗ MISSING"
            }
        );

        assert!(
            be.is_some(),
            "Block entity at {:?} should be present in merged region",
            (x, y, z)
        );
    }

    // Now check what happens when we export and reload
    let bytes = schematic::to_schematic(&schematic).expect("Failed to export");
    let reloaded = schematic::from_schematic(&bytes).expect("Failed to reload");

    println!("\nAfter export/reload:");
    let entities_after = reloaded.get_block_entities_as_list();
    println!("  Block entities: {}", entities_after.len());

    for be in &entities_after {
        println!("    {} at {:?}", be.id, be.position);
    }

    assert_eq!(
        entities_after.len(),
        positions.len(),
        "Should preserve all block entities after export/reload"
    );

    // Verify each position
    for &(x, y, z) in &positions {
        let found = entities_after.iter().any(|be| be.position == (x, y, z));
        assert!(
            found,
            "Block entity at position {:?} should be preserved",
            (x, y, z)
        );
    }
}

#[test]
fn test_region_merge_adjusts_block_entity_positions() {
    println!("\n======================================================");
    println!("Test 7: Region Merge Block Entity Position Adjustment");
    println!("======================================================");

    // Create a region with blocks and block entities
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (5, 5, 5));

    region.set_block(2, 2, 2, BlockState::new("minecraft:chest".to_string()));
    let be = BlockEntity::new("minecraft:chest".to_string(), (2, 2, 2));
    region.add_block_entity(be);

    println!(
        "Initial region: pos={:?}, size={:?}",
        region.position, region.size
    );
    println!("Block entity at: (2, 2, 2)");

    // Now trigger expansion to negative coordinates
    region.set_block(
        -10,
        -10,
        -10,
        BlockState::new("minecraft:stone".to_string()),
    );

    println!("\nAfter expansion:");
    println!("  Region pos={:?}, size={:?}", region.position, region.size);

    // Check if block entity position is still correct
    let be_at_original = region.block_entities.get(&(2, 2, 2));
    println!(
        "  Block entity at (2, 2, 2): {}",
        if be_at_original.is_some() {
            "✓ Found"
        } else {
            "✗ MISSING"
        }
    );

    // Check if we can still get the block
    let block_at_original = region.get_block(2, 2, 2);
    println!(
        "  Block at (2, 2, 2): {:?}",
        block_at_original.map(|b| &b.name)
    );

    assert!(
        be_at_original.is_some(),
        "Block entity should still be at (2, 2, 2) after expansion"
    );
    assert!(
        block_at_original.is_some(),
        "Block should still be at (2, 2, 2) after expansion"
    );

    println!("\n✓ Block entities maintain their world positions during expansion");
}

#[test]
fn test_schematic_export_with_tight_bounds_option() {
    println!("\n==================================================");
    println!("Test 8: Schematic Export Should Use Tight Bounds");
    println!("==================================================");

    let mut schematic = UniversalSchematic::new("Test".to_string());

    // Create a simple 3x3x3 cube in the center
    for x in 0..=2 {
        for y in 0..=2 {
            for z in 0..=2 {
                schematic.set_block(x, y, z, BlockState::new("minecraft:stone".to_string()));
            }
        }
    }

    let allocated = schematic.get_dimensions();
    let tight = schematic.get_tight_dimensions();

    println!("Structure: 3x3x3 cube");
    println!("Allocated dimensions: {:?}", allocated);
    println!("Tight dimensions: {:?}", tight);

    assert_eq!(
        tight,
        (3, 3, 3),
        "Tight dimensions should be exactly (3, 3, 3)"
    );

    // Export the schematic
    let bytes = schematic::to_schematic(&schematic).expect("Failed to export");

    // Reload and check dimensions
    let reloaded = schematic::from_schematic(&bytes).expect("Failed to reload");
    let reloaded_dims = reloaded.get_dimensions();

    println!("\nAfter export/reload:");
    println!("  Dimensions: {:?}", reloaded_dims);

    // DESIRED: Reloaded dimensions should match tight bounds (3, 3, 3)
    // ACTUAL: Will be larger due to exporting allocated bounds

    if reloaded_dims != tight {
        println!(
            "\n⚠ BUG CONFIRMED: Exported dimensions {:?} don't match tight bounds {:?}",
            reloaded_dims, tight
        );
        println!("This causes incorrect positioning when pasting in-game!");
    } else {
        println!("\n✓ Dimensions correctly match tight bounds");
    }
}
