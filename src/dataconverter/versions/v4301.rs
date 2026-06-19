//! V4301 (25w02a + 3) — schematic-relevant subset of `V4301.java`.
//!
//! ENTITY_EQUIPMENT structure converter that folds the legacy flat equipment
//! fields into a single `equipment` compound (V4301.java:249-307):
//!   * `body_armor_item` -> `equipment.body`, `saddle` -> `equipment.saddle`.
//!   * `ArmorItems` list (0..4) -> feet/legs/chest/head.
//!   * `HandItems` list (0..2) -> mainhand/offhand.
//! Each source item is only kept if it actually has an `id` (empty `{}` slots are
//! dropped). The legacy fields are removed, and `equipment` is only written when
//! non-empty.
//!
//! Plus the ENTITY_EQUIPMENT structure walker that recurses ITEM_STACK over every
//! equipment slot (V4301.java:309-323).
//!
//! VERSION = V25W02A(4298) + 3. Entirely schematic-relevant (ENTITY_EQUIPMENT).

use std::sync::Arc;

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};
use super::super::walker::convert;

const VERSION: i32 = 4301;

/// `ArmorItems` index -> equipment slot (V4301.java:250-255).
const ARMOR_SLOTS: &[&str] = &["feet", "legs", "chest", "head"];
/// `HandItems` index -> equipment slot (V4301.java:257-260).
const HAND_SLOTS: &[&str] = &["mainhand", "offhand"];

/// `filterItem` (V4301.java:262-264): keep the item only if it has an `id`.
fn keep_item(item: &NbtMap) -> bool {
    item.has_key("id")
}

/// `moveItems` (V4301.java:272-284): for each of the first `names.len()` entries
/// of `src[src_path]` that is a compound carrying an `id`, copy it into
/// `dst[names[i]]`.
fn move_items(src: &NbtMap, src_path: &str, names: &[&str], dst: &mut NbtMap) {
    let list = match src.get_list(src_path) {
        Some(l) => l,
        None => return,
    };
    let len = list.len().min(names.len());
    for (i, name) in names.iter().enumerate().take(len) {
        if let Some(item) = list[i].as_compound_ref() {
            if keep_item(item) {
                dst.set_map(name, item.clone());
            }
        }
    }
}

/// Reverse of `move_items`: take the slots `names[i]` out of `equipment` and
/// rebuild the fixed-length legacy list `dst[dst_path]` (one entry per name,
/// empty `{}` for any missing slot — the canonical legacy placeholder, see
/// V100.java:32). Only emits the list when at least one slot survived.
fn rebuild_list(equipment: &mut NbtMap, names: &[&str], dst_path: &str, dst: &mut NbtMap) {
    if !names.iter().any(|name| equipment.has_key(name)) {
        return;
    }
    let list: Vec<NbtValue> = names
        .iter()
        .map(|name| match equipment.take(name) {
            Some(NbtValue::Compound(item)) => NbtValue::Compound(item),
            Some(_) => {
                report_loss(
                    VERSION,
                    LossKind::UnsupportedInTarget,
                    Severity::Loss,
                    format!(
                        "equipment.{name} is not an item compound and has no legacy {dst_path} slot representation; using empty placeholder"
                    ),
                );
                NbtValue::Compound(NbtMap::new())
            }
            None => NbtValue::Compound(NbtMap::new()),
        })
        .collect();
    dst.set_list(dst_path, list);
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity_equipment.add_structure_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let mut equipment = NbtMap::new();

            // body / saddle: only if present and carrying an id.
            if let Some(body) = data.get_map("body_armor_item") {
                if keep_item(body) {
                    equipment.set_map("body", body.clone());
                }
            }
            if let Some(saddle) = data.get_map("saddle") {
                if keep_item(saddle) {
                    equipment.set_map("saddle", saddle.clone());
                }
            }

            move_items(data, "ArmorItems", ARMOR_SLOTS, &mut equipment);
            move_items(data, "HandItems", HAND_SLOTS, &mut equipment);

            data.take("ArmorItems");
            data.take("HandItems");
            data.take("body_armor_item");
            data.take("saddle");

            if !equipment.inner().is_empty() {
                data.set_map("equipment", equipment);
            }
        }),
    );

    // Reverse: unfold the `equipment` compound back into the legacy flat fields
    // (bucket B, structural). Each equipment slot name deterministically maps to
    // a legacy field / list index, so this is the exact inverse:
    //   * `equipment.body` -> `body_armor_item`, `equipment.saddle` -> `saddle`.
    //   * feet/legs/chest/head -> a fixed-length-4 `ArmorItems` list (indices 0..3).
    //   * mainhand/offhand   -> a fixed-length-2 `HandItems` list (indices 0..1).
    // The forward dropped slots that lacked an `id`; the canonical legacy shape
    // (see V100.java:28-41) always carried empty `{}` placeholders for unused
    // slots, so reconstructing missing slots as `{}` is the exact preimage, not
    // loss (cheatsheet rule 11). The legacy lists/fields are only emitted when at
    // least one corresponding slot survived, mirroring the forward's "only write
    // `equipment` when non-empty" guard.
    reg.entity_equipment.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let mut equipment = match data.take("equipment") {
                Some(NbtValue::Compound(m)) => m,
                // Wrong type or absent: nothing to restore.
                other => {
                    if let Some(v) = other {
                        data.set_generic("equipment", v);
                    }
                    return;
                }
            };

            // body / saddle map straight back to their legacy field names.
            if let Some(body) = equipment.take("body") {
                match body {
                    NbtValue::Compound(body) => data.set_map("body_armor_item", body),
                    _ => report_loss(
                        VERSION,
                        LossKind::UnsupportedInTarget,
                        Severity::Loss,
                        "equipment.body is not an item compound and has no legacy body_armor_item representation; dropping it",
                    ),
                }
            }
            if let Some(saddle) = equipment.take("saddle") {
                match saddle {
                    NbtValue::Compound(saddle) => data.set_map("saddle", saddle),
                    _ => report_loss(
                        VERSION,
                        LossKind::UnsupportedInTarget,
                        Severity::Loss,
                        "equipment.saddle is not an item compound and has no legacy saddle representation; dropping it",
                    ),
                }
            }

            rebuild_list(&mut equipment, ARMOR_SLOTS, "ArmorItems", data);
            rebuild_list(&mut equipment, HAND_SLOTS, "HandItems", data);

            for key in equipment.keys() {
                equipment.take(&key);
                report_loss(
                    VERSION,
                    LossKind::UnsupportedInTarget,
                    Severity::Loss,
                    format!("equipment.{key} has no pre-4301 legacy slot representation; dropping it"),
                );
            }
        }),
    );

    reg.entity_equipment.add_structure_walker(
        VERSION,
        0,
        Arc::new(|reg, data, from, to| {
            if let Some(equipment) = data.get_map_mut("equipment") {
                for slot in [
                    "mainhand", "offhand", "feet", "legs", "chest", "head", "body", "saddle",
                ] {
                    convert(reg, &reg.item_stack, equipment, slot, from, to);
                }
            }
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::loss;
    use crate::dataconverter::registry::{convert_reverse_under_session, registry};

    #[test]
    fn reverse_equipment_reports_non_compound_and_unknown_slots() {
        let mut data = NbtMap::new();
        let mut equipment = NbtMap::new();
        equipment.set_string("head", "not an item");
        equipment.set_string("unknown_slot", "extra");
        data.set_map("equipment", equipment);

        let reg = registry();
        let (_, report) = loss::run_reverse(|| {
            convert_reverse_under_session(&reg.entity_equipment, &mut data, VERSION, VERSION - 1);
        });

        assert_eq!(report.loss_count(), 2);
        assert!(data.get_map("equipment").is_none());
        assert!(data.get_list("ArmorItems").is_some());
    }
}
