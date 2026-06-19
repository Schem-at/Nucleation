//! V4064 (24w36a + 1) — schematic-relevant subset of `V4064.java`.
//!
//! ITEM_STACK structure converter: the boolean `minecraft:fire_resistant`
//! component is replaced by a `minecraft:damage_resistant` compound with
//! `types = "#minecraft:is_fire"`.
//!
//! Nothing non-schematic is present in this version.
//!
//! VERSION = MCVersions.V24W36A (4063) + 1 = 4064.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::types::ValueExt;

const VERSION: i32 = 4064;

pub fn register(reg: &mut RegistryBuilder) {
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let components = match data.get_map_mut("components") {
                Some(c) => c,
                None => return,
            };
            if components.has_key("minecraft:fire_resistant") {
                components.take("minecraft:fire_resistant");
                let mut damage_resistant = NbtMap::new();
                damage_resistant.set_string("types", "#minecraft:is_fire");
                components.set_map("minecraft:damage_resistant", damage_resistant);
            }
        }),
    );

    // Reverse of the structure converter above (V4064.java:21-28).
    //
    // Forward replaced the unit component `minecraft:fire_resistant` with the
    // compound `minecraft:damage_resistant = { types: "#minecraft:is_fire" }`.
    // The reverse restores the unit component when it sees that exact marker.
    //
    // Lossless (rule 11): `damage_resistant` with `types == "#minecraft:is_fire"`
    // is the unambiguous forward output, and `fire_resistant` was a unit
    // component whose canonical value is the empty compound `{}` — restoring a
    // default the old format always carried is exact, not loss. A
    // `damage_resistant` carrying any other `types` (or a non-string `types`,
    // e.g. an inline tag list) was authored directly and is left untouched.
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let components = match data.get_map_mut("components") {
                Some(c) => c,
                None => return,
            };
            let is_fire = components
                .get_map("minecraft:damage_resistant")
                .and_then(|dr| dr.get("types"))
                .and_then(ValueExt::as_str)
                == Some("#minecraft:is_fire");
            if is_fire {
                components.take("minecraft:damage_resistant");
                components.set_map("minecraft:fire_resistant", NbtMap::new());
            }
        }),
    );
}
