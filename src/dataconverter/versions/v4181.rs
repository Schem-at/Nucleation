//! V4181 (1.21.4-pre1 + 2) — schematic-relevant subset of `V4181.java`.
//!
//! VERSION = MCVersions.V1_21_4_PRE1 (4179) + 2 = 4181.
//!
//! Ported (TILE_ENTITY per-id converter, V4181.java:152-171): furnace-family
//! field renames, applied to `minecraft:furnace`, `minecraft:smoker`, and
//! `minecraft:blast_furnace`:
//!   * `CookTime`      -> `cooking_time_spent`
//!   * `CookTimeTotal` -> `cooking_total_time`
//!   * `BurnTime`      -> `lit_time_remaining`
//! then duplicate the (renamed) `lit_time_remaining` value into `lit_total_time`,
//! preserving the original tag type (Java's `getGeneric`/`setGeneric`).
//!
//! Nothing non-schematic is present in this version.

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4181;

/// The shared furnace-family converter (V4181.java:153-167).
fn convert_furnace(data: &mut NbtMap) {
    data.rename_key("CookTime", "cooking_time_spent");
    data.rename_key("CookTimeTotal", "cooking_total_time");
    data.rename_key("BurnTime", "lit_time_remaining");

    // Duplicate lit_time_remaining -> lit_total_time, preserving the tag type.
    if let Some(lit_time_remaining) = data.get("lit_time_remaining").cloned() {
        data.set_generic("lit_total_time", lit_time_remaining);
    }
}

/// Inverse of `convert_furnace`. Lossless (bucket A renames + bucket D additive):
///   * drop `lit_total_time` — the forward created it solely as a copy of
///     `lit_time_remaining`/`BurnTime`; the pre-V4181 furnace schema never had
///     it, so removing it is exact (no surviving user data is lost).
///   * `lit_time_remaining`  -> `BurnTime`
///   * `cooking_total_time`  -> `CookTimeTotal`
///   * `cooking_time_spent`  -> `CookTime`
/// Each modern field maps uniquely back to its old name, so no loss is reported.
fn revert_furnace(data: &mut NbtMap) {
    if let Some(lit_total_time) = data.get("lit_total_time") {
        if data
            .get("lit_time_remaining")
            .map(|remaining| remaining != lit_total_time)
            .unwrap_or(true)
        {
            report_loss(
                VERSION,
                LossKind::FingerprintCollapse,
                Severity::Loss,
                "furnace lit_total_time differed from lit_time_remaining; legacy BurnTime can store only one value",
            );
        }
    }
    data.take("lit_total_time");

    data.rename_key("lit_time_remaining", "BurnTime");
    data.rename_key("cooking_total_time", "CookTimeTotal");
    data.rename_key("cooking_time_spent", "CookTime");
}

pub fn register(reg: &mut RegistryBuilder) {
    for id in [
        "minecraft:furnace",
        "minecraft:smoker",
        "minecraft:blast_furnace",
    ] {
        reg.tile_entity.add_converter_for_id(
            id,
            VERSION,
            0,
            Box::new(|data: &mut NbtMap, _from, _to| convert_furnace(data)),
        );
        reg.tile_entity.add_reverse_converter_for_id(
            id,
            VERSION,
            0,
            Box::new(|data: &mut NbtMap, _from, _to| revert_furnace(data)),
        );
    }
}
