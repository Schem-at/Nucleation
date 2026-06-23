//! V3807 (24w04a + 1) — schematic-relevant subset of `V3807.java`.
//!
//! Ported (step 0): the TILE_ENTITY walker for `minecraft:vault`, recursing the
//! ITEM_STACKs nested in the vault's config/server_data/shared_data
//! (V3807.java:25-31):
//!   * `config.key_item`         — single item
//!   * `server_data.items_to_eject` — item list
//!   * `shared_data.display_item` — single item
//!
//! Skipped (non-schematic): step 1's SAVED_DATA_MAP_DATA structure converter
//! (`banners[].Pos` block-pos flattening) — SAVED_DATA never appears in a
//! schematic file.
//!
//! VERSION = MCVersions.V24W04A (3806) + 1 = 3807.

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::{convert, convert_list};

const VERSION: i32 = 3807;

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "minecraft:vault",
        Arc::new(|reg, root, from, to| {
            if let Some(config) = root.get_map_mut("config") {
                convert(reg, &reg.item_stack, config, "key_item", from, to);
            }
            if let Some(server_data) = root.get_map_mut("server_data") {
                convert_list(
                    reg,
                    &reg.item_stack,
                    server_data,
                    "items_to_eject",
                    from,
                    to,
                );
            }
            if let Some(shared_data) = root.get_map_mut("shared_data") {
                convert(reg, &reg.item_stack, shared_data, "display_item", from, to);
            }
        }),
    );
}
