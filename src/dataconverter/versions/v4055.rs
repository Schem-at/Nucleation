//! V4055 (1.21.1 + 100) — schematic-relevant subset of `V4055.java`.
//!
//! `ConverterAbstractAttributesRename` with a prefix-stripping renamer: an
//! attribute id is namespaced, and if it begins with one of the legacy category
//! prefixes (`generic.` / `horse.` / `player.` / `zombie.`, each itself
//! namespaced to `minecraft:<prefix>`) the prefix is removed and the remainder
//! re-namespaced (`minecraft:<remainder>`).
//!
//! Schematic-relevant registrations ported (PLAYER is skipped):
//!   * DATA_COMPONENTS converter: rename `type` on each
//!     `minecraft:attribute_modifiers.modifiers[]`.
//!   * ENTITY converter: rename `id` on each `attributes[]`.
//!
//! VERSION = MCVersions.V1_21_1 (3955) + 100 = 4055.

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 4055;

/// Legacy prefixes removed by this rename, in priority order.
const PREFIXES_TO_REMOVE: &[&str] = &["generic.", "horse.", "player.", "zombie."];

/// Port of `NamespaceUtil.correctNamespace`: prepend `minecraft:` to an
/// unnamespaced value that parses as a resource location, otherwise return the
/// value unchanged. (The inputs here — attribute ids and the static prefixes —
/// are always valid resource-location paths.)
fn correct_namespace(value: &str) -> String {
    let (namespace, path) = match value.find(':') {
        Some(i) => (&value[..i], &value[i + 1..]),
        None => ("minecraft", value),
    };
    let ns_ok = namespace
        .bytes()
        .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'-'));
    let path_ok = path
        .bytes()
        .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'-' | b'/'));
    if ns_ok && path_ok {
        format!("{namespace}:{path}")
    } else {
        value.to_string()
    }
}

/// The V4055 renamer: returns `Some(new_id)` if a prefix matched, else `None`.
fn rename(input: &str) -> Option<String> {
    let namespaced_input = correct_namespace(input);
    for prefix in PREFIXES_TO_REMOVE {
        let namespaced_prefix = correct_namespace(prefix);
        if let Some(rest) = namespaced_input.strip_prefix(&namespaced_prefix) {
            return Some(format!("minecraft:{rest}"));
        }
    }
    None
}

/// Rename a string field in `map` through the V4055 renamer (RenameHelper.renameString).
fn rename_string(map: &mut NbtMap, key: &str) {
    if let Some(cur) = map.get_string(key).map(|s| s.to_string()) {
        if let Some(new) = rename(&cur) {
            map.set_string(key, new);
        }
    }
}

/// Inverse table: modern (unprefixed) attribute id -> its single canonical
/// pre-V4055 (1.21.1, data version 3955) prefixed form. The forward strips one
/// of `generic./horse./player./zombie.`; to reverse we must re-attach the prefix
/// each attribute actually carried in 1.21.1.
///
/// This is the authoritative 1.21.1 attribute set (derived from the legacy
/// `attribute.name.<prefix>.<name>` translation keys in vanilla 26.1.2
/// `assets/minecraft/lang/deprecated.json`), pruned to the form each id had
/// *immediately before* V4055:
///   * `jump_strength` -> `generic.jump_strength` (NOT `horse.`): the
///     `horse.jump_strength -> generic.jump_strength` rename happened earlier in
///     V3814, so by V4055 no live id is `horse.`-prefixed.
///   * `block_interaction_range` / `entity_interaction_range` -> `player.`:
///     they moved from `generic.` to `player.` in a pre-1.21 snapshot, handled
///     by earlier converters; the stale `generic.*` keys in deprecated.json are
///     not V4055 inputs.
///
/// Every modern id maps to exactly one preimage (vanilla never had both
/// `generic.X` and `player.X` for the same `X` at the same time), so the inverse
/// is one-to-one and LOSSLESS (cheatsheet rule 11): the prefix the forward
/// dropped was always a fixed default for each attribute, so restoring it is
/// exact, not a guess.
const ATTRIBUTE_REVERSE: &[(&str, &str)] = &[
    // generic.*
    ("minecraft:armor", "minecraft:generic.armor"),
    (
        "minecraft:armor_toughness",
        "minecraft:generic.armor_toughness",
    ),
    ("minecraft:attack_damage", "minecraft:generic.attack_damage"),
    (
        "minecraft:attack_knockback",
        "minecraft:generic.attack_knockback",
    ),
    ("minecraft:attack_speed", "minecraft:generic.attack_speed"),
    ("minecraft:burning_time", "minecraft:generic.burning_time"),
    (
        "minecraft:explosion_knockback_resistance",
        "minecraft:generic.explosion_knockback_resistance",
    ),
    (
        "minecraft:fall_damage_multiplier",
        "minecraft:generic.fall_damage_multiplier",
    ),
    ("minecraft:flying_speed", "minecraft:generic.flying_speed"),
    ("minecraft:follow_range", "minecraft:generic.follow_range"),
    ("minecraft:gravity", "minecraft:generic.gravity"),
    ("minecraft:jump_strength", "minecraft:generic.jump_strength"),
    (
        "minecraft:knockback_resistance",
        "minecraft:generic.knockback_resistance",
    ),
    ("minecraft:luck", "minecraft:generic.luck"),
    (
        "minecraft:max_absorption",
        "minecraft:generic.max_absorption",
    ),
    ("minecraft:max_health", "minecraft:generic.max_health"),
    (
        "minecraft:movement_efficiency",
        "minecraft:generic.movement_efficiency",
    ),
    (
        "minecraft:movement_speed",
        "minecraft:generic.movement_speed",
    ),
    ("minecraft:oxygen_bonus", "minecraft:generic.oxygen_bonus"),
    (
        "minecraft:safe_fall_distance",
        "minecraft:generic.safe_fall_distance",
    ),
    ("minecraft:scale", "minecraft:generic.scale"),
    ("minecraft:step_height", "minecraft:generic.step_height"),
    (
        "minecraft:water_movement_efficiency",
        "minecraft:generic.water_movement_efficiency",
    ),
    // player.*
    (
        "minecraft:block_break_speed",
        "minecraft:player.block_break_speed",
    ),
    (
        "minecraft:block_interaction_range",
        "minecraft:player.block_interaction_range",
    ),
    (
        "minecraft:entity_interaction_range",
        "minecraft:player.entity_interaction_range",
    ),
    (
        "minecraft:mining_efficiency",
        "minecraft:player.mining_efficiency",
    ),
    (
        "minecraft:sneaking_speed",
        "minecraft:player.sneaking_speed",
    ),
    (
        "minecraft:submerged_mining_speed",
        "minecraft:player.submerged_mining_speed",
    ),
    (
        "minecraft:sweeping_damage_ratio",
        "minecraft:player.sweeping_damage_ratio",
    ),
    // zombie.*
    (
        "minecraft:spawn_reinforcements",
        "minecraft:zombie.spawn_reinforcements",
    ),
];

/// Inverse of `rename`: re-attach the legacy prefix to a modern attribute id.
/// Returns `Some(old)` when the id is in the known 1.21.1 attribute set, else
/// `None` (leave unrecognized / already-prefixed / modded ids untouched, which
/// also makes a forward->reverse round trip idempotent).
fn rename_inverse(input: &str) -> Option<&'static str> {
    let namespaced = correct_namespace(input);
    ATTRIBUTE_REVERSE
        .iter()
        .find(|(new, _)| *new == namespaced)
        .map(|(_, old)| *old)
}

/// Inverse of `rename_string`: rename a modern attribute id back to its prefixed
/// pre-V4055 form.
fn rename_string_inverse(map: &mut NbtMap, key: &str) {
    if let Some(cur) = map.get_string(key).map(|s| s.to_string()) {
        if let Some(old) = rename_inverse(&cur) {
            map.set_string(key, old);
        } else if cur.starts_with("minecraft:")
            && !PREFIXES_TO_REMOVE
                .iter()
                .any(|prefix| cur.starts_with(&correct_namespace(prefix)))
        {
            report_loss(
                VERSION,
                LossKind::RenameAmbiguous,
                Severity::Approximated,
                format!(
                    "attribute id '{cur}' is not in the V4055 reverse table; it may have lost a generic/horse/player/zombie prefix"
                ),
            );
        }
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    // DATA_COMPONENTS: rename `type` on each attribute modifier.
    reg.data_components.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let attribute_modifiers = match data.get_map_mut("minecraft:attribute_modifiers") {
                Some(m) => m,
                None => return,
            };
            if let Some(modifiers) = attribute_modifiers.get_list_mut("modifiers") {
                for el in modifiers.iter_mut() {
                    if let Some(m) = el.as_compound_mut() {
                        rename_string(m, "type");
                    }
                }
            }
        }),
    );
    // REVERSE (lossless): re-attach the legacy prefix to each modifier `type`.
    reg.data_components.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let attribute_modifiers = match data.get_map_mut("minecraft:attribute_modifiers") {
                Some(m) => m,
                None => return,
            };
            if let Some(modifiers) = attribute_modifiers.get_list_mut("modifiers") {
                for el in modifiers.iter_mut() {
                    if let Some(m) = el.as_compound_mut() {
                        rename_string_inverse(m, "type");
                    }
                }
            }
        }),
    );

    // ENTITY: rename `id` on each attribute.
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(modifiers) = data.get_list_mut("attributes") {
                for el in modifiers.iter_mut() {
                    if let Some(m) = el.as_compound_mut() {
                        rename_string(m, "id");
                    }
                }
            }
        }),
    );
    // REVERSE (lossless): re-attach the legacy prefix to each attribute `id`.
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(modifiers) = data.get_list_mut("attributes") {
                for el in modifiers.iter_mut() {
                    if let Some(m) = el.as_compound_mut() {
                        rename_string_inverse(m, "id");
                    }
                }
            }
        }),
    );
}
