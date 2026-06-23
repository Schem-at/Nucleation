//! V1490 (18w20a+1) — schematic-relevant subset of `V1490.java`.
//!
//! VERSION = MCVersions.V18W20A + 1 = 1489 + 1 = 1490.
//!
//! Ported:
//!   * BLOCK_NAME / BLOCK_STATE / FLAT_BLOCK_STATE rename `melon_block` ->
//!     `melon` (via `register_block_rename`).
//!   * ITEM_NAME renames `melon_block` -> `melon`, `melon` -> `melon_slice`,
//!     `speckled_melon` -> `glistering_melon_slice`.
//!
//! Everything in V1490 is schematic-relevant; nothing skipped.

use super::super::helpers::{map_renamer, register_block_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 1490;

/// Block rename (V1490.java:15-19).
const BLOCK_RENAMES: &[(&str, &str)] = &[("minecraft:melon_block", "minecraft:melon")];

/// Item renames (V1490.java:20-26). `melon_block`->`melon` and `melon`->
/// `melon_slice` are independent single-pass HashMap lookups (no chaining),
/// matching Java's `HashMap::get`.
const ITEM_RENAMES: &[(&str, &str)] = &[
    ("minecraft:melon_block", "minecraft:melon"),
    ("minecraft:melon", "minecraft:melon_slice"),
    (
        "minecraft:speckled_melon",
        "minecraft:glistering_melon_slice",
    ),
];

pub fn register(reg: &mut RegistryBuilder) {
    register_block_rename(reg, VERSION, map_renamer(BLOCK_RENAMES));
    register_item_rename(reg, VERSION, map_renamer(ITEM_RENAMES));
}
