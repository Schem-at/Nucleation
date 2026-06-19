//! V2696 (21w07a + 1) — schematic-relevant subset of `V2696.java`.
//!
//! `grimstone*` -> `deepslate*` item + block renames (V2696.java:14-34). Both the
//! ITEM_NAME and the BLOCK_NAME/BLOCK_STATE/FLAT_BLOCK_STATE forms are renamed
//! (Java registers the same map on `ConverterAbstractItemRename` and
//! `ConverterAbstractBlockRename`). Nothing non-schematic is present.
//!
//! VERSION = MCVersions.V21W07A (2695) + 1 = 2696.

use super::super::helpers::{map_renamer, register_block_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 2696;

const RENAMES: &[(&str, &str)] = &[
    ("minecraft:grimstone", "minecraft:deepslate"),
    ("minecraft:grimstone_slab", "minecraft:cobbled_deepslate_slab"),
    ("minecraft:grimstone_stairs", "minecraft:cobbled_deepslate_stairs"),
    ("minecraft:grimstone_wall", "minecraft:cobbled_deepslate_wall"),
    ("minecraft:polished_grimstone", "minecraft:polished_deepslate"),
    ("minecraft:polished_grimstone_slab", "minecraft:polished_deepslate_slab"),
    ("minecraft:polished_grimstone_stairs", "minecraft:polished_deepslate_stairs"),
    ("minecraft:polished_grimstone_wall", "minecraft:polished_deepslate_wall"),
    ("minecraft:grimstone_tiles", "minecraft:deepslate_tiles"),
    ("minecraft:grimstone_tile_slab", "minecraft:deepslate_tile_slab"),
    ("minecraft:grimstone_tile_stairs", "minecraft:deepslate_tile_stairs"),
    ("minecraft:grimstone_tile_wall", "minecraft:deepslate_tile_wall"),
    ("minecraft:grimstone_bricks", "minecraft:deepslate_bricks"),
    ("minecraft:grimstone_brick_slab", "minecraft:deepslate_brick_slab"),
    ("minecraft:grimstone_brick_stairs", "minecraft:deepslate_brick_stairs"),
    ("minecraft:grimstone_brick_wall", "minecraft:deepslate_brick_wall"),
    ("minecraft:chiseled_grimstone", "minecraft:chiseled_deepslate"),
];

pub fn register(reg: &mut RegistryBuilder) {
    register_item_rename(reg, VERSION, map_renamer(RENAMES));
    register_block_rename(reg, VERSION, map_renamer(RENAMES));
}
