//! V3692 (23w46a+1, `V23W46A + 1` = 3692) — schematic-relevant subset of
//! `V3692.java`.
//!
//! The `minecraft:grass` block/item was renamed to `minecraft:short_grass`.
//! Ported via the standard block-rename (BLOCK_NAME + BLOCK_STATE `Name` +
//! FLAT_BLOCK_STATE) and item-rename (ITEM_NAME) helpers. Bijective single-pair
//! rename, trivially reversible. No non-schematic registrations exist in this
//! version.

use super::super::helpers::{map_renamer, register_block_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 3692;

/// `GRASS_RENAME` (V3692.java:13-17).
pub const GRASS_RENAME: &[(&str, &str)] = &[("minecraft:grass", "minecraft:short_grass")];

pub fn register(reg: &mut RegistryBuilder) {
    register_block_rename(reg, VERSION, map_renamer(GRASS_RENAME));
    register_item_rename(reg, VERSION, map_renamer(GRASS_RENAME));
}
