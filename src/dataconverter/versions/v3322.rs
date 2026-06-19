//! V3322 (23w04a + 1) — schematic-relevant subset of `V3322.java`.
//!
//! The "entity effect" fix (V3322.java:26-66): for the effect lists `Effects`,
//! `ActiveEffects`, and `CustomPotionEffects`, each effect entry that carries a
//! `FactorCalculationData` map gets `effect_changed_timestamp` removed and
//! `ticks_active` written as `effect_changed_timestamp - Duration` (both default
//! to -1 when absent; `Duration` is read off the effect element, the timestamp
//! off `FactorCalculationData`).
//!
//! The Java file registers `entityEffectFix` on both PLAYER and ENTITY; only the
//! ENTITY registration is schematic-relevant, so the PLAYER one is skipped.
//!
//! It also registers an ITEM_STACK converter (V3322.java:68-80): when the item
//! `id` is one of potion/splash_potion/lingering_potion/tipped_arrow, the same
//! effect-list fix is applied to `tag.CustomPotionEffects`.
//!
//! VERSION = MCVersions.V23W04A (3321) + 1 = 3322.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 3322;

/// Item ids whose `tag.CustomPotionEffects` get the fix (V3322.java:17-24).
const EFFECT_ITEM_TYPES: &[&str] = &[
    "minecraft:potion",
    "minecraft:splash_potion",
    "minecraft:lingering_potion",
    "minecraft:tipped_arrow",
];

/// Port of `updateEffectList(root, path)` (V3322.java:26-52).
fn update_effect_list(root: &mut NbtMap, path: &str) {
    let effects = match root.get_list_mut(path) {
        Some(list) => list,
        None => return,
    };

    for el in effects.iter_mut() {
        // getList(path, MAP) only yields map elements; non-maps are ignored.
        let data = match el.as_compound_mut() {
            Some(map) => map,
            None => continue,
        };

        // Duration is read off the effect element (default -1).
        let duration = data.get_i32("Duration").unwrap_or(-1);

        let factor_data = match data.get_map_mut("FactorCalculationData") {
            Some(map) => map,
            None => continue,
        };

        let timestamp = factor_data.get_i32("effect_changed_timestamp").unwrap_or(-1);
        factor_data.take("effect_changed_timestamp");

        let ticks_active = timestamp - duration;
        factor_data.set_i32("ticks_active", ticks_active);
    }
}

/// Inverse of `update_effect_list`: restore `effect_changed_timestamp` and drop
/// `ticks_active` on every effect carrying `FactorCalculationData`.
///
/// Forward set `ticks_active = effect_changed_timestamp - Duration` (both default
/// -1) and removed `effect_changed_timestamp`. `Duration` lives on the effect
/// element and is untouched by the forward, so the original timestamp is exactly
/// recoverable as `ticks_active + Duration` — lossless (no `report_loss`).
fn restore_effect_list(root: &mut NbtMap, path: &str) {
    let effects = match root.get_list_mut(path) {
        Some(list) => list,
        None => return,
    };

    for el in effects.iter_mut() {
        let data = match el.as_compound_mut() {
            Some(map) => map,
            None => continue,
        };

        // Duration is read off the effect element (default -1), mirroring forward.
        let duration = data.get_i32("Duration").unwrap_or(-1);

        let factor_data = match data.get_map_mut("FactorCalculationData") {
            Some(map) => map,
            None => continue,
        };

        let ticks_active = factor_data.get_i32("ticks_active").unwrap_or(-1);
        factor_data.take("ticks_active");

        let timestamp = ticks_active + duration;
        factor_data.set_i32("effect_changed_timestamp", timestamp);
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    // ENTITY: Effects / ActiveEffects / CustomPotionEffects effect-list fix.
    // (The identical PLAYER registration is non-schematic and skipped.)
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            update_effect_list(data, "Effects");
            update_effect_list(data, "ActiveEffects");
            update_effect_list(data, "CustomPotionEffects");
        }),
    );

    // Reverse: restore effect_changed_timestamp / drop ticks_active (lossless).
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            restore_effect_list(data, "Effects");
            restore_effect_list(data, "ActiveEffects");
            restore_effect_list(data, "CustomPotionEffects");
        }),
    );

    // ITEM_STACK: for potion-like items, fix tag.CustomPotionEffects.
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let is_effect_item = data
                .get_string("id")
                .map(|id| EFFECT_ITEM_TYPES.contains(&id))
                .unwrap_or(false);
            if !is_effect_item {
                return;
            }

            if let Some(tag) = data.get_map_mut("tag") {
                update_effect_list(tag, "CustomPotionEffects");
            }
        }),
    );

    // Reverse: for potion-like items, restore tag.CustomPotionEffects (lossless).
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let is_effect_item = data
                .get_string("id")
                .map(|id| EFFECT_ITEM_TYPES.contains(&id))
                .unwrap_or(false);
            if !is_effect_item {
                return;
            }

            if let Some(tag) = data.get_map_mut("tag") {
                restore_effect_list(tag, "CustomPotionEffects");
            }
        }),
    );
}
