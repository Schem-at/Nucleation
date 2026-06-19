//! V2529 (20w17a) — schematic-relevant subset of `V2529.java`.
//!
//! ENTITY `minecraft:strider`: force `NoGravity` off. Java reads `NoGravity`
//! (default false) and, if true, sets it false (V2529.java:13-21).
//!
//! VERSION = MCVersions.V20W17A = 2529.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 2529;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:strider",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_bool("NoGravity").unwrap_or(false) {
                data.set_bool("NoGravity", false);
            }
        }),
    );
}
