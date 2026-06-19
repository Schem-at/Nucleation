//! V2505 (20w06a + 1 = 2505) — schematic-relevant subset of `V2505.java`.
//!
//! (1) ENTITY `minecraft:villager` converter (V2505.java:15-39): migrates the
//!     `Brain.memories` format. Every memory value `Brain.memories.<key>` is
//!     wrapped into a new compound `{value: <orig>}`. Bails out early (no-op) if
//!     `Brain` or `Brain.memories` is absent.
//! (2) ENTITY `minecraft:piglin` walker (V2505.java:41): recurse `Inventory` as
//!     an item-stack list (`DataWalkerItemLists("Inventory")`).
//!
//! VERSION = MCVersions.V20W06A (2504) + 1 = 2505. Nothing non-schematic present.

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::item_lists;

const VERSION: i32 = 2505;

pub fn register(reg: &mut RegistryBuilder) {
    // (1) Wrap each Brain.memories.<key> generic value into {value: <orig>}.
    reg.entity.add_converter_for_id(
        "minecraft:villager",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let brain = match data.get_map_mut("Brain") {
                Some(b) => b,
                None => return,
            };

            if brain.get_map("memories").is_none() {
                return;
            }
            let memories = brain.get_map_mut("memories").unwrap();

            for key in memories.keys() {
                // getGeneric(key) returns the existing value (non-null because
                // it is an existing key); take it out and wrap it.
                if let Some(value) = memories.take(&key) {
                    let mut wrapped = NbtMap::new();
                    wrapped.set_generic("value", value);
                    memories.set_map(&key, wrapped);
                }
            }
        }),
    );

    // (1-reverse) Unwrap each Brain.memories.<key> from {value: <orig>} back to
    // <orig>. Exact inverse of the forward wrap (bucket B, lossless): the forward
    // moved the original value into the `value` field of a fresh compound, so the
    // reverse pulls that `value` back out. Bails out early (no-op) if `Brain` or
    // `Brain.memories` is absent, mirroring the forward guards.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:villager",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let brain = match data.get_map_mut("Brain") {
                Some(b) => b,
                None => return,
            };

            if brain.get_map("memories").is_none() {
                return;
            }
            let memories = brain.get_map_mut("memories").unwrap();

            for key in memories.keys() {
                let Some(value) = memories.take(&key) else {
                    continue;
                };
                match value {
                    NbtValue::Compound(mut wrapper) => {
                        if let Some(unwrapped) = wrapper.take("value") {
                            if !wrapper.is_empty() {
                                report_loss(
                                    VERSION,
                                    LossKind::UnsupportedInTarget,
                                    Severity::Loss,
                                    format!(
                                        "villager Brain.memories `{key}` wrapper had modern-only fields that cannot be represented before V2505"
                                    ),
                                );
                            }
                            memories.set_generic(&key, unwrapped);
                        } else {
                            report_loss(
                                VERSION,
                                LossKind::UnsupportedInTarget,
                                Severity::Approximated,
                                format!(
                                    "villager Brain.memories `{key}` wrapper had no `value`; preserved compound as legacy memory value"
                                ),
                            );
                            memories.set_map(&key, wrapper);
                        }
                    }
                    other => {
                        memories.set_generic(&key, other);
                    }
                }
            }
        }),
    );

    // (2) Piglin inventory item-list walker.
    reg.entity
        .add_walker(VERSION, 0, "minecraft:piglin", item_lists(&["Inventory"]));
}

#[cfg(test)]
mod tests {
    use crate::dataconverter::convert_entity_reverse;
    use crate::dataconverter::types::{MapExt, ValueExt};
    use crate::nbt::{NbtMap, NbtValue};

    #[test]
    fn reverse_reports_villager_memory_wrapper_metadata() {
        let mut wrapper = NbtMap::new();
        wrapper.set_string("value", "minecraft:home");
        wrapper.set_i32("ttl", 20);
        let mut memories = NbtMap::new();
        memories.set_map("minecraft:home", wrapper);
        let mut brain = NbtMap::new();
        brain.set_map("memories", memories);
        let mut villager = NbtMap::new();
        villager.set_string("id", "minecraft:villager");
        villager.set_map("Brain", brain);

        let report = convert_entity_reverse(&mut villager, 2505, 2504);

        let memory = villager
            .get_map("Brain")
            .unwrap()
            .get_map("memories")
            .unwrap()
            .get("minecraft:home")
            .unwrap();
        assert_eq!(memory.as_str(), Some("minecraft:home"));
        assert_eq!(report.loss_count(), 1);
        assert!(report.summary().contains("modern-only fields"));
    }

    #[test]
    fn reverse_preserves_non_compound_memory_values() {
        let mut memories = NbtMap::new();
        memories.set_generic("minecraft:test", NbtValue::Int(7));
        let mut brain = NbtMap::new();
        brain.set_map("memories", memories);
        let mut villager = NbtMap::new();
        villager.set_string("id", "minecraft:villager");
        villager.set_map("Brain", brain);

        let report = convert_entity_reverse(&mut villager, 2505, 2504);

        assert_eq!(
            villager
                .get_map("Brain")
                .unwrap()
                .get_map("memories")
                .unwrap()
                .get_i32("minecraft:test"),
            Some(7)
        );
        assert!(report.is_empty());
    }
}
