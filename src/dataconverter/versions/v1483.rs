//! V1483 (18w16a) — schematic-relevant subset of `V1483.java`.
//!
//! VERSION = MCVersions.V18W16A = 1483.
//!
//! Ported:
//!   * ENTITY id rename `minecraft:puffer_fish` -> `minecraft:pufferfish`
//!     (also renames ENTITY_NAME).
//!   * ITEM_NAME rename `minecraft:puffer_fish_spawn_egg` ->
//!     `minecraft:pufferfish_spawn_egg`.
//!   * `copyWalkers` so the walkers registered under the old entity id also
//!     fire under the new id.
//!
//! Nothing in V1483 is non-schematic; everything here is ported.

use super::super::helpers::{map_renamer, register_entity_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 1483;

/// `RENAMED_ENTITY_IDS` (V1483.java:15-19).
pub const RENAMED_ENTITY_IDS: &[(&str, &str)] =
    &[("minecraft:puffer_fish", "minecraft:pufferfish")];

/// `RENAMED_ITEM_IDS` (V1483.java:20-24).
pub const RENAMED_ITEM_IDS: &[(&str, &str)] = &[(
    "minecraft:puffer_fish_spawn_egg",
    "minecraft:pufferfish_spawn_egg",
)];

pub fn register(reg: &mut RegistryBuilder) {
    // copyWalkers from the old entity id onto the new one (V1483.java:26).
    reg.entity
        .copy_walkers(VERSION, 0, "minecraft:puffer_fish", "minecraft:pufferfish");

    register_entity_rename(reg, VERSION, map_renamer(RENAMED_ENTITY_IDS));
    register_item_rename(reg, VERSION, map_renamer(RENAMED_ITEM_IDS));
}
