//! V4294 (1.21.4 + 105) — schematic-relevant subset of `V4294.java`.
//!
//! VERSION = MCVersions.V1_21_4 (4189) + 105 = 4294.
//!
//! Ported (BLOCK_STATE structure converter, V4294.java:896-916): the
//! `minecraft:creaking_heart` block replaced its boolean-string `active` property
//! with the `creaking_heart_state` enum (`active == "true"` -> `awake`, else
//! `uprooted`).
//!
//! Nothing non-schematic is present in this version.

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4294;

pub fn register(reg: &mut RegistryBuilder) {
    reg.block_state.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_string("Name") != Some("minecraft:creaking_heart") {
                return;
            }

            let properties = match data.get_map_mut("Properties") {
                Some(p) => p,
                None => return,
            };

            let active = match properties.get_string("active") {
                Some(a) => a.to_string(),
                None => return,
            };
            properties.take("active");
            properties.set_string(
                "creaking_heart_state",
                if active == "true" {
                    "awake"
                } else {
                    "uprooted"
                },
            );
        }),
    );

    // Inverse of the BLOCK_STATE converter above (V4294.java:896-916). The
    // `creaking_heart_state` enum uniquely encodes the old boolean-string
    // `active` property: forward maps "true" -> "awake" and "false" ->
    // "uprooted", so the reverse maps "awake" -> "true" and any other value
    // ("uprooted") -> "false". Lossless for real downgrades (bucket B).
    reg.block_state.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_string("Name") != Some("minecraft:creaking_heart") {
                return;
            }

            let properties = match data.get_map_mut("Properties") {
                Some(p) => p,
                None => return,
            };

            let state = match properties.get_string("creaking_heart_state") {
                Some(s) => s.to_string(),
                None => return,
            };
            properties.take("creaking_heart_state");
            if state != "awake" {
                report_loss(
                    VERSION,
                    LossKind::FingerprintCollapse,
                    Severity::Approximated,
                    "creaking_heart_state is not awake; legacy active value collapsed to false preimage",
                );
            }
            properties.set_string("active", if state == "awake" { "true" } else { "false" });
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::registry::convert_block_state_reverse;

    #[test]
    fn reverse_non_awake_creaking_heart_reports_collapsed_active_value() {
        let mut data = NbtMap::new();
        data.set_string("Name", "minecraft:creaking_heart");
        let mut properties = NbtMap::new();
        properties.set_string("creaking_heart_state", "uprooted");
        data.set_map("Properties", properties);

        let report = convert_block_state_reverse(&mut data, VERSION, VERSION - 1);

        assert_eq!(report.len(), 1);
        assert_eq!(report.entries[0].kind, LossKind::FingerprintCollapse);
        assert_eq!(
            data.get_map("Properties").unwrap().get_string("active"),
            Some("false")
        );
    }
}
