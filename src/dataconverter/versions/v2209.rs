//! V2209 (19w40a + 1) — schematic-relevant subset of `V2209.java`.
//!
//! Ported: the BLOCK and ITEM renames `minecraft:bee_hive` ->
//! `minecraft:beehive` (V2209.java:18-25). Bijective rename (bucket A),
//! trivially reversible.
//!
//! VERSION = MCVersions.V19W40A + 1 = 2208 + 1 = 2209.
//!
//! Skipped (non-schematic): the POI rename of the same id —
//! ConverterAbstractPOIRename targets POI, which never appears in a schematic.

use super::super::helpers::{map_renamer, register_block_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 2209;

/// `(old, new)` rename. Shared between the block and item renames.
pub const BEE_HIVE_RENAMES: &[(&str, &str)] = &[("minecraft:bee_hive", "minecraft:beehive")];

pub fn register(reg: &mut RegistryBuilder) {
    register_block_rename(reg, VERSION, map_renamer(BEE_HIVE_RENAMES));
    register_item_rename(reg, VERSION, map_renamer(BEE_HIVE_RENAMES));
}
