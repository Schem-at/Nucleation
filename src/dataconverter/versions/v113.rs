//! V113 (15w33c + 1) — schematic-relevant subset of `V113.java`.
//!
//! VERSION = MCVersions.V15W33C (112) + 1 = 113.
//!
//! Ported (ENTITY structure converter, V113.java:27-37): the `checkList` helper
//! removes the `HandDropChances` (len 2) and `ArmorDropChances` (len 4) float
//! lists from a mob entity when they are absent, the wrong length, or entirely
//! zero. A list is kept only when it is exactly the required length AND at least
//! one element is non-zero; otherwise the key is removed (Java
//! `V113.checkList`).
//!
//! Nothing non-schematic is present in this version.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 113;

/// `V113.checkList`: remove `data[id]` unless it is a float list of exactly
/// `required_length` containing at least one non-zero element.
fn check_list(data: &mut NbtMap, id: &str, required_length: usize) {
    if let Some(list) = data.get_list(id) {
        if list.len() == required_length {
            // Keep the list if any element is non-zero (Java `getFloat(i) != 0.0F`).
            for elem in list {
                if elem.as_number_f64().map(|v| v as f32).unwrap_or(0.0) != 0.0 {
                    return;
                }
            }
        }
    }

    data.take(id);
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            check_list(data, "HandDropChances", 2);
            check_list(data, "ArmorDropChances", 4);
        }),
    );
}
