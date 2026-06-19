//! V2508 (20w08a + 1) — schematic-relevant subset of `V2508.java`.
//!
//! `warped_fungi`/`crimson_fungi` -> `warped_fungus`/`crimson_fungus`, applied to
//! both BLOCK_NAME (+ BLOCK_STATE Name + FLAT_BLOCK_STATE) and ITEM_NAME
//! (V2508.java:14-23). Nothing non-schematic is present.
//!
//! VERSION = MCVersions.V20W08A (2507) + 1 = 2508.

use super::super::helpers::{map_renamer, register_block_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 2508;

const RENAMES: &[(&str, &str)] = &[
    ("minecraft:warped_fungi", "minecraft:warped_fungus"),
    ("minecraft:crimson_fungi", "minecraft:crimson_fungus"),
];

pub fn register(reg: &mut RegistryBuilder) {
    register_block_rename(reg, VERSION, map_renamer(RENAMES));
    register_item_rename(reg, VERSION, map_renamer(RENAMES));
}
