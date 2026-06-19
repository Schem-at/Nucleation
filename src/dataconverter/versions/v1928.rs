//! V1928 (19w04b+1) — schematic-relevant subset of `V1928.java`.
//!
//! Renames the illager beast to the ravager:
//!   * ENTITY id + ENTITY_NAME `minecraft:illager_beast` -> `minecraft:ravager`
//!     (V1928.java:14-18).
//!   * ITEM_NAME `minecraft:illager_beast_spawn_egg` ->
//!     `minecraft:ravager_spawn_egg` (V1928.java:19-23).
//!
//! The `registerMob` call is commented out in Java (became a simple-walker
//! registration in 1.21.5), so nothing else is registered here.
//!
//! VERSION = MCVersions.V19W04B (1927) + 1 = 1928.

use super::super::helpers::{map_renamer, register_entity_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 1928;

pub fn register(reg: &mut RegistryBuilder) {
    register_entity_rename(
        reg,
        VERSION,
        map_renamer(&[("minecraft:illager_beast", "minecraft:ravager")]),
    );
    register_item_rename(
        reg,
        VERSION,
        map_renamer(&[("minecraft:illager_beast_spawn_egg", "minecraft:ravager_spawn_egg")]),
    );
}
