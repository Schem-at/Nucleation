//! V2838 (21w40a) — schematic-relevant subset of `V2838.java`.
//!
//! Ported: the BIOME value-type rename (`ConverterAbstractStringValueTypeRename`
//! on BIOME) that collapses the pre-1.18 biome ids into their 1.18 successors
//! (e.g. `minecraft:snowy_tundra` -> `minecraft:snowy_plains`). BIOME ids appear
//! in schematic biome palettes, so this is kept.
//!
//! Note: `BIOME_UPDATE` is *not* injective — several old biomes collapse onto the
//! same new biome (e.g. both `desert_hills` and `desert_lakes` -> `desert`). The
//! forward rename is lossy and cannot be auto-inverted; that is handled by the
//! reverse engine, not here. `register_value_rename`/`map_renamer` only use the
//! forward direction, so the non-injective table is fine for this registration.
//!
//! VERSION = MCVersions.V21W40A = 2838.

use std::sync::Arc;

use super::super::helpers::{map_renamer, register_value_rename, RenameSpec};
use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 2838;

/// `(old, new)` 1.18 biome renames (V2838.java:14-55).
pub const BIOME_UPDATE: &[(&str, &str)] = &[
    ("minecraft:badlands_plateau", "minecraft:badlands"),
    ("minecraft:bamboo_jungle_hills", "minecraft:bamboo_jungle"),
    ("minecraft:birch_forest_hills", "minecraft:birch_forest"),
    ("minecraft:dark_forest_hills", "minecraft:dark_forest"),
    ("minecraft:desert_hills", "minecraft:desert"),
    ("minecraft:desert_lakes", "minecraft:desert"),
    (
        "minecraft:giant_spruce_taiga_hills",
        "minecraft:old_growth_spruce_taiga",
    ),
    (
        "minecraft:giant_spruce_taiga",
        "minecraft:old_growth_spruce_taiga",
    ),
    (
        "minecraft:giant_tree_taiga_hills",
        "minecraft:old_growth_pine_taiga",
    ),
    (
        "minecraft:giant_tree_taiga",
        "minecraft:old_growth_pine_taiga",
    ),
    (
        "minecraft:gravelly_mountains",
        "minecraft:windswept_gravelly_hills",
    ),
    ("minecraft:jungle_edge", "minecraft:sparse_jungle"),
    ("minecraft:jungle_hills", "minecraft:jungle"),
    ("minecraft:modified_badlands_plateau", "minecraft:badlands"),
    (
        "minecraft:modified_gravelly_mountains",
        "minecraft:windswept_gravelly_hills",
    ),
    ("minecraft:modified_jungle_edge", "minecraft:sparse_jungle"),
    ("minecraft:modified_jungle", "minecraft:jungle"),
    (
        "minecraft:modified_wooded_badlands_plateau",
        "minecraft:wooded_badlands",
    ),
    ("minecraft:mountain_edge", "minecraft:windswept_hills"),
    ("minecraft:mountains", "minecraft:windswept_hills"),
    (
        "minecraft:mushroom_field_shore",
        "minecraft:mushroom_fields",
    ),
    ("minecraft:shattered_savanna", "minecraft:windswept_savanna"),
    (
        "minecraft:shattered_savanna_plateau",
        "minecraft:windswept_savanna",
    ),
    ("minecraft:snowy_mountains", "minecraft:snowy_plains"),
    ("minecraft:snowy_taiga_hills", "minecraft:snowy_taiga"),
    ("minecraft:snowy_taiga_mountains", "minecraft:snowy_taiga"),
    ("minecraft:snowy_tundra", "minecraft:snowy_plains"),
    ("minecraft:stone_shore", "minecraft:stony_shore"),
    ("minecraft:swamp_hills", "minecraft:swamp"),
    ("minecraft:taiga_hills", "minecraft:taiga"),
    ("minecraft:taiga_mountains", "minecraft:taiga"),
    (
        "minecraft:tall_birch_forest",
        "minecraft:old_growth_birch_forest",
    ),
    (
        "minecraft:tall_birch_hills",
        "minecraft:old_growth_birch_forest",
    ),
    (
        "minecraft:wooded_badlands_plateau",
        "minecraft:wooded_badlands",
    ),
    ("minecraft:wooded_hills", "minecraft:forest"),
    ("minecraft:wooded_mountains", "minecraft:windswept_forest"),
    ("minecraft:lofty_peaks", "minecraft:jagged_peaks"),
    ("minecraft:snowcapped_peaks", "minecraft:frozen_peaks"),
];

fn reverse_biome_update(biome: &str) -> Option<(&'static str, bool)> {
    let mut first = None;
    let mut count = 0;
    for (old, new) in BIOME_UPDATE {
        if *new == biome {
            if first.is_none() {
                first = Some(*old);
            }
            count += 1;
        }
    }
    first.map(|old| (old, count > 1))
}

pub fn register(reg: &mut RegistryBuilder) {
    let forward = map_renamer(BIOME_UPDATE).forward();
    let reverse = Arc::new(|id: &str| {
        reverse_biome_update(id).map(|(old, ambiguous)| {
            if ambiguous {
                report_loss(
                    VERSION,
                    LossKind::RenameAmbiguous,
                    Severity::Approximated,
                    format!(
                        "biome `{id}` has multiple pre-1.18 biome preimages; restored canonical `{old}`"
                    ),
                );
            }
            old.to_string()
        })
    });
    register_value_rename(
        &mut reg.biome,
        VERSION,
        0,
        RenameSpec::custom(forward, reverse),
    );
}

#[cfg(test)]
mod tests {
    use crate::dataconverter::loss;
    use crate::dataconverter::registry::registry;
    use crate::dataconverter::version::{encode_versions, MAX_STEP};
    use crate::nbt::NbtValue;

    #[test]
    fn reverse_reports_ambiguous_biome_collapse() {
        let mut biome = NbtValue::String("minecraft:desert".to_string());
        let reg = registry();

        let (_, report) = loss::run_reverse(|| {
            reg.biome.convert(
                &mut biome,
                encode_versions(2838, MAX_STEP),
                encode_versions(2837, MAX_STEP),
            );
        });

        assert_eq!(
            biome,
            NbtValue::String("minecraft:desert_hills".to_string())
        );
        assert_eq!(report.loss_count(), 0);
        assert_eq!(report.len(), 1);
        assert!(report.summary().contains("multiple pre-1.18"));
    }
}
