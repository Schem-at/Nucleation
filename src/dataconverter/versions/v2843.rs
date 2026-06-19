//! V2843 (21w42a + 3) — schematic-relevant subset of `V2843.java`.
//!
//! VERSION = MCVersions.V21W42A + 3 = 2840 + 3 = 2843.
//!
//! Kept:
//!   * BIOME value rename `minecraft:deep_warm_ocean` -> `minecraft:warm_ocean`
//!     (V2843.java:20-24). This is the `ConverterAbstractStringValueTypeRename`
//!     registered on `MCTypeRegistry.BIOME`; BIOME is a schematic-relevant value
//!     type (it appears in chunk-section biome palettes carried by structures).
//!
//! Skipped (non-schematic): the CHUNK structure converter that moves
//! out-of-bound block/fluid ticks into `UpgradeData` (V2843.java:26-67), and the
//! CHUNK structure walker (V2843.java:70-97) — both target CHUNK, which never
//! appears in a schematic file. (The walker's only sub-targets here are CHUNK's
//! own entities/block_entities/sections/structures, all reached via CHUNK.)

use std::sync::Arc;

use super::super::helpers::{map_renamer, register_value_rename, RenameSpec};
use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 2843;

/// `(old, new)` BIOME renames (V2843.java:20-24). The explicit table is
/// one-entry, but `minecraft:warm_ocean` itself was also a valid old biome, so
/// reversing `warm_ocean` to `deep_warm_ocean` is an approximation.
const BIOME_RENAMES: &[(&str, &str)] = &[("minecraft:deep_warm_ocean", "minecraft:warm_ocean")];

pub fn register(reg: &mut RegistryBuilder) {
    let forward = map_renamer(BIOME_RENAMES).forward();
    let reverse = Arc::new(|id: &str| {
        if id == "minecraft:warm_ocean" {
            report_loss(
                VERSION,
                LossKind::RenameAmbiguous,
                Severity::Approximated,
                "biome `minecraft:warm_ocean` may have been legacy warm_ocean or deep_warm_ocean; restored deep_warm_ocean",
            );
            Some("minecraft:deep_warm_ocean".to_string())
        } else {
            None
        }
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
    fn reverse_reports_warm_ocean_ambiguous_preimage() {
        let mut biome = NbtValue::String("minecraft:warm_ocean".to_string());
        let reg = registry();

        let (_, report) = loss::run_reverse(|| {
            reg.biome.convert(
                &mut biome,
                encode_versions(2843, MAX_STEP),
                encode_versions(2842, MAX_STEP),
            );
        });

        assert_eq!(
            biome,
            NbtValue::String("minecraft:deep_warm_ocean".to_string())
        );
        assert_eq!(report.loss_count(), 0);
        assert_eq!(report.len(), 1);
        assert!(report.summary().contains("deep_warm_ocean"));
    }
}
