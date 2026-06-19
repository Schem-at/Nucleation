//! V1480 (18w14a+1) — coral block + item renames; cites `V1480.java`.
//!
//! Ported:
//!   * BLOCK rename (`ConverterAbstractBlockRename.register(VERSION, RENAMED_IDS)`,
//!     V1480.java:40): 20-entry coral map. The block-rename helper registers on
//!     BLOCK_NAME + BLOCK_STATE(Name) + FLAT_BLOCK_STATE.
//!   * ITEM rename (`ConverterAbstractItemRename.register(VERSION, RENAMED_IDS)`,
//!     V1480.java:41): same 20-entry coral map applied as ITEM_NAME renames.
//!
//! VERSION = MCVersions.V18W14A + 1 = 1479 + 1 = 1480.
//!
//! Nothing else is present in `V1480.java`.

use super::super::helpers::{map_renamer, register_block_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 1480;

/// Renamed coral ids (V1480.java:16-35); applied to both blocks and items.
const RENAMED_IDS: &[(&str, &str)] = &[
    ("minecraft:blue_coral", "minecraft:tube_coral_block"),
    ("minecraft:pink_coral", "minecraft:brain_coral_block"),
    ("minecraft:purple_coral", "minecraft:bubble_coral_block"),
    ("minecraft:red_coral", "minecraft:fire_coral_block"),
    ("minecraft:yellow_coral", "minecraft:horn_coral_block"),
    ("minecraft:blue_coral_plant", "minecraft:tube_coral"),
    ("minecraft:pink_coral_plant", "minecraft:brain_coral"),
    ("minecraft:purple_coral_plant", "minecraft:bubble_coral"),
    ("minecraft:red_coral_plant", "minecraft:fire_coral"),
    ("minecraft:yellow_coral_plant", "minecraft:horn_coral"),
    ("minecraft:blue_coral_fan", "minecraft:tube_coral_fan"),
    ("minecraft:pink_coral_fan", "minecraft:brain_coral_fan"),
    ("minecraft:purple_coral_fan", "minecraft:bubble_coral_fan"),
    ("minecraft:red_coral_fan", "minecraft:fire_coral_fan"),
    ("minecraft:yellow_coral_fan", "minecraft:horn_coral_fan"),
    ("minecraft:blue_dead_coral", "minecraft:dead_tube_coral"),
    ("minecraft:pink_dead_coral", "minecraft:dead_brain_coral"),
    ("minecraft:purple_dead_coral", "minecraft:dead_bubble_coral"),
    ("minecraft:red_dead_coral", "minecraft:dead_fire_coral"),
    ("minecraft:yellow_dead_coral", "minecraft:dead_horn_coral"),
];

pub fn register(reg: &mut RegistryBuilder) {
    register_block_rename(reg, VERSION, map_renamer(RENAMED_IDS));
    register_item_rename(reg, VERSION, map_renamer(RENAMED_IDS));
}
