//! V3083 (22w12a + 1) — schematic-relevant subset of `V3083.java`.
//!
//! Kept: the ENTITY `minecraft:allay` walkers — an item list (`Inventory`) and a
//! `GameEventListenerWalker` (`listener.event.game_event` -> GAME_EVENT_NAME).
//! Both walkers are registered separately, matching Java (the engine runs every
//! walker registered for an id).
//!
//! Skipped (non-schematic): the commented-out allay mob registration.
//!
//! VERSION = MCVersions.V22W12A (3082) + 1 = 3083.

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::{convert_value, item_lists};

const VERSION: i32 = 3083;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_walker(VERSION, 0, "minecraft:allay", item_lists(&["Inventory"]));
    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:allay",
        Arc::new(|reg, data, from, to| {
            // GameEventListenerWalker: listener.event.game_event -> GAME_EVENT_NAME.
            if let Some(listener) = data.get_map_mut("listener") {
                if let Some(event) = listener.get_map_mut("event") {
                    convert_value(&reg.game_event_name, event, "game_event", from, to);
                }
            }
        }),
    );
}
