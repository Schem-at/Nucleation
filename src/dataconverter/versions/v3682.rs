//! V3682 (23w41a+1, `V23W41A + 1` = 3682) — schematic-relevant subset of
//! `V3682.java`.
//!
//! Registers the `minecraft:crafter` block entity as a "named inventory":
//! `V1458.namedInventory(VERSION, "minecraft:crafter")` expands (V1458.java:28-35)
//! to a TILE_ENTITY `CustomName` TEXT_COMPONENT walker plus an `Items` itemstack
//! list walker. Both are schematic-relevant (crafter is a block entity).

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::walker::{convert, item_lists};

const VERSION: i32 = 3682;

pub fn register(reg: &mut RegistryBuilder) {
    // named(): CustomName -> TEXT_COMPONENT.
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "minecraft:crafter",
        Arc::new(|reg, data, from, to| {
            convert(reg, &reg.text_component, data, "CustomName", from, to)
        }),
    );
    // + Items itemstack list.
    reg.tile_entity
        .add_walker(VERSION, 0, "minecraft:crafter", item_lists(&["Items"]));
}
