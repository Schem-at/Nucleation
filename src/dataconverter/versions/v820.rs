//! V820 (1.11+1) — schematic-relevant subset.
//!
//! Port of DataConverterJava .../versions/V820.java: a single item rename,
//! `minecraft:totem` -> `minecraft:totem_of_undying`. Bijective (bucket A),
//! trivially reversible.

use super::super::helpers::{map_renamer, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 820;

/// `(old, new)` item renames from V820.java:13-17.
pub const TOTEM_RENAMES: &[(&str, &str)] = &[("minecraft:totem", "minecraft:totem_of_undying")];

pub fn register(reg: &mut RegistryBuilder) {
    register_item_rename(reg, VERSION, map_renamer(TOTEM_RENAMES));
}
