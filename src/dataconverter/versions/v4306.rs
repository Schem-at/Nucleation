//! V4306 (25w03a + 2) — schematic-relevant subset of `V4306.java`.
//!
//! ENTITY id-converter for `minecraft:potion`: the generic thrown-potion entity
//! is split by its held `Item`. If the item is `minecraft:lingering_potion` the
//! entity becomes `minecraft:lingering_potion`, otherwise
//! `minecraft:splash_potion` (V4306.java:439-454).
//!
//! Plus `copyWalkers` to carry the `minecraft:potion` walkers onto both new ids,
//! and an explicit no-op walker on the now-vestigial `minecraft:potion`
//! (V4306.java:456-458).
//!
//! VERSION = V25W03A(4304) + 2. Entirely schematic-relevant (ENTITY).

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4306;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:potion",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let is_lingering = data.get_map("Item").and_then(|item| item.get_string("id"))
                == Some("minecraft:lingering_potion");
            data.set_string(
                "id",
                if is_lingering {
                    "minecraft:lingering_potion"
                } else {
                    "minecraft:splash_potion"
                },
            );
        }),
    );

    // Reverse: the forward split keys the new entity id on the held `Item`, so
    // both `minecraft:splash_potion` and `minecraft:lingering_potion` came from
    // the single legacy `minecraft:potion` id. The discriminator survives in the
    // intact `Item`, so collapsing either new id back to `minecraft:potion` is
    // exact (lossless — rule 11). No `report_loss`.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:splash_potion",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.set_string("id", "minecraft:potion");
        }),
    );
    reg.entity.add_reverse_converter_for_id(
        "minecraft:lingering_potion",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.set_string("id", "minecraft:potion");
        }),
    );

    reg.entity
        .copy_walkers(VERSION, 0, "minecraft:potion", "minecraft:splash_potion");
    reg.entity
        .copy_walkers(VERSION, 0, "minecraft:potion", "minecraft:lingering_potion");
    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:potion",
        Arc::new(|_reg, _data, _from, _to| {}),
    );
}
