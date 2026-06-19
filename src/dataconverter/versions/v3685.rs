//! V3685 (23w42a+1, `V23W42A + 1` = 3685) — schematic-relevant subset of
//! `V3685.java`.
//!
//! Arrow/trident entities gained an `item` itemstack. Ported:
//!   * ENTITY `minecraft:trident` converter: rename `Trident` -> `item`.
//!   * ENTITY `minecraft:arrow` converter: synthesize `item = {id, Count:1}` where
//!     id is `minecraft:tipped_arrow` when `Potion != minecraft:empty`, else
//!     `minecraft:arrow`.
//!   * ENTITY `minecraft:spectral_arrow` converter: synthesize
//!     `item = {id:"minecraft:spectral_arrow", Count:1}`.
//!   * Walkers for trident/spectral_arrow/arrow: `inBlockState` -> BLOCK_STATE and
//!     `item` -> ITEM_STACK.
//!
//! Nothing non-schematic here (all are schematic entities).

use std::sync::Arc;

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::{convert, items};

const VERSION: i32 = 3685;

/// `getType` (V3685.java:16-18): empty potion -> arrow, else tipped_arrow.
fn arrow_type(arrow: &NbtMap) -> &'static str {
    let potion = arrow.get_string("Potion").unwrap_or("minecraft:empty");
    if potion == "minecraft:empty" {
        "minecraft:arrow"
    } else {
        "minecraft:tipped_arrow"
    }
}

/// `createItem` (V3685.java:20-27): `{id, Count}`.
fn create_item(id: &str, count: i32) -> NbtMap {
    let mut ret = NbtMap::new();
    ret.set_string("id", id);
    ret.set_i32("Count", count);
    ret
}

fn is_synthesized_item(data: &NbtMap, expected_id: &str) -> bool {
    let Some(item) = data.get_map("item") else {
        return true;
    };
    item.get_string("id") == Some(expected_id)
        && item.get_i32("Count").unwrap_or(1) == 1
        && item
            .keys()
            .into_iter()
            .all(|key| key == "id" || key == "Count")
}

/// `registerArrowEntity` (V3685.java:29-33): inBlockState -> BLOCK_STATE,
/// item -> ITEM_STACK.
fn register_arrow_entity(reg: &mut RegistryBuilder, id: &'static str) {
    reg.entity.add_walker(
        VERSION,
        0,
        id,
        Arc::new(|reg, data, from, to| {
            convert(reg, &reg.block_state, data, "inBlockState", from, to)
        }),
    );
    reg.entity.add_walker(VERSION, 0, id, items(&["item"]));
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:trident",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.rename_key("Trident", "item");
        }),
    );
    // Reverse: undo the manual `Trident` -> `item` rename (lossless, bucket A).
    // This rename is done via add_converter_for_id (not map_renamer), so it is
    // NOT auto-inverted by the engine — invert it explicitly. Matches the NEW key
    // (`item`).
    reg.entity.add_reverse_converter_for_id(
        "minecraft:trident",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.rename_key("item", "Trident");
        }),
    );
    reg.entity.add_converter_for_id(
        "minecraft:arrow",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let item = create_item(arrow_type(data), 1);
            data.set_map("item", item);
        }),
    );
    // Reverse: drop the synthesized `item` (bucket D additive). The forward derives
    // it entirely from the still-present `Potion` field (arrow vs tipped_arrow), so
    // removal is exact — no loss.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:arrow",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let expected = arrow_type(data);
            if !is_synthesized_item(data, expected) {
                report_loss(
                    VERSION,
                    LossKind::UnsupportedInTarget,
                    Severity::Loss,
                    "arrow item field contains modern-only state with no pre-23w42a entity representation",
                );
            }
            data.take("item");
        }),
    );
    reg.entity.add_converter_for_id(
        "minecraft:spectral_arrow",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let item = create_item("minecraft:spectral_arrow", 1);
            data.set_map("item", item);
        }),
    );
    // Reverse: drop the synthesized constant `item` (bucket D additive). The forward
    // adds a fixed `{id:"minecraft:spectral_arrow", Count:1}` that the old format
    // never carried, so removal is exact — no loss.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:spectral_arrow",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if !is_synthesized_item(data, "minecraft:spectral_arrow") {
                report_loss(
                    VERSION,
                    LossKind::UnsupportedInTarget,
                    Severity::Loss,
                    "spectral_arrow item field contains modern-only state with no pre-23w42a entity representation",
                );
            }
            data.take("item");
        }),
    );

    register_arrow_entity(reg, "minecraft:trident");
    register_arrow_entity(reg, "minecraft:spectral_arrow");
    register_arrow_entity(reg, "minecraft:arrow");
}
