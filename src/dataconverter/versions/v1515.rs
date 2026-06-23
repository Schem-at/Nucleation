//! V1515 (1.13-pre7+2) — schematic-relevant subset of `V1515.java`.
//!
//! Kept:
//!   * BLOCK rename (`RENAMED_BLOCK_IDS`): the standing coral-fan ids ->
//!     `*_coral_wall_fan` (V1515.java:13-21). `register_block_rename` covers
//!     BLOCK_NAME, the BLOCK_STATE `Name` field, and FLAT_BLOCK_STATE.
//!
//! VERSION = MCVersions.V1_13_PRE7 + 2 = 1513 + 2 = 1515.
//!
//! Nothing non-schematic is present in this version.

use super::super::helpers::{map_renamer, register_block_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 1515;

/// `RENAMED_BLOCK_IDS` (V1515.java:13-21).
const RENAMED_BLOCK_IDS: &[(&str, &str)] = &[
    ("minecraft:tube_coral_fan", "minecraft:tube_coral_wall_fan"),
    (
        "minecraft:brain_coral_fan",
        "minecraft:brain_coral_wall_fan",
    ),
    (
        "minecraft:bubble_coral_fan",
        "minecraft:bubble_coral_wall_fan",
    ),
    ("minecraft:fire_coral_fan", "minecraft:fire_coral_wall_fan"),
    ("minecraft:horn_coral_fan", "minecraft:horn_coral_wall_fan"),
];

pub fn register(reg: &mut RegistryBuilder) {
    register_block_rename(reg, VERSION, map_renamer(RENAMED_BLOCK_IDS));
}
