//! V3093 (22w17a) — schematic-relevant subset of `V3093.java`.
//!
//! Kept: the ENTITY `minecraft:goat` converter, which sets `HasLeftHorn` and
//! `HasRightHorn` to `true` (goats gained two-horn state in 22w17a).
//!
//! VERSION = MCVersions.V22W17A = 3093.

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 3093;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:goat",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.set_bool("HasLeftHorn", true);
            data.set_bool("HasRightHorn", true);
        }),
    );

    // Reverse: bucket D (additive default). 22w17a added the two-horn booleans;
    // pre-3093 goats had no `HasLeftHorn`/`HasRightHorn`. The forward sets both to
    // `true` unconditionally, so drop them when they equal that default — lossless
    // for genuine downgrades (the old format never carried these fields).
    reg.entity.add_reverse_converter_for_id(
        "minecraft:goat",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            for key in ["HasLeftHorn", "HasRightHorn"] {
                match data.get_bool(key) {
                    Some(true) => {
                        data.take(key);
                    }
                    Some(false) => {
                        data.take(key);
                        report_loss(
                            VERSION,
                            LossKind::UnsupportedInTarget,
                            Severity::Loss,
                            format!("goat {key}=false has no representation before 3093; horn state dropped"),
                        );
                    }
                    None if data.has_key(key) => {
                        data.take(key);
                        report_loss(
                            VERSION,
                            LossKind::UnsupportedInTarget,
                            Severity::Loss,
                            format!("goat {key} is not a boolean and has no representation before 3093; dropped"),
                        );
                    }
                    None => {}
                }
            }
        }),
    );
}
