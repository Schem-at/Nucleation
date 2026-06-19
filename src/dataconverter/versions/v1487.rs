//! V1487 (18w19b+2) — schematic-relevant subset of `V1487.java`.
//!
//! VERSION = MCVersions.V18W19B + 2 = 1485 + 2 = 1487.
//!
//! Ported:
//!   * ITEM_NAME renames `minecraft:prismarine_bricks_slab` ->
//!     `minecraft:prismarine_brick_slab` and `minecraft:prismarine_bricks_stairs`
//!     -> `minecraft:prismarine_brick_stairs` (ConverterAbstractItemRename).
//!   * BLOCK_NAME / BLOCK_STATE(Name) / FLAT_BLOCK_STATE renames for the same two
//!     ids (ConverterAbstractBlockRename) via `register_block_rename`.
//!
//! Nothing in V1487 is non-schematic; everything here is ported.

use super::super::helpers::{map_renamer, register_block_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 1487;

/// `remap` (V1487.java:15-20) — shared by both the item and block renames.
pub const RENAMED_IDS: &[(&str, &str)] = &[
    ("minecraft:prismarine_bricks_slab", "minecraft:prismarine_brick_slab"),
    ("minecraft:prismarine_bricks_stairs", "minecraft:prismarine_brick_stairs"),
];

pub fn register(reg: &mut RegistryBuilder) {
    // ConverterAbstractItemRename.register(VERSION, remap::get) (V1487.java:22).
    register_item_rename(reg, VERSION, map_renamer(RENAMED_IDS));

    // ConverterAbstractBlockRename.register(VERSION, remap::get) (V1487.java:23) —
    // covers BLOCK_NAME, the BLOCK_STATE `Name` field, and FLAT_BLOCK_STATE.
    register_block_rename(reg, VERSION, map_renamer(RENAMED_IDS));
}
