//! V3828 (24w14a + 1) — schematic-relevant subset of `V3828.java`.
//!
//! Drops a villager trade's `buyB` item when it holds nothing
//! (`minecraft:air` or a non-positive count) (V3828.java:14-33). The id is
//! compared after `NamespaceUtil.correctNamespace` (default `minecraft:air`
//! when absent).
//!
//! Nothing non-schematic exists in this version file.
//!
//! VERSION = MCVersions.V24W14A (3827) + 1 = 3828.

use crate::nbt::NbtMap;

use super::super::helpers::correct_namespace;
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 3828;

pub fn register(reg: &mut RegistryBuilder) {
    reg.villager_trade.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_map("buyB").is_none() {
                return;
            }
            let buy_b = data.get_map("buyB").unwrap();
            // getString("id", "minecraft:air") — default when absent.
            let id = correct_namespace(buy_b.get_string("id").unwrap_or("minecraft:air"));
            // getInt("count", 0) — default 0 when absent.
            let count = buy_b.get_i32("count").unwrap_or(0);

            // Fix DFU: use count <= 0 instead of count == 0.
            if id == "minecraft:air" || count <= 0 {
                data.take("buyB");
            }
        }),
    );
}
