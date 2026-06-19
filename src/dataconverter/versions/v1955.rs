//! V1955 (1.14.1-pre1) — schematic-relevant subset of `V1955.java`.
//!
//! Ported: the ENTITY converters for `minecraft:villager` and
//! `minecraft:zombie_villager` that synthesize a profession `level`
//! (VillagerData.level) from the trade count and an `Xp` value from the level
//! (V1955.java:42-92).
//!
//! VERSION = MCVersions.V1_14_1_PRE1 = 1955.
//!
//! Implementation note: there are no rename helpers for this — it is bespoke
//! field synthesis, so it is implemented inline with NbtMap/MapExt. `Mth.clamp`
//! is an ordinary integer clamp. Java's `hasKey("Xp", NUMBER)` is matched by
//! testing whether `Xp` reads as a number (get_i64 returns Some); `getInt`
//! defaults to 0 when absent, which we mirror with `unwrap_or(0)` (and
//! `getInt(key, def)` with `unwrap_or(def)`).
//!
//! Nothing non-schematic is present in this version.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 1955;

/// V1955.java:18-25: minimum xp for a level, clamped to the threshold table.
const LEVEL_XP_THRESHOLDS: [i32; 5] = [0, 10, 50, 100, 150];

fn clamp_i32(value: i32, min: i32, max: i32) -> i32 {
    value.max(min).min(max)
}

fn get_min_xp_per_level(level: i32) -> i32 {
    let idx = clamp_i32(level - 1, 0, (LEVEL_XP_THRESHOLDS.len() as i32) - 1) as usize;
    LEVEL_XP_THRESHOLDS[idx]
}

fn add_level(data: &mut NbtMap, level: i32) {
    if data.get_map("VillagerData").is_none() {
        data.set_map("VillagerData", NbtMap::new());
    }
    if let Some(villager_data) = data.get_map_mut("VillagerData") {
        villager_data.set_i32("level", level);
    }
}

fn add_xp_from_level(data: &mut NbtMap, level: i32) {
    data.set_i32("Xp", get_min_xp_per_level(level));
}

pub fn register(reg: &mut RegistryBuilder) {
    // minecraft:villager (V1955.java:42-67).
    reg.entity.add_converter_for_id(
        "minecraft:villager",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // getInt("level") defaults to 0 when VillagerData or level absent.
            let mut level = data
                .get_map("VillagerData")
                .and_then(|vd| vd.get_i32("level"))
                .unwrap_or(0);

            if level == 0 || level == 1 {
                // count recipes in Offers.Recipes (list of maps).
                let recipe_count = data
                    .get_map("Offers")
                    .and_then(|offers| offers.get_list("Recipes"))
                    .map(|recipes| recipes.len() as i32)
                    .unwrap_or(0);

                level = clamp_i32(recipe_count / 2, 1, 5);
                if level > 1 {
                    add_level(data, level);
                }
            }

            // hasKey("Xp", NUMBER): only add when Xp is absent or not a number.
            let has_xp = data.get("Xp").and_then(ValueExt::as_number_i64).is_some();
            if !has_xp {
                add_xp_from_level(data, level);
            }
        }),
    );

    // No reverse converter for minecraft:villager (identity inverse, rule 10).
    //
    // The forward (V1955.java:42-67) is a pure additive backfill: it only ever
    // *adds* `Xp` (and, when the computed level > 1, `VillagerData.level`) when
    // those fields are missing — it never rewrites existing values. Both `Xp`
    // and `VillagerData.level` are perfectly valid fields in the pre-1955
    // (1.14.1-pre1) villager schema, so leaving them in place on downgrade is
    // correct. The synthesized values are *derived* (from recipe count / level),
    // not a fixed default, and the new format always carries `Xp`, so we cannot
    // tell a synthesized `Xp` from a genuine one. Removing it would therefore
    // drop legitimate data with no benefit (bucket D where removal is unsafe),
    // and keeping it is harmless. Hence: identity inverse, no reverse converter,
    // no loss report.

    // minecraft:zombie_villager (V1955.java:69-89).
    reg.entity.add_converter_for_id(
        "minecraft:zombie_villager",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // getNumber("Xp") == null -> Xp absent or not a number.
            let has_xp = data.get("Xp").and_then(ValueExt::as_number_i64).is_some();
            if !has_xp {
                // getInt("level", 1) defaults to 1 when VillagerData absent;
                // when VillagerData is present, getInt("level") (no default) is 1.
                let level = match data.get_map("VillagerData") {
                    None => 1,
                    Some(vd) => vd.get_i32("level").unwrap_or(1),
                };
                data.set_i32("Xp", get_min_xp_per_level(level));
            }
        }),
    );

    // No reverse converter for minecraft:zombie_villager (identity inverse,
    // rule 10). Same reasoning as the villager case above: the forward
    // (V1955.java:69-89) only backfills `Xp` (derived from level, default level
    // 1) when it is absent, never touching an existing value. `Xp` is a valid
    // pre-1955 zombie-villager field, the new format always carries it, and a
    // synthesized `Xp` is indistinguishable from a real one. Keeping it on
    // downgrade is harmless; removing it would be lossy with no upside. Identity
    // inverse — nothing to register, no loss report.
}
