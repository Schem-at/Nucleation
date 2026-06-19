//! V1475 (18w10b+1) — block rename only; cites `V1475.java`.
//!
//! Ported:
//!   * BLOCK rename (`ConverterAbstractBlockRename.register`, V1475.java:13-18):
//!     `minecraft:flowing_water` -> `minecraft:water` and
//!     `minecraft:flowing_lava` -> `minecraft:lava`. The block-rename helper
//!     registers on BLOCK_NAME + BLOCK_STATE(Name) + FLAT_BLOCK_STATE.
//!
//! VERSION = MCVersions.V18W10B + 1 = 1474 + 1 = 1475.
//!
//! Nothing else is present in `V1475.java`.

use std::sync::Arc;

use super::super::helpers::RenameSpec;
use super::super::helpers::{map_renamer, register_block_rename};
use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 1475;

/// Renamed blocks (V1475.java:13-18).
const RENAMED_BLOCKS: &[(&str, &str)] = &[
    ("minecraft:flowing_water", "minecraft:water"),
    ("minecraft:flowing_lava", "minecraft:lava"),
];

pub fn register(reg: &mut RegistryBuilder) {
    let forward = map_renamer(RENAMED_BLOCKS).forward();
    let reverse = Arc::new(|id: &str| match id {
        "minecraft:water" => {
            report_loss(
                VERSION,
                LossKind::RenameAmbiguous,
                Severity::Approximated,
                "minecraft:water may have been legacy water or flowing_water; restoring flowing_water",
            );
            Some("minecraft:flowing_water".to_string())
        }
        "minecraft:lava" => {
            report_loss(
                VERSION,
                LossKind::RenameAmbiguous,
                Severity::Approximated,
                "minecraft:lava may have been legacy lava or flowing_lava; restoring flowing_lava",
            );
            Some("minecraft:flowing_lava".to_string())
        }
        _ => None,
    });
    register_block_rename(reg, VERSION, RenameSpec::custom(forward, reverse));
}
