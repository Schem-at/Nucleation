//! V3082 (22w11a + 2) — schematic-relevant subset of `V3082.java`.
//!
//! Kept: the ENTITY `minecraft:chest_boat` walker (item list `Items`).
//!
//! VERSION = MCVersions.V22W11A (3080) + 2 = 3082.

use super::super::registry::RegistryBuilder;
use super::super::walker::item_lists;

const VERSION: i32 = 3082;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity
        .add_walker(VERSION, 0, "minecraft:chest_boat", item_lists(&["Items"]));
}
