//! V1802 (1.13.2 + 171) — schematic-relevant subset of `V1802.java`.
//!
//! VERSION = MCVersions.V1_13_2 (1631) + 171 = 1802.
//!
//! Ported (all schematic-relevant):
//!   * BLOCK renames: stone_slab -> smooth_stone_slab, sign -> oak_sign,
//!     wall_sign -> oak_wall_sign (V1802.java:14-19). `register_block_rename`
//!     covers BLOCK_NAME + BLOCK_STATE Name + FLAT_BLOCK_STATE.
//!   * ITEM renames: stone_slab -> smooth_stone_slab, sign -> oak_sign
//!     (V1802.java:20-24).
//!
//! Note: the item-rename table differs from the block table (the item table has
//! no `wall_sign` entry, since wall_sign is not an item), so the two tables are
//! kept separate exactly as in Java.

use super::super::helpers::{map_renamer, register_block_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 1802;

/// Block renames (V1802.java:14-19).
pub const BLOCK_RENAMES: &[(&str, &str)] = &[
    ("minecraft:stone_slab", "minecraft:smooth_stone_slab"),
    ("minecraft:sign", "minecraft:oak_sign"),
    ("minecraft:wall_sign", "minecraft:oak_wall_sign"),
];

/// Item renames (V1802.java:20-24).
pub const ITEM_RENAMES: &[(&str, &str)] = &[
    ("minecraft:stone_slab", "minecraft:smooth_stone_slab"),
    ("minecraft:sign", "minecraft:oak_sign"),
];

pub fn register(reg: &mut RegistryBuilder) {
    register_block_rename(reg, VERSION, map_renamer(BLOCK_RENAMES));
    register_item_rename(reg, VERSION, map_renamer(ITEM_RENAMES));
}
