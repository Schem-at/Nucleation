//! V2717 (1.17-pre1 + 1) — schematic-relevant subset of `V2717.java`.
//!
//! `azalea_leaves_flowers` -> `flowering_azalea_leaves` item + block rename
//! (V2717.java:14-22). Both the ITEM_NAME and the
//! BLOCK_NAME/BLOCK_STATE/FLAT_BLOCK_STATE forms are renamed (Java registers the
//! same map on `ConverterAbstractItemRename` and `ConverterAbstractBlockRename`).
//! Nothing non-schematic is present.
//!
//! VERSION = MCVersions.V1_17_PRE1 (2716) + 1 = 2717.

use super::super::helpers::{map_renamer, register_block_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 2717;

const RENAMES: &[(&str, &str)] = &[(
    "minecraft:azalea_leaves_flowers",
    "minecraft:flowering_azalea_leaves",
)];

pub fn register(reg: &mut RegistryBuilder) {
    register_item_rename(reg, VERSION, map_renamer(RENAMES));
    register_block_rename(reg, VERSION, map_renamer(RENAMES));
}
