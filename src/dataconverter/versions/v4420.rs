//! V4420 (1.21.5 + 95) — schematic-relevant subset of
//! `DataConverterJava/.../versions/V4420.java`.
//!
//! The `area_effect_cloud` entity's `Particle` field is renamed to
//! `custom_particle` (V4420.java:15-22), and a PARTICLE walker is registered for
//! that new path. Both registrations target ENTITY, so both are ported.

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::convert;

const VERSION: i32 = 4420;

pub fn register(reg: &mut RegistryBuilder) {
    // Rename `Particle` -> `custom_particle` (RenameHelper.renameSingle, i.e. only
    // when the key is present).
    reg.entity.add_converter_for_id(
        "minecraft:area_effect_cloud",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.rename_key("Particle", "custom_particle");
        }),
    );

    // Reverse: undo the rename, `custom_particle` -> `Particle`. Lossless key
    // rename (no-op when key absent), so no loss report.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:area_effect_cloud",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.rename_key("custom_particle", "Particle");
        }),
    );

    // Walk the renamed path through PARTICLE (DataWalkerTypePaths<PARTICLE, "custom_particle">).
    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:area_effect_cloud",
        Arc::new(|reg, data, from, to| {
            convert(reg, &reg.particle, data, "custom_particle", from, to);
        }),
    );
}
