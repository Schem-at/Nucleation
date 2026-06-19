//! V2684 (20w48a + 1) — schematic-relevant subset of `V2684.java`.
//!
//! TILE_ENTITY walker for `minecraft:sculk_sensor`: the GameEventListenerWalker
//! descends `listener.event.game_event` and converts it through GAME_EVENT_NAME
//! (V2684.java:11-13, GameEventListenerWalker.java:11-19).
//!
//! VERSION = MCVersions.V20W48A (2683) + 1 = 2684.
//!
//! Nothing non-schematic is present in this version.

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::convert_value;

const VERSION: i32 = 2684;

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "minecraft:sculk_sensor",
        Arc::new(|reg, data, from, to| {
            // data.getMap("listener") -> getMap("event") -> convert "game_event".
            if let Some(listener) = data.get_map_mut("listener") {
                if let Some(event) = listener.get_map_mut("event") {
                    convert_value(&reg.game_event_name, event, "game_event", from, to);
                }
            }
        }),
    );
}
