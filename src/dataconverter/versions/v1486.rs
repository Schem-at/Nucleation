//! V1486 (18w19b+1) — schematic-relevant subset of `V1486.java`.
//!
//! VERSION = MCVersions.V18W19B + 1 = 1485 + 1 = 1486.
//!
//! Ported:
//!   * ENTITY id renames `minecraft:salmon_mob` -> `minecraft:salmon`,
//!     `minecraft:cod_mob` -> `minecraft:cod` (also renames ENTITY_NAME).
//!   * ITEM_NAME renames for the matching spawn eggs.
//!   * `copyWalkers` so the V99 walkers registered under the old entity ids
//!     also fire under the new ids.
//!
//! Nothing in V1486 is non-schematic; everything here is ported.

use super::super::helpers::{map_renamer, register_entity_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 1486;

/// `RENAMED_ENTITY_IDS` (V1486.java:16-21).
pub const RENAMED_ENTITY_IDS: &[(&str, &str)] = &[
    ("minecraft:salmon_mob", "minecraft:salmon"),
    ("minecraft:cod_mob", "minecraft:cod"),
];

/// `RENAMED_ITEM_IDS` (V1486.java:22-27).
pub const RENAMED_ITEM_IDS: &[(&str, &str)] = &[
    ("minecraft:salmon_mob_spawn_egg", "minecraft:salmon_spawn_egg"),
    ("minecraft:cod_mob_spawn_egg", "minecraft:cod_spawn_egg"),
];

pub fn register(reg: &mut RegistryBuilder) {
    // copyWalkers from the old entity ids onto the new ones (V1486.java:31-32).
    reg.entity.copy_walkers(VERSION, 0, "minecraft:cod_mob", "minecraft:cod");
    reg.entity.copy_walkers(VERSION, 0, "minecraft:salmon_mob", "minecraft:salmon");

    register_entity_rename(reg, VERSION, map_renamer(RENAMED_ENTITY_IDS));
    register_item_rename(reg, VERSION, map_renamer(RENAMED_ITEM_IDS));
}
