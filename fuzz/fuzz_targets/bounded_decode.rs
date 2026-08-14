#![no_main]

use libfuzzer_sys::fuzz_target;
use nucleation::formats::limits::DecodeLimits;
use nucleation::formats::manager::get_manager;

fuzz_target!(|data: &[u8]| {
    let limits = DecodeLimits {
        max_input_bytes: 1024 * 1024,
        max_decompressed_bytes: 4 * 1024 * 1024,
        max_dimension: 512,
        max_volume: 4 * 1024 * 1024,
        max_regions: 128,
        max_palette_entries: 16_384,
        max_entities: 16_384,
        max_block_entities: 65_536,
        max_nbt_depth: 32,
        max_nbt_string_bytes: 64 * 1024,
        max_nbt_collection_items: 1_000_000,
        max_nbt_nodes: 1_000_000,
    };
    let manager = get_manager();
    let manager = manager.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let _ = manager.read_bounded(data, &limits);
});
