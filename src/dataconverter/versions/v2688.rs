//! V2688 (20w51a + 1) — schematic-relevant subset of `V2688.java`.
//!
//! ENTITY walker for `minecraft:glow_item_frame`: its single `Item` slot is an
//! itemstack and must be recursed (V2688.java:14, DataWalkerItems("Item")).
//!
//! VERSION = MCVersions.V20W51A (2687) + 1 = 2688.
//!
//! Skipped (non-schematic): the commented-out `registerMob("minecraft:glow_squid")`
//! (a no-op in the Java source).

use super::super::registry::RegistryBuilder;
use super::super::walker::items;

const VERSION: i32 = 2688;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity
        .add_walker(VERSION, 0, "minecraft:glow_item_frame", items(&["Item"]));
}
