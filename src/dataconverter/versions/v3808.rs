//! V3808 (24w04a + 2) — schematic-relevant subset of `V3808.java`.
//!
//! The "body armor" migration for horses/llamas: the old armor/decor slot
//! compound is moved to `body_armor_item`, a `body_armor_drop_chance` of 2.0F is
//! written, and (for the horse) the now-unused chest-slot armor entry is cleared
//! (V3808.java:32-75).
//!
//! Ported (all ENTITY, schematic-relevant):
//!   * step 0 — `minecraft:horse`: `ArmorItem` -> body armor (clears
//!     `ArmorItems[2]` / `ArmorDropChances[2]`), plus a `SaddleItem` walker.
//!   * step 1 — `minecraft:llama`: `DecorItem` -> body armor (no clear), plus
//!     `Items` (item-list) + `SaddleItem` walkers.
//!   * step 2 — `minecraft:trader_llama`: same as llama.
//!
//! Nothing non-schematic exists in this version file.
//!
//! VERSION = MCVersions.V24W04A (3806) + 2 = 3808.

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};
use super::super::walker::{item_lists, items};

const VERSION: i32 = 3808;

/// `BodyArmorConverter.convert` (V3808.java:46-69): move `path` to
/// `body_armor_item`, set the drop chance, and (optionally) clear the chest
/// armor slot.
fn body_armor_convert(data: &mut NbtMap, path: &str, clear_armor: bool) {
    let prev = match data.take(path) {
        Some(NbtValue::Compound(m)) => m,
        // Java: `getMap(path) == null` -> return (a non-compound also reads null).
        Some(_) => return,
        None => return,
    };

    data.set_map("body_armor_item", prev);
    data.set_f32("body_armor_drop_chance", 2.0);

    if clear_armor {
        if let Some(armor) = data.get_list_mut("ArmorItems") {
            if armor.len() > 2 {
                armor[2] = NbtValue::Compound(NbtMap::new());
            }
        }
        if let Some(chances) = data.get_list_mut("ArmorDropChances") {
            if chances.len() > 2 {
                chances[2] = NbtValue::Float(0.085);
            }
        }
    }
}

/// Inverse of `body_armor_convert`: move `body_armor_item` back to `path` and
/// drop the additive `body_armor_drop_chance`.
///
/// Lossless for real downgrades: the forward was a rename (`path` ->
/// `body_armor_item`) plus an additive default (`body_armor_drop_chance` =
/// 2.0F). The new `body_armor_item` uniquely encodes the old `path` compound,
/// so reversing it is exact.
///
/// `clear_armor` (the horse case) is intentionally NOT undone: the forward
/// overwrote `ArmorItems[2]`/`ArmorDropChances[2]`, but in the pre-3808 horse
/// format the chest armor slot was unused and those slots carried exactly the
/// values the forward wrote (empty item, 0.085F). The modern data therefore
/// already holds the canonical old values; there is nothing to restore and no
/// information is lost.
fn body_armor_reverse(data: &mut NbtMap, path: &str) {
    let prev = match data.take("body_armor_item") {
        Some(NbtValue::Compound(m)) => m,
        Some(other) => {
            data.set_generic("body_armor_item", other);
            return;
        }
        None => return,
    };

    data.set_map(path, prev);
    if let Some(chance) = data.take("body_armor_drop_chance") {
        let is_forward_default = chance.as_number_f64().map(|v| v == 2.0).unwrap_or(false);
        if !is_forward_default {
            report_loss(
                VERSION,
                LossKind::UnsupportedInTarget,
                Severity::Loss,
                format!("{path} body_armor_drop_chance is not representable before 3808; dropped"),
            );
        }
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    // Step 0 — horse.
    reg.entity.add_converter_for_id(
        "minecraft:horse",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| body_armor_convert(data, "ArmorItem", true)),
    );
    reg.entity.add_reverse_converter_for_id(
        "minecraft:horse",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| body_armor_reverse(data, "ArmorItem")),
    );
    reg.entity
        .add_walker(VERSION, 0, "minecraft:horse", items(&["SaddleItem"]));

    // Step 1 — llama.
    reg.entity.add_converter_for_id(
        "minecraft:llama",
        VERSION,
        1,
        Box::new(|data: &mut NbtMap, _from, _to| body_armor_convert(data, "DecorItem", false)),
    );
    reg.entity.add_reverse_converter_for_id(
        "minecraft:llama",
        VERSION,
        1,
        Box::new(|data: &mut NbtMap, _from, _to| body_armor_reverse(data, "DecorItem")),
    );
    reg.entity
        .add_walker(VERSION, 1, "minecraft:llama", item_lists(&["Items"]));
    reg.entity
        .add_walker(VERSION, 1, "minecraft:llama", items(&["SaddleItem"]));

    // Step 2 — trader_llama.
    reg.entity.add_converter_for_id(
        "minecraft:trader_llama",
        VERSION,
        2,
        Box::new(|data: &mut NbtMap, _from, _to| body_armor_convert(data, "DecorItem", false)),
    );
    reg.entity.add_reverse_converter_for_id(
        "minecraft:trader_llama",
        VERSION,
        2,
        Box::new(|data: &mut NbtMap, _from, _to| body_armor_reverse(data, "DecorItem")),
    );
    reg.entity
        .add_walker(VERSION, 2, "minecraft:trader_llama", item_lists(&["Items"]));
    reg.entity
        .add_walker(VERSION, 2, "minecraft:trader_llama", items(&["SaddleItem"]));
}
