//! V3327 (23w06a + 1) — TILE_ENTITY walkers for decorated pots and suspicious
//! sand item payloads.
//!
//! V3327.java registers ITEM_NAME walking for decorated_pot `shards`, ITEM_STACK
//! walking for decorated_pot `item`, and ITEM_STACK walking for suspicious_sand
//! `item`.
use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::walker::{convert_value_list, items};

const VERSION: i32 = 3327;

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "minecraft:decorated_pot",
        Arc::new(|reg, data, from, to| {
            convert_value_list(&reg.item_name, data, "shards", from, to);
        }),
    );
    reg.tile_entity
        .add_walker(VERSION, 0, "minecraft:decorated_pot", items(&["item"]));

    // V3327.java:15 — DataWalkerItems("item") walks the single item stack through ITEM_STACK.
    reg.tile_entity
        .add_walker(VERSION, 0, "minecraft:suspicious_sand", items(&["item"]));
}
