//! V4533 (1.21.8 + 93) — schematic-relevant subset of
//! `DataConverterJava/.../versions/V4533.java`.
//!
//! Registers a TILE_ENTITY walker for `minecraft:shelf` that recurses its
//! `Items` item-list so contained item stacks are themselves converted
//! (V4533.java:12). Nothing non-schematic is present in this version.
//!
//! VERSION = MCVersions.V1_21_8 (4440) + 93 = 4533.

use super::super::registry::RegistryBuilder;
use super::super::walker::item_lists;

const VERSION: i32 = 4533;

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity
        .add_walker(VERSION, 0, "minecraft:shelf", item_lists(&["Items"]));
}
