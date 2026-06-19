//! V4187 (1.21.4-rc2 + 1) — schematic-relevant subset of `V4187.java`.
//!
//! VERSION = MCVersions.V1_21_4_RC2 (4186) + 1 = 4187.
//!
//! Ported (ENTITY per-id attribute-base updaters, V4187.java:207-262): adjust the
//! `minecraft:follow_range` attribute base value of several mobs, matching
//! `ConverterEntityAttributesBaseValueUpdater`. For each listed entity id, the
//! converter scans the `attributes` list, finds the modifier whose
//! namespace-corrected `id` is `minecraft:follow_range`, and rewrites its `base`:
//!   * villager / bee / allay / llama : 48.0 -> 16.0
//!   * piglin_brute                   : 16.0 -> 12.0
//!   * warden                         : 16.0 -> 24.0
//!
//! Nothing non-schematic is present in this version.

use crate::nbt::NbtMap;

use super::super::helpers::correct_namespace_or_null;
use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 4187;

/// `ConverterEntityAttributesBaseValueUpdater.convert` (restricted to the
/// `attributes` list form used in 1.21.x): for each attribute modifier whose
/// namespace-corrected `id` equals `attribute_id`, apply `updater` to `base`.
fn update_attribute_base(data: &mut NbtMap, attribute_id: &str, updater: fn(f64) -> f64) {
    let modifiers = match data.get_list_mut("attributes") {
        Some(l) => l,
        None => return,
    };

    for modifier in modifiers.iter_mut() {
        let modifier = match modifier.as_compound_mut() {
            Some(m) => m,
            None => continue,
        };

        // NamespaceUtil.correctNamespace(modifier.getString("id", "")).
        let id = modifier.get_string("id").unwrap_or("");
        let corrected = correct_namespace_or_null(id).unwrap_or_else(|| id.to_string());
        if corrected != attribute_id {
            continue;
        }

        let base = modifier.get_f64("base").unwrap_or(0.0);
        modifier.set_f64("base", updater(base));
    }
}

fn reverse_update_attribute_base(
    data: &mut NbtMap,
    attribute_id: &str,
    from_base: f64,
    to_base: f64,
) {
    let modifiers = match data.get_list_mut("attributes") {
        Some(l) => l,
        None => return,
    };

    for modifier in modifiers.iter_mut() {
        let modifier = match modifier.as_compound_mut() {
            Some(m) => m,
            None => continue,
        };

        let id = modifier.get_string("id").unwrap_or("");
        let corrected = correct_namespace_or_null(id).unwrap_or_else(|| id.to_string());
        if corrected != attribute_id {
            continue;
        }

        let base = modifier.get_f64("base").unwrap_or(0.0);
        if base == from_base {
            modifier.set_f64("base", to_base);
            report_loss(
                VERSION,
                LossKind::FingerprintCollapse,
                Severity::Approximated,
                format!(
                    "follow_range base {from_base} may be a modern authored value or a V4187 default rewrite; restored legacy default {to_base}"
                ),
            );
        }
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    fn follow_range_48_to_16(curr: f64) -> f64 {
        if curr == 48.0 {
            16.0
        } else {
            curr
        }
    }

    for id in [
        "minecraft:villager",
        "minecraft:bee",
        "minecraft:allay",
        "minecraft:llama",
    ] {
        reg.entity.add_converter_for_id(
            id,
            VERSION,
            0,
            Box::new(|data: &mut NbtMap, _from, _to| {
                update_attribute_base(data, "minecraft:follow_range", follow_range_48_to_16);
            }),
        );
        reg.entity.add_reverse_converter_for_id(
            id,
            VERSION,
            0,
            Box::new(|data: &mut NbtMap, _from, _to| {
                reverse_update_attribute_base(data, "minecraft:follow_range", 16.0, 48.0);
            }),
        );
    }

    reg.entity.add_converter_for_id(
        "minecraft:piglin_brute",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            update_attribute_base(data, "minecraft:follow_range", |curr| {
                if curr == 16.0 {
                    12.0
                } else {
                    curr
                }
            });
        }),
    );
    // Reverse of piglin_brute: forward mapped 16.0 -> 12.0, so undo 12.0 -> 16.0.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:piglin_brute",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            reverse_update_attribute_base(data, "minecraft:follow_range", 12.0, 16.0);
        }),
    );

    reg.entity.add_converter_for_id(
        "minecraft:warden",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            update_attribute_base(data, "minecraft:follow_range", |curr| {
                if curr == 16.0 {
                    24.0
                } else {
                    curr
                }
            });
        }),
    );
    // Reverse of warden: forward mapped 16.0 -> 24.0, so undo 24.0 -> 16.0.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:warden",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            reverse_update_attribute_base(data, "minecraft:follow_range", 24.0, 16.0);
        }),
    );
}
