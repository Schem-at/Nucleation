//! V4305 (25w03a + 1) — schematic-relevant subset of `V4305.java`.
//!
//! BLOCK_STATE structure converter: for `minecraft:test_block`, rename the
//! `Properties.test_block_mode` property to `Properties.mode`
//! (V4305.java:398-419).
//!
//! VERSION = V25W03A(4304) + 1. Entirely schematic-relevant (BLOCK_STATE).

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4305;

pub fn register(reg: &mut RegistryBuilder) {
    reg.block_state.add_structure_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_string("Name") != Some("minecraft:test_block") {
                return;
            }
            let properties = match data.get_map_mut("Properties") {
                Some(p) => p,
                None => return,
            };
            let mode = match properties.get_string("test_block_mode") {
                Some(s) => s.to_string(),
                None => return,
            };
            properties.take("test_block_mode");
            properties.set_string("mode", mode);
        }),
    );

    // Reverse: exact inverse rename for `minecraft:test_block` —
    // `Properties.mode` -> `Properties.test_block_mode`. Lossless (bucket B):
    // the `mode` property uniquely encodes the old `test_block_mode` value, so
    // no loss is reported.
    reg.block_state.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_string("Name") != Some("minecraft:test_block") {
                return;
            }
            let properties = match data.get_map_mut("Properties") {
                Some(p) => p,
                None => return,
            };
            let mode = match properties.get_string("mode") {
                Some(s) => s.to_string(),
                None => return,
            };
            properties.take("mode");
            properties.set_string("test_block_mode", mode);
        }),
    );
}
