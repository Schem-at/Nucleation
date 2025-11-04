#![cfg(target_arch = "wasm32")]

use nucleation::wasm::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

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

#[wasm_bindgen_test]
fn test_basic_chunk_iteration() {
    let schematic = create_test_schematic();

    // Test basic chunk iteration
    let chunks = schematic.chunks(2, 2, 2);

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
fn test_lazy_chunk_iterator() {
    let schematic = create_test_schematic();

    // Create lazy iterator
    let mut iterator = schematic.create_lazy_chunk_iterator(2, 2, 2, "bottom_up", 0.0, 0.0, 0.0);

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
        }
    }

    // Check for duplicates (which could indicate iterator problems)
    all_blocks.sort();
    let mut unique_positions = Vec::new();
    let mut duplicates = Vec::new();

    for (x, y, z, _palette_idx) in &all_blocks {
        let pos = (*x, *y, *z);
        if unique_positions.contains(&pos) {
            duplicates.push(pos);
        } else {
            unique_positions.push(pos);
        }
    }

    if !duplicates.is_empty() {
        panic!(
            "Chunk iterator returned duplicate blocks at positions: {:?}",
            duplicates
        );
    }

    assert!(all_blocks.len() > 0, "Should have found some blocks");
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

    assert!(total_blocks > 0, "Should have some blocks");
    assert!(
        non_air_blocks <= total_blocks,
        "Non-air blocks should not exceed total"
    );
    assert!(palette_size > 0, "Should have some palette entries");
}
