//! V4541 (25w34b + 1) — schematic-relevant subset of
//! `DataConverterJava/.../versions/V4541.java`.
//!
//! Renames `minecraft:chain` to `minecraft:iron_chain` for both block ids
//! (BLOCK_NAME + BLOCK_STATE Name + FLAT_BLOCK_STATE) and item ids (ITEM_NAME +
//! ITEM_STACK id) (V4541.java:19-22). Bijective rename, trivially reversible.
//!
//! VERSION = MCVersions.V25W34B (4540) + 1 = 4541.

use super::super::helpers::{map_renamer, register_block_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 4541;

/// `(old, new)` chain rename, shared by the block and item passes.
pub const CHAIN_RENAMES: &[(&str, &str)] = &[("minecraft:chain", "minecraft:iron_chain")];

pub fn register(reg: &mut RegistryBuilder) {
    register_block_rename(reg, VERSION, map_renamer(CHAIN_RENAMES));
    register_item_rename(reg, VERSION, map_renamer(CHAIN_RENAMES));
}
