//! V2680 (1.16.5 + 94) — schematic-relevant subset of `V2680.java`.
//!
//! `grass_path` -> `dirt_path` rename, applied to both ITEM_NAME and the block
//! types (BLOCK_NAME / BLOCK_STATE Name / FLAT_BLOCK_STATE) (V2680.java:13-23).
//! Bijective rename (bucket A), trivially reversible.
//!
//! VERSION = MCVersions.V1_16_5 (2586) + 94 = 2680.

use super::super::helpers::{map_renamer, register_block_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 2680;

/// `(old, new)` rename, shared by the item and block renamers.
pub const GRASS_PATH_RENAME: &[(&str, &str)] = &[("minecraft:grass_path", "minecraft:dirt_path")];

pub fn register(reg: &mut RegistryBuilder) {
    register_item_rename(reg, VERSION, map_renamer(GRASS_PATH_RENAME));
    register_block_rename(reg, VERSION, map_renamer(GRASS_PATH_RENAME));
}
