//! V1800 (1.13.2 + 169) — schematic-relevant subset of `V1800.java`.
//!
//! VERSION = MCVersions.V1_13_2 (1631) + 169 = 1800.
//!
//! Ported:
//!   * ITEM_NAME renames for the dye renames (cactus_green -> green_dye, etc.)
//!     (V1800.java:15-24).
//!   * ENTITY walker for `minecraft:pillager`: its `Inventory` is an item-list
//!     (V1800.java:27).
//!
//! Skipped (non-schematic / no-op): the commented-out `registerMob` for panda
//! (became a simple mob and registers nothing).

use super::super::helpers::{map_renamer, register_item_rename};
use super::super::registry::RegistryBuilder;
use super::super::walker::item_lists;

const VERSION: i32 = 1800;

/// `RENAMED_ITEM_IDS` (V1800.java:15-21).
pub const RENAMED_ITEM_IDS: &[(&str, &str)] = &[
    ("minecraft:cactus_green", "minecraft:green_dye"),
    ("minecraft:rose_red", "minecraft:red_dye"),
    ("minecraft:dandelion_yellow", "minecraft:yellow_dye"),
];

pub fn register(reg: &mut RegistryBuilder) {
    register_item_rename(reg, VERSION, map_renamer(RENAMED_ITEM_IDS));

    reg.entity
        .add_walker(VERSION, 0, "minecraft:pillager", item_lists(&["Inventory"]));
}
