//! V1500 (18w22c+1) — schematic-relevant subset of `V1500.java`.
//!
//! Kept:
//!   * TILE_ENTITY `addConverterForId("DUMMY", ...)` that sets `keepPacked = true`
//!     (V1500.java:13-19). Java's `addConverterForId` only runs the converter when
//!     the compound's `id` equals the given id literally (IDDataType.java:23-32),
//!     so this fires only for a tile entity whose `id == "DUMMY"` — ported
//!     faithfully via `add_converter_for_id("DUMMY", ...)`.
//!
//! VERSION = MCVersions.V18W22C + 1 = 1499 + 1 = 1500.
//!
//! Nothing non-schematic is present in this version.

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 1500;

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity.add_converter_for_id(
        "DUMMY",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.set_bool("keepPacked", true);
        }),
    );

    // Reverse of V1500.java:13-19 (bucket D, additive default). The forward
    // unconditionally sets `keepPacked = true` on a DUMMY tile entity, so the
    // inverse drops the field the forward added. Only remove it when it equals
    // the value the forward wrote (`true`) so we never clobber a user-supplied
    // `keepPacked = false`. The id is unchanged in this version, so we match the
    // same literal id "DUMMY".
    reg.tile_entity.add_reverse_converter_for_id(
        "DUMMY",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_bool("keepPacked") == Some(true) {
                data.take("keepPacked");
            }
        }),
    );
}
