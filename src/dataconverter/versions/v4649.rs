//! V4649 (1.21.10 + 93) — schematic-relevant subset of `V4649.java`.
//!
//! Kept:
//!   * DATA_COMPONENTS structure converter: the `minecraft:consumable`
//!     component's `animation` value `"spear"` is renamed to `"trident"`
//!     (V4649.java:13-29).
//!
//! VERSION = MCVersions.V1_21_10 (4556) + 93 = 4649.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4649;

pub fn register(reg: &mut RegistryBuilder) {
    // consumable animation "spear" -> "trident" (V4649.java:13-29).
    reg.data_components.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(consumable) = data.get_map_mut("minecraft:consumable") {
                if consumable.get_string("animation") == Some("spear") {
                    consumable.set_string("animation", "trident");
                }
            }
        }),
    );
    // Reverse: undo the consumable animation rename "trident" -> "spear".
    // The forward (V4649.java:21-25) renamed the legacy "spear" animation to
    // "trident"; this converter introduces "trident" as the new name for that
    // value, so the inverse is exact and lossless (bucket A).
    reg.data_components.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(consumable) = data.get_map_mut("minecraft:consumable") {
                if consumable.get_string("animation") == Some("trident") {
                    consumable.set_string("animation", "spear");
                }
            }
        }),
    );
}
