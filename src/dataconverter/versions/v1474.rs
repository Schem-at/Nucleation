//! V1474 (18w10b) — schematic-relevant subset of `V1474.java`.
//!
//! VERSION = MCVersions.V18W10B = 1474.
//!
//! Ported (V1474.java:14-31):
//!   * ENTITY converter for `minecraft:shulker`: if int `Color` == 10, set the
//!     byte `Color` to 16 (migrate the old purple==10 sentinel to no-color).
//!   * BLOCK_NAME / BLOCK_STATE(Name) / FLAT_BLOCK_STATE rename
//!     `minecraft:purple_shulker_box` -> `minecraft:shulker_box`
//!     (via `register_block_rename`).
//!   * ITEM_NAME rename `minecraft:purple_shulker_box` -> `minecraft:shulker_box`
//!     (via `register_item_rename`).
//!
//! Everything in V1474 is schematic-relevant; nothing skipped.

use super::super::helpers::{map_renamer, register_block_rename, register_item_rename};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 1474;

/// Rename `purple_shulker_box` -> `shulker_box` (V1474.java:25-30).
const RENAMES: &[(&str, &str)] = &[(
    "minecraft:purple_shulker_box",
    "minecraft:shulker_box",
)];

pub fn register(reg: &mut RegistryBuilder) {
    // ENTITY shulker Color migration (V1474.java:15-23).
    // `getInt("Color")` defaults to 0 when absent, so it can only equal 10
    // when actually present.
    reg.entity.add_converter_for_id(
        "minecraft:shulker",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_i32("Color").unwrap_or(0) == 10 {
                data.set_byte("Color", 16);
            }
        }),
    );
    // Reverse of the shulker Color migration (V1474.java:15-23).
    // Forward mapped the old purple sentinel (int Color == 10) to the new
    // no-color value (byte Color == 16). In this version's forward-output
    // schema, Color == 16 is exactly that sentinel, so reversing 16 -> 10 is
    // the exact inverse (bucket B, lossless — no report_loss). We restore the
    // old int representation, since the pre-1474 format read Color as an int.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:shulker",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_i32("Color").unwrap_or(0) == 16 {
                data.set_i32("Color", 10);
            }
        }),
    );

    // BLOCK_NAME + BLOCK_STATE(Name) + FLAT_BLOCK_STATE (V1474.java:25-27).
    register_block_rename(reg, VERSION, map_renamer(RENAMES));
    // ITEM_NAME (V1474.java:28-30).
    register_item_rename(reg, VERSION, map_renamer(RENAMES));
}
