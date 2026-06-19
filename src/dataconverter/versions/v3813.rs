//! V3813 (24w05b + 2) — schematic-relevant subset of `V3813.java`.
//!
//! 1.20.5 flattened a number of `{X,Y,Z}` block-position compounds into `int[3]`
//! arrays and lower-cased their keys. The shared `RootPositionConverter` does, for
//! each `(from -> to)` pair: flatten the `{X,Y,Z}` compound at `from` into an
//! `int[3]`, then rename the key `from` -> `to`.
//!
//! Ported (schematic-relevant):
//!   * ENTITY per-id: `bee` (HivePos/FlowerPos), `end_crystal` (BeamTarget),
//!     `wandering_trader` (WanderTarget), and the patrolling mobs (PatrolTarget).
//!   * ENTITY structure converter: `Leash` -> `leash`.
//!   * TILE_ENTITY per-id: `beehive` (FlowerPos), `end_gateway` (ExitPortal).
//!   * ITEM_STACK `minecraft:compass`: flatten `tag.LodestonePos` in place (no
//!     rename).
//!
//! Skipped (non-schematic): the SAVED_DATA_MAP_DATA structure converter
//! (frames/banners) and its structure walker — SAVED_DATA never appears in a
//! schematic file.
//!
//! VERSION = MCVersions.V24W05B (3811) + 2 = 3813.

use crate::nbt::{NbtMap, NbtValue};

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 3813;

const PATROLLING_MOBS: &[&str] = &[
    "minecraft:witch",
    "minecraft:ravager",
    "minecraft:pillager",
    "minecraft:illusioner",
    "minecraft:evoker",
    "minecraft:vindicator",
];

/// `V3807.flattenBlockPos(data, path)`: if `data[path]` is a `{X,Y,Z}` compound
/// with all three numbers present, replace it with an `int[3]` `[x,y,z]`.
fn flatten_block_pos(data: &mut NbtMap, path: &str) {
    let pos = match data.get_map(path) {
        Some(p) => p,
        None => return,
    };
    let (x, y, z) = match (pos.get_i32("X"), pos.get_i32("Y"), pos.get_i32("Z")) {
        (Some(x), Some(y), Some(z)) => (x, y, z),
        _ => return,
    };
    data.set_generic(path, NbtValue::IntArray(vec![x, y, z]));
}

/// `flattenBlockPos(data, from); RenameHelper.renameSingle(data, from, to)`.
fn flatten_and_rename(data: &mut NbtMap, from: &str, to: &str) {
    flatten_block_pos(data, from);
    data.rename_key(from, to);
}

/// Inverse of `flatten_block_pos`: if `data[path]` is an `int[3]` `[x,y,z]`,
/// replace it with a `{X,Y,Z}` compound. Lossless — the array uniquely encodes
/// the three ints the old `{X,Y,Z}` compound held.
fn unflatten_block_pos(data: &mut NbtMap, path: &str) {
    let (x, y, z) = match data.get(path) {
        Some(NbtValue::IntArray(v)) if v.len() == 3 => (v[0], v[1], v[2]),
        _ => return,
    };
    let mut pos = NbtMap::new();
    pos.set_i32("X", x);
    pos.set_i32("Y", y);
    pos.set_i32("Z", z);
    data.set_map(path, pos);
}

/// Inverse of `flatten_and_rename`: rename `to` -> `from`, then unflatten the
/// `int[3]` back into a `{X,Y,Z}` compound. Bucket B (lossless structural).
fn rename_and_unflatten(data: &mut NbtMap, from: &str, to: &str) {
    data.rename_key(to, from);
    unflatten_block_pos(data, from);
}

pub fn register(reg: &mut RegistryBuilder) {
    // ENTITY per-id position renames.
    reg.entity.add_converter_for_id(
        "minecraft:bee",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            flatten_and_rename(data, "HivePos", "hive_pos");
            flatten_and_rename(data, "FlowerPos", "flower_pos");
        }),
    );
    // Reverse (lossless, bucket B): hive_pos -> HivePos, flower_pos -> FlowerPos.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:bee",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            rename_and_unflatten(data, "HivePos", "hive_pos");
            rename_and_unflatten(data, "FlowerPos", "flower_pos");
        }),
    );
    reg.entity.add_converter_for_id(
        "minecraft:end_crystal",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            flatten_and_rename(data, "BeamTarget", "beam_target");
        }),
    );
    // Reverse (lossless, bucket B): beam_target -> BeamTarget.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:end_crystal",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            rename_and_unflatten(data, "BeamTarget", "beam_target");
        }),
    );
    reg.entity.add_converter_for_id(
        "minecraft:wandering_trader",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            flatten_and_rename(data, "WanderTarget", "wander_target");
        }),
    );
    // Reverse (lossless, bucket B): wander_target -> WanderTarget.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:wandering_trader",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            rename_and_unflatten(data, "WanderTarget", "wander_target");
        }),
    );
    for id in PATROLLING_MOBS {
        reg.entity.add_converter_for_id(
            *id,
            VERSION,
            0,
            Box::new(|data: &mut NbtMap, _from, _to| {
                flatten_and_rename(data, "PatrolTarget", "patrol_target");
            }),
        );
        // Reverse (lossless, bucket B): patrol_target -> PatrolTarget.
        reg.entity.add_reverse_converter_for_id(
            *id,
            VERSION,
            0,
            Box::new(|data: &mut NbtMap, _from, _to| {
                rename_and_unflatten(data, "PatrolTarget", "patrol_target");
            }),
        );
    }

    // ENTITY structure converter (all entities): Leash -> leash.
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            flatten_and_rename(data, "Leash", "leash");
        }),
    );
    // Reverse (lossless, bucket B): leash -> Leash.
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            rename_and_unflatten(data, "Leash", "leash");
        }),
    );

    // TILE_ENTITY per-id position renames.
    reg.tile_entity.add_converter_for_id(
        "minecraft:beehive",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            flatten_and_rename(data, "FlowerPos", "flower_pos");
        }),
    );
    // Reverse (lossless, bucket B): flower_pos -> FlowerPos.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:beehive",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            rename_and_unflatten(data, "FlowerPos", "flower_pos");
        }),
    );
    reg.tile_entity.add_converter_for_id(
        "minecraft:end_gateway",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            flatten_and_rename(data, "ExitPortal", "exit_portal");
        }),
    );
    // Reverse (lossless, bucket B): exit_portal -> ExitPortal.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:end_gateway",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            rename_and_unflatten(data, "ExitPortal", "exit_portal");
        }),
    );

    // ITEM_STACK compass: flatten tag.LodestonePos in place (no rename).
    reg.item_stack.add_converter_for_id(
        "minecraft:compass",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(tag) = data.get_map_mut("tag") {
                flatten_block_pos(tag, "LodestonePos");
            }
        }),
    );
    // Reverse (lossless, bucket B): unflatten tag.LodestonePos int[3] back into
    // the {X,Y,Z} compound (no rename, mirroring the forward).
    reg.item_stack.add_reverse_converter_for_id(
        "minecraft:compass",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(tag) = data.get_map_mut("tag") {
                unflatten_block_pos(tag, "LodestonePos");
            }
        }),
    );
}
