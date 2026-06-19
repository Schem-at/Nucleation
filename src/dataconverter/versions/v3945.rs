//! V3945 (24w20a + 1) — schematic-relevant subset of `V3945.java`.
//!
//! Attribute modifiers move from the old `{uuid:int[4], name, amount}` shape to
//! the new `{id, amount, ...}` shape, with the legacy UUID/name pairs resolved
//! to stable string ids (`remapModifiers`). Three registrations are ported
//! (PLAYER is skipped):
//!   * ITEM_STACK converter: remap
//!     `components["minecraft:attribute_modifiers"]["modifiers"]`.
//!   * ENTITY converter: rename `Attributes`->`attributes`, each attribute's
//!     `Name/Base/Modifiers`, each modifier's `UUID/Name/Amount` and the
//!     int `Operation` -> string `operation`, then `remapModifiers`.
//!   * TILE_ENTITY `minecraft:jukebox` converter: collapse
//!     `IsPlaying/TickCount/RecordStartTick` into `ticks_since_song_started`.
//!
//! VERSION = MCVersions.V24W20A (3944) + 1 = 3945.

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 3945;

/// Legacy modifier UUID (canonical lowercase string) -> stable id (V3945.java:21-63).
const UUID_TO_ID: &[(&str, &str)] = &[
    (
        "736565d2-e1a7-403d-a3f8-1aeb3e302542",
        "minecraft:creative_mode_block_range",
    ),
    (
        "98491ef6-97b1-4584-ae82-71a8cc85cf73",
        "minecraft:creative_mode_entity_range",
    ),
    (
        "91aeaa56-376b-4498-935b-2f7f68070635",
        "minecraft:effect.speed",
    ),
    (
        "7107de5e-7ce8-4030-940e-514c1f160890",
        "minecraft:effect.slowness",
    ),
    (
        "af8b6e3f-3328-4c0a-aa36-5ba2bb9dbef3",
        "minecraft:effect.haste",
    ),
    (
        "55fced67-e92a-486e-9800-b47f202c4386",
        "minecraft:effect.mining_fatigue",
    ),
    (
        "648d7064-6a60-4f59-8abe-c2c23a6dd7a9",
        "minecraft:effect.strength",
    ),
    (
        "c0105bf3-aef8-46b0-9ebc-92943757ccbe",
        "minecraft:effect.jump_boost",
    ),
    (
        "22653b89-116e-49dc-9b6b-9971489b5be5",
        "minecraft:effect.weakness",
    ),
    (
        "5d6f0ba2-1186-46ac-b896-c61c5cee99cc",
        "minecraft:effect.health_boost",
    ),
    (
        "eae29cf0-701e-4ed6-883a-96f798f3dab5",
        "minecraft:effect.absorption",
    ),
    (
        "03c3c89d-7037-4b42-869f-b146bcb64d2e",
        "minecraft:effect.luck",
    ),
    (
        "cc5af142-2bd2-4215-b636-2605aed11727",
        "minecraft:effect.unluck",
    ),
    ("6555be74-63b3-41f1-a245-77833b3c2562", "minecraft:evil"),
    (
        "1eaf83ff-7207-4596-b37a-d7a07b3ec4ce",
        "minecraft:powder_snow",
    ),
    (
        "662a6b8d-da3e-4c1c-8813-96ea6097278d",
        "minecraft:sprinting",
    ),
    (
        "020e0dfb-87ae-4653-9556-831010e291a0",
        "minecraft:attacking",
    ),
    ("766bfa64-11f3-11ea-8d71-362b9e155667", "minecraft:baby"),
    ("7e0292f2-9434-48d5-a29f-9583af7df27f", "minecraft:covered"),
    (
        "9e362924-01de-4ddd-a2b2-d0f7a405a174",
        "minecraft:suffocating",
    ),
    ("5cd17e52-a79a-43d3-a529-90fde04b181e", "minecraft:drinking"),
    ("b9766b59-9566-4402-bc1f-2ee2a276d836", "minecraft:baby"),
    (
        "49455a49-7ec5-45ba-b886-3b90b23a1718",
        "minecraft:attacking",
    ),
    (
        "845db27c-c624-495f-8c9f-6020a9a58b6b",
        "minecraft:armor.boots",
    ),
    (
        "d8499b04-0e66-4726-ab29-64469d734e0d",
        "minecraft:armor.leggings",
    ),
    (
        "9f3d476d-c118-4544-8365-64846904b48e",
        "minecraft:armor.chestplate",
    ),
    (
        "2ad3f246-fee1-4e67-b886-69fd380bb150",
        "minecraft:armor.helmet",
    ),
    (
        "c1c72771-8b8e-ba4a-ace0-81a93c8928b2",
        "minecraft:armor.body",
    ),
    (
        "b572ecd2-ac0c-4071-abde-9594af072a37",
        "minecraft:enchantment.fire_protection",
    ),
    (
        "40a9968f-5c66-4e2f-b7f4-2ec2f4b3e450",
        "minecraft:enchantment.blast_protection",
    ),
    (
        "07a65791-f64d-4e79-86c7-f83932f007ec",
        "minecraft:enchantment.respiration",
    ),
    (
        "60b1b7db-fffd-4ad0-817c-d6c6a93d8a45",
        "minecraft:enchantment.aqua_affinity",
    ),
    (
        "11dc269a-4476-46c0-aff3-9e17d7eb6801",
        "minecraft:enchantment.depth_strider",
    ),
    (
        "87f46a96-686f-4796-b035-22e16ee9e038",
        "minecraft:enchantment.soul_speed",
    ),
    (
        "b9716dbd-50df-4080-850e-70347d24e687",
        "minecraft:enchantment.soul_speed",
    ),
    (
        "92437d00-c3a7-4f2e-8f6c-1f21585d5dd0",
        "minecraft:enchantment.swift_sneak",
    ),
    (
        "5d3d087b-debe-4037-b53e-d84f3ff51f17",
        "minecraft:enchantment.sweeping_edge",
    ),
    (
        "3ceb37c0-db62-46b5-bd02-785457b01d96",
        "minecraft:enchantment.efficiency",
    ),
    (
        "cb3f55d3-645c-4f38-a497-9c13a33db5cf",
        "minecraft:base_attack_damage",
    ),
    (
        "fa233e1c-4180-4865-b01b-bcce9785aca3",
        "minecraft:base_attack_speed",
    ),
];

/// Legacy modifier name -> stable id (V3945.java:65-73).
const NAME_TO_ID: &[(&str, &str)] = &[
    ("Random spawn bonus", "minecraft:random_spawn_bonus"),
    (
        "Random zombie-spawn bonus",
        "minecraft:zombie_random_spawn_bonus",
    ),
    ("Leader zombie bonus", "minecraft:leader_zombie_bonus"),
    (
        "Zombie reinforcement callee charge",
        "minecraft:reinforcement_callee_charge",
    ),
    (
        "Zombie reinforcement caller charge",
        "minecraft:reinforcement_caller_charge",
    ),
];

/// `makeUUID(int[4])`: build the canonical lowercase UUID string, or `None` if
/// the array is absent or not length 4.
fn make_uuid(arr: Option<&[i32]>) -> Option<String> {
    let arr = arr?;
    if arr.len() != 4 {
        return None;
    }
    let most = ((arr[0] as i64) << 32) | ((arr[1] as i64) & 0xFFFF_FFFF);
    let least = ((arr[2] as i64) << 32) | ((arr[3] as i64) & 0xFFFF_FFFF);
    Some(format_uuid(most as u64, least as u64))
}

/// Port of `java.util.UUID.toString` (8-4-4-4-12 lowercase hex).
fn format_uuid(most: u64, least: u64) -> String {
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        (most >> 32) & 0xFFFF_FFFF,
        (most >> 16) & 0xFFFF,
        most & 0xFFFF,
        (least >> 48) & 0xFFFF,
        least & 0xFFFF_FFFF_FFFF,
    )
}

/// Read an int array (`int[]`) field, leniently (`getInts`).
fn get_ints<'a>(map: &'a NbtMap, key: &str) -> Option<&'a [i32]> {
    match map.get(key) {
        Some(NbtValue::IntArray(v)) => Some(v.as_slice()),
        _ => None,
    }
}

/// `remapModifiers`: collapse the modifier list, resolving uuid/name to ids and
/// deduplicating (name matches accumulate `amount`). Preserves first-seen order
/// (LinkedHashMap semantics).
fn remap_modifiers(list: &[NbtValue]) -> Vec<NbtValue> {
    // (resolved id, modifier map). Vec preserves insertion order; lookups are
    // linear which is fine for the small modifier lists in practice.
    let mut ret: Vec<(String, NbtMap)> = Vec::new();

    for el in list {
        let modifier = match el.as_compound_ref() {
            Some(m) => m.clone(),
            None => continue,
        };

        let uuid = make_uuid(get_ints(&modifier, "uuid"));
        let name = modifier.get_string("name").unwrap_or("").to_string();

        let remapped_uuid = uuid
            .as_deref()
            .and_then(|u| UUID_TO_ID.iter().find(|(k, _)| *k == u).map(|(_, v)| *v));
        let remapped_name = NAME_TO_ID
            .iter()
            .find(|(k, _)| *k == name.as_str())
            .map(|(_, v)| *v);

        if let Some(id) = remapped_uuid {
            let mut m = modifier;
            m.take("uuid");
            m.take("name");
            m.set_string("id", id);
            upsert(&mut ret, id, m);
        } else if let Some(id) = remapped_name {
            if let Some((_, existing)) = ret.iter_mut().find(|(k, _)| k == id) {
                let sum = existing.get_f64("amount").unwrap_or(0.0)
                    + modifier.get_f64("amount").unwrap_or(0.0);
                existing.set_f64("amount", sum);
            } else {
                let mut m = modifier;
                m.take("uuid");
                m.take("name");
                m.set_string("id", id);
                upsert(&mut ret, id, m);
            }
        } else {
            let id = format!("minecraft:{}", uuid.as_deref().unwrap_or("unknown"));
            let mut m = modifier;
            m.set_string("id", &id);
            upsert(&mut ret, &id, m);
        }
    }

    ret.into_iter()
        .map(|(_, m)| NbtValue::Compound(m))
        .collect()
}

/// Insert/replace by id (LinkedHashMap.put: overwrites value, keeps position).
fn upsert(ret: &mut Vec<(String, NbtMap)>, id: &str, m: NbtMap) {
    if let Some(slot) = ret.iter_mut().find(|(k, _)| k == id) {
        slot.1 = m;
    } else {
        ret.push((id.to_string(), m));
    }
}

/// Map the legacy integer `Operation` to the new string `operation`.
fn operation_str(op: i32) -> &'static str {
    match op {
        0 => "add_value",
        1 => "add_multiplied_base",
        2 => "add_multiplied_total",
        _ => "invalid",
    }
}

/// Shared ENTITY/PLAYER attribute converter (PLAYER not registered here).
fn convert_entity_attributes(data: &mut NbtMap) {
    data.rename_key("Attributes", "attributes");

    let attributes = match data.get_list_mut("attributes") {
        Some(l) => l,
        None => return,
    };

    for el in attributes.iter_mut() {
        let attribute = match el.as_compound_mut() {
            Some(a) => a,
            None => continue,
        };

        attribute.rename_key("Name", "id");
        attribute.rename_key("Base", "base");
        attribute.rename_key("Modifiers", "modifiers");

        let modifiers = match attribute.get_list_mut("modifiers") {
            Some(m) => m,
            None => continue,
        };

        for m_el in modifiers.iter_mut() {
            if let Some(modifier) = m_el.as_compound_mut() {
                modifier.rename_key("UUID", "uuid");
                modifier.rename_key("Name", "name");
                modifier.rename_key("Amount", "amount");
                let op = modifier.get_i32("Operation").unwrap_or(0);
                modifier.take("Operation");
                modifier.set_string("operation", operation_str(op));
            }
        }

        // remapModifiers over the just-normalized list.
        let remapped = remap_modifiers(modifiers);
        attribute.set_list("modifiers", remapped);
    }
}

// ---------------------------------------------------------------------------
// Reverse (new -> old) helpers.
// ---------------------------------------------------------------------------

/// Parse a canonical UUID string into a length-4 `int[]` (inverse of
/// `make_uuid`/`format_uuid`): split into `(most, least)` u64 then unpack each
/// into two i32 words. Returns `None` if the string is not a valid UUID.
fn uuid_string_to_ints(s: &str) -> Option<Vec<i32>> {
    let hex: String = s.chars().filter(|c| *c != '-').collect();
    if hex.len() != 32 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let most = u64::from_str_radix(&hex[0..16], 16).ok()?;
    let least = u64::from_str_radix(&hex[16..32], 16).ok()?;
    Some(vec![
        (most >> 32) as u32 as i32,
        (most & 0xFFFF_FFFF) as u32 as i32,
        (least >> 32) as u32 as i32,
        (least & 0xFFFF_FFFF) as u32 as i32,
    ])
}

/// Inverse of `operation_str`: new string `operation` -> legacy int `Operation`.
/// `"invalid"` had no integer preimage; fall back to 0 (`add_value`).
fn operation_int(op: &str) -> i32 {
    match op {
        "add_value" => 0,
        "add_multiplied_base" => 1,
        "add_multiplied_total" => 2,
        _ => {
            report_loss(
                VERSION,
                LossKind::Other,
                Severity::Approximated,
                format!("unknown attribute modifier operation '{op}' cannot be represented before V3945; used add_value"),
            );
            0
        }
    }
}

/// Resolve a stable `id` back to its legacy UUID string, choosing the canonical
/// (first-listed) preimage when several legacy UUIDs collapsed onto one id.
/// Returns `(uuid_string, ambiguous)` where `ambiguous` flags the multi-preimage
/// ids (`minecraft:baby`, `minecraft:attacking`, `minecraft:enchantment.soul_speed`).
fn id_to_uuid(id: &str) -> Option<(&'static str, bool)> {
    let mut found: Option<&'static str> = None;
    let mut count = 0usize;
    for (uuid, mapped_id) in UUID_TO_ID {
        if *mapped_id == id {
            count += 1;
            if found.is_none() {
                found = Some(uuid);
            }
        }
    }
    found.map(|u| (u, count > 1))
}

/// Resolve a stable `id` back to its legacy modifier `name` (1:1 table).
fn id_to_name(id: &str) -> Option<&'static str> {
    NAME_TO_ID.iter().find(|(_, v)| *v == id).map(|(k, _)| *k)
}

/// Reverse of `remap_modifiers` for a single modifier compound, in place.
///
/// Forward set `id` from a legacy `uuid`/`name`; reverse reconstructs the legacy
/// shape. For UUID-resolved ids we restore the `uuid` `int[4]` exactly but the
/// descriptive `name` was dropped and cannot be recovered (loss). For
/// name-resolved ids we restore `name` but the original per-modifier `uuid` is
/// gone, and the forward also accumulated `amount` across duplicates so a split
/// cannot be reconstructed (loss). Unknown ids (`minecraft:<uuid>`) kept their
/// `uuid`/`name`, so we only strip the synthetic `id`.
fn unremap_modifier(modifier: &mut NbtMap) {
    let id = match modifier.get_string("id") {
        Some(s) => s.to_string(),
        None => return,
    };

    if let Some((uuid, ambiguous)) = id_to_uuid(&id) {
        modifier.take("id");
        if let Some(arr) = uuid_string_to_ints(uuid) {
            modifier.set_generic("uuid", NbtValue::IntArray(arr));
        }
        // The legacy descriptive `name` was removed by the forward remap.
        report_loss(
            VERSION,
            LossKind::RenameAmbiguous,
            Severity::Loss,
            "attribute modifier name dropped when id was resolved from uuid; cannot restore",
        );
        if ambiguous {
            report_loss(
                VERSION,
                LossKind::FingerprintCollapse,
                Severity::Approximated,
                "attribute modifier id had multiple legacy uuid preimages; chose canonical one",
            );
        }
    } else if let Some(name) = id_to_name(&id) {
        modifier.take("id");
        modifier.set_string("name", name);
        // The forward dropped the per-modifier uuid and merged duplicate
        // name-keyed modifiers by summing `amount`; neither is recoverable.
        report_loss(
            VERSION,
            LossKind::FingerprintCollapse,
            Severity::Loss,
            "name-resolved attribute modifier lost its uuid and any duplicate-merge split",
        );
    } else if let Some(rest) = id.strip_prefix("minecraft:") {
        // Unknown branch: forward kept `uuid`/`name` and only added a synthetic
        // id. If the uuid is missing, reconstruct it from the id text.
        modifier.take("id");
        if get_ints(modifier, "uuid").is_none() && rest != "unknown" {
            if let Some(arr) = uuid_string_to_ints(rest) {
                modifier.set_generic("uuid", NbtValue::IntArray(arr));
            }
        }
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    // ITEM_STACK: remap component attribute modifiers.
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let components = match data.get_map_mut("components") {
                Some(c) => c,
                None => return,
            };
            let attribute_modifiers = match components.get_map_mut("minecraft:attribute_modifiers")
            {
                Some(a) => a,
                None => return,
            };
            if let Some(modifiers) = attribute_modifiers.get_list("modifiers") {
                let remapped = remap_modifiers(modifiers);
                attribute_modifiers.set_list("modifiers", remapped);
            }
        }),
    );

    // ITEM_STACK reverse: undo the `remap_modifiers` id resolution, restoring
    // the legacy `{uuid, name, amount}` shape. Lossy (see `unremap_modifier`).
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let components = match data.get_map_mut("components") {
                Some(c) => c,
                None => return,
            };
            let attribute_modifiers = match components.get_map_mut("minecraft:attribute_modifiers")
            {
                Some(a) => a,
                None => return,
            };
            if let Some(modifiers) = attribute_modifiers.get_list_mut("modifiers") {
                for el in modifiers.iter_mut() {
                    if let Some(modifier) = el.as_compound_mut() {
                        unremap_modifier(modifier);
                    }
                }
            }
        }),
    );

    // ENTITY: rename + remap attributes.
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            convert_entity_attributes(data);
        }),
    );

    // ENTITY reverse: undo the attribute rename + modifier remap. Operates on
    // the forward-output schema (`attributes`/`id`/`base`/`modifiers`/string
    // `operation`). Lossy on dropped modifier names / merged duplicates.
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(attributes) = data.get_list_mut("attributes") {
                for el in attributes.iter_mut() {
                    let attribute = match el.as_compound_mut() {
                        Some(a) => a,
                        None => continue,
                    };

                    if let Some(modifiers) = attribute.get_list_mut("modifiers") {
                        for m_el in modifiers.iter_mut() {
                            if let Some(modifier) = m_el.as_compound_mut() {
                                // Reverse remap_modifiers (id -> uuid/name).
                                unremap_modifier(modifier);
                                // Reverse the per-modifier field renames.
                                let op = modifier
                                    .get_string("operation")
                                    .map(operation_int)
                                    .unwrap_or(0);
                                modifier.take("operation");
                                modifier.set_i32("Operation", op);
                                modifier.rename_key("amount", "Amount");
                                modifier.rename_key("name", "Name");
                                modifier.rename_key("uuid", "UUID");
                            }
                        }
                    }

                    // Reverse the per-attribute field renames.
                    attribute.rename_key("modifiers", "Modifiers");
                    attribute.rename_key("base", "Base");
                    attribute.rename_key("id", "Name");
                }
            }

            // Reverse the top-level list rename.
            data.rename_key("attributes", "Attributes");
        }),
    );

    // TILE_ENTITY minecraft:jukebox: collapse playback bookkeeping.
    reg.tile_entity.add_converter_for_id(
        "minecraft:jukebox",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let playing_for = data.get_i64("TickCount").unwrap_or(0)
                - data.get_i64("RecordStartTick").unwrap_or(0);

            data.take("IsPlaying");
            data.take("TickCount");
            data.take("RecordStartTick");

            if playing_for > 0 {
                data.set_i64("ticks_since_song_started", playing_for);
            }
        }),
    );

    // TILE_ENTITY minecraft:jukebox reverse: re-expand the playback bookkeeping.
    // The forward collapsed `IsPlaying/TickCount/RecordStartTick` into the single
    // relative `ticks_since_song_started`. We reconstruct a *consistent* triple
    // (origin at tick 0) but the original absolute `TickCount`/`RecordStartTick`
    // values are unrecoverable, and a non-playing record with no remaining song
    // (`ticks_since_song_started` absent) cannot be distinguished from an empty
    // jukebox -> approximated.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:jukebox",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            match data.take("ticks_since_song_started") {
                Some(v) => {
                    let elapsed = v.as_number_i64().unwrap_or(0).max(0);
                    data.set_bool("IsPlaying", true);
                    data.set_i64("RecordStartTick", 0);
                    data.set_i64("TickCount", elapsed);
                    report_loss(
                        VERSION,
                        LossKind::Other,
                        Severity::Approximated,
                        "jukebox absolute TickCount/RecordStartTick reconstructed from relative offset (origin 0)",
                    );
                }
                None => {
                    data.set_bool("IsPlaying", false);
                    data.set_i64("RecordStartTick", 0);
                    data.set_i64("TickCount", 0);
                    report_loss(
                        VERSION,
                        LossKind::FingerprintCollapse,
                        Severity::Approximated,
                        "jukebox without ticks_since_song_started cannot distinguish an idle record from an empty jukebox; reconstructed stopped playback at tick 0",
                    );
                }
            }
        }),
    );
}
