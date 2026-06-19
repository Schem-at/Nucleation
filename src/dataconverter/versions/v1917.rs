//! V1917 (18w49a+1) — schematic-relevant subset of `V1917.java`.
//!
//! ENTITY converter for `minecraft:cat`: a `CatType` of 9 is remapped to 10
//! (V1917.java:13-21). Java reads `getInt("CatType")` which defaults to 0 when
//! absent.
//!
//! VERSION = MCVersions.V18W49A (1916) + 1 = 1917.
//!
//! Reverse: the forward remaps the single old `CatType` value 9 -> 10 (to make
//! room for a texture inserted at index 9 in 18w49a). Pre-1917 cats only used
//! indices 0..=9, so the new value 10 could ONLY have originated from old 9.
//! The inverse `10 -> 9` is therefore exact (rule 11: the new value uniquely
//! encodes the old discriminator) — no loss reporting.

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 1917;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:cat",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_i32("CatType").unwrap_or(0) == 9 {
                data.set_i32("CatType", 10);
            }
        }),
    );
    // Reverse of the above: undo 9 -> 10. Exact inverse (see module doc).
    reg.entity.add_reverse_converter_for_id(
        "minecraft:cat",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_i32("CatType").unwrap_or(0) == 10 {
                data.set_i32("CatType", 9);
            }
        }),
    );
}
