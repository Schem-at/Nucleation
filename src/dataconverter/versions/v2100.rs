//! V2100 (1.14.4 + 124) — schematic-relevant subset of `V2100.java`.
//!
//! Ported: the TILE_ENTITY `minecraft:beehive` walker, which recurses each
//! stored bee's `EntityData` compound through ENTITY (V2100.java:38-47).
//!
//! VERSION = MCVersions.V1_14_4 + 124 = 1976 + 124 = 2100.
//!
//! Skipped (non-schematic): the RECIPE rename (`minecraft:sugar` ->
//! `minecraft:sugar_from_sugar_cane`) and the ADVANCEMENTS rename
//! (`recipes/misc/sugar` -> `recipes/misc/sugar_from_sugar_cane`) — RECIPE and
//! ADVANCEMENTS never appear in a schematic file.

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};
use super::super::walker::convert;

const VERSION: i32 = 2100;

pub fn register(reg: &mut RegistryBuilder) {
    // Walk each bee's EntityData through ENTITY (V2100.java:38-46).
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "minecraft:beehive",
        Arc::new(|reg, data, from, to| {
            if let Some(bees) = data.get_list_mut("Bees") {
                for bee in bees.iter_mut() {
                    if let Some(bee_map) = bee.as_compound_mut() {
                        convert(reg, &reg.entity, bee_map, "EntityData", from, to);
                    }
                }
            }
        }),
    );
}
