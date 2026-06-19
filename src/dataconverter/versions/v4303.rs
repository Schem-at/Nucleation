//! V4303 (25w02a + 5) — schematic-relevant subset of `V4303.java`.
//!
//! ENTITY structure converter: the legacy float `FallDistance` becomes a double
//! `fall_distance` (V4303.java:363-378). Java reads `getFloat("FallDistance",
//! 0.0f)` then widens to double.
//!
//! Skipped (non-schematic): the same converter is also registered on PLAYER
//! (V4303.java:379), which never appears in a schematic file.
//!
//! VERSION = V25W02A(4298) + 5.

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4303;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if !data.has_key("FallDistance") {
                return;
            }
            // getFloat(.., 0.0f): a non-float value reads as 0.0.
            let fall_distance = data.get_f64("FallDistance").unwrap_or(0.0);
            data.take("FallDistance");
            data.set_f64("fall_distance", fall_distance);
        }),
    );

    // Reverse: lossless structural rename (bucket B). The double `fall_distance`
    // uniquely determines the legacy float `FallDistance`; narrowing back to f32
    // is the exact inverse of the forward widening (the value originated as a
    // float). Mirrors V4303.java:363-378.
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if !data.has_key("fall_distance") {
                return;
            }
            let fall_distance = data.get_f64("fall_distance").unwrap_or(0.0);
            data.take("fall_distance");
            if (fall_distance as f32) as f64 != fall_distance {
                report_loss(
                    VERSION,
                    LossKind::FingerprintCollapse,
                    Severity::Approximated,
                    "fall_distance double cannot be represented exactly as legacy FallDistance float",
                );
            }
            data.set_f32("FallDistance", fall_distance as f32);
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::registry::convert_entity_reverse;

    #[test]
    fn reverse_precise_double_fall_distance_reports_float_narrowing() {
        let mut data = crate::nbt::NbtMap::new();
        data.set_string("id", "minecraft:pig");
        data.set_f64("fall_distance", 1.0000000000000002);

        let report = convert_entity_reverse(&mut data, VERSION, VERSION - 1);

        assert_eq!(report.len(), 1);
        assert_eq!(report.entries[0].kind, LossKind::FingerprintCollapse);
        assert!(data.has_key("FallDistance"));
    }
}
