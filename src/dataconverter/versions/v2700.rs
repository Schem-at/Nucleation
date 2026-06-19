//! V2700 (21w10a + 1) — schematic-relevant subset of `V2700.java`.
//!
//! Cave-vine block renames (V2700.java:12-19): `cave_vines_head` -> `cave_vines`
//! and `cave_vines_body` -> `cave_vines_plant`. Java only registers
//! `ConverterAbstractBlockRename` here (no item rename), so we only register the
//! block-side rename. Nothing non-schematic is present.
//!
//! VERSION = MCVersions.V21W10A (2699) + 1 = 2700.

use super::super::helpers::{map_renamer, register_block_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 2700;

const RENAMES: &[(&str, &str)] = &[
    ("minecraft:cave_vines_head", "minecraft:cave_vines"),
    ("minecraft:cave_vines_body", "minecraft:cave_vines_plant"),
];

pub fn register(reg: &mut RegistryBuilder) {
    register_block_rename(reg, VERSION, map_renamer(RENAMES));
}
