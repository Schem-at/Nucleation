//! V3833 (1.20.5-pre4 + 1) — schematic-relevant subset of `V3833.java`.
//!
//! Drops a `brushable_block`'s `item` field when it holds nothing
//! (`minecraft:air` or a non-positive count) (V3833.java:14-24).
//!
//! Nothing non-schematic exists in this version file.
//!
//! The id is compared after `NamespaceUtil.correctNamespace` (default `air` when
//! absent); the engine does not export that fn, so the parse rule is inlined.
//!
//! VERSION = MCVersions.V1_20_5_PRE4 (3832) + 1 = 3833.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 3833;

/// Port of `NamespaceUtil.correctNamespace`: default-namespace an unnamespaced,
/// parseable resource location; otherwise return the input unchanged.
fn correct_namespace(value: &str) -> String {
    if value.contains(':') {
        return value.to_string();
    }
    let valid = !value.is_empty()
        && value
            .bytes()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'-' | b'/'));
    if valid {
        format!("minecraft:{value}")
    } else {
        value.to_string()
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity.add_converter_for_id(
        "minecraft:brushable_block",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let item = match data.get_map("item") {
                Some(i) => i,
                None => return,
            };
            // getString("id", "minecraft:air") — default when absent.
            let id = correct_namespace(item.get_string("id").unwrap_or("minecraft:air"));
            // getInt("count", 0) — default 0 when absent.
            let count = item.get_i32("count").unwrap_or(0);

            // Fix DFU: use count <= 0 instead of count == 0.
            if id == "minecraft:air" || count <= 0 {
                data.take("item");
            }
        }),
    );
}
