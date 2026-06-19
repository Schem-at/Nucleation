//! V3800 (1.20.4 + 100) — schematic-relevant subset of `V3800.java`.
//!
//! A single ITEM_NAME rename: `minecraft:scute` -> `minecraft:turtle_scute`
//! (V3800.java:18-22). Bijective, trivially reversible.
//!
//! VERSION = MCVersions.V1_20_4 (3700) + 100 = 3800.

use super::super::helpers::{map_renamer, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 3800;

/// `(old, new)` item renames (V3800.java:18-20).
pub const ITEM_RENAMES: &[(&str, &str)] = &[("minecraft:scute", "minecraft:turtle_scute")];

pub fn register(reg: &mut RegistryBuilder) {
    register_item_rename(reg, VERSION, map_renamer(ITEM_RENAMES));
}
