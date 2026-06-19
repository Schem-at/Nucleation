//! V4081 (1.21.2 + 1) — schematic-relevant subset of `V4081.java`.
//!
//! ENTITY `minecraft:salmon` converter: a salmon whose `type` is not `"large"`
//! is normalised to `"medium"` (the previously-implicit default sizing). A
//! `"large"` salmon is left untouched.
//!
//! Nothing non-schematic is present in this version.
//!
//! VERSION = MCVersions.V1_21_2 (4080) + 1 = 4081.

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4081;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:salmon",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_string("type") == Some("large") {
                return;
            }
            data.set_string("type", "medium");
        }),
    );
}
