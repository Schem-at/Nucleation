//! V3938 (1.20.6 + 99) — schematic-relevant subset of `V3938.java`.
//!
//! Adds walkers for the arrow entities (`minecraft:spectral_arrow`,
//! `minecraft:arrow`): `inBlockState` -> BLOCK_STATE, and the new `item` /
//! `weapon` single-itemstack fields -> ITEM_STACK.
//!
//! Nothing non-schematic is present in this version.
//!
//! VERSION = MCVersions.V1_20_6 (3839) + 99 = 3938.

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::walker::{convert, items};

const VERSION: i32 = 3938;

fn register_arrow(reg: &mut RegistryBuilder, id: &str) {
    // DataWalkerTypePaths<>(BLOCK_STATE, "inBlockState").
    reg.entity.add_walker(
        VERSION,
        0,
        id,
        Arc::new(|reg, data, from, to| {
            convert(reg, &reg.block_state, data, "inBlockState", from, to);
        }),
    );
    // DataWalkerItems("item", "weapon").
    reg.entity
        .add_walker(VERSION, 0, id, items(&["item", "weapon"]));
}

pub fn register(reg: &mut RegistryBuilder) {
    register_arrow(reg, "minecraft:spectral_arrow");
    register_arrow(reg, "minecraft:arrow");
}
