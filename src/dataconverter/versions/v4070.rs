//! V4070 (24w39a + 1) — schematic-relevant subset of `V4070.java`.
//!
//! Adds the `Items` item-list walker for the pale-oak chest boat entity
//! (`minecraft:pale_oak_chest_boat`). The plain `minecraft:pale_oak_boat` is a
//! simple entity with no contained data.
//!
//! Nothing non-schematic is present in this version.
//!
//! VERSION = MCVersions.V24W39A (4069) + 1 = 4070.

use super::super::registry::RegistryBuilder;
use super::super::walker::item_lists;

const VERSION: i32 = 4070;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:pale_oak_chest_boat",
        item_lists(&["Items"]),
    );
}
