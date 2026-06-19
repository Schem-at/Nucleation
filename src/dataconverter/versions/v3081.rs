//! V3081 (22w11a + 1) — schematic-relevant subset of `V3081.java`.
//!
//! Kept: the ENTITY `minecraft:warden` walker, a `GameEventListenerWalker` that
//! descends `listener.event.game_event` -> GAME_EVENT_NAME.
//!
//! Skipped (non-schematic): the commented-out warden mob registration (commented
//! out in Java).
//!
//! VERSION = MCVersions.V22W11A (3080) + 1 = 3081.

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::convert_value;

const VERSION: i32 = 3081;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:warden",
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
