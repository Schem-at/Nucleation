//! V4296 (1.21.4 + 107) — schematic-relevant subset of `V4296.java`.
//!
//! VERSION = MCVersions.V1_21_4 (4189) + 107 = 4296.
//!
//! Ported (ENTITY per-id converter, V4296.java:955-961): the
//! `minecraft:area_effect_cloud` entity gains a `potion_duration_scale` float of
//! `0.25` (the new default for migrated clouds).
//!
//! Nothing non-schematic is present in this version.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4296;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:area_effect_cloud",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            data.set_f32("potion_duration_scale", 0.25);
        }),
    );

    // Reverse of the additive default (bucket D): the old format had no
    // `potion_duration_scale`, so drop the field. Only remove it when it equals
    // the value the forward added (`0.25`) to avoid clobbering a user-set scale.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:area_effect_cloud",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_f64("potion_duration_scale") == Some(0.25) {
                data.take("potion_duration_scale");
            }
        }),
    );
}
