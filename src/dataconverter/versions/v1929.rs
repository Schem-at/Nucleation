//! V1929 (19w04b+2) — schematic-relevant subset of `V1929.java`.
//!
//! ENTITY walkers:
//!   * `minecraft:wandering_trader`: `Inventory` is a list of item stacks and
//!     `Offers.Recipes` recurse through VILLAGER_TRADE (V1929.java:13-19).
//!   * `minecraft:trader_llama`: `SaddleItem` and `DecorItem` are single item
//!     stacks, and `Items` is a list of item stacks (V1929.java:21-28).
//!
//! VERSION = MCVersions.V19W04B (1927) + 2 = 1929.

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::{convert, convert_list};

const VERSION: i32 = 1929;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:wandering_trader",
        Arc::new(|reg, data, from, to| {
            convert_list(reg, &reg.item_stack, data, "Inventory", from, to);
            if let Some(offers) = data.get_map_mut("Offers") {
                convert_list(reg, &reg.villager_trade, offers, "Recipes", from, to);
            }
        }),
    );

    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:trader_llama",
        Arc::new(|reg, data, from, to| {
            convert(reg, &reg.item_stack, data, "SaddleItem", from, to);
            convert(reg, &reg.item_stack, data, "DecorItem", from, to);
            convert_list(reg, &reg.item_stack, data, "Items", from, to);
        }),
    );
}
