//! V3204 (1.19.2 + 84) — TILE_ENTITY walker for minecraft:chiseled_bookshelf recursing
//! "Items" as item stacks; cites V3204.java. No other registrations present in the Java file.
use super::super::registry::RegistryBuilder;
use super::super::walker::item_lists;

const VERSION: i32 = 3204;

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "minecraft:chiseled_bookshelf",
        item_lists(&["Items"]),
    );
}
