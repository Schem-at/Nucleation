//! V2535 (20w19a + 1) — schematic-relevant subset of `V2535.java`.
//!
//! ENTITY `minecraft:shulker`: subtract 180 from the yaw (`Rotation[0]`)
//! (V2535.java:15-30). Rotation is stored as a FLOAT list; the value is read and
//! written back as a float so the entity-load doesn't discard it.
//!
//! VERSION = MCVersions.V20W19A (2534) + 1 = 2535.

use crate::nbt::{NbtMap, NbtValue};

use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 2535;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:shulker",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let rotation = match data.get_list_mut("Rotation") {
                Some(r) => r,
                None => return,
            };
            if rotation.is_empty() {
                return;
            }
            // getFloat(0) defaults to 0.0 for a non-float element.
            let yaw = rotation[0].as_number_f64().unwrap_or(0.0) as f32;
            rotation[0] = NbtValue::Float(yaw - 180.0);
        }),
    );

    // Reverse: add 180 back to the yaw. The forward `yaw - 180` is exactly
    // invertible by `yaw + 180` (the value is mod-360 in vanilla but the engine
    // stores the raw float; this is the lossless inverse of the forward step).
    reg.entity.add_reverse_converter_for_id(
        "minecraft:shulker",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let rotation = match data.get_list_mut("Rotation") {
                Some(r) => r,
                None => return,
            };
            if rotation.is_empty() {
                return;
            }
            let yaw = rotation[0].as_number_f64().unwrap_or(0.0) as f32;
            rotation[0] = NbtValue::Float(yaw + 180.0);
        }),
    );
}
