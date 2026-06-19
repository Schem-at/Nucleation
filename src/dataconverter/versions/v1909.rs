//! V1909 (18w45a+1) — schematic-relevant subset of `V1909.java`.
//!
//! Registers a TILE_ENTITY walker for `minecraft:jigsaw` whose `final_state`
//! field is a FLAT_BLOCK_STATE string (V1909.java:12). Java uses
//! `DataWalkerTypePaths(FLAT_BLOCK_STATE, "final_state")`, i.e. a single value
//! descent; we express it inline via `convert_value` on `reg.flat_block_state`.
//!
//! VERSION = MCVersions.V18W45A (1908) + 1 = 1909.

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::walker::convert_value;

const VERSION: i32 = 1909;

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "minecraft:jigsaw",
        Arc::new(|reg, data, from, to| {
            convert_value(&reg.flat_block_state, data, "final_state", from, to);
        }),
    );
}
