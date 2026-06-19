//! V2511 (20w09a + 1) — schematic-relevant subset of `V2511.java`.
//!
//! Projectile/throwable owner-UUID migration to the modern int-array `OwnerUUID`
//! plus a couple of structural moves (V2511.java:29-95):
//!   * throwable (`egg`/`ender_pearl`/`experience_bottle`/`snowball`/`potion`):
//!     drop the `owner` compound, lift its `M`/`L` longs into `OwnerUUID`.
//!   * `potion`: move the `Potion` compound into `Item` (empty map if absent).
//!   * `llama_spit`: drop `Owner`, lift its `OwnerUUIDMost`/`OwnerUUIDLeast`.
//!   * `arrow`/`spectral_arrow`/`trident`: lift `OwnerUUIDMost`/`OwnerUUIDLeast`
//!     into `OwnerUUID` and drop the old longs.
//!   * ENTITY walker for `minecraft:potion` over the `Item` itemstack.
//!
//! `setUUID` only writes when BOTH longs are non-zero (note: `&&`, unlike V2514's
//! `||`), matching the Java exactly.
//!
//! VERSION = MCVersions.V20W09A (2510) + 1 = 2511.

use crate::nbt::{NbtMap, NbtValue};

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::items;

const VERSION: i32 = 2511;

/// `createUUIDArray(most, least)` (V2511.java:14-21): pack two longs into an
/// int[4] (most-hi, most-lo, least-hi, least-lo).
fn create_uuid_array(most: i64, least: i64) -> Vec<i32> {
    vec![
        (((most as u64) >> 32) as u32) as i32,
        (most as u32) as i32,
        (((least as u64) >> 32) as u32) as i32,
        (least as u32) as i32,
    ]
}

/// `setUUID` (V2511.java:23-27): write `OwnerUUID` int-array, but only when both
/// halves are non-zero.
fn set_uuid(data: &mut NbtMap, most: i64, least: i64) {
    if most != 0 && least != 0 {
        data.set_generic("OwnerUUID", NbtValue::IntArray(create_uuid_array(most, least)));
    }
}

/// Exact inverse of `create_uuid_array`: read an `int[4]` field as a `(most, least)`
/// long pair. Each i32 is a 32-bit slice of a 64-bit long, so recombining loses
/// nothing. Returns `None` if the field is missing or not length 4.
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

fn throwable(data: &mut NbtMap, _from: super::super::version::EncodedVersion, _to: super::super::version::EncodedVersion) {
    let owner = match data.take("owner") {
        Some(NbtValue::Compound(m)) => m,
        _ => {
            // remove still happened above; if not a compound, nothing to lift.
            return;
        }
    };
    let most = owner.get_i64("M").unwrap_or(0);
    let least = owner.get_i64("L").unwrap_or(0);
    set_uuid(data, most, least);
}

pub fn register(reg: &mut RegistryBuilder) {
    for id in ["minecraft:egg", "minecraft:ender_pearl", "minecraft:experience_bottle", "minecraft:snowball", "minecraft:potion"] {
        reg.entity.add_converter_for_id(id, VERSION, 0, Box::new(throwable));
        // Reverse: int-array `OwnerUUID` -> the `owner` compound with `M`/`L` longs
        // (exact inverse of create_uuid_array; lossless). The forward only wrote
        // OwnerUUID when both halves were non-zero, so a missing/short array means
        // there is no owner to restore — matching the forward's drop of all-zero UUIDs.
        reg.entity.add_reverse_converter_for_id(id, VERSION, 0, Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some((most, least)) = read_uuid_array(data, "OwnerUUID") {
                data.take("OwnerUUID");
                let mut owner = NbtMap::new();
                owner.set_i64("M", most);
                owner.set_i64("L", least);
                data.set_map("owner", owner);
            }
        }));
    }

    // potion: move Potion -> Item (empty map if absent).
    reg.entity.add_converter_for_id(
        "minecraft:potion",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let potion = match data.take("Potion") {
                Some(NbtValue::Compound(m)) => m,
                _ => NbtMap::new(),
            };
            data.set_map("Item", potion);
        }),
    );
    // Reverse: move `Item` back to `Potion`. Lossless rename of the compound.
    // The forward used an empty map when `Potion` was absent; reverse simply moves
    // whatever `Item` holds back to `Potion` (an empty `Item` round-trips to an empty
    // `Potion`, which the old format also treated as "no potion"). Runs BEFORE the
    // throwable reverse (engine iterates reverse converters in reverse registration
    // order), mirroring the forward order throwable -> potion for `minecraft:potion`.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:potion",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(NbtValue::Compound(item)) = data.take("Item") {
                data.set_map("Potion", item);
            }
        }),
    );

    // llama_spit: drop Owner, lift OwnerUUIDMost/Least.
    reg.entity.add_converter_for_id(
        "minecraft:llama_spit",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let owner = match data.take("Owner") {
                Some(NbtValue::Compound(m)) => m,
                _ => return,
            };
            let most = owner.get_i64("OwnerUUIDMost").unwrap_or(0);
            let least = owner.get_i64("OwnerUUIDLeast").unwrap_or(0);
            set_uuid(data, most, least);
        }),
    );
    // Reverse: top-level int-array `OwnerUUID` -> the `Owner` compound holding
    // `OwnerUUIDMost`/`OwnerUUIDLeast` longs (exact inverse; lossless). Forward only
    // wrote OwnerUUID when both halves were non-zero, so a missing array restores nothing.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:llama_spit",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some((most, least)) = read_uuid_array(data, "OwnerUUID") {
                data.take("OwnerUUID");
                let mut owner = NbtMap::new();
                owner.set_i64("OwnerUUIDMost", most);
                owner.set_i64("OwnerUUIDLeast", least);
                data.set_map("Owner", owner);
            }
        }),
    );

    // arrow family: lift OwnerUUIDMost/Least into OwnerUUID; drop the old longs.
    for id in ["minecraft:arrow", "minecraft:spectral_arrow", "minecraft:trident"] {
        reg.entity.add_converter_for_id(
            id,
            VERSION,
            0,
            Box::new(|data: &mut NbtMap, _from, _to| {
                let most = data.get_i64("OwnerUUIDMost").unwrap_or(0);
                let least = data.get_i64("OwnerUUIDLeast").unwrap_or(0);
                set_uuid(data, most, least);
                data.take("OwnerUUIDMost");
                data.take("OwnerUUIDLeast");
            }),
        );
        // Reverse: top-level int-array `OwnerUUID` -> `OwnerUUIDMost`/`OwnerUUIDLeast`
        // longs, dropping the array (exact inverse; lossless). Forward only wrote
        // OwnerUUID when both halves were non-zero, so a missing array restores nothing.
        reg.entity.add_reverse_converter_for_id(
            id,
            VERSION,
            0,
            Box::new(|data: &mut NbtMap, _from, _to| {
                if let Some((most, least)) = read_uuid_array(data, "OwnerUUID") {
                    data.take("OwnerUUID");
                    data.set_i64("OwnerUUIDMost", most);
                    data.set_i64("OwnerUUIDLeast", least);
                }
            }),
        );
    }

    // ENTITY walker for potion's Item itemstack.
    reg.entity.add_walker(VERSION, 0, "minecraft:potion", items(&["Item"]));
}
