//! V4314 (25w06a + 1) — schematic-relevant subset of `V4314.java`.
//!
//! ENTITY converters that collapse split `X/Y/Z` block-position fields into a
//! single int-array field, plus a few field renames:
//!   * all living entities: `SleepingX/Y/Z` -> `sleeping_pos` (V4314.java:933-941,
//!     also registered on PLAYER which we skip).
//!   * `minecraft:vex`: `BoundX/Y/Z` -> `bound_pos`, `LifeTicks` -> `life_ticks`.
//!   * `minecraft:phantom`: `AX/AY/AZ` -> `anchor_pos`, `Size` -> `size`.
//!   * `minecraft:turtle`: drop `TravelPosX/Y/Z`, `HomePosX/Y/Z` -> `home_pos`,
//!     `HasEgg` -> `has_egg`.
//!   * attached-block entities (item_frame / glow_item_frame / painting /
//!     leash_knot): `TileX/Y/Z` -> `block_pos`.
//!
//! Skipped (non-schematic): both PLAYER structure converters — the
//! `livingEntityConverter` registered on PLAYER, and the spawn/respawn +
//! `enteredNetherPosition` migration (V4314.java:941-978) — PLAYER never appears
//! in a schematic file.
//!
//! `convertBlockPosition` is V4309's shared helper (V4309.java:704-723), inlined
//! here because V4309 itself only touches non-schematic SAVED_DATA types and so
//! is not ported.
//!
//! VERSION = V25W06A(4313) + 1.

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4314;

/// `V4309.makeBlockPosition` + `convertBlockPosition` (V4309.java:704-723): read
/// three numeric coords, remove them, and write a 3-int array at `to_path`. A
/// missing coord aborts (leaving the originals in place).
fn convert_block_position(data: &mut NbtMap, x_path: &str, y_path: &str, z_path: &str, to_path: &str) {
    let x = match data.get_i32(x_path) {
        Some(v) => v,
        None => return,
    };
    let y = match data.get_i32(y_path) {
        Some(v) => v,
        None => return,
    };
    let z = match data.get_i32(z_path) {
        Some(v) => v,
        None => return,
    };
    data.take(x_path);
    data.take(y_path);
    data.take(z_path);
    data.set_generic(to_path, NbtValue::IntArray(vec![x, y, z]));
}

/// Inverse of `convert_block_position`: if `data[to_path]` is an `int[3]`
/// `[x,y,z]`, remove it and write the three split `x_path/y_path/z_path` ints.
/// Lossless (bucket B) — the array uniquely encodes the three ints the old split
/// fields held; a missing/wrong-length array aborts (leaving data untouched).
fn unconvert_block_position(data: &mut NbtMap, x_path: &str, y_path: &str, z_path: &str, to_path: &str) {
    let (x, y, z) = match data.get(to_path) {
        Some(NbtValue::IntArray(v)) if v.len() == 3 => (v[0], v[1], v[2]),
        _ => return,
    };
    data.take(to_path);
    data.set_i32(x_path, x);
    data.set_i32(y_path, y);
    data.set_i32(z_path, z);
}

pub fn register(reg: &mut RegistryBuilder) {
    // Living-entity sleeping position (registered on every ENTITY).
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            convert_block_position(data, "SleepingX", "SleepingY", "SleepingZ", "sleeping_pos");
        }),
    );
    // Reverse (lossless, bucket B): sleeping_pos int[3] -> SleepingX/Y/Z.
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            unconvert_block_position(data, "SleepingX", "SleepingY", "SleepingZ", "sleeping_pos");
        }),
    );

    reg.entity.add_converter_for_id(
        "minecraft:vex",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            convert_block_position(data, "BoundX", "BoundY", "BoundZ", "bound_pos");
            data.rename_key("LifeTicks", "life_ticks");
        }),
    );
    // Reverse (lossless, bucket B): life_ticks -> LifeTicks, bound_pos int[3] ->
    // BoundX/Y/Z. (Both field renames here are inline closure renames, not
    // map_renamer registrations, so they need explicit inversion.)
    reg.entity.add_reverse_converter_for_id(
        "minecraft:vex",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.rename_key("life_ticks", "LifeTicks");
            unconvert_block_position(data, "BoundX", "BoundY", "BoundZ", "bound_pos");
        }),
    );
    reg.entity.add_converter_for_id(
        "minecraft:phantom",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            convert_block_position(data, "AX", "AY", "AZ", "anchor_pos");
            data.rename_key("Size", "size");
        }),
    );
    // Reverse (lossless, bucket B): size -> Size, anchor_pos int[3] -> AX/AY/AZ.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:phantom",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.rename_key("size", "Size");
            unconvert_block_position(data, "AX", "AY", "AZ", "anchor_pos");
        }),
    );
    reg.entity.add_converter_for_id(
        "minecraft:turtle",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.take("TravelPosX");
            data.take("TravelPosY");
            data.take("TravelPosZ");
            convert_block_position(data, "HomePosX", "HomePosY", "HomePosZ", "home_pos");
            data.rename_key("HasEgg", "has_egg");
        }),
    );
    // Reverse: has_egg -> HasEgg (lossless), home_pos int[3] -> HomePosX/Y/Z
    // (lossless). TravelPosX/Y/Z (the turtle's travel target) was DROPPED by the
    // forward with no modern representation, so it cannot be restored — bucket C,
    // report_loss. The old turtle defaults a missing travel target, so we add
    // nothing back.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:turtle",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.rename_key("has_egg", "HasEgg");
            unconvert_block_position(data, "HomePosX", "HomePosY", "HomePosZ", "home_pos");
            report_loss(
                VERSION,
                LossKind::UnsupportedInTarget,
                Severity::Loss,
                "turtle TravelPosX/Y/Z was dropped on upgrade and cannot be restored",
            );
        }),
    );

    // Attached-block entities: TileX/Y/Z -> block_pos.
    for id in ["minecraft:item_frame", "minecraft:glow_item_frame", "minecraft:painting", "minecraft:leash_knot"] {
        reg.entity.add_converter_for_id(
            id,
            VERSION,
            0,
            Box::new(|data, _from, _to| {
                convert_block_position(data, "TileX", "TileY", "TileZ", "block_pos");
            }),
        );
        // Reverse (lossless, bucket B): block_pos int[3] -> TileX/Y/Z.
        reg.entity.add_reverse_converter_for_id(
            id,
            VERSION,
            0,
            Box::new(|data, _from, _to| {
                unconvert_block_position(data, "TileX", "TileY", "TileZ", "block_pos");
            }),
        );
    }
}
