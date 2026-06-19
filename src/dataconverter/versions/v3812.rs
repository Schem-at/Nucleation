//! V3812 (24w05b + 1) — schematic-relevant subset of `V3812.java`.
//!
//! The 1.20.5 wolf-health buff: a wolf whose `generic.max_health` attribute has
//! `Base == 20.0` gets it bumped to `40.0`, and (when that happened) its current
//! `Health` is doubled (V3812.java:14-40).
//!
//! Nothing non-schematic exists in this version file.
//!
//! The attribute `Name` is compared after `NamespaceUtil.correctNamespace`; the
//! engine does not export that fn, so the (namespace-defaulting) parse rule is
//! inlined.
//!
//! VERSION = MCVersions.V24W05B (3811) + 1 = 3812.

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 3812;

/// Port of `NamespaceUtil.correctNamespace`: default-namespace an unnamespaced,
/// parseable resource location; otherwise return the input unchanged.
fn correct_namespace(value: &str) -> String {
    if value.contains(':') {
        return value.to_string();
    }
    let valid = !value.is_empty()
        && value
            .bytes()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'-' | b'/'));
    if valid {
        format!("minecraft:{value}")
    } else {
        value.to_string()
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:wolf",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let mut double_health = false;

            if let Some(attributes) = data.get_list_mut("Attributes") {
                for el in attributes.iter_mut() {
                    let attribute = match el.as_compound_mut() {
                        Some(a) => a,
                        None => continue,
                    };

                    let name = attribute.get_string("Name").map(correct_namespace);
                    if name.as_deref() != Some("minecraft:generic.max_health") {
                        continue;
                    }

                    // getDouble("Base", 0.0) — default 0.0 when absent.
                    let base = attribute.get_f64("Base").unwrap_or(0.0);
                    if base == 20.0 {
                        attribute.set_f64("Base", 40.0);
                        double_health = true;
                    }
                }
            }

            if double_health {
                // getFloat("Health", 0.0) — default 0.0 when absent.
                let health = data.get_f64("Health").unwrap_or(0.0) as f32;
                data.set_f32("Health", health * 2.0);
            }
        }),
    );

    // Reverse of the 1.20.5 wolf-health buff (V3812.java:14-40). The forward
    // only doubled `generic.max_health` Base when it was exactly 20.0 (the old
    // wolf default), simultaneously doubling `Health`. So a modern wolf with
    // Base == 40.0 has the unambiguous old preimage Base == 20.0 with halved
    // Health — restoring the default the old format always carried (rule 11),
    // hence lossless, no report_loss.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:wolf",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let mut halve_health = false;

            if let Some(attributes) = data.get_list_mut("Attributes") {
                for el in attributes.iter_mut() {
                    let attribute = match el.as_compound_mut() {
                        Some(a) => a,
                        None => continue,
                    };

                    let name = attribute.get_string("Name").map(correct_namespace);
                    if name.as_deref() != Some("minecraft:generic.max_health") {
                        continue;
                    }

                    let base = attribute.get_f64("Base").unwrap_or(0.0);
                    if base == 40.0 {
                        report_loss(
                            VERSION,
                            LossKind::FingerprintCollapse,
                            Severity::Approximated,
                            "wolf max_health Base=40.0 may be a modern authored value; downgrading to the pre-3812 default Base=20.0",
                        );
                        attribute.set_f64("Base", 20.0);
                        halve_health = true;
                    }
                }
            }

            if halve_health {
                let health = data.get_f64("Health").unwrap_or(0.0) as f32;
                data.set_f32("Health", health / 2.0);
            }
        }),
    );
}
