//! V1920 (18w50a+1) — schematic-relevant subset of `V1920.java`.
//!
//! Kept:
//!   * TILE_ENTITY walker for `minecraft:campfire`: its `Items` field is a list
//!     of item stacks (V1920.java:71).
//!
//! Skipped (non-schematic): the CHUNK structure converter that relocates the
//! `New_Village` structure start/reference to `Village` (V1920.java:15-56), and
//! the STRUCTURE_FEATURE structure converter renaming `minecraft:new_village` to
//! `minecraft:village` (V1920.java:58-69). CHUNK and STRUCTURE_FEATURE never
//! appear in a schematic file.
//!
//! VERSION = MCVersions.V18W50A (1919) + 1 = 1920.

use super::super::registry::RegistryBuilder;
use super::super::walker::item_lists;

const VERSION: i32 = 1920;

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity
        .add_walker(VERSION, 0, "minecraft:campfire", item_lists(&["Items"]));
}
