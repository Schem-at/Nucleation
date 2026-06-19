//! V4291 (1.21.4 + 102) — schematic-relevant subset of `V4291.java`.
//!
//! VERSION = MCVersions.V1_21_4 (4189) + 102 = 4291.
//!
//! Ported (TEXT_COMPONENT structure converter, V4291.java:595-622): formatting
//! fields that were stored as strings (`"true"` / `"false"`) become real
//! booleans for the paths `bold`, `italic`, `underlined`, `strikethrough`,
//! `obfuscated`, and the extra `interpret` field. Only compound text components
//! carry these fields; list components are handled element-wise by the walker and
//! plain-string components have no formatting.
//!
//! `Boolean.parseBoolean` is case-insensitive on `"true"` and treats everything
//! else as `false`.
//!
//! Nothing non-schematic is present in this version.

use crate::nbt::{NbtMap, NbtValue};

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4291;

const BOOLEAN_PATHS_TO_CONVERT: &[&str] = &[
    "bold",
    "italic",
    "underlined",
    "strikethrough",
    "obfuscated",
    "interpret",
];

/// `convertToBoolean`: when `data[path]` is a string, parse it as a boolean
/// (`Boolean.parseBoolean`) and store the boolean back.
fn convert_to_boolean(data: &mut NbtMap, path: &str) {
    let value = match data.get_string(path) {
        Some(v) => v.to_string(),
        None => return,
    };
    data.set_bool(path, value.eq_ignore_ascii_case("true"));
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.text_component.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            for path in BOOLEAN_PATHS_TO_CONVERT {
                convert_to_boolean(data, path);
            }
        }),
    );

    // Reverse: the forward turned the old string formatting flags
    // (`"true"`/`"false"`) into real boolean (Byte) tags. Restore the canonical
    // old string form so a pre-V4291 reader sees them. The old format always
    // stored these as the strings `"true"`/`"false"`, and the modern Byte value
    // uniquely encodes which one (non-zero -> "true", zero -> "false"), so this
    // is a lossless inverse for real downgrades (rule 11). Only Byte/boolean
    // values are rewritten; anything else (e.g. a string left over from a
    // newer-than-this version) is left untouched.
    reg.text_component.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            for path in BOOLEAN_PATHS_TO_CONVERT {
                if let Some(NbtValue::Byte(v)) = data.get(*path) {
                    let s = if *v != 0 { "true" } else { "false" };
                    data.set_string(*path, s);
                }
            }
        }),
    );
}
