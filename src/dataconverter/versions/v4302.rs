//! V4302 (25w02a + 4) — schematic-relevant subset of `V4302.java`.
//!
//! TILE_ENTITY walkers for `minecraft:test_instance_block` (V4302.java:342-344):
//!   * `data`   -> a single TEXT_COMPONENT.
//!   * `errors` -> a list of TEXT_COMPONENTs.
//!
//! VERSION = V25W02A(4298) + 4. Entirely schematic-relevant (TILE_ENTITY).

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::walker::{convert, convert_list};

const VERSION: i32 = 4302;

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "minecraft:test_instance_block",
        Arc::new(|reg, data, from, to| {
            convert(reg, &reg.text_component, data, "data", from, to);
        }),
    );
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "minecraft:test_instance_block",
        Arc::new(|reg, data, from, to| {
            convert_list(reg, &reg.text_component, data, "errors", from, to);
        }),
    );
}
