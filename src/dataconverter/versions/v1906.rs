//! V1906 (18w43c + 3) — schematic-relevant subset of `V1906.java`.
//!
//! VERSION = MCVersions.V18W43C (1903) + 3 = 1906.
//!
//! Ported (the entire version is schematic-relevant — all TILE_ENTITY walkers):
//!   * `namedInventory` walkers for `minecraft:barrel`, `minecraft:smoker`,
//!     `minecraft:blast_furnace` (V1906.java:14-16).
//!   * `minecraft:lectern` Book item walker (V1906.java:17).
//!
//! `V1458.namedInventory(version, id)` is `V1458.named` + a `DataWalkerItemLists`
//! over `Items` (V1458.java:28-35), where `named` adds a `DataWalkerTypePaths`
//! that routes the `CustomName` path through TEXT_COMPONENT. V1458 is not yet
//! ported, so the two component walkers are registered inline here (identical to
//! how `register_inventory` works in v99.rs, plus the CustomName text walker).

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::walker::{convert, item_lists, items};

const VERSION: i32 = 1906;

/// `V1458.named(version, id)` — route `CustomName` through TEXT_COMPONENT.
fn register_named(reg: &mut RegistryBuilder, id: &str) {
    reg.tile_entity.add_walker(
        VERSION,
        0,
        id,
        Arc::new(|reg, data, from, to| {
            convert(reg, &reg.text_component, data, "CustomName", from, to)
        }),
    );
}

/// `V1458.namedInventory(version, id)` — `named` + an `Items` item-list walker.
fn register_named_inventory(reg: &mut RegistryBuilder, id: &str) {
    register_named(reg, id);
    reg.tile_entity
        .add_walker(VERSION, 0, id, item_lists(&["Items"]));
}

pub fn register(reg: &mut RegistryBuilder) {
    register_named_inventory(reg, "minecraft:barrel");
    register_named_inventory(reg, "minecraft:smoker");
    register_named_inventory(reg, "minecraft:blast_furnace");

    reg.tile_entity
        .add_walker(VERSION, 0, "minecraft:lectern", items(&["Book"]));
}
