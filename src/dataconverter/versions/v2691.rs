//! V2691 (21w05a + 1) — schematic-relevant subset of `V2691.java`.
//!
//! Copper-block item + block renames (V2691.java:14-26): `waxed_copper` gains its
//! `_block` suffix, and the three oxidation-stage `*_copper_block` ids lose theirs.
//! Both the ITEM_NAME and the BLOCK_NAME/BLOCK_STATE/FLAT_BLOCK_STATE forms are
//! renamed (Java registers the same map on `ConverterAbstractItemRename` and
//! `ConverterAbstractBlockRename`). Nothing non-schematic is present.
//!
//! VERSION = MCVersions.V21W05A (2690) + 1 = 2691.

use super::super::helpers::{map_renamer, register_block_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 2691;

const RENAMES: &[(&str, &str)] = &[
    ("minecraft:waxed_copper", "minecraft:waxed_copper_block"),
    ("minecraft:oxidized_copper_block", "minecraft:oxidized_copper"),
    ("minecraft:weathered_copper_block", "minecraft:weathered_copper"),
    ("minecraft:exposed_copper_block", "minecraft:exposed_copper"),
];

pub fn register(reg: &mut RegistryBuilder) {
    register_item_rename(reg, VERSION, map_renamer(RENAMES));
    register_block_rename(reg, VERSION, map_renamer(RENAMES));
}
