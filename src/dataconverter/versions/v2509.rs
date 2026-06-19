//! V2509 (20w08a + 2) — schematic-relevant subset of `V2509.java`.
//!
//! Zombie-pigman -> zombified-piglin renames:
//!   * ITEM_NAME `zombie_pigman_spawn_egg` -> `zombified_piglin_spawn_egg`.
//!   * ENTITY id + ENTITY_NAME `zombie_pigman` -> `zombified_piglin`.
//! (V2509.java:13-23). The commented-out `registerMob` is intentionally omitted.
//!
//! VERSION = MCVersions.V20W08A (2507) + 2 = 2509.

use super::super::helpers::{map_renamer, register_entity_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 2509;

const ITEM_RENAMES: &[(&str, &str)] =
    &[("minecraft:zombie_pigman_spawn_egg", "minecraft:zombified_piglin_spawn_egg")];

const ENTITY_RENAMES: &[(&str, &str)] = &[("minecraft:zombie_pigman", "minecraft:zombified_piglin")];

pub fn register(reg: &mut RegistryBuilder) {
    register_item_rename(reg, VERSION, map_renamer(ITEM_RENAMES));
    register_entity_rename(reg, VERSION, map_renamer(ENTITY_RENAMES));
}
