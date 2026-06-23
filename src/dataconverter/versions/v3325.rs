//! V3325 (V23W05A + 2 = 3325) — display-entity walkers; cites `V3325.java`.
//!
//! Ported (all ENTITY, schematic-relevant), step 0:
//!   * `minecraft:item_display` — `DataWalkerItems("item")`: the held item stack
//!     walks through ITEM_STACK (V3325.java:13).
//!   * `minecraft:block_display` — `DataWalkerTypePaths(BLOCK_STATE,
//!     "block_state")`: the `block_state` walks through BLOCK_STATE
//!     (V3325.java:14).
//!   * `minecraft:text_display` — `DataWalkerTypePaths(TEXT_COMPONENT, "text")`:
//!     the `text` field walks through TEXT_COMPONENT (V3325.java:15).
//!
//! Nothing non-schematic exists in this version file.

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::walker::{convert, items};

const VERSION: i32 = 3325;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity
        .add_walker(VERSION, 0, "minecraft:item_display", items(&["item"]));

    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:block_display",
        Arc::new(|reg, data, from, to| {
            convert(reg, &reg.block_state, data, "block_state", from, to)
        }),
    );

    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:text_display",
        Arc::new(|reg, data, from, to| convert(reg, &reg.text_component, data, "text", from, to)),
    );
}
