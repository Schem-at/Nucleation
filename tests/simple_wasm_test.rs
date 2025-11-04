#![cfg(target_arch = "wasm32")]

use nucleation::wasm::*;
use wasm_bindgen_test::*;

// Configure for Node.js testing - no configuration needed, use --node flag

#[wasm_bindgen_test]
fn test_schematic_creation() {
    let mut schematic = SchematicWrapper::new();

    // Just verify basic functionality
    schematic.set_block(0, 0, 0, "minecraft:stone");

    // Get basic chunk info
    let chunks = schematic.chunks(2, 2, 2);
    assert!(chunks.length() > 0, "Should have chunks");
}
