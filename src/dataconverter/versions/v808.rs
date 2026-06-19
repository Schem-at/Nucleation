//! V808 (16w38a+1) — schematic-relevant subset.
//!
//! Port of DataConverterJava .../versions/V808.java:
//!   * ENTITY `minecraft:shulker`: default `Color` byte to 10 when absent
//!     (registered at step 1, matching the Java `new DataConverter<>(VERSION, 1)`).
//!   * TILE_ENTITY `minecraft:shulker_box`: walk its `Items` item list.
//!
//! Nothing is skipped — both registrations target schematic types.

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::item_lists;

const VERSION: i32 = 808;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:shulker",
        VERSION,
        1,
        Box::new(|data, _from, _to| {
            // hasKey("Color", NUMBER): default only when no numeric Color present.
            if data.get_i64("Color").is_none() {
                data.set_byte("Color", 10);
            }
        }),
    );

    // Reverse: the forward added the default `Color=10` when absent (bucket D
    // additive default). The pre-808 shulker NBT had no `Color` field and rendered
    // as the default (purple/10), so dropping `Color` when it equals 10 restores the
    // exact old shape. We only remove the value the forward set; any other color was
    // user-authored and must be preserved. No loss: a purple shulker was represented
    // by the *absence* of Color in the old format anyway.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:shulker",
        VERSION,
        1,
        Box::new(|data, _from, _to| {
            if data.get_i64("Color") == Some(10) {
                data.take("Color");
            }
        }),
    );

    reg.tile_entity
        .add_walker(VERSION, 0, "minecraft:shulker_box", item_lists(&["Items"]));
}
