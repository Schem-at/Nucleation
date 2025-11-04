#![cfg(target_arch = "wasm32")]

use js_sys::{Array, Object, Reflect};
use nucleation::wasm::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::console;

// Configure for Node.js testing - no configuration needed, use --node flag

// Helper function to create a test schematic
fn create_test_schematic() -> SchematicWrapper {
    let mut schematic = SchematicWrapper::new();

    // Create a simple 3x3x3 cube of blocks
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                if x == 0 || x == 2 || y == 0 || y == 2 || z == 0 || z == 2 {
                    // Walls are stone
                    schematic.set_block(x, y, z, "minecraft:stone");
                } else {
                    // Interior is air (or we could use different blocks)
                    schematic.set_block(x, y, z, "minecraft:air");
                }
            }
        }
    }

    // Add some special blocks for testing
    schematic.set_block(1, 1, 1, "minecraft:diamond_block");
    schematic.set_block(0, 0, 0, "minecraft:emerald_block");
    schematic.set_block(2, 2, 2, "minecraft:gold_block");

    schematic
}

// Helper function to load a real test schematic
// In WASM, we can't use std::fs, so we just create a test schematic
fn load_test_schematic() -> SchematicWrapper {
    create_test_schematic()
}

#[wasm_bindgen_test]
fn test_basic_chunk_iteration() {
    let schematic = create_test_schematic();

    // Test basic chunk iteration
    let chunks = schematic.chunks(2, 2, 2);

    console_log!("Basic chunk test - Number of chunks: {}", chunks.length());

    // Verify we have at least 1 chunk
    assert!(chunks.length() > 0, "Should have at least one chunk");

    // Check first chunk structure
    let first_chunk = chunks.get(0);
    assert!(!first_chunk.is_null(), "First chunk should not be null");
}

#[wasm_bindgen_test]
fn test_chunk_indices_optimization() {
    let schematic = create_test_schematic();

    // Test optimized chunk iteration with indices
    let chunks_indices = schematic.chunks_indices(2, 2, 2);
    let chunks_regular = schematic.chunks(2, 2, 2);

    console_log!("Chunks indices length: {}", chunks_indices.length());
    console_log!("Chunks regular length: {}", chunks_regular.length());

    // Both should have the same number of chunks
    assert_eq!(
        chunks_indices.length(),
        chunks_regular.length(),
        "Optimized and regular chunks should have same count"
    );

    // Verify structure of indices chunks
    if chunks_indices.length() > 0 {
        let first_chunk = chunks_indices.get(0);
        assert!(
            !first_chunk.is_null(),
            "First indexed chunk should not be null"
        );

        // Check that it has the expected properties
        let chunk_obj = first_chunk.dyn_into::<js_sys::Object>().unwrap();
        assert!(js_sys::Reflect::has(&chunk_obj, &"chunk_x".into()).unwrap());
        assert!(js_sys::Reflect::has(&chunk_obj, &"chunk_y".into()).unwrap());
        assert!(js_sys::Reflect::has(&chunk_obj, &"chunk_z".into()).unwrap());
        assert!(js_sys::Reflect::has(&chunk_obj, &"blocks".into()).unwrap());
    }
}

#[wasm_bindgen_test]
fn test_chunk_loading_strategies() {
    let schematic = create_test_schematic();

    let strategies = [
        "bottom_up",
        "top_down",
        "distance_to_camera",
        "center_outward",
        "random",
    ];

    for strategy in &strategies {
        console_log!("Testing strategy: {}", strategy);

        let chunks = schematic.chunks_with_strategy(2, 2, 2, strategy, 0.0, 0.0, 0.0);
        assert!(
            chunks.length() > 0,
            "Strategy {} should produce chunks",
            strategy
        );

        let chunks_indices =
            schematic.chunks_indices_with_strategy(2, 2, 2, strategy, 0.0, 0.0, 0.0);
        assert_eq!(
            chunks.length(),
            chunks_indices.length(),
            "Strategy {} should produce same count for both methods",
            strategy
        );
    }
}

#[wasm_bindgen_test]
fn test_lazy_chunk_iterator() {
    let schematic = create_test_schematic();

    // Create lazy iterator
    let mut iterator = schematic.create_lazy_chunk_iterator(2, 2, 2, "bottom_up", 0.0, 0.0, 0.0);

    console_log!("Total chunks in lazy iterator: {}", iterator.total_chunks());

    let total_chunks = iterator.total_chunks();
    assert!(total_chunks > 0, "Should have chunks to iterate over");

    let mut chunks_retrieved = 0;

    // Test iteration
    while iterator.has_next() {
        let chunk = iterator.next();
        assert!(!chunk.is_null(), "Chunk should not be null");

        // Verify chunk structure
        let chunk_obj = chunk.dyn_into::<js_sys::Object>().unwrap();
        assert!(js_sys::Reflect::has(&chunk_obj, &"chunk_x".into()).unwrap());
        assert!(js_sys::Reflect::has(&chunk_obj, &"chunk_y".into()).unwrap());
        assert!(js_sys::Reflect::has(&chunk_obj, &"chunk_z".into()).unwrap());
        assert!(js_sys::Reflect::has(&chunk_obj, &"blocks".into()).unwrap());
        assert!(js_sys::Reflect::has(&chunk_obj, &"index".into()).unwrap());
        assert!(js_sys::Reflect::has(&chunk_obj, &"total".into()).unwrap());

        chunks_retrieved += 1;

        // Safety break to prevent infinite loops
        if chunks_retrieved > 100 {
            panic!("Too many chunks, possible infinite loop");
        }
    }

    assert_eq!(
        chunks_retrieved, total_chunks,
        "Should retrieve exactly the expected number of chunks"
    );

    // Test reset functionality
    iterator.reset();
    assert_eq!(
        iterator.current_position(),
        0,
        "Reset should set position to 0"
    );
    assert!(iterator.has_next(), "Should have next after reset");
}

#[wasm_bindgen_test]
fn test_lazy_iterator_skip_functionality() {
    let schematic = create_test_schematic();
    let mut iterator = schematic.create_lazy_chunk_iterator(2, 2, 2, "bottom_up", 0.0, 0.0, 0.0);

    let total_chunks = iterator.total_chunks();
    if total_chunks > 2 {
        // Test skip to middle
        iterator.skip_to(total_chunks / 2);
        assert_eq!(iterator.current_position(), total_chunks / 2);

        // Test skip beyond end
        iterator.skip_to(total_chunks + 10);
        assert_eq!(iterator.current_position(), total_chunks);
        assert!(
            !iterator.has_next(),
            "Should not have next when skipped beyond end"
        );
    }
}

#[wasm_bindgen_test]
fn test_chunk_blocks_consistency() {
    let schematic = create_test_schematic();

    // Get blocks using different methods and compare
    let chunk_size = 2;
    let chunks = schematic.chunks(chunk_size, chunk_size, chunk_size);
    let chunks_indices = schematic.chunks_indices(chunk_size, chunk_size, chunk_size);

    if chunks.length() > 0 && chunks_indices.length() > 0 {
        // Compare first chunk
        let regular_chunk = chunks.get(0);
        let indices_chunk = chunks_indices.get(0);

        let regular_obj = regular_chunk.dyn_into::<js_sys::Object>().unwrap();
        let indices_obj = indices_chunk.dyn_into::<js_sys::Object>().unwrap();

        // Get chunk coordinates - they should match
        let reg_x = js_sys::Reflect::get(&regular_obj, &"chunk_x".into()).unwrap();
        let ind_x = js_sys::Reflect::get(&indices_obj, &"chunk_x".into()).unwrap();
        assert_eq!(
            reg_x.as_f64(),
            ind_x.as_f64(),
            "Chunk X coordinates should match"
        );

        let reg_y = js_sys::Reflect::get(&regular_obj, &"chunk_y".into()).unwrap();
        let ind_y = js_sys::Reflect::get(&indices_obj, &"chunk_y".into()).unwrap();
        assert_eq!(
            reg_y.as_f64(),
            ind_y.as_f64(),
            "Chunk Y coordinates should match"
        );

        let reg_z = js_sys::Reflect::get(&regular_obj, &"chunk_z".into()).unwrap();
        let ind_z = js_sys::Reflect::get(&indices_obj, &"chunk_z".into()).unwrap();
        assert_eq!(
            reg_z.as_f64(),
            ind_z.as_f64(),
            "Chunk Z coordinates should match"
        );
    }
}

#[wasm_bindgen_test]
fn test_specific_chunk_retrieval() {
    let schematic = create_test_schematic();

    // Test getting specific chunk blocks
    let chunk_blocks = schematic.get_chunk_blocks(0, 0, 0, 2, 2, 2);
    console_log!("Chunk blocks count: {}", chunk_blocks.length());

    let chunk_blocks_indices = schematic.get_chunk_blocks_indices(0, 0, 0, 2, 2, 2);
    console_log!(
        "Chunk blocks indices count: {}",
        chunk_blocks_indices.length()
    );

    // Both methods should return blocks in the same region
    // The indices version might have fewer blocks if it skips air
    assert!(
        chunk_blocks_indices.length() <= chunk_blocks.length(),
        "Indices method should return same or fewer blocks (air filtering)"
    );
}

#[wasm_bindgen_test]
fn test_palette_consistency() {
    let schematic = create_test_schematic();

    // Get all palettes
    let all_palettes = schematic.get_all_palettes();
    assert!(!all_palettes.is_null(), "Palettes should not be null");

    let palettes_obj = all_palettes.dyn_into::<js_sys::Object>().unwrap();
    assert!(
        js_sys::Reflect::has(&palettes_obj, &"default".into()).unwrap(),
        "Should have default palette"
    );

    // Get default palette specifically
    let default_palette = schematic.get_default_region_palette();
    assert!(
        !default_palette.is_null(),
        "Default palette should not be null"
    );

    let default_array = default_palette.dyn_into::<js_sys::Array>().unwrap();
    console_log!("Default palette size: {}", default_array.length());

    // Should have at least the blocks we added (stone, air, diamond, emerald, gold)
    assert!(
        default_array.length() >= 4,
        "Should have at least our test blocks in palette"
    );
}

#[wasm_bindgen_test]
fn test_optimization_info() {
    let schematic = create_test_schematic();

    let optimization_info = schematic.get_optimization_info();
    assert!(
        !optimization_info.is_null(),
        "Optimization info should not be null"
    );

    let info_obj = optimization_info.dyn_into::<js_sys::Object>().unwrap();

    // Check expected properties
    assert!(js_sys::Reflect::has(&info_obj, &"total_blocks".into()).unwrap());
    assert!(js_sys::Reflect::has(&info_obj, &"non_air_blocks".into()).unwrap());
    assert!(js_sys::Reflect::has(&info_obj, &"palette_size".into()).unwrap());
    assert!(js_sys::Reflect::has(&info_obj, &"compression_ratio".into()).unwrap());

    let total_blocks = js_sys::Reflect::get(&info_obj, &"total_blocks".into())
        .unwrap()
        .as_f64()
        .unwrap() as u32;
    let non_air_blocks = js_sys::Reflect::get(&info_obj, &"non_air_blocks".into())
        .unwrap()
        .as_f64()
        .unwrap() as u32;
    let palette_size = js_sys::Reflect::get(&info_obj, &"palette_size".into())
        .unwrap()
        .as_f64()
        .unwrap() as u32;

    console_log!(
        "Total blocks: {}, Non-air blocks: {}, Palette size: {}",
        total_blocks,
        non_air_blocks,
        palette_size
    );

    assert!(total_blocks > 0, "Should have some blocks");
    assert!(
        non_air_blocks <= total_blocks,
        "Non-air blocks should not exceed total"
    );
    assert!(palette_size > 0, "Should have some palette entries");
}

#[wasm_bindgen_test]
fn test_large_chunk_sizes() {
    let schematic = create_test_schematic();

    // Test with larger chunk sizes
    let large_chunks = schematic.chunks(10, 10, 10);
    console_log!("Large chunks count: {}", large_chunks.length());

    // With our 3x3x3 schematic, a 10x10x10 chunk should contain everything
    assert_eq!(
        large_chunks.length(),
        1,
        "Large chunk should contain entire small schematic"
    );

    // Test with very small chunk sizes
    let small_chunks = schematic.chunks(1, 1, 1);
    console_log!("Small chunks count: {}", small_chunks.length());

    // Each block should be in its own chunk
    assert!(
        small_chunks.length() > 0,
        "Should have multiple small chunks"
    );
}

// Test to verify false values issue is detected
#[wasm_bindgen_test]
fn test_chunk_iterator_data_integrity() {
    let schematic = create_test_schematic();

    // Test the lazy iterator for data integrity
    let mut iterator = schematic.create_lazy_chunk_iterator(2, 2, 2, "bottom_up", 0.0, 0.0, 0.0);

    let mut all_blocks = Vec::new();
    let mut chunk_positions = Vec::new();

    while iterator.has_next() {
        let chunk = iterator.next();
        let chunk_obj = chunk.dyn_into::<js_sys::Object>().unwrap();

        // Get chunk position
        let chunk_x = js_sys::Reflect::get(&chunk_obj, &"chunk_x".into())
            .unwrap()
            .as_f64()
            .unwrap() as i32;
        let chunk_y = js_sys::Reflect::get(&chunk_obj, &"chunk_y".into())
            .unwrap()
            .as_f64()
            .unwrap() as i32;
        let chunk_z = js_sys::Reflect::get(&chunk_obj, &"chunk_z".into())
            .unwrap()
            .as_f64()
            .unwrap() as i32;

        chunk_positions.push((chunk_x, chunk_y, chunk_z));

        // Get blocks array
        let blocks = js_sys::Reflect::get(&chunk_obj, &"blocks".into()).unwrap();
        let blocks_array = blocks.dyn_into::<js_sys::Array>().unwrap();

        // Verify each block's data
        for i in 0..blocks_array.length() {
            let block_data = blocks_array.get(i);
            let block_array = block_data.dyn_into::<js_sys::Array>().unwrap();

            // Each block should be [x, y, z, palette_index]
            assert_eq!(
                block_array.length(),
                4,
                "Block data should have 4 elements [x,y,z,palette_index]"
            );

            let x = block_array.get(0).as_f64().unwrap() as i32;
            let y = block_array.get(1).as_f64().unwrap() as i32;
            let z = block_array.get(2).as_f64().unwrap() as i32;
            let palette_index = block_array.get(3).as_f64().unwrap() as u32;

            // Verify coordinates are within expected range for our test schematic
            assert!(
                x >= 0 && x < 3,
                "X coordinate should be in range [0,3): got {}",
                x
            );
            assert!(
                y >= 0 && y < 3,
                "Y coordinate should be in range [0,3): got {}",
                y
            );
            assert!(
                z >= 0 && z < 3,
                "Z coordinate should be in range [0,3): got {}",
                z
            );

            // Palette index should be reasonable (not a huge number indicating corruption)
            assert!(
                palette_index < 1000,
                "Palette index seems corrupted: {}",
                palette_index
            );

            // Store for duplicate detection
            all_blocks.push((x, y, z, palette_index));

            console_log!(
                "Block at ({},{},{}) has palette index {}",
                x,
                y,
                z,
                palette_index
            );
        }
    }

    console_log!(
        "Processed {} chunks with {} total blocks",
        chunk_positions.len(),
        all_blocks.len()
    );
    console_log!("Chunk positions: {:?}", chunk_positions);

    // Check for duplicates (which could indicate iterator problems)
    all_blocks.sort();
    let mut unique_positions = Vec::new();
    let mut duplicates = Vec::new();

    for (x, y, z, palette_idx) in &all_blocks {
        let pos = (*x, *y, *z);
        if unique_positions.contains(&pos) {
            duplicates.push(pos);
        } else {
            unique_positions.push(pos);
        }
    }

    if !duplicates.is_empty() {
        console_log!("Found duplicate positions: {:?}", duplicates);
        panic!(
            "Chunk iterator returned duplicate blocks at positions: {:?}",
            duplicates
        );
    }

    assert!(all_blocks.len() > 0, "Should have found some blocks");
    console_log!("Data integrity test passed - no duplicate blocks found");
}
