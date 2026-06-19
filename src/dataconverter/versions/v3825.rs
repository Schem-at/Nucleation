//! V3825 (24w12a + 1) — schematic-relevant subset of `V3825.java`.
//!
//! Ported (all schematic-relevant):
//!   * ITEM_STACK structure converter: for `white_banner` / `filled_map`, a
//!     `custom_name` component whose translate key is one of the known "standard"
//!     names is promoted to an `item_name` component (V3825.java:30-73).
//!   * TILE_ENTITY `minecraft:banner`: an ominous-banner `CustomName` becomes an
//!     `item_name` component plus a `hide_additional_tooltip` component
//!     (V3825.java:74-91).
//!   * TILE_ENTITY `minecraft:trial_spawner` walker: recurse the spawner entities
//!     in normal_config/ominous_config `spawn_potentials[].data.entity` and
//!     `spawn_data.entity` (V3825.java:93-105).
//!   * TILE_ENTITY `minecraft:trial_spawner` converter: gather the loose config
//!     keys into a `normal_config` sub-map (V3825.java:106-138).
//!   * ENTITY `minecraft:ominous_item_spawner` walker: `item` -> ITEM_STACK
//!     (V3825.java:139).
//!
//! Nothing non-schematic exists in this version file.
//!
//! `ComponentUtils.retrieveTranslationString` is ported with `serde_json`: parse
//! the string; if it is a JSON object with a string `translate` field, return it.
//!
//! VERSION = MCVersions.V24W12A (3824) + 1 = 3825.

use std::sync::Arc;

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};
use super::super::walker::{convert, items};

const VERSION: i32 = 3825;

const BANNER_NAMES: &[&str] = &["block.minecraft.ominous_banner"];

const MAP_NAMES: &[&str] = &[
    "filled_map.buried_treasure",
    "filled_map.explorer_jungle",
    "filled_map.explorer_swamp",
    "filled_map.mansion",
    "filled_map.monument",
    "filled_map.trial_chambers",
    "filled_map.village_desert",
    "filled_map.village_plains",
    "filled_map.village_savanna",
    "filled_map.village_snowy",
    "filled_map.village_taiga",
];

const TRIAL_SPAWNER_NORMAL_CONFIG_KEYS: &[&str] = &[
    "spawn_range",
    "total_mobs",
    "simultaneous_mobs",
    "total_mobs_added_per_player",
    "simultaneous_mobs_added_per_player",
    "ticks_between_spawn",
    "spawn_potentials",
    "loot_tables_to_eject",
    "items_to_drop_when_ominous",
];

/// `ComponentUtils.retrieveTranslationString`: parse `possible_json`; if it is a
/// JSON object with a string `translate` field, return that string; else `None`.
fn retrieve_translation_string(possible_json: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(possible_json).ok()?;
    value.get("translate")?.as_str().map(|s| s.to_string())
}

/// `convertName` (V3825.java:33-49): promote a matching `custom_name` to
/// `item_name`.
fn convert_name(components: &mut NbtMap, standard_names: &[&str]) {
    let custom_name = match components.get_string("minecraft:custom_name") {
        Some(s) => s.to_string(),
        None => return,
    };
    let translation = match retrieve_translation_string(&custom_name) {
        Some(t) => t,
        None => return,
    };
    if standard_names.contains(&translation.as_str()) {
        components.take("minecraft:custom_name");
        components.set_string("minecraft:item_name", custom_name);
    }
}

/// Inverse of `convertName`: demote a matching `item_name` back to
/// `custom_name`. The forward only promoted names whose translate key is in
/// `standard_names`, so an `item_name` matching that set is exactly the
/// preimage — lossless. A pre-existing `item_name` with a non-standard
/// translate key (or non-translatable text) is left alone.
fn revert_name(components: &mut NbtMap, standard_names: &[&str]) {
    let item_name = match components.get_string("minecraft:item_name") {
        Some(s) => s.to_string(),
        None => return,
    };
    let translation = match retrieve_translation_string(&item_name) {
        Some(t) => t,
        None => return,
    };
    if standard_names.contains(&translation.as_str()) {
        components.take("minecraft:item_name");
        components.set_string("minecraft:custom_name", item_name);
        report_loss(
            VERSION,
            LossKind::FingerprintCollapse,
            Severity::Approximated,
            "standard item_name may be a modern value or a promoted legacy custom_name; restored legacy custom_name",
        );
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    // ITEM_STACK: white_banner / filled_map custom_name promotion.
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let id = match data.get_string("id") {
                Some(s) => s.to_string(),
                None => return,
            };
            let components = match data.get_map_mut("components") {
                Some(c) => c,
                None => return,
            };
            match id.as_str() {
                "minecraft:white_banner" => convert_name(components, BANNER_NAMES),
                "minecraft:filled_map" => convert_name(components, MAP_NAMES),
                _ => {}
            }
        }),
    );

    // Reverse: demote a standard-named `item_name` back to `custom_name`.
    // The forward only promoted `custom_name`s whose translate key is in the
    // per-id standard set, so an `item_name` matching that set is the exact
    // preimage — lossless (rule 11). Items that legitimately carried an
    // `item_name` with a non-standard translate key are left untouched.
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let id = match data.get_string("id") {
                Some(s) => s.to_string(),
                None => return,
            };
            let components = match data.get_map_mut("components") {
                Some(c) => c,
                None => return,
            };
            match id.as_str() {
                "minecraft:white_banner" => revert_name(components, BANNER_NAMES),
                "minecraft:filled_map" => revert_name(components, MAP_NAMES),
                _ => {}
            }
        }),
    );

    // TILE_ENTITY banner: ominous-banner CustomName -> item_name component.
    reg.tile_entity.add_converter_for_id(
        "minecraft:banner",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let custom_name = match data.get_string("CustomName") {
                Some(s) => s.to_string(),
                None => return,
            };
            if retrieve_translation_string(&custom_name).as_deref()
                != Some("block.minecraft.ominous_banner")
            {
                return;
            }

            data.take("CustomName");

            // getOrCreateMap("components").
            if data.get_map("components").is_none() {
                data.set_map("components", NbtMap::new());
            }
            let components = data.get_map_mut("components").unwrap();
            components.set_string("minecraft:item_name", custom_name);
            components.set_map("minecraft:hide_additional_tooltip", NbtMap::new());
        }),
    );

    // Reverse: restore the ominous-banner `CustomName` from the `item_name`
    // component, dropping the `hide_additional_tooltip` the forward added.
    // The forward fired only when the (now removed) `CustomName` translated to
    // `block.minecraft.ominous_banner`; that same translate key on `item_name`
    // is the exact preimage, so this is lossless. `components` is dropped if it
    // becomes empty (it carried only the two forward-added entries).
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:banner",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let components = match data.get_map_mut("components") {
                Some(c) => c,
                None => return,
            };
            let item_name = match components.get_string("minecraft:item_name") {
                Some(s) => s.to_string(),
                None => return,
            };
            if retrieve_translation_string(&item_name).as_deref()
                != Some("block.minecraft.ominous_banner")
            {
                return;
            }

            components.take("minecraft:item_name");
            if components.take("minecraft:hide_additional_tooltip").is_some() {
                report_loss(
                    VERSION,
                    LossKind::ComponentDropped,
                    Severity::Approximated,
                    "removed ominous banner hide_additional_tooltip while restoring legacy CustomName",
                );
            }
            if components.inner().is_empty() {
                data.take("components");
            }

            data.set_string("CustomName", item_name);
            report_loss(
                VERSION,
                LossKind::FingerprintCollapse,
                Severity::Approximated,
                "ominous banner item_name may be modern or converted from legacy CustomName; restored legacy CustomName",
            );
        }),
    );

    // TILE_ENTITY trial_spawner walker.
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "minecraft:trial_spawner",
        Arc::new(|reg, data, from, to| {
            for config_key in ["normal_config", "ominous_config"] {
                if let Some(config) = data.get_map_mut(config_key) {
                    // convertListPath(ENTITY, config, "spawn_potentials", "data", "entity"):
                    // for each spawn_potentials entry, descend data.entity. The Rust
                    // helper only descends one level, so do the inner hop inline.
                    if let Some(list) = config.get_list_mut("spawn_potentials") {
                        for el in list.iter_mut() {
                            if let Some(entry) = el.as_compound_mut() {
                                if let Some(inner) = entry.get_map_mut("data") {
                                    convert(reg, &reg.entity, inner, "entity", from, to);
                                }
                            }
                        }
                    }
                }
            }

            if let Some(spawn_data) = data.get_map_mut("spawn_data") {
                convert(reg, &reg.entity, spawn_data, "entity", from, to);
            }
        }),
    );

    // TILE_ENTITY trial_spawner converter: gather loose keys into normal_config.
    reg.tile_entity.add_converter_for_id(
        "minecraft:trial_spawner",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let mut normal_config = NbtMap::new();
            for key in TRIAL_SPAWNER_NORMAL_CONFIG_KEYS {
                if let Some(value) = data.take(key) {
                    normal_config.set_generic(key, value);
                }
            }
            if !normal_config.inner().is_empty() {
                data.set_map("normal_config", normal_config);
            }
        }),
    );

    // Reverse: scatter `normal_config`'s entries back to the top level. The
    // forward moved each of NORMAL_CONFIG_KEYS that was present into a
    // `normal_config` sub-map (and only those keys), so lifting every entry
    // back out and removing the now-empty sub-map is the exact inverse —
    // lossless structural move (bucket B). We pull all of `normal_config`'s
    // entries (not just the known key list) so any value the forward placed
    // there is restored.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:trial_spawner",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let normal_config = match data.take("normal_config") {
                Some(NbtValue::Compound(m)) => m,
                // Not a compound (or absent): nothing to undo. Restore if it
                // was some other tag we shouldn't have removed.
                Some(other) => {
                    data.set_generic("normal_config", other);
                    return;
                }
                None => return,
            };
            for (key, value) in normal_config {
                data.set_generic(&key, value);
            }
        }),
    );

    // ENTITY ominous_item_spawner walker: item -> ITEM_STACK.
    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:ominous_item_spawner",
        items(&["item"]),
    );
}
