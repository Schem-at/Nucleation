//! V4763 (V4763.java) — sets `VillagerDataFinalized = true` on existing
//! villager-type ENTITY data (V4763.java:13-22). Registered for both
//! `minecraft:villager` and `minecraft:zombie_villager`. ENTITY is a
//! schematic-relevant type, so this is ported in full. Nothing in this version
//! targets a non-schematic type.
//!
//! VERSION = MCVersions.V1_21_11 (4671) + 92 = 4763.

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4763;

pub fn register(reg: &mut RegistryBuilder) {
    // setBoolean stores a Byte(1) — see MapExt::set_bool (matches Java setBoolean).
    reg.entity.add_converter_for_id(
        "minecraft:villager",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.set_bool("VillagerDataFinalized", true);
        }),
    );
    // Reverse (bucket D, additive default): the pre-V4763 villager format had no
    // `VillagerDataFinalized` field, so the forward converter's only effect was to
    // add it as Byte(1). Drop it on downgrade — but only when it equals the value
    // the forward set (true), so any unrelated user value is preserved. Lossless,
    // no report_loss.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:villager",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_bool("VillagerDataFinalized") == Some(true) {
                data.take("VillagerDataFinalized");
            }
        }),
    );
    reg.entity.add_converter_for_id(
        "minecraft:zombie_villager",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.set_bool("VillagerDataFinalized", true);
        }),
    );
    // Reverse (bucket D, additive default): mirror of the villager case for
    // zombie villagers — remove the added `VillagerDataFinalized` when it is the
    // forward-set value. Lossless, no report_loss.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:zombie_villager",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_bool("VillagerDataFinalized") == Some(true) {
                data.take("VillagerDataFinalized");
            }
        }),
    );
}
