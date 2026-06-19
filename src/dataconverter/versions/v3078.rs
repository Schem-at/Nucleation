//! V3078 (1.18.2 + 103) — schematic-relevant subset of `V3078.java`.
//!
//! Kept: the TILE_ENTITY `minecraft:sculk_shrieker` walker, which is a
//! `GameEventListenerWalker` — it descends into `listener.event.game_event` and
//! converts it through the GAME_EVENT_NAME value type.
//!
//! Skipped (non-schematic): the commented-out frog/tadpole mob registrations
//! (commented out in Java; nothing emitted).
//!
//! VERSION = MCVersions.V1_18_2 (2975) + 103 = 3078.

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::convert_value;

const VERSION: i32 = 3078;

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "minecraft:sculk_shrieker",
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
