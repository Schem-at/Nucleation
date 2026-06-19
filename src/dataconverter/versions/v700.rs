//! V700 (1.11 snapshot, MCVersions.V1_10_2 + 188 = 700) — schematic-relevant
//! subset of V700.java.
//!
//! Splits the legacy `Guardian` entity into `Guardian` / `ElderGuardian` based on
//! the boolean `Elder` flag (which is then removed). Schematic-relevant: it
//! mutates an ENTITY's `id`. The commented-out `registerMob("ElderGuardian")` in
//! the Java is a no-op (simple mob) and nothing to port.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 700;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "Guardian",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // Java: data.getBoolean("Elder") defaults to false when absent.
            if data.get_bool("Elder").unwrap_or(false) {
                data.set_string("id", "ElderGuardian");
            }
            data.take("Elder");
        }),
    );

    // Reverse of V700.java:13-22. The forward split the legacy `Guardian`
    // entity into `Guardian` / `ElderGuardian` by reading the boolean `Elder`
    // flag (then removing it). The new id uniquely encodes the discriminator,
    // so this is exact/lossless (rule 11): `ElderGuardian` means `Elder=true`,
    // a plain `Guardian` means `Elder=false`. The legacy format always carried
    // the `Elder` boolean, so restoring it is exact, not data loss.
    //
    // `add_reverse_converter_for_id` matches the NEW id; any inverse id-rename
    // (flattening to `minecraft:elder_guardian` etc.) runs later in the
    // descending sweep, so we match the un-namespaced ids this converter emits.
    reg.entity.add_reverse_converter_for_id(
        "ElderGuardian",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            data.set_string("id", "Guardian");
            data.set_bool("Elder", true);
        }),
    );
    reg.entity.add_reverse_converter_for_id(
        "Guardian",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // Plain Guardian: forward removed the `Elder` flag (it was false);
            // restore the default the legacy format always carried.
            data.set_bool("Elder", false);
        }),
    );
}
