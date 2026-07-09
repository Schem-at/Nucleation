//! V4061 (24w34a + 1) — schematic-relevant subset of `V4061.java`.
//!
//! TILE_ENTITY `minecraft:trial_spawner` converter: a spawner whose inline
//! `normal_config` / `ominous_config` compounds exactly match one of the known
//! vanilla trial-chamber spawner presets is collapsed to a pair of string
//! references (`<key>/normal`, `<key>/ominous`).
//!
//! The Java version builds its lookup table by parsing SNBT templates and, for
//! each preset, accepts three encodings of the ominous config: the raw ominous
//! compound, the ominous compound merged onto a copy of the normal compound, and
//! that merged compound with vanilla defaults stripped (`removeDefaults`). We
//! mirror this exactly: SNBT is parsed via `quartz_nbt::snbt`, `merge` and
//! `removeDefaults` are reimplemented over [`NbtMap`], and matching uses the
//! derived structural equality (which, like Java's `MapType.equals`, is
//! type-sensitive: `1.0f`/`1`/`1b` compare as Float/Int/Byte).
//!
//! Nothing non-schematic is present in this version.
//!
//! VERSION = MCVersions.V24W34A (4060) + 1 = 4061.

use std::sync::LazyLock;

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 4061;

/// `(keyPath, normalSNBT, ominousSNBT)` presets (V4061.java:21-34).
const CONVERSIONS: &[(&str, &str, &str)] = &[
    (
        "trial_chamber/breeze",
        "{simultaneous_mobs: 1.0f, simultaneous_mobs_added_per_player: 0.5f, spawn_potentials: [{data: {entity: {id: \"minecraft:breeze\"}}, weight: 1}], ticks_between_spawn: 20, total_mobs: 2.0f, total_mobs_added_per_player: 1.0f}",
        "{loot_tables_to_eject: [{data: \"minecraft:spawners/ominous/trial_chamber/key\", weight: 3}, {data: \"minecraft:spawners/ominous/trial_chamber/consumables\", weight: 7}], simultaneous_mobs: 2.0f, total_mobs: 4.0f}",
    ),
    (
        "trial_chamber/melee/husk",
        "{simultaneous_mobs: 3.0f, simultaneous_mobs_added_per_player: 0.5f, spawn_potentials: [{data: {entity: {id: \"minecraft:husk\"}}, weight: 1}], ticks_between_spawn: 20}",
        "{loot_tables_to_eject: [{data: \"minecraft:spawners/ominous/trial_chamber/key\", weight: 3}, {data: \"minecraft:spawners/ominous/trial_chamber/consumables\", weight: 7}], spawn_potentials: [{data: {entity: {id: \"minecraft:husk\"}, equipment: {loot_table: \"minecraft:equipment/trial_chamber_melee\", slot_drop_chances: 0.0f}}, weight: 1}]}",
    ),
    (
        "trial_chamber/melee/spider",
        "{simultaneous_mobs: 3.0f, simultaneous_mobs_added_per_player: 0.5f, spawn_potentials: [{data: {entity: {id: \"minecraft:spider\"}}, weight: 1}], ticks_between_spawn: 20}",
        "{loot_tables_to_eject: [{data: \"minecraft:spawners/ominous/trial_chamber/key\", weight: 3}, {data: \"minecraft:spawners/ominous/trial_chamber/consumables\", weight: 7}],simultaneous_mobs: 4.0f, total_mobs: 12.0f}",
    ),
    (
        "trial_chamber/melee/zombie",
        "{simultaneous_mobs: 3.0f, simultaneous_mobs_added_per_player: 0.5f, spawn_potentials: [{data: {entity: {id: \"minecraft:zombie\"}}, weight: 1}], ticks_between_spawn: 20}",
        "{loot_tables_to_eject: [{data: \"minecraft:spawners/ominous/trial_chamber/key\", weight: 3}, {data: \"minecraft:spawners/ominous/trial_chamber/consumables\", weight: 7}],spawn_potentials: [{data: {entity: {id: \"minecraft:zombie\"}, equipment: {loot_table: \"minecraft:equipment/trial_chamber_melee\", slot_drop_chances: 0.0f}}, weight: 1}]}",
    ),
    (
        "trial_chamber/ranged/poison_skeleton",
        "{simultaneous_mobs: 3.0f, simultaneous_mobs_added_per_player: 0.5f, spawn_potentials: [{data: {entity: {id: \"minecraft:bogged\"}}, weight: 1}], ticks_between_spawn: 20}",
        "{loot_tables_to_eject: [{data: \"minecraft:spawners/ominous/trial_chamber/key\", weight: 3}, {data: \"minecraft:spawners/ominous/trial_chamber/consumables\", weight: 7}],spawn_potentials: [{data: {entity: {id: \"minecraft:bogged\"}, equipment: {loot_table: \"minecraft:equipment/trial_chamber_ranged\", slot_drop_chances: 0.0f}}, weight: 1}]}",
    ),
    (
        "trial_chamber/ranged/skeleton",
        "{simultaneous_mobs: 3.0f, simultaneous_mobs_added_per_player: 0.5f, spawn_potentials: [{data: {entity: {id: \"minecraft:skeleton\"}}, weight: 1}], ticks_between_spawn: 20}",
        "{loot_tables_to_eject: [{data: \"minecraft:spawners/ominous/trial_chamber/key\", weight: 3}, {data: \"minecraft:spawners/ominous/trial_chamber/consumables\", weight: 7}], spawn_potentials: [{data: {entity: {id: \"minecraft:skeleton\"}, equipment: {loot_table: \"minecraft:equipment/trial_chamber_ranged\", slot_drop_chances: 0.0f}}, weight: 1}]}",
    ),
    (
        "trial_chamber/ranged/stray",
        "{simultaneous_mobs: 3.0f, simultaneous_mobs_added_per_player: 0.5f, spawn_potentials: [{data: {entity: {id: \"minecraft:stray\"}}, weight: 1}], ticks_between_spawn: 20}",
        "{loot_tables_to_eject: [{data: \"minecraft:spawners/ominous/trial_chamber/key\", weight: 3}, {data: \"minecraft:spawners/ominous/trial_chamber/consumables\", weight: 7}], spawn_potentials: [{data: {entity: {id: \"minecraft:stray\"}, equipment: {loot_table: \"minecraft:equipment/trial_chamber_ranged\", slot_drop_chances: 0.0f}}, weight: 1}]}",
    ),
    (
        "trial_chamber/slow_ranged/poison_skeleton",
        "{simultaneous_mobs: 4.0f, simultaneous_mobs_added_per_player: 2.0f, spawn_potentials: [{data: {entity: {id: \"minecraft:bogged\"}}, weight: 1}], ticks_between_spawn: 160}",
        "{loot_tables_to_eject: [{data: \"minecraft:spawners/ominous/trial_chamber/key\", weight: 3}, {data: \"minecraft:spawners/ominous/trial_chamber/consumables\", weight: 7}], spawn_potentials: [{data: {entity: {id: \"minecraft:bogged\"}, equipment: {loot_table: \"minecraft:equipment/trial_chamber_ranged\", slot_drop_chances: 0.0f}}, weight: 1}]}",
    ),
    (
        "trial_chamber/slow_ranged/skeleton",
        "{simultaneous_mobs: 4.0f, simultaneous_mobs_added_per_player: 2.0f, spawn_potentials: [{data: {entity: {id: \"minecraft:skeleton\"}}, weight: 1}], ticks_between_spawn: 160}",
        "{loot_tables_to_eject: [{data: \"minecraft:spawners/ominous/trial_chamber/key\", weight: 3}, {data: \"minecraft:spawners/ominous/trial_chamber/consumables\", weight: 7}], spawn_potentials: [{data: {entity: {id: \"minecraft:skeleton\"}, equipment: {loot_table: \"minecraft:equipment/trial_chamber_ranged\", slot_drop_chances: 0.0f}}, weight: 1}]}",
    ),
    (
        "trial_chamber/slow_ranged/stray",
        "{simultaneous_mobs: 4.0f, simultaneous_mobs_added_per_player: 2.0f, spawn_potentials: [{data: {entity: {id: \"minecraft:stray\"}}, weight: 1}], ticks_between_spawn: 160}",
        "{loot_tables_to_eject: [{data: \"minecraft:spawners/ominous/trial_chamber/key\", weight: 3}, {data: \"minecraft:spawners/ominous/trial_chamber/consumables\", weight: 7}],spawn_potentials: [{data: {entity: {id: \"minecraft:stray\"}, equipment: {loot_table: \"minecraft:equipment/trial_chamber_ranged\", slot_drop_chances: 0.0f}}, weight: 1}]}",
    ),
    (
        "trial_chamber/small_melee/baby_zombie",
        "{simultaneous_mobs: 2.0f, simultaneous_mobs_added_per_player: 0.5f, spawn_potentials: [{data: {entity: {IsBaby: 1b, id: \"minecraft:zombie\"}}, weight: 1}], ticks_between_spawn: 20}",
        "{loot_tables_to_eject: [{data: \"minecraft:spawners/ominous/trial_chamber/key\", weight: 3}, {data: \"minecraft:spawners/ominous/trial_chamber/consumables\", weight: 7}], spawn_potentials: [{data: {entity: {IsBaby: 1b, id: \"minecraft:zombie\"}, equipment: {loot_table: \"minecraft:equipment/trial_chamber_melee\", slot_drop_chances: 0.0f}}, weight: 1}]}",
    ),
    (
        "trial_chamber/small_melee/cave_spider",
        "{simultaneous_mobs: 3.0f, simultaneous_mobs_added_per_player: 0.5f, spawn_potentials: [{data: {entity: {id: \"minecraft:cave_spider\"}}, weight: 1}], ticks_between_spawn: 20}",
        "{loot_tables_to_eject: [{data: \"minecraft:spawners/ominous/trial_chamber/key\", weight: 3}, {data: \"minecraft:spawners/ominous/trial_chamber/consumables\", weight: 7}], simultaneous_mobs: 4.0f, total_mobs: 12.0f}",
    ),
    (
        "trial_chamber/small_melee/silverfish",
        "{simultaneous_mobs: 3.0f, simultaneous_mobs_added_per_player: 0.5f, spawn_potentials: [{data: {entity: {id: \"minecraft:silverfish\"}}, weight: 1}], ticks_between_spawn: 20}",
        "{loot_tables_to_eject: [{data: \"minecraft:spawners/ominous/trial_chamber/key\", weight: 3}, {data: \"minecraft:spawners/ominous/trial_chamber/consumables\", weight: 7}], simultaneous_mobs: 4.0f, total_mobs: 12.0f}",
    ),
    (
        "trial_chamber/small_melee/slime",
        "{simultaneous_mobs: 3.0f, simultaneous_mobs_added_per_player: 0.5f, spawn_potentials: [{data: {entity: {Size: 1, id: \"minecraft:slime\"}}, weight: 3}, {data: {entity: {Size: 2, id: \"minecraft:slime\"}}, weight: 1}], ticks_between_spawn: 20}",
        "{loot_tables_to_eject: [{data: \"minecraft:spawners/ominous/trial_chamber/key\", weight: 3}, {data: \"minecraft:spawners/ominous/trial_chamber/consumables\", weight: 7}], simultaneous_mobs: 4.0f, total_mobs: 12.0f}",
    ),
];

/// One preset: the normal config and the set of accepted ominous encodings.
struct Preset {
    full_key: String,
    normal: NbtMap,
    ominous_variants: Vec<NbtMap>,
}

/// Parse an SNBT template into an [`NbtMap`] (templates are static and known
/// valid, so a parse failure is a programmer error).
fn parse_snbt(snbt: &str) -> NbtMap {
    let compound = quartz_nbt::snbt::parse(snbt).expect("valid V4061 SNBT template");
    NbtMap::from_quartz_nbt(&compound)
}

/// `CompoundTag.merge`: recursively merge `other` onto `base` (compound children
/// merge, everything else overwrites). Returns the merged copy.
fn merge(base: &NbtMap, other: &NbtMap) -> NbtMap {
    let mut out = base.clone();
    for (k, v) in other.iter() {
        if let NbtValue::Compound(other_child) = v {
            if let Some(NbtValue::Compound(base_child)) = out.get(k) {
                let merged = merge(base_child, other_child);
                out.set_map(k, merged);
                continue;
            }
        }
        out.set_generic(k, v.clone());
    }
    out
}

/// `removeDefaults`: drop fields that equal the vanilla trial-spawner defaults.
fn remove_defaults(config: &mut NbtMap) {
    if config.get_i32("spawn_range") == Some(4) {
        config.take("spawn_range");
    }
    if config.get("total_mobs").and_then(ValueExt::as_number_f64) == Some(6.0) {
        config.take("total_mobs");
    }
    if config
        .get("simultaneous_mobs")
        .and_then(ValueExt::as_number_f64)
        == Some(2.0)
    {
        config.take("simultaneous_mobs");
    }
    if config
        .get("total_mobs_added_per_player")
        .and_then(ValueExt::as_number_f64)
        == Some(2.0)
    {
        config.take("total_mobs_added_per_player");
    }
    if config
        .get("simultaneous_mobs_added_per_player")
        .and_then(ValueExt::as_number_f64)
        == Some(1.0)
    {
        config.take("simultaneous_mobs_added_per_player");
    }
    if config.get_i32("ticks_between_spawn") == Some(40) {
        config.take("ticks_between_spawn");
    }
}

/// Build the preset table once (mirrors V4061.java's static initializer).
static PRESETS: LazyLock<Vec<Preset>> = LazyLock::new(|| {
    let mut presets = Vec::with_capacity(CONVERSIONS.len());
    for (key_path, normal_snbt, ominous_snbt) in CONVERSIONS {
        let full_key = format!("minecraft:{key_path}");

        let normal = parse_snbt(normal_snbt);
        let ominous = parse_snbt(ominous_snbt);

        let ominous_merged = merge(&normal, &ominous);

        let mut ominous_merged_defaults = ominous_merged.clone();
        remove_defaults(&mut ominous_merged_defaults);

        presets.push(Preset {
            full_key,
            normal,
            ominous_variants: vec![ominous, ominous_merged, ominous_merged_defaults],
        });
    }
    presets
});

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity.add_converter_for_id(
        "minecraft:trial_spawner",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let normal_config = match data.get_map("normal_config") {
                Some(m) => m.clone(),
                None => return,
            };
            let ominous_config = match data.get_map("ominous_config") {
                Some(m) => m.clone(),
                None => return,
            };

            let matched = PRESETS.iter().find(|p| {
                p.normal == normal_config && p.ominous_variants.contains(&ominous_config)
            });

            if let Some(preset) = matched {
                data.set_string("normal_config", format!("{}/normal", preset.full_key));
                data.set_string("ominous_config", format!("{}/ominous", preset.full_key));
            }
        }),
    );

    // Reverse (24w34a+1 -> 24w34a): expand the collapsed preset references back
    // into inline `normal_config` / `ominous_config` compounds.
    //
    // The forward only fires when `normal_config` matched a preset's normal
    // compound exactly AND `ominous_config` matched one of that preset's three
    // accepted ominous encodings (raw ominous, ominous merged onto normal, and
    // that merged compound with vanilla defaults stripped); on a hit it replaces
    // both inline compounds with the string references `<key>/normal` and
    // `<key>/ominous`. So the reverse fires when both fields are the *string*
    // references for the same preset key.
    //
    // `normal_config` is exact: a preset key maps to exactly one normal compound
    // (no information was discarded), so restoring `preset.normal` is lossless.
    //
    // `ominous_config` is lossy: three distinct ominous encodings all collapse to
    // the same `<key>/ominous` string, so the original encoding is unrecoverable.
    // We restore the literal `ominous` template (the first registered preimage,
    // `ominous_variants[0]`) as a best-effort substitution and report it. This
    // round-trips every input whose ominous_config was the raw template, and the
    // structurally-equivalent merged/defaults-stripped inputs are restored to a
    // semantically equal config (rule 11: genuinely merged with no surviving
    // discriminator).
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:trial_spawner",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let normal_ref = match data.get_string("normal_config") {
                Some(s) => s.to_string(),
                None => return,
            };
            let ominous_ref = match data.get_string("ominous_config") {
                Some(s) => s.to_string(),
                None => return,
            };

            // Both references must name the same preset key.
            let normal_key = match normal_ref.strip_suffix("/normal") {
                Some(k) => k,
                None => return,
            };
            let ominous_key = match ominous_ref.strip_suffix("/ominous") {
                Some(k) => k,
                None => return,
            };
            if normal_key != ominous_key {
                return;
            }

            let preset = match PRESETS.iter().find(|p| p.full_key == normal_key) {
                Some(p) => p,
                None => return,
            };

            data.set_map("normal_config", preset.normal.clone());
            // ominous_variants[0] is the raw `ominous` SNBT template.
            data.set_map("ominous_config", preset.ominous_variants[0].clone());

            report_loss(
                VERSION,
                LossKind::FingerprintCollapse,
                Severity::Approximated,
                "trial_spawner ominous_config preset reference expanded to the canonical ominous template; the original encoding (raw / merged / defaults-stripped) is not recoverable",
            );
        }),
    );
}
