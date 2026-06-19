//! V2514 (20w11a + 1) — schematic-relevant subset of `V2514.java`.
//!
//! The big entity/tile-entity/item UUID migration from `*Most`/`*Least` longs (or
//! `M`/`L` compounds, or UUID strings) to the modern int-array form
//! (V2514.java:296-588).
//!
//! Ported (schematic-relevant):
//!   * ENTITY structure converter: `UUID` least/most -> int-array `UUID`.
//!   * per-id converters over the abstract-horse / tameable / animal / mob /
//!     living / projectile sets, plus the special cases (bee, zombified_piglin,
//!     fox, item, shulker_bullet, area_effect_cloud, zombie_villager,
//!     evoker_fangs, piglin).
//!   * TILE_ENTITY `conduit` (target_uuid -> Target) and `skull` (Owner -> SkullOwner).
//!   * ITEM_STACK: AttributeModifiers UUIDs + player_head SkullOwner Id.
//!
//! Skipped (non-schematic): PLAYER structure converter (RootVehicle), LEVEL
//! (WanderingTraderId / DimensionData / CustomBossEvents), SAVED_DATA_RAIDS.
//!
//! The `replace_uuid_least_most` / `replace_uuid_ml_tag` / `create_uuid_from_longs`
//! helpers use Java's `||` (either half non-zero) rule, distinct from V2511's `&&`.
//! (V2516 duplicates `replaceUUIDLeastMost` locally rather than importing it, so
//! these helpers stay private to this file.)
//!
//! VERSION = MCVersions.V20W11A (2513) + 1 = 2514.

use crate::nbt::{NbtMap, NbtValue};

use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 2514;

const ABSTRACT_HORSES: &[&str] = &[
    "minecraft:donkey",
    "minecraft:horse",
    "minecraft:llama",
    "minecraft:mule",
    "minecraft:skeleton_horse",
    "minecraft:trader_llama",
    "minecraft:zombie_horse",
];

const TAMEABLE_ANIMALS: &[&str] = &["minecraft:cat", "minecraft:parrot", "minecraft:wolf"];

const ANIMALS: &[&str] = &[
    "minecraft:bee",
    "minecraft:chicken",
    "minecraft:cow",
    "minecraft:fox",
    "minecraft:mooshroom",
    "minecraft:ocelot",
    "minecraft:panda",
    "minecraft:pig",
    "minecraft:polar_bear",
    "minecraft:rabbit",
    "minecraft:sheep",
    "minecraft:turtle",
    "minecraft:hoglin",
];

const MOBS: &[&str] = &[
    "minecraft:bat",
    "minecraft:blaze",
    "minecraft:cave_spider",
    "minecraft:cod",
    "minecraft:creeper",
    "minecraft:dolphin",
    "minecraft:drowned",
    "minecraft:elder_guardian",
    "minecraft:ender_dragon",
    "minecraft:enderman",
    "minecraft:endermite",
    "minecraft:evoker",
    "minecraft:ghast",
    "minecraft:giant",
    "minecraft:guardian",
    "minecraft:husk",
    "minecraft:illusioner",
    "minecraft:magma_cube",
    "minecraft:pufferfish",
    "minecraft:zombified_piglin",
    "minecraft:salmon",
    "minecraft:shulker",
    "minecraft:silverfish",
    "minecraft:skeleton",
    "minecraft:slime",
    "minecraft:snow_golem",
    "minecraft:spider",
    "minecraft:squid",
    "minecraft:stray",
    "minecraft:tropical_fish",
    "minecraft:vex",
    "minecraft:villager",
    "minecraft:iron_golem",
    "minecraft:vindicator",
    "minecraft:pillager",
    "minecraft:wandering_trader",
    "minecraft:witch",
    "minecraft:wither",
    "minecraft:wither_skeleton",
    "minecraft:zombie",
    "minecraft:zombie_villager",
    "minecraft:phantom",
    "minecraft:ravager",
    "minecraft:piglin",
];

const LIVING_ENTITIES: &[&str] = &["minecraft:armor_stand"];

const PROJECTILES: &[&str] = &[
    "minecraft:arrow",
    "minecraft:dragon_fireball",
    "minecraft:firework_rocket",
    "minecraft:fireball",
    "minecraft:llama_spit",
    "minecraft:small_fireball",
    "minecraft:snowball",
    "minecraft:spectral_arrow",
    "minecraft:egg",
    "minecraft:ender_pearl",
    "minecraft:experience_bottle",
    "minecraft:potion",
    "minecraft:trident",
    "minecraft:wither_skull",
];

// --- UUID helpers (V2514.java:114-178) --------------------------------------

/// `createUUIDArray(most, least)`: pack two longs into an int[4].
fn create_uuid_array(most: i64, least: i64) -> Vec<i32> {
    vec![
        (((most as u64) >> 32) as u32) as i32,
        (most as u32) as i32,
        (((least as u64) >> 32) as u32) as i32,
        (least as u32) as i32,
    ]
}

/// Port of `UUID.fromString(s)` -> `(most, least)`. Returns `None` for any value
/// that is not a valid dash-separated hex UUID (Java throws `IllegalArgumentException`).
fn parse_uuid(s: &str) -> Option<(i64, i64)> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return None;
    }
    let p = |hex: &str| u64::from_str_radix(hex, 16).ok();
    let mut most = p(parts[0])? & 0xffff_ffff;
    most <<= 16;
    most |= p(parts[1])? & 0xffff;
    most <<= 16;
    most |= p(parts[2])? & 0xffff;

    let mut least = p(parts[3])? & 0xffff;
    least <<= 48;
    least |= p(parts[4])? & 0xffff_ffff_ffff;

    Some((most as i64, least as i64))
}

/// `createUUIDFromString(data, path)`: parse a UUID string into an int[4].
fn create_uuid_from_string(data: &NbtMap, path: &str) -> Option<Vec<i32>> {
    let s = data.get_string(path)?;
    let (most, least) = parse_uuid(s)?;
    Some(create_uuid_array(most, least))
}

/// `createUUIDFromLongs(data, most, least)`: int[4] iff EITHER half is non-zero
/// (Java uses `||` here — note the contrast with V2511's `&&`).
fn create_uuid_from_longs(data: &NbtMap, most: &str, least: &str) -> Option<Vec<i32>> {
    let most_bits = data.get_i64(most).unwrap_or(0);
    let least_bits = data.get_i64(least).unwrap_or(0);
    if most_bits != 0 || least_bits != 0 {
        Some(create_uuid_array(most_bits, least_bits))
    } else {
        None
    }
}

/// `replaceUUIDString(data, oldPath, newPath)`.
fn replace_uuid_string(data: &mut NbtMap, old_path: &str, new_path: &str) {
    if let Some(new_uuid) = create_uuid_from_string(data, old_path) {
        data.take(old_path);
        data.set_generic(new_path, NbtValue::IntArray(new_uuid));
    }
}

/// `replaceUUIDMLTag(data, oldPath, newPath)`: read the `M`/`L` compound at
/// `oldPath`, remove it, then write the int-array (if non-zero) at `newPath`.
fn replace_uuid_ml_tag(data: &mut NbtMap, old_path: &str, new_path: &str) {
    let uuid = data.get_map(old_path).and_then(|m| create_uuid_from_longs(m, "M", "L"));
    data.take(old_path);
    if let Some(uuid) = uuid {
        data.set_generic(new_path, NbtValue::IntArray(uuid));
    }
}

/// `replaceUUIDLeastMost(data, prefix, newPath)`: read `<prefix>Most`/`<prefix>Least`.
fn replace_uuid_least_most(data: &mut NbtMap, prefix: &str, new_path: &str) {
    let most_path = format!("{prefix}Most");
    let least_path = format!("{prefix}Least");
    if let Some(uuid) = create_uuid_from_longs(data, &most_path, &least_path) {
        data.take(&most_path);
        data.take(&least_path);
        data.set_generic(new_path, NbtValue::IntArray(uuid));
    }
}

// --- reverse UUID helpers ---------------------------------------------------
//
// All three forward forms pack a UUID into an int[4] via `create_uuid_array`
// (most-hi, most-lo, least-hi, least-lo). The reverse simply unpacks that int[4]
// back into the original (most, least) pair, which is exact: each i32 is a 32-bit
// slice of a 64-bit long, so recombining loses nothing. Absent / wrong-length
// arrays restore nothing (mirroring the forward's "EITHER half non-zero" guard,
// which dropped all-zero UUIDs).

/// Read an `int[4]` field as a `(most, least)` long pair — the exact inverse of
/// `create_uuid_array`. Returns `None` if the field is missing or not length 4.
fn read_uuid_array(data: &NbtMap, path: &str) -> Option<(i64, i64)> {
    let arr = match data.get(path) {
        Some(NbtValue::IntArray(v)) => v.as_slice(),
        _ => return None,
    };
    if arr.len() != 4 {
        return None;
    }
    let most = ((arr[0] as i64) << 32) | ((arr[1] as i64) & 0xFFFF_FFFF);
    let least = ((arr[2] as i64) << 32) | ((arr[3] as i64) & 0xFFFF_FFFF);
    Some((most, least))
}

/// Port of `java.util.UUID.toString` (8-4-4-4-12 lowercase hex) — the canonical
/// inverse of `parse_uuid` (`UUID.fromString` round-trips to canonical form).
fn format_uuid(most: i64, least: i64) -> String {
    let (most, least) = (most as u64, least as u64);
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        (most >> 32) & 0xFFFF_FFFF,
        (most >> 16) & 0xFFFF,
        most & 0xFFFF,
        (least >> 48) & 0xFFFF,
        least & 0xFFFF_FFFF_FFFF,
    )
}

/// Inverse of `replace_uuid_string`: int-array at `new_path` -> canonical UUID
/// string at `old_path`.
fn restore_uuid_string(data: &mut NbtMap, old_path: &str, new_path: &str) {
    if let Some((most, least)) = read_uuid_array(data, new_path) {
        data.take(new_path);
        data.set_string(old_path, format_uuid(most, least));
    }
}

/// Inverse of `replace_uuid_ml_tag`: int-array at `new_path` -> `{M, L}` compound
/// at `old_path`.
fn restore_uuid_ml_tag(data: &mut NbtMap, old_path: &str, new_path: &str) {
    if let Some((most, least)) = read_uuid_array(data, new_path) {
        data.take(new_path);
        let mut ml = NbtMap::new();
        ml.set_i64("M", most);
        ml.set_i64("L", least);
        data.set_map(old_path, ml);
    }
}

/// Inverse of `replace_uuid_least_most`: int-array at `new_path` -> `<prefix>Most`
/// / `<prefix>Least` longs.
fn restore_uuid_least_most(data: &mut NbtMap, prefix: &str, new_path: &str) {
    if let Some((most, least)) = read_uuid_array(data, new_path) {
        data.take(new_path);
        data.set_i64(&format!("{prefix}Most"), most);
        data.set_i64(&format!("{prefix}Least"), least);
    }
}

// --- reverse per-entity update routines -------------------------------------

fn restore_entity_uuid(data: &mut NbtMap) {
    restore_uuid_least_most(data, "UUID", "UUID");
}

fn restore_living_entity(data: &mut NbtMap) {
    let attributes = match data.get_list_mut("Attributes") {
        Some(a) => a,
        None => return,
    };
    for attr_el in attributes.iter_mut() {
        let attr = match attr_el.as_compound_mut() {
            Some(a) => a,
            None => continue,
        };
        let modifiers = match attr.get_list_mut("Modifiers") {
            Some(m) => m,
            None => continue,
        };
        for mod_el in modifiers.iter_mut() {
            if let Some(modifier) = mod_el.as_compound_mut() {
                restore_uuid_least_most(modifier, "UUID", "UUID");
            }
        }
    }
}

fn restore_mob(data: &mut NbtMap) {
    restore_living_entity(data);
    if let Some(leash) = data.get_map_mut("Leash") {
        restore_uuid_least_most(leash, "UUID", "UUID");
    }
}

fn restore_animal(data: &mut NbtMap) {
    restore_mob(data);
    restore_uuid_least_most(data, "LoveCause", "LoveCause");
}

fn restore_animal_owner(data: &mut NbtMap) {
    restore_animal(data);
    restore_uuid_string(data, "OwnerUUID", "Owner");
}

fn restore_hurt_by(data: &mut NbtMap) {
    restore_uuid_string(data, "HurtBy", "HurtBy");
}

/// Inverse of `update_fox`: `Trusted` (list of int[4]) -> `TrustedUUIDs` (list of
/// `{M, L}` compounds). Exact; the forward dropped only all-zero entries, which
/// reverse cannot resurrect but those carried no information.
fn restore_fox(data: &mut NbtMap) {
    let pairs: Vec<(i64, i64)> = match data.get_list("Trusted") {
        Some(list) => list
            .iter()
            .filter_map(|el| match el {
                NbtValue::IntArray(v) if v.len() == 4 => {
                    let most = ((v[0] as i64) << 32) | ((v[1] as i64) & 0xFFFF_FFFF);
                    let least = ((v[2] as i64) << 32) | ((v[3] as i64) & 0xFFFF_FFFF);
                    Some((most, least))
                }
                _ => None,
            })
            .collect(),
        None => return,
    };
    data.take("Trusted");
    let new_list: Vec<NbtValue> = pairs
        .into_iter()
        .map(|(most, least)| {
            let mut ml = NbtMap::new();
            ml.set_i64("M", most);
            ml.set_i64("L", least);
            NbtValue::Compound(ml)
        })
        .collect();
    data.set_list("TrustedUUIDs", new_list);
}

fn restore_item(data: &mut NbtMap) {
    restore_uuid_ml_tag(data, "Owner", "Owner");
    restore_uuid_ml_tag(data, "Thrower", "Thrower");
}

fn restore_shulker_bullet(data: &mut NbtMap) {
    restore_uuid_ml_tag(data, "Owner", "Owner");
    restore_uuid_ml_tag(data, "Target", "Target");
}

fn restore_area_effect_cloud(data: &mut NbtMap) {
    restore_uuid_least_most(data, "OwnerUUID", "Owner");
}

fn restore_zombie_villager(data: &mut NbtMap) {
    restore_uuid_least_most(data, "ConversionPlayer", "ConversionPlayer");
}

fn restore_evoker_fangs(data: &mut NbtMap) {
    restore_uuid_least_most(data, "OwnerUUID", "Owner");
}

fn restore_piglin(data: &mut NbtMap) {
    let brain = match data.get_map_mut("Brain") {
        Some(b) => b,
        None => return,
    };
    let memories = match brain.get_map_mut("memories") {
        Some(m) => m,
        None => return,
    };
    if let Some(angry_at) = memories.get_map_mut("minecraft:angry_at") {
        restore_uuid_string(angry_at, "value", "value");
    }
}

fn restore_projectile(data: &mut NbtMap) {
    if let Some(owner) = data.take("Owner") {
        data.set_generic("OwnerUUID", owner);
    }
}

// --- per-entity update routines (V2514.java:180-294) ------------------------

fn update_entity_uuid(data: &mut NbtMap) {
    replace_uuid_least_most(data, "UUID", "UUID");
}

fn update_living_entity(data: &mut NbtMap) {
    let attributes = match data.get_list_mut("Attributes") {
        Some(a) => a,
        None => return,
    };
    for attr_el in attributes.iter_mut() {
        let attr = match attr_el.as_compound_mut() {
            Some(a) => a,
            None => continue,
        };
        let modifiers = match attr.get_list_mut("Modifiers") {
            Some(m) => m,
            None => continue,
        };
        for mod_el in modifiers.iter_mut() {
            if let Some(modifier) = mod_el.as_compound_mut() {
                replace_uuid_least_most(modifier, "UUID", "UUID");
            }
        }
    }
}

fn update_mob(data: &mut NbtMap) {
    update_living_entity(data);
    if let Some(leash) = data.get_map_mut("Leash") {
        replace_uuid_least_most(leash, "UUID", "UUID");
    }
}

fn update_animal(data: &mut NbtMap) {
    update_mob(data);
    replace_uuid_least_most(data, "LoveCause", "LoveCause");
}

fn update_animal_owner(data: &mut NbtMap) {
    update_animal(data);
    replace_uuid_string(data, "OwnerUUID", "Owner");
}

fn update_hurt_by(data: &mut NbtMap) {
    replace_uuid_string(data, "HurtBy", "HurtBy");
}

fn update_fox(data: &mut NbtMap) {
    // Collect the new int-arrays first (immutable reads), then rebuild Trusted.
    let trusted: Vec<Vec<i32>> = match data.get_list("TrustedUUIDs") {
        Some(list) => list
            .iter()
            .filter_map(|el| el.as_compound_ref())
            .filter_map(|m| create_uuid_from_longs(m, "M", "L"))
            .collect(),
        None => return,
    };
    data.take("TrustedUUIDs");
    let new_list: Vec<NbtValue> = trusted.into_iter().map(NbtValue::IntArray).collect();
    data.set_list("Trusted", new_list);
}

fn update_item(data: &mut NbtMap) {
    replace_uuid_ml_tag(data, "Owner", "Owner");
    replace_uuid_ml_tag(data, "Thrower", "Thrower");
}

fn update_shulker_bullet(data: &mut NbtMap) {
    replace_uuid_ml_tag(data, "Owner", "Owner");
    replace_uuid_ml_tag(data, "Target", "Target");
}

fn update_area_effect_cloud(data: &mut NbtMap) {
    replace_uuid_least_most(data, "OwnerUUID", "Owner");
}

fn update_zombie_villager(data: &mut NbtMap) {
    replace_uuid_least_most(data, "ConversionPlayer", "ConversionPlayer");
}

fn update_evoker_fangs(data: &mut NbtMap) {
    replace_uuid_least_most(data, "OwnerUUID", "Owner");
}

fn update_piglin(data: &mut NbtMap) {
    let brain = match data.get_map_mut("Brain") {
        Some(b) => b,
        None => return,
    };
    let memories = match brain.get_map_mut("memories") {
        Some(m) => m,
        None => return,
    };
    // replaceUUIDString(angryAt, "value", "value") with angryAt possibly null.
    if let Some(angry_at) = memories.get_map_mut("minecraft:angry_at") {
        replace_uuid_string(angry_at, "value", "value");
    }
}

fn update_projectile(data: &mut NbtMap) {
    if let Some(owner_uuid) = data.take("OwnerUUID") {
        data.set_generic("Owner", owner_uuid);
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    // ENTITY structure converter: UUID least/most -> int-array.
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| update_entity_uuid(data)),
    );
    // Reverse: int-array UUID -> UUIDMost/UUIDLeast (lossless).
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| restore_entity_uuid(data)),
    );

    for id in ABSTRACT_HORSES.iter().chain(TAMEABLE_ANIMALS.iter()) {
        reg.entity.add_converter_for_id(*id, VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| update_animal_owner(data)));
        // Reverse: int-array UUIDs -> least/most (LoveCause, Leash, Attributes) and Owner -> OwnerUUID string (lossless).
        reg.entity.add_reverse_converter_for_id(*id, VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| restore_animal_owner(data)));
    }
    for id in ANIMALS {
        reg.entity.add_converter_for_id(*id, VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| update_animal(data)));
        reg.entity.add_reverse_converter_for_id(*id, VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| restore_animal(data)));
    }
    for id in MOBS {
        reg.entity.add_converter_for_id(*id, VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| update_mob(data)));
        reg.entity.add_reverse_converter_for_id(*id, VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| restore_mob(data)));
    }
    for id in LIVING_ENTITIES {
        reg.entity.add_converter_for_id(*id, VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| update_living_entity(data)));
        reg.entity.add_reverse_converter_for_id(*id, VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| restore_living_entity(data)));
    }
    for id in PROJECTILES {
        reg.entity.add_converter_for_id(*id, VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| update_projectile(data)));
        // Reverse: rename generic Owner -> OwnerUUID (no format change; lossless).
        reg.entity.add_reverse_converter_for_id(*id, VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| restore_projectile(data)));
    }

    reg.entity.add_converter_for_id("minecraft:bee", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| update_hurt_by(data)));
    reg.entity.add_reverse_converter_for_id("minecraft:bee", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| restore_hurt_by(data)));
    reg.entity.add_converter_for_id("minecraft:zombified_piglin", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| update_hurt_by(data)));
    reg.entity.add_reverse_converter_for_id("minecraft:zombified_piglin", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| restore_hurt_by(data)));
    reg.entity.add_converter_for_id("minecraft:fox", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| update_fox(data)));
    reg.entity.add_reverse_converter_for_id("minecraft:fox", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| restore_fox(data)));
    reg.entity.add_converter_for_id("minecraft:item", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| update_item(data)));
    reg.entity.add_reverse_converter_for_id("minecraft:item", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| restore_item(data)));
    reg.entity.add_converter_for_id("minecraft:shulker_bullet", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| update_shulker_bullet(data)));
    reg.entity.add_reverse_converter_for_id("minecraft:shulker_bullet", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| restore_shulker_bullet(data)));
    reg.entity.add_converter_for_id("minecraft:area_effect_cloud", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| update_area_effect_cloud(data)));
    reg.entity.add_reverse_converter_for_id("minecraft:area_effect_cloud", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| restore_area_effect_cloud(data)));
    reg.entity.add_converter_for_id("minecraft:zombie_villager", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| update_zombie_villager(data)));
    reg.entity.add_reverse_converter_for_id("minecraft:zombie_villager", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| restore_zombie_villager(data)));
    reg.entity.add_converter_for_id("minecraft:evoker_fangs", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| update_evoker_fangs(data)));
    reg.entity.add_reverse_converter_for_id("minecraft:evoker_fangs", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| restore_evoker_fangs(data)));
    reg.entity.add_converter_for_id("minecraft:piglin", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| update_piglin(data)));
    reg.entity.add_reverse_converter_for_id("minecraft:piglin", VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| restore_piglin(data)));

    // TILE_ENTITY.
    reg.tile_entity.add_converter_for_id(
        "minecraft:conduit",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| replace_uuid_ml_tag(data, "target_uuid", "Target")),
    );
    // Reverse: int-array Target -> {M, L} compound at target_uuid (lossless).
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:conduit",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| restore_uuid_ml_tag(data, "target_uuid", "Target")),
    );
    reg.tile_entity.add_converter_for_id(
        "minecraft:skull",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let mut owner = match data.take("Owner") {
                Some(NbtValue::Compound(m)) => m,
                _ => return,
            };
            replace_uuid_string(&mut owner, "Id", "Id");
            data.set_map("SkullOwner", owner);
        }),
    );
    // Reverse: move SkullOwner compound back to Owner, int-array Id -> UUID string (lossless).
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:skull",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let mut owner = match data.take("SkullOwner") {
                Some(NbtValue::Compound(m)) => m,
                _ => return,
            };
            restore_uuid_string(&mut owner, "Id", "Id");
            data.set_map("Owner", owner);
        }),
    );

    // ITEM_STACK: tag.AttributeModifiers UUIDs + player_head SkullOwner Id.
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let is_player_head = data.get_string("id") == Some("minecraft:player_head");
            let tag = match data.get_map_mut("tag") {
                Some(t) => t,
                None => return,
            };

            // updateAttributeModifiers(tag).
            if let Some(attributes) = tag.get_list_mut("AttributeModifiers") {
                for el in attributes.iter_mut() {
                    if let Some(attr) = el.as_compound_mut() {
                        replace_uuid_least_most(attr, "UUID", "UUID");
                    }
                }
            }

            // updateSkullOwner(tag): replaceUUIDString(tag.SkullOwner, "Id", "Id").
            if is_player_head {
                if let Some(skull_owner) = tag.get_map_mut("SkullOwner") {
                    replace_uuid_string(skull_owner, "Id", "Id");
                }
            }
        }),
    );
    // Reverse: int-array AttributeModifiers UUIDs -> UUIDMost/Least, and
    // player_head SkullOwner Id int-array -> UUID string (lossless).
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let is_player_head = data.get_string("id") == Some("minecraft:player_head");
            let tag = match data.get_map_mut("tag") {
                Some(t) => t,
                None => return,
            };

            if let Some(attributes) = tag.get_list_mut("AttributeModifiers") {
                for el in attributes.iter_mut() {
                    if let Some(attr) = el.as_compound_mut() {
                        restore_uuid_least_most(attr, "UUID", "UUID");
                    }
                }
            }

            if is_player_head {
                if let Some(skull_owner) = tag.get_map_mut("SkullOwner") {
                    restore_uuid_string(skull_owner, "Id", "Id");
                }
            }
        }),
    );
}
