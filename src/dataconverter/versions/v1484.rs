//! V1484 (18w19a) — seagrass block + item renames; cites `V1484.java`.
//!
//! Ported:
//!   * ITEM rename (`ConverterAbstractItemRename.register(VERSION, renamed::get)`,
//!     V1484.java:25): seagrass map applied as ITEM_NAME renames.
//!   * BLOCK rename (`ConverterAbstractBlockRename.register(VERSION, renamed::get)`,
//!     V1484.java:26): same seagrass map. The block-rename helper registers on
//!     BLOCK_NAME + BLOCK_STATE(Name) + FLAT_BLOCK_STATE.
//!
//! VERSION = MCVersions.V18W19A = 1484.
//!
//! Skipped: the CHUNK Heightmaps structure converter (V1484.java:28-71) — out of
//! scope for the schematic data-converter (chunk-only).

use super::super::helpers::{map_renamer, register_block_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 1484;

/// Renamed seagrass ids (V1484.java:18-23); applied to both items and blocks.
const RENAMED_IDS: &[(&str, &str)] = &[
    ("minecraft:sea_grass", "minecraft:seagrass"),
    ("minecraft:tall_sea_grass", "minecraft:tall_seagrass"),
];

pub fn register(reg: &mut RegistryBuilder) {
    register_item_rename(reg, VERSION, map_renamer(RENAMED_IDS));
    register_block_rename(reg, VERSION, map_renamer(RENAMED_IDS));
}
