//! V3448 (23w14a + 3) — schematic-relevant: `minecraft:decorated_pot`
//! block-entity (V3448.java:15-25).
//!
//! VERSION = MCVersions.V23W14A (3445) + 3 = 3448.
//!
//! Ported, all TILE_ENTITY:
//!   * Walker for `sherds`: a list of item-id strings, each converted through
//!     ITEM_NAME (Java `DataWalkerListPaths<>(ITEM_NAME, "sherds")`).
//!   * Walker for `item`: a single item stack (Java `DataWalkerItems("item")`).
//!   * Converter renaming the field `shards` -> `sherds`
//!     (`RenameHelper.renameSingle`).
//!
//! Nothing in V3448 is non-schematic.

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::{convert_value_list, items};

const VERSION: i32 = 3448;

pub fn register(reg: &mut RegistryBuilder) {
    // `sherds` is a list of item-id strings -> ITEM_NAME each.
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "minecraft:decorated_pot",
        Arc::new(|reg, data, from, to| {
            convert_value_list(&reg.item_name, data, "sherds", from, to);
        }),
    );
    // `item` is a single item stack.
    reg.tile_entity
        .add_walker(VERSION, 0, "minecraft:decorated_pot", items(&["item"]));

    // Field rename shards -> sherds.
    reg.tile_entity.add_converter_for_id(
        "minecraft:decorated_pot",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.rename_key("shards", "sherds");
        }),
    );
    // Reverse: undo the field rename sherds -> shards (lossless, bucket A).
    // Id is unchanged by the forward converter, so match the same id.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:decorated_pot",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.rename_key("sherds", "shards");
        }),
    );
}
