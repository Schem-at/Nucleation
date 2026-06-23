//! V2516 (20w12a + 1) — schematic-relevant subset of `V2516.java`.
//!
//! ENTITY `villager`/`zombie_villager`: migrate each `Gossips[].Target` UUID from
//! `TargetMost`/`TargetLeast` longs to the int-array form, reusing V2514's
//! `replaceUUIDLeastMost` (V2516.java:14-33).
//!
//! The helper is duplicated here (rather than importing from V2514) so this file
//! is self-contained; it is identical to `V2514.replaceUUIDLeastMost` — int[4]
//! iff EITHER half is non-zero (`||`).
//!
//! VERSION = MCVersions.V20W12A (2515) + 1 = 2516.

use crate::nbt::{NbtMap, NbtValue};

use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 2516;

fn create_uuid_array(most: i64, least: i64) -> Vec<i32> {
    vec![
        (((most as u64) >> 32) as u32) as i32,
        (most as u32) as i32,
        (((least as u64) >> 32) as u32) as i32,
        (least as u32) as i32,
    ]
}

/// `V2514.replaceUUIDLeastMost(data, prefix, newPath)`.
fn replace_uuid_least_most(data: &mut NbtMap, prefix: &str, new_path: &str) {
    let most_path = format!("{prefix}Most");
    let least_path = format!("{prefix}Least");
    let most = data.get_i64(&most_path).unwrap_or(0);
    let least = data.get_i64(&least_path).unwrap_or(0);
    if most != 0 || least != 0 {
        data.take(&most_path);
        data.take(&least_path);
        data.set_generic(new_path, NbtValue::IntArray(create_uuid_array(most, least)));
    }
}

fn convert_gossips(data: &mut NbtMap) {
    if let Some(gossips) = data.get_list_mut("Gossips") {
        for el in gossips.iter_mut() {
            if let Some(gossip) = el.as_compound_mut() {
                replace_uuid_least_most(gossip, "Target", "Target");
            }
        }
    }
}

// --- reverse --------------------------------------------------------------
//
// The forward packs the `<prefix>Most`/`<prefix>Least` longs into an int[4] via
// `create_uuid_array` (most-hi, most-lo, least-hi, least-lo). Reverse unpacks that
// int[4] back into the original (most, least) pair — exact, since each i32 is a
// 32-bit slice of a 64-bit long, so recombining is lossless. Absent / wrong-length
// arrays restore nothing (mirroring the forward's "EITHER half non-zero" guard,
// which dropped all-zero UUIDs that carried no information). No `report_loss`.

/// Inverse of `replace_uuid_least_most`: int-array at `new_path` -> `<prefix>Most`
/// / `<prefix>Least` longs.
fn restore_uuid_least_most(data: &mut NbtMap, prefix: &str, new_path: &str) {
    let (most, least) = match data.get(new_path) {
        Some(NbtValue::IntArray(v)) if v.len() == 4 => {
            let most = ((v[0] as i64) << 32) | ((v[1] as i64) & 0xFFFF_FFFF);
            let least = ((v[2] as i64) << 32) | ((v[3] as i64) & 0xFFFF_FFFF);
            (most, least)
        }
        _ => return,
    };
    data.take(new_path);
    data.set_i64(&format!("{prefix}Most"), most);
    data.set_i64(&format!("{prefix}Least"), least);
}

fn restore_gossips(data: &mut NbtMap) {
    if let Some(gossips) = data.get_list_mut("Gossips") {
        for el in gossips.iter_mut() {
            if let Some(gossip) = el.as_compound_mut() {
                restore_uuid_least_most(gossip, "Target", "Target");
            }
        }
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:villager",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| convert_gossips(data)),
    );
    reg.entity.add_reverse_converter_for_id(
        "minecraft:villager",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| restore_gossips(data)),
    );
    reg.entity.add_converter_for_id(
        "minecraft:zombie_villager",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| convert_gossips(data)),
    );
    reg.entity.add_reverse_converter_for_id(
        "minecraft:zombie_villager",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| restore_gossips(data)),
    );
}
