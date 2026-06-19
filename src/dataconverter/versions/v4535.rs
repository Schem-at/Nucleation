//! V4535 (25w31a + 1) — schematic-relevant subset of
//! `DataConverterJava/.../versions/V4535.java`.
//!
//! ENTITY converter for `minecraft:copper_golem`: the integer `weather_state`
//! oxidation value is rewritten to a string enum
//! (1->"exposed", 2->"weathered", 3->"oxidized", default->"unaffected")
//! (V4535.java:13-36). Java reads `getInt("weather_state", 0)`, so an absent or
//! out-of-range value falls into the default ("unaffected"). Nothing
//! non-schematic is present in this version.
//!
//! VERSION = MCVersions.V25W31A (4534) + 1 = 4535.

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4535;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:copper_golem",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // getInt("weather_state", 0): default 0 maps to "unaffected".
            let state = match data.get_i64("weather_state").unwrap_or(0) {
                1 => "exposed",
                2 => "weathered",
                3 => "oxidized",
                _ => "unaffected",
            };
            data.set_string("weather_state", state);
        }),
    );

    // Reverse: string oxidation enum -> integer weather_state. The string
    // uniquely encodes the integer the forward produced
    // ("exposed"->1, "weathered"->2, "oxidized"->3, else->0), so this is the
    // exact inverse for real downgrades. "unaffected" maps back to 0, the
    // canonical preimage the old format always carried via getInt(..., 0).
    reg.entity.add_reverse_converter_for_id(
        "minecraft:copper_golem",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let state = match data.get_string("weather_state") {
                Some("exposed") => 1,
                Some("weathered") => 2,
                Some("oxidized") => 3,
                Some("unaffected") | None => 0,
                Some(other) => {
                    report_loss(
                        VERSION,
                        LossKind::UnsupportedInTarget,
                        Severity::Loss,
                        format!("unknown copper_golem weather_state {other:?}; restoring legacy unaffected state"),
                    );
                    0
                }
            };
            data.set_i32("weather_state", state);
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::registry::convert_entity_reverse;

    #[test]
    fn reverse_unknown_copper_golem_weather_state_reports() {
        let mut data = crate::nbt::NbtMap::new();
        data.set_string("id", "minecraft:copper_golem");
        data.set_string("weather_state", "stormy");

        let report = convert_entity_reverse(&mut data, VERSION, VERSION - 1);

        assert_eq!(report.loss_count(), 1);
        assert_eq!(data.get_i32("weather_state"), Some(0));
    }
}
