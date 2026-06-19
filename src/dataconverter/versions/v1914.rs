//! V1914 (18w48a) — schematic-relevant subset of `V1914.java`.
//!
//! TILE_ENTITY converter for `minecraft:chest`: the block-entity `LootTable`
//! `minecraft:chests/village_blacksmith` is rewritten to
//! `minecraft:chests/village/village_weaponsmith` (V1914.java:13-24). Java reads
//! `getString("LootTable")` (null when absent) and only rewrites on an exact
//! match.
//!
//! VERSION = MCVersions.V18W48A (1914).

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 1914;

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity.add_converter_for_id(
        "minecraft:chest",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_string("LootTable") == Some("minecraft:chests/village_blacksmith") {
                data.set_string("LootTable", "minecraft:chests/village/village_weaponsmith");
            }
        }),
    );

    // Reverse of V1914.java:13-24. The forward rewrites the chest `LootTable`
    // `minecraft:chests/village_blacksmith` -> `.../village/village_weaponsmith`.
    // Only this single value is remapped in this version, so the new value
    // uniquely encodes the old one for real downgrades — exact inverse, lossless
    // (bucket A, a value rename). Matches the NEW id `minecraft:chest` (unchanged).
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:chest",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_string("LootTable")
                == Some("minecraft:chests/village/village_weaponsmith")
            {
                data.set_string("LootTable", "minecraft:chests/village_blacksmith");
            }
        }),
    );
}
