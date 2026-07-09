//! V100 (15w32a) — ENTITY: split the legacy flat `Equipment`/`DropChances`
//! lists into `HandItems`/`ArmorItems` and `HandDropChances`/`ArmorDropChances`
//! (mainhand = index 0, armor = indices 1..4); cites V100.java:21-74.
//! Plus an ENTITY_EQUIPMENT structure walker recursing ITEM_STACK over the new
//! equipment paths (V100.java:75-82). The commented-out `registerMob(...)`
//! blocks (V100.java:84-118) are intentionally skipped.

use std::sync::Arc;

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::{convert, convert_list};

const VERSION: i32 = 100;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // Java: getList("Equipment", ObjectType.MAP) — only a list whose
            // elements are maps; then remove the key unconditionally.
            let equipment: Option<Vec<NbtMap>> = match data.take("Equipment") {
                Some(NbtValue::List(items)) => {
                    if items.iter().all(|v| matches!(v, NbtValue::Compound(_))) {
                        Some(
                            items
                                .into_iter()
                                .map(|v| match v {
                                    NbtValue::Compound(m) => m,
                                    _ => unreachable!(),
                                })
                                .collect(),
                        )
                    } else {
                        // Mixed/non-map list reads as null in Java.
                        None
                    }
                }
                _ => None,
            };

            if let Some(equipment) = equipment {
                if !equipment.is_empty() && data.get_list("HandItems").is_none() {
                    let mut hand_items: Vec<NbtValue> = Vec::new();
                    hand_items.push(NbtValue::Compound(equipment[0].clone()));
                    hand_items.push(NbtValue::Compound(NbtMap::new()));
                    data.set_list("HandItems", hand_items);
                }

                if equipment.len() > 1 && data.get_list("ArmorItems").is_none() {
                    let mut armor_items: Vec<NbtValue> = Vec::new();
                    for i in 1..equipment.len().min(5) {
                        armor_items.push(NbtValue::Compound(equipment[i].clone()));
                    }
                    data.set_list("ArmorItems", armor_items);
                }
            }

            // Java: getList("DropChances", ObjectType.FLOAT) — only a list whose
            // elements are floats; then remove the key unconditionally.
            let drop_chances: Option<Vec<f32>> = match data.take("DropChances") {
                Some(NbtValue::List(items)) => {
                    if items.iter().all(|v| matches!(v, NbtValue::Float(_))) {
                        Some(
                            items
                                .into_iter()
                                .map(|v| match v {
                                    NbtValue::Float(f) => f,
                                    _ => unreachable!(),
                                })
                                .collect(),
                        )
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(drop_chances) = drop_chances {
                if data.get_list("HandDropChances").is_none() {
                    let mut hand_drop_chances: Vec<NbtValue> = Vec::new();
                    if !drop_chances.is_empty() {
                        hand_drop_chances.push(NbtValue::Float(drop_chances[0]));
                    } else {
                        hand_drop_chances.push(NbtValue::Float(0.0));
                    }
                    hand_drop_chances.push(NbtValue::Float(0.0));
                    data.set_list("HandDropChances", hand_drop_chances);
                }

                if data.get_list("ArmorDropChances").is_none() {
                    let mut armor_drop_chances: Vec<NbtValue> = Vec::new();
                    for i in 1..5 {
                        if i < drop_chances.len() {
                            armor_drop_chances.push(NbtValue::Float(drop_chances[i]));
                        } else {
                            armor_drop_chances.push(NbtValue::Float(0.0));
                        }
                    }
                    data.set_list("ArmorDropChances", armor_drop_chances);
                }
            }
        }),
    );

    // Reverse of V100.java:21-74 — merge the split lists back into the legacy
    // flat `Equipment`/`DropChances` and strip the forward-added padding.
    //
    // Forward built:
    //   HandItems = [Equipment[0], {} ]                  (mainhand + empty offhand pad)
    //   ArmorItems = Equipment[1..min(5)]                (feet,legs,chest,head)
    //   HandDropChances = [DropChances[0] or 0.0, 0.0]   (mainhand + 0.0 offhand pad)
    //   ArmorDropChances = DropChances[1..5] (0.0-padded) (feet,legs,chest,head)
    // so the legacy preimage is Equipment = [mainhand, ..armor] and
    // DropChances = [mainhand_chance, ..armor_chances]. The offhand slot
    // (HandItems[1] / HandDropChances[1]) has no representation in the legacy
    // schema; the forward always put a default there, but by the time this runs
    // newer versions are already undone and could carry a real offhand value.
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // Read & remove the new item lists (Java getList equivalents: only
            // accept lists, treat anything else as absent).
            let hand_items = take_map_list(data, "HandItems");
            let armor_items = take_map_list(data, "ArmorItems");

            if hand_items.is_some() || armor_items.is_some() {
                let hand = hand_items.unwrap_or_default();
                let armor = armor_items.unwrap_or_default();

                // Offhand item (HandItems[1]) cannot survive the merge — drop it
                // if it carries real data.
                if let Some(offhand) = hand.get(1) {
                    if let NbtValue::Compound(m) = offhand {
                        if !m.is_empty() {
                            report_loss(
                                VERSION,
                                LossKind::UnsupportedInTarget,
                                Severity::Loss,
                                "entity offhand item (HandItems[1]) has no legacy Equipment slot",
                            );
                        }
                    }
                }

                let mut equipment: Vec<NbtValue> = Vec::new();
                // Equipment[0] = mainhand; if HandItems is empty, fall back to {}
                // to mirror the forward's index-0 element.
                equipment.push(
                    hand.into_iter()
                        .next()
                        .unwrap_or_else(|| NbtValue::Compound(NbtMap::new())),
                );
                // Equipment[1..] = the armor slots, in order.
                equipment.extend(armor);

                data.set_list("Equipment", equipment);
            }

            let hand_drop = take_float_list(data, "HandDropChances");
            let armor_drop = take_float_list(data, "ArmorDropChances");

            if hand_drop.is_some() || armor_drop.is_some() {
                let hand = hand_drop.unwrap_or_default();
                let armor = armor_drop.unwrap_or_default();

                // Offhand drop chance (HandDropChances[1]) is likewise lost.
                if let Some(&offhand_chance) = hand.get(1) {
                    if offhand_chance != 0.0 {
                        report_loss(
                            VERSION,
                            LossKind::UnsupportedInTarget,
                            Severity::Loss,
                            "entity offhand drop chance (HandDropChances[1]) has no legacy DropChances slot",
                        );
                    }
                }

                let mut drop_chances: Vec<NbtValue> = Vec::new();
                // DropChances[0] = mainhand drop chance.
                drop_chances.push(NbtValue::Float(hand.into_iter().next().unwrap_or(0.0)));
                // DropChances[1..] = the armor drop chances, in order.
                for c in armor {
                    drop_chances.push(NbtValue::Float(c));
                }

                data.set_list("DropChances", drop_chances);
            }
        }),
    );

    reg.entity_equipment.add_structure_walker(
        VERSION,
        0,
        Arc::new(|reg, data, from, to| {
            convert_list(reg, &reg.item_stack, data, "ArmorItems", from, to);
            convert_list(reg, &reg.item_stack, data, "HandItems", from, to);
            convert(reg, &reg.item_stack, data, "body_armor_item", from, to);
            convert(reg, &reg.item_stack, data, "saddle", from, to);
        }),
    );
}

/// Remove `key` and, mirroring Java `getList(key, ObjectType.MAP)`, return its
/// elements only when every element is a compound (otherwise treat as absent).
fn take_map_list(data: &mut NbtMap, key: &str) -> Option<Vec<NbtValue>> {
    match data.take(key) {
        Some(NbtValue::List(items)) => {
            if items.iter().all(|v| matches!(v, NbtValue::Compound(_))) {
                Some(items)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Remove `key` and, mirroring Java `getList(key, ObjectType.FLOAT)`, return its
/// float elements only when every element is a float (otherwise absent).
fn take_float_list(data: &mut NbtMap, key: &str) -> Option<Vec<f32>> {
    match data.take(key) {
        Some(NbtValue::List(items)) => {
            if items.iter().all(|v| matches!(v, NbtValue::Float(_))) {
                Some(
                    items
                        .into_iter()
                        .map(|v| match v {
                            NbtValue::Float(f) => f,
                            _ => unreachable!(),
                        })
                        .collect(),
                )
            } else {
                None
            }
        }
        _ => None,
    }
}
