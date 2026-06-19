//! V4439 (1.21.8-rc1) — schematic-relevant subset of
//! `DataConverterJava/.../versions/V4439.java`.
//!
//! Removes the `fall_distance` field from any entity when it is exactly `0.0`
//! (V4439.java:13-21). The Java code uses `getDouble("fall_distance", -1.0)` so a
//! missing field reads as `-1.0` and is left untouched; we mirror that by reading
//! the value and only removing on an exact `0.0` match.

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4439;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_f64("fall_distance").unwrap_or(-1.0) == 0.0 {
                data.take("fall_distance");
            }
        }),
    );

    // Reverse: the forward dropped `fall_distance` whenever it was exactly 0.0
    // (a value the older format always carried explicitly). Restore that default
    // when the field is absent. We only set it when missing so we never clobber a
    // surviving non-zero value. Bucket D additive-default; exact per rule 11
    // (re-adding a default the old format always stored is lossless, no report).
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if !data.has_key("fall_distance") {
                data.set_f64("fall_distance", 0.0);
            }
        }),
    );
}
