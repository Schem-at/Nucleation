//! V4300 (25w02a + 2) — schematic-relevant subset of `V4300.java`.
//!
//! The saddle item moved out of the legacy mob fields into the new `saddle`
//! slot (later folded under ENTITY_EQUIPMENT by V4301). The ENTITY structure
//! converter (V4300.java:162-218):
//!   * "saddle item" mobs (horse family, camel, llama, trader_llama): rename
//!     `SaddleItem` -> `saddle` and, if it existed, set a guaranteed saddle drop
//!     chance (`drop_chances.saddle = 2.0`).
//!   * "saddle flag" mobs (pig, strider): the boolean `Saddle` flag becomes an
//!     actual `saddle` item (`{id:minecraft:saddle, count:1}`) plus the
//!     guaranteed drop chance, when set.
//!
//! Plus walkers: llama/trader_llama/donkey/mule keep an `Items` item-list walker
//! (their inventory), and horse/skeleton_horse/zombie_horse get an explicit
//! no-op walker (V4300.java:221-228).
//!
//! VERSION = V25W02A(4298) + 2. Everything here is schematic-relevant (ENTITY).

use std::sync::Arc;

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::item_lists;

const VERSION: i32 = 4300;

/// Mobs that stored the saddle as a `SaddleItem` compound (V4300.java:163-174).
const SADDLE_ITEM_ENTITIES: &[&str] = &[
    "minecraft:horse",
    "minecraft:skeleton_horse",
    "minecraft:zombie_horse",
    "minecraft:donkey",
    "minecraft:mule",
    "minecraft:camel",
    "minecraft:llama",
    "minecraft:trader_llama",
];

/// Mobs that stored the saddle as a boolean `Saddle` flag (V4300.java:175-180).
const SADDLE_FLAG_ENTITIES: &[&str] = &["minecraft:pig", "minecraft:strider"];

/// `setGuaranteedSaddleDropChance` — `drop_chances.saddle = 2.0f`
/// (V4300.java:182-184). `getOrCreateMap` semantics: reuse an existing
/// `drop_chances` compound if present, else create one.
fn set_guaranteed_saddle_drop_chance(data: &mut NbtMap) {
    if data.get_map("drop_chances").is_none() {
        data.set_map("drop_chances", NbtMap::new());
    }
    data.get_map_mut("drop_chances")
        .unwrap()
        .set_f32("saddle", 2.0);
}

/// Inverse of `set_guaranteed_saddle_drop_chance`: remove the `saddle` entry the
/// forward added to `drop_chances`, and drop the `drop_chances` compound itself
/// if that leaves it empty (i.e. the forward had created it from scratch). The
/// old format had no saddle drop-chance slot, so this is exact (bucket D).
fn remove_guaranteed_saddle_drop_chance(data: &mut NbtMap) {
    if let Some(drop_chances) = data.get_map_mut("drop_chances") {
        if let Some(chance) = drop_chances.get_f64("saddle") {
            if chance != 2.0 {
                report_loss(
                    VERSION,
                    LossKind::UnsupportedInTarget,
                    Severity::Loss,
                    "saddle drop_chances.saddle has no pre-4300 representation; dropping it",
                );
            }
        }
        drop_chances.take("saddle");
        if drop_chances.is_empty() {
            data.take("drop_chances");
        }
    }
}

fn is_canonical_saddle_item(item: &NbtMap) -> bool {
    item.get_string("id") == Some("minecraft:saddle") && item.get_i64("count").unwrap_or(1) == 1
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let id = match data.get_string("id") {
                Some(s) => s.to_string(),
                None => return,
            };

            if SADDLE_ITEM_ENTITIES.contains(&id.as_str()) {
                // renameSingle returns false (no-op) when the source key is absent.
                if !data.has_key("SaddleItem") {
                    return;
                }
                data.rename_key("SaddleItem", "saddle");
                set_guaranteed_saddle_drop_chance(data);
            } else if SADDLE_FLAG_ENTITIES.contains(&id.as_str()) {
                let saddle = data.get_bool("Saddle").unwrap_or(false);
                data.take("Saddle");

                if !saddle {
                    return;
                }

                let mut saddle_item = NbtMap::new();
                saddle_item.set_string("id", "minecraft:saddle");
                saddle_item.set_i32("count", 1);
                data.set_map("saddle", saddle_item);

                set_guaranteed_saddle_drop_chance(data);
            }
        }),
    );

    // Reverse of the saddle structure converter (new -> old). The entity id is
    // unchanged by the forward, so we branch on it exactly like the forward.
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let id = match data.get_string("id") {
                Some(s) => s.to_string(),
                None => return,
            };

            if SADDLE_ITEM_ENTITIES.contains(&id.as_str()) {
                // Forward renamed `SaddleItem` -> `saddle` and added the saddle
                // drop chance only when `SaddleItem` existed. Undo both: rename
                // `saddle` back and drop the added drop-chance. Lossless — the
                // new `saddle` slot uniquely encodes the old `SaddleItem`.
                if !data.has_key("saddle") {
                    return;
                }
                data.rename_key("saddle", "SaddleItem");
                remove_guaranteed_saddle_drop_chance(data);
            } else if SADDLE_FLAG_ENTITIES.contains(&id.as_str()) {
                // Forward replaced the boolean `Saddle` flag with a `saddle` item
                // (`{id:minecraft:saddle, count:1}`) + drop chance when set, and
                // always removed `Saddle`. Undo: presence of the `saddle` item
                // means `Saddle=true`, absence means `Saddle=false`. Pigs/striders
                // could only ever wear a vanilla saddle, so the item maps back to
                // the boolean exactly (the old format always carried `Saddle`).
                let had_saddle = data.has_key("saddle");
                if let Some(saddle) = data.get_map("saddle") {
                    if !is_canonical_saddle_item(saddle) {
                        report_loss(
                            VERSION,
                            LossKind::UnsupportedInTarget,
                            Severity::Loss,
                            "pig/strider saddle item data has no legacy boolean representation; restoring Saddle=true",
                        );
                    }
                } else if had_saddle {
                    report_loss(
                        VERSION,
                        LossKind::UnsupportedInTarget,
                        Severity::Loss,
                        "non-compound pig/strider saddle value has no legacy boolean representation; restoring Saddle=true",
                    );
                }
                data.take("saddle");
                remove_guaranteed_saddle_drop_chance(data);
                data.set_bool("Saddle", had_saddle);
            }
        }),
    );

    // Inventory walkers (saddle moved out; `Items` is the carried inventory).
    reg.entity
        .add_walker(VERSION, 0, "minecraft:llama", item_lists(&["Items"]));
    reg.entity
        .add_walker(VERSION, 0, "minecraft:trader_llama", item_lists(&["Items"]));
    reg.entity
        .add_walker(VERSION, 0, "minecraft:donkey", item_lists(&["Items"]));
    reg.entity
        .add_walker(VERSION, 0, "minecraft:mule", item_lists(&["Items"]));

    // Explicit no-op walkers (replace any earlier inventory walker).
    let noop: super::super::engine::Walker = Arc::new(|_reg, _data, _from, _to| {});
    reg.entity
        .add_walker(VERSION, 0, "minecraft:horse", noop.clone());
    reg.entity
        .add_walker(VERSION, 0, "minecraft:skeleton_horse", noop.clone());
    reg.entity
        .add_walker(VERSION, 0, "minecraft:zombie_horse", noop);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::registry::convert_entity_reverse;

    #[test]
    fn reverse_pig_custom_saddle_and_drop_chance_reports_losses() {
        let mut data = NbtMap::new();
        data.set_string("id", "minecraft:pig");
        let mut saddle = NbtMap::new();
        saddle.set_string("id", "minecraft:diamond");
        saddle.set_i32("count", 2);
        data.set_map("saddle", saddle);
        let mut chances = NbtMap::new();
        chances.set_f32("saddle", 0.5);
        data.set_map("drop_chances", chances);

        let report = convert_entity_reverse(&mut data, VERSION, VERSION - 1);

        assert_eq!(report.loss_count(), 2);
        assert_eq!(data.get_bool("Saddle"), Some(true));
        assert!(data.get_map("drop_chances").is_none());
    }
}
