//! V3568 (23w31a+1, `V23W31A + 1` = 3568) — schematic-relevant subset of
//! `V3568.java`.
//!
//! The 1.20.2 mob-effect format rewrite: numeric effect ids -> namespaced
//! string ids, and the legacy CamelCase effect fields -> snake_case. Ported:
//!   * TILE_ENTITY `minecraft:beacon`: `Primary`/`Secondary` (numeric) ->
//!     `primary_effect`/`secondary_effect` (string id).
//!   * ENTITY `minecraft:mooshroom`: legacy `EffectId`/`EffectDuration` ->
//!     `stew_effects` list of `{id, duration}`.
//!   * ENTITY `minecraft:arrow`: `CustomPotionEffects` -> `custom_potion_effects`.
//!   * ENTITY `minecraft:area_effect_cloud`: `Effects` -> `effects`.
//!   * ENTITY structure converter (living entities): `ActiveEffects` ->
//!     `active_effects`.
//!   * ITEM_STACK structure converter: `minecraft:suspicious_stew` `Effects` ->
//!     `effects` (+ `EffectId`/`EffectDuration` -> `id`/`duration`), and for
//!     potion/splash_potion/lingering_potion/tipped_arrow,
//!     `CustomPotionEffects` -> `custom_potion_effects`.
//!
//! Skipped (non-schematic): `MCTypeRegistry.PLAYER.addStructureConverter(...)`
//! (the same living-entity converter applied to PLAYER) — PLAYER never appears
//! in a schematic file.
//!
//! Implemented inline with NbtMap/NbtValue + the rename map; the per-effect
//! reconstruction (numeric->string id, recursive hidden_effect, list rebuild) is
//! not expressible with the rename/walker helpers.

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 3568;

/// `EFFECT_ID_MAP` (V3568.java:19-54): numeric effect id -> namespaced id. Index
/// 0 and unmapped slots are `None` (Java leaves the array slot null).
const EFFECT_ID_MAP: &[Option<&str>] = &[
    None,                                  // 0
    Some("minecraft:speed"),               // 1
    Some("minecraft:slowness"),            // 2
    Some("minecraft:haste"),               // 3
    Some("minecraft:mining_fatigue"),      // 4
    Some("minecraft:strength"),            // 5
    Some("minecraft:instant_health"),      // 6
    Some("minecraft:instant_damage"),      // 7
    Some("minecraft:jump_boost"),          // 8
    Some("minecraft:nausea"),              // 9
    Some("minecraft:regeneration"),        // 10
    Some("minecraft:resistance"),          // 11
    Some("minecraft:fire_resistance"),     // 12
    Some("minecraft:water_breathing"),     // 13
    Some("minecraft:invisibility"),        // 14
    Some("minecraft:blindness"),           // 15
    Some("minecraft:night_vision"),        // 16
    Some("minecraft:hunger"),              // 17
    Some("minecraft:weakness"),            // 18
    Some("minecraft:poison"),              // 19
    Some("minecraft:wither"),              // 20
    Some("minecraft:health_boost"),        // 21
    Some("minecraft:absorption"),          // 22
    Some("minecraft:saturation"),          // 23
    Some("minecraft:glowing"),             // 24
    Some("minecraft:levitation"),          // 25
    Some("minecraft:luck"),                // 26
    Some("minecraft:unluck"),              // 27
    Some("minecraft:slow_falling"),        // 28
    Some("minecraft:conduit_power"),       // 29
    Some("minecraft:dolphins_grace"),      // 30
    Some("minecraft:bad_omen"),            // 31
    Some("minecraft:hero_of_the_village"), // 32
    Some("minecraft:darkness"),            // 33
];

const EFFECT_ITEMS: &[&str] = &[
    "minecraft:potion",
    "minecraft:splash_potion",
    "minecraft:lingering_potion",
    "minecraft:tipped_arrow",
];

/// `MOB_EFFECT_RENAMES` (V3568.java:93-102).
const MOB_EFFECT_RENAMES: &[(&str, &str)] = &[
    ("Ambient", "ambient"),
    ("Amplifier", "amplifier"),
    ("Duration", "duration"),
    ("ShowParticles", "show_particles"),
    ("ShowIcon", "show_icon"),
    ("FactorCalculationData", "factor_calculation_data"),
    ("HiddenEffect", "hidden_effect"),
];

/// `readLegacyEffect` (V3568.java:65-73): numeric id at `path` -> namespaced id.
fn read_legacy_effect(data: &NbtMap, path: &str) -> Option<&'static str> {
    let id = data.get(path)?.as_number_i64()? as i32;
    if id >= 0 && (id as usize) < EFFECT_ID_MAP.len() {
        EFFECT_ID_MAP[id as usize]
    } else {
        None
    }
}

/// `convertLegacyEffect` (V3568.java:75-91): remove numeric `legacy_path`, and
/// when it maps to a known id write that string to `new_path`.
fn convert_legacy_effect(data: &mut NbtMap, legacy_path: &str, new_path: &str) {
    let id = data
        .get(legacy_path)
        .and_then(ValueExt::as_number_i64)
        .map(|v| v as i32);
    data.take(legacy_path);

    let id = match id {
        Some(v) => v,
        None => return,
    };
    let new_id = if id >= 0 && (id as usize) < EFFECT_ID_MAP.len() {
        EFFECT_ID_MAP[id as usize]
    } else {
        None
    };
    if let Some(new_id) = new_id {
        data.set_string(new_path, new_id);
    }
}

/// `convertMobEffect` (V3568.java:104-116): `Id`->`id`, field renames, recurse
/// into `hidden_effect`.
fn convert_mob_effect(mob_effect: &mut NbtMap) {
    convert_legacy_effect(mob_effect, "Id", "id");

    for (old, new) in MOB_EFFECT_RENAMES {
        mob_effect.rename_key(old, new);
    }

    if let Some(hidden) = mob_effect.get_map_mut("hidden_effect") {
        convert_mob_effect(hidden);
    }
}

/// `convertMobEffectList` (V3568.java:118-130): rename the list path and convert
/// each effect map.
fn convert_mob_effect_list(data: &mut NbtMap, old_path: &str, new_path: &str) {
    let mut effects = match data.take(old_path) {
        Some(NbtValue::List(l)) => l,
        Some(other) => {
            // Wrong type: Java's getList returns null -> no-op; restore it.
            data.set_generic(old_path, other);
            return;
        }
        None => return,
    };

    for el in effects.iter_mut() {
        if let Some(map) = el.as_compound_mut() {
            convert_mob_effect(map);
        }
    }

    data.set_list(new_path, effects);
}

/// `updateSuspiciousStew` (V3568.java:140-143): rewrite `EffectId`/`EffectDuration`
/// on `into` from the values read off `from`, mapping the numeric effect id.
fn update_suspicious_stew(from: &NbtMap, into: &mut NbtMap) {
    let id = read_legacy_effect(from, "EffectId").map(|s| s.to_string());
    let duration = from.get("EffectDuration").cloned();

    into.take("EffectId");
    if let Some(id) = id {
        into.set_string("id", id);
    }
    into.take("EffectDuration");
    if let Some(duration) = duration {
        into.set_generic("duration", duration);
    }
}

// ---------------------------------------------------------------------------
// Reverse helpers (new -> old). EFFECT_ID_MAP is a bijection over its defined
// slots (every namespaced id appears at most once), so string -> numeric is a
// well-defined inverse. A string id with no numeric preimage (an effect that
// did not exist in the legacy table) cannot be represented in the old numeric
// format -> best-effort drop + report_loss.
// ---------------------------------------------------------------------------

/// Inverse of `EFFECT_ID_MAP`: namespaced id -> numeric id. `None` if the id
/// was never assigned a numeric value in the legacy table.
fn legacy_effect_id(name: &str) -> Option<i32> {
    EFFECT_ID_MAP
        .iter()
        .position(|slot| *slot == Some(name))
        .map(|i| i as i32)
}

/// Inverse of `convert_legacy_effect`: namespaced id at `new_path` -> numeric id
/// at `legacy_path`. Lossless when the id is in the legacy table; when it is not
/// (a post-3568 effect would already have been downgraded, so this only fires
/// for a genuinely unrepresentable id) the field is dropped + loss reported.
fn revert_legacy_effect(data: &mut NbtMap, legacy_path: &str, new_path: &str) {
    let name = match data.take(new_path) {
        Some(NbtValue::String(s)) => s,
        Some(other) => {
            // Wrong type: forward only writes a string; leave untouched.
            data.set_generic(new_path, other);
            return;
        }
        None => return,
    };
    match legacy_effect_id(&name) {
        Some(id) => data.set_i32(legacy_path, id),
        None => report_loss(
            VERSION,
            LossKind::UnsupportedInTarget,
            Severity::Loss,
            "mob effect id has no legacy numeric mapping; dropped",
        ),
    }
}

/// Inverse of `convert_mob_effect`: `id`->`Id`, reverse the manual field
/// renames, recurse into `HiddenEffect`. (These renames are applied manually in
/// the forward converter, not via the auto-inverted rename engine, so they must
/// be reversed here.)
fn revert_mob_effect(mob_effect: &mut NbtMap) {
    revert_legacy_effect(mob_effect, "Id", "id");

    for (old, new) in MOB_EFFECT_RENAMES {
        mob_effect.rename_key(new, old);
    }

    if let Some(hidden) = mob_effect.get_map_mut("HiddenEffect") {
        revert_mob_effect(hidden);
    }
}

/// Inverse of `convert_mob_effect_list`: rename the list path back and revert
/// each effect map.
fn revert_mob_effect_list(data: &mut NbtMap, old_path: &str, new_path: &str) {
    let mut effects = match data.take(new_path) {
        Some(NbtValue::List(l)) => l,
        Some(other) => {
            data.set_generic(new_path, other);
            return;
        }
        None => return,
    };

    for el in effects.iter_mut() {
        if let Some(map) = el.as_compound_mut() {
            revert_mob_effect(map);
        }
    }

    data.set_list(old_path, effects);
}

/// Inverse of `update_suspicious_stew`: `id`/`duration` -> `EffectId`/
/// `EffectDuration`. `id` -> numeric `EffectId` via the legacy table (loss if
/// unrepresentable).
fn revert_suspicious_stew(into: &mut NbtMap) {
    let name = match into.take("id") {
        Some(NbtValue::String(s)) => Some(s),
        Some(other) => {
            into.set_generic("id", other);
            None
        }
        None => None,
    };
    if let Some(name) = name {
        match legacy_effect_id(&name) {
            Some(id) => into.set_i32("EffectId", id),
            None => report_loss(
                VERSION,
                LossKind::UnsupportedInTarget,
                Severity::Loss,
                "suspicious stew effect id has no legacy numeric mapping; dropped",
            ),
        }
    }

    if let Some(duration) = into.take("duration") {
        into.set_generic("EffectDuration", duration);
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    // TILE_ENTITY beacon.
    reg.tile_entity.add_converter_for_id(
        "minecraft:beacon",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            convert_legacy_effect(data, "Primary", "primary_effect");
            convert_legacy_effect(data, "Secondary", "secondary_effect");
        }),
    );
    // Reverse beacon: namespaced primary/secondary effect ids -> numeric
    // Primary/Secondary. Lossless for legacy-table ids.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:beacon",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            revert_legacy_effect(data, "Primary", "primary_effect");
            revert_legacy_effect(data, "Secondary", "secondary_effect");
        }),
    );

    // ENTITY mooshroom: build stew_effects from EffectId/EffectDuration.
    reg.entity.add_converter_for_id(
        "minecraft:mooshroom",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let mut new_effect = NbtMap::new();
            update_suspicious_stew(data, &mut new_effect);

            data.take("EffectId");
            data.take("EffectDuration");

            if new_effect.iter().next().is_some() {
                data.set_list("stew_effects", vec![NbtValue::Compound(new_effect)]);
            }
        }),
    );
    // Reverse mooshroom: the forward packed a single {id,duration} into a
    // one-element `stew_effects` list. Restore `EffectId`/`EffectDuration` from
    // that first element, then drop `stew_effects`. The mooshroom only ever
    // carried one stew effect, so taking element 0 is exact.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:mooshroom",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            match data.take("stew_effects") {
                Some(NbtValue::List(stew)) => {
                    if stew.len() > 1 {
                        report_loss(
                            VERSION,
                            LossKind::UnsupportedInTarget,
                            Severity::Loss,
                            "mooshroom stew_effects has multiple effects; only the first can be represented before 3568",
                        );
                    }
                    if stew.iter().skip(1).any(|v| !matches!(v, NbtValue::Compound(_))) {
                        report_loss(
                            VERSION,
                            LossKind::UnsupportedInTarget,
                            Severity::Loss,
                            "mooshroom stew_effects contains non-compound extra entries; dropped",
                        );
                    }
                    if let Some(first) = stew.into_iter().next() {
                        if let NbtValue::Compound(mut effect) = first {
                        revert_suspicious_stew(&mut effect);
                        // Move restored fields back onto the entity root.
                        if let Some(v) = effect.take("EffectId") {
                            data.set_generic("EffectId", v);
                        }
                        if let Some(v) = effect.take("EffectDuration") {
                            data.set_generic("EffectDuration", v);
                        }
                            if !effect.is_empty() {
                                report_loss(
                                    VERSION,
                                    LossKind::UnsupportedInTarget,
                                    Severity::Loss,
                                    "mooshroom stew_effects entry has extra fields with no pre-3568 representation; dropped",
                                );
                            }
                        } else {
                            report_loss(
                                VERSION,
                                LossKind::UnsupportedInTarget,
                                Severity::Loss,
                                "mooshroom stew_effects first entry is not a compound; dropped",
                            );
                        }
                    }
                }
                Some(other) => {
                    data.set_generic("stew_effects", other);
                }
                None => {}
            }
        }),
    );

    // ENTITY arrow.
    reg.entity.add_converter_for_id(
        "minecraft:arrow",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            convert_mob_effect_list(data, "CustomPotionEffects", "custom_potion_effects");
        }),
    );
    // Reverse arrow: custom_potion_effects -> CustomPotionEffects (+ per-effect).
    reg.entity.add_reverse_converter_for_id(
        "minecraft:arrow",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            revert_mob_effect_list(data, "CustomPotionEffects", "custom_potion_effects");
        }),
    );

    // ENTITY area_effect_cloud.
    reg.entity.add_converter_for_id(
        "minecraft:area_effect_cloud",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            convert_mob_effect_list(data, "Effects", "effects");
        }),
    );
    // Reverse area_effect_cloud: effects -> Effects (+ per-effect).
    reg.entity.add_reverse_converter_for_id(
        "minecraft:area_effect_cloud",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            revert_mob_effect_list(data, "Effects", "effects");
        }),
    );

    // ENTITY structure converter (living entities). (The identical PLAYER
    // registration is intentionally skipped: PLAYER is non-schematic.)
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            convert_mob_effect_list(data, "ActiveEffects", "active_effects");
        }),
    );
    // Reverse living-entity structure converter: active_effects ->
    // ActiveEffects (+ per-effect).
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            revert_mob_effect_list(data, "ActiveEffects", "active_effects");
        }),
    );

    // ITEM_STACK structure converter.
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|root, _from, _to| {
            let id = root.get_string("id").map(|s| s.to_string());
            let tag = match root.get_map_mut("tag") {
                Some(t) => t,
                None => return,
            };

            if id.as_deref() == Some("minecraft:suspicious_stew") {
                tag.rename_key("Effects", "effects");
                if let Some(NbtValue::List(mut effects)) = tag.take("effects") {
                    for el in effects.iter_mut() {
                        if let Some(map) = el.as_compound_mut() {
                            // updateSuspiciousStew(effect, effect): read+write same map.
                            let id = read_legacy_effect(map, "EffectId").map(|s| s.to_string());
                            let duration = map.get("EffectDuration").cloned();
                            map.take("EffectId");
                            if let Some(id) = id {
                                map.set_string("id", id);
                            }
                            map.take("EffectDuration");
                            if let Some(duration) = duration {
                                map.set_generic("duration", duration);
                            }
                        }
                    }
                    tag.set_list("effects", effects);
                }
                return;
            }

            if id
                .as_deref()
                .map(|s| EFFECT_ITEMS.contains(&s))
                .unwrap_or(false)
            {
                convert_mob_effect_list(tag, "CustomPotionEffects", "custom_potion_effects");
            }
        }),
    );
    // Reverse ITEM_STACK structure converter.
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|root, _from, _to| {
            let id = root.get_string("id").map(|s| s.to_string());
            let tag = match root.get_map_mut("tag") {
                Some(t) => t,
                None => return,
            };

            if id.as_deref() == Some("minecraft:suspicious_stew") {
                // Per-element id/duration -> EffectId/EffectDuration, then
                // effects -> Effects. updateSuspiciousStew(effect, effect) was
                // read+write same map, so its inverse is the same map too.
                if let Some(NbtValue::List(mut effects)) = tag.take("effects") {
                    for el in effects.iter_mut() {
                        if let Some(map) = el.as_compound_mut() {
                            revert_suspicious_stew(map);
                        }
                    }
                    tag.set_list("effects", effects);
                }
                tag.rename_key("effects", "Effects");
                return;
            }

            if id
                .as_deref()
                .map(|s| EFFECT_ITEMS.contains(&s))
                .unwrap_or(false)
            {
                revert_mob_effect_list(tag, "CustomPotionEffects", "custom_potion_effects");
            }
        }),
    );
}
