//! V2528 (20w16a + 2) — schematic-relevant subset of `V2528.java`.
//!
//! Soul-fire torch/lantern renames:
//!   * ITEM_NAME: `soul_fire_torch` -> `soul_torch`, `soul_fire_lantern` ->
//!     `soul_lantern`.
//!   * BLOCK_NAME (+ BLOCK_STATE Name + FLAT_BLOCK_STATE): `soul_fire_torch` ->
//!     `soul_torch`, `soul_fire_wall_torch` -> `soul_wall_torch`,
//!     `soul_fire_lantern` -> `soul_lantern`.
//! (V2528.java:14-26). Nothing non-schematic is present.
//!
//! VERSION = MCVersions.V20W16A (2526) + 2 = 2528.

use super::super::helpers::{map_renamer, register_block_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 2528;

const ITEM_RENAMES: &[(&str, &str)] = &[
    ("minecraft:soul_fire_torch", "minecraft:soul_torch"),
    ("minecraft:soul_fire_lantern", "minecraft:soul_lantern"),
];

const BLOCK_RENAMES: &[(&str, &str)] = &[
    ("minecraft:soul_fire_torch", "minecraft:soul_torch"),
    (
        "minecraft:soul_fire_wall_torch",
        "minecraft:soul_wall_torch",
    ),
    ("minecraft:soul_fire_lantern", "minecraft:soul_lantern"),
];

pub fn register(reg: &mut RegistryBuilder) {
    register_item_rename(reg, VERSION, map_renamer(ITEM_RENAMES));
    register_block_rename(reg, VERSION, map_renamer(BLOCK_RENAMES));
}
