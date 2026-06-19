//! V2552 (20w20b + 15) — schematic-relevant subset of `V2552.java`.
//!
//! BIOME value rename `minecraft:nether` -> `minecraft:nether_wastes`
//! (V2552.java:13-18). Nothing non-schematic is present.
//!
//! VERSION = MCVersions.V20W20B (2537) + 15 = 2552.

use super::super::helpers::{map_renamer, register_value_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 2552;

const RENAMES: &[(&str, &str)] = &[("minecraft:nether", "minecraft:nether_wastes")];

pub fn register(reg: &mut RegistryBuilder) {
    register_value_rename(&mut reg.biome, VERSION, 0, map_renamer(RENAMES));
}
