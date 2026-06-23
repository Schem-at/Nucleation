//! V1470 (18w08a, 1470) — `minecraft:trident` entity walkers; cites V1470.java.
//!
//! VERSION = MCVersions.V18W08A = 1470.
//!
//! Ported (V1470.java:22-23):
//!   * ENTITY walker for `minecraft:trident` routing `inBlockState` through
//!     BLOCK_STATE (`DataWalkerTypePaths<BLOCK_STATE, "inBlockState">`).
//!   * ENTITY walker for `minecraft:trident` routing the `Trident` item field
//!     through ITEM_STACK (`DataWalkerItems("Trident")`).
//!
//! Skipped: the commented-out `registerMob(...)` blocks (V1470.java:13-20),
//! which are no-ops in the Java source.

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::walker::{convert, items};

const VERSION: i32 = 1470;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:trident",
        Arc::new(|reg, data, from, to| {
            convert(reg, &reg.block_state, data, "inBlockState", from, to)
        }),
    );
    reg.entity
        .add_walker(VERSION, 0, "minecraft:trident", items(&["Trident"]));
}
