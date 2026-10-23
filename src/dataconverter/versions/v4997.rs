//! V4997 (26.3 snapshot) — schematic-relevant subset of Mojang schema `V4997`.
//!
//! Upstream DataConverter has no step past V4892 yet; this ports Mojang's 26.3
//! datafixer (`net.minecraft.util.datafix.DataFixers`, schema 4997, and
//! `net.minecraft.util.datafix.schemas.V4997`).
//!
//! Ported:
//!   * ENTITY walker for the new `minecraft:poplar_chest_boat`: `Items` is an
//!     ITEM_STACK list (V4997 `registerEntities`). The plain
//!     `minecraft:poplar_boat` is a simple entity with no contained data
//!     (mirrors `v4070.rs` for the pale-oak boats).
//!
//! Skipped (non-schematic): `BlendingDataFix` re-run on CHUNK at 4997
//! (`DataFixers.BLENDING_VERSION` moved from 4882 to 4997; see `V4882.java` for
//! the upstream equivalent at the old version).
//!
//! VERSION = 4997 (Mojang schema id).

use super::super::registry::RegistryBuilder;
use super::super::walker::item_lists;

const VERSION: i32 = 4997;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:poplar_chest_boat",
        item_lists(&["Items"]),
    );
}

#[cfg(test)]
mod tests {
    use crate::dataconverter::convert_entity;
    use crate::dataconverter::types::{MapExt, ValueExt};
    use crate::nbt::{NbtMap, NbtValue};

    #[test]
    fn poplar_chest_boat_items_are_walked() {
        let mut item = NbtMap::new();
        item.set_string("id", "stone");
        item.set_i32("count", 1);
        let mut boat = NbtMap::new();
        boat.set_string("id", "minecraft:poplar_chest_boat");
        boat.set_list("Items", vec![NbtValue::Compound(item)]);

        convert_entity(&mut boat, super::VERSION - 1, super::VERSION);
        let items = boat.get_list("Items").unwrap();
        assert_eq!(
            items[0].as_compound_ref().unwrap().get_string("id"),
            Some("minecraft:stone")
        );
    }
}
