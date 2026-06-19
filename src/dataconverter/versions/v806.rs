//! V806 (16w36a+1) — schematic-relevant subset.
//!
//! Port of DataConverterJava .../versions/V806.java. Ensures potion-type item
//! stacks (potion / splash_potion / lingering_potion / tipped_arrow) carry a
//! `tag.Potion` string, defaulting it to `minecraft:water` when absent. Creates
//! the `tag` compound if missing, matching the Java updater.
//!
//! V806.java only touches ITEM_STACK, so nothing is skipped here.

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 806;

/// Item ids whose stacks get a defaulted `tag.Potion` (V806.java:32-35).
const POTION_ITEM_IDS: &[&str] = &[
    "minecraft:potion",
    "minecraft:splash_potion",
    "minecraft:lingering_potion",
    "minecraft:tipped_arrow",
];

pub fn register(reg: &mut RegistryBuilder) {
    for &id in POTION_ITEM_IDS {
        reg.item_stack.add_converter_for_id(
            id,
            VERSION,
            0,
            Box::new(|data, _from, _to| {
                // Ensure `tag` exists (the Java updater creates it when absent).
                if data.get_map_mut("tag").is_none() {
                    data.set_map("tag", NbtMap::new());
                }
                let tag = match data.get_map_mut("tag") {
                    Some(t) => t,
                    None => return,
                };
                // Java checks hasKey("Potion", STRING): only a String counts as present.
                let has_string_potion = matches!(tag.get("Potion"), Some(NbtValue::String(_)));
                if !has_string_potion {
                    tag.set_string("Potion", "minecraft:water");
                }
            }),
        );

        // Reverse: the forward only ADDS a defaulted `tag.Potion = "minecraft:water"`
        // when no String `Potion` was present (V806.java:24-26). Pre-806, the absence
        // of `Potion` was the implicit water bottle, so the inverse strips the default
        // to restore the old "no Potion tag" shape. This is ambiguous: a potion that
        // legitimately carries Potion="minecraft:water" in the new format is
        // indistinguishable from one the forward defaulted, so dropping it is
        // best-effort (Approximated). We leave any other `Potion` value untouched, and
        // never remove `tag` itself (it may hold other data).
        reg.item_stack.add_reverse_converter_for_id(
            id,
            VERSION,
            0,
            Box::new(|data, _from, _to| {
                let tag = match data.get_map_mut("tag") {
                    Some(t) => t,
                    None => return,
                };
                if tag.get_string("Potion") == Some("minecraft:water") {
                    tag.take("Potion");
                    report_loss(
                        VERSION,
                        LossKind::RenameAmbiguous,
                        Severity::Approximated,
                        "potion item: dropped defaulted tag.Potion=\"minecraft:water\" \
                         (cannot distinguish a forward-added default from an \
                         explicitly-set water bottle)",
                    );
                }
                if data.get_map("tag").map(MapExt::is_empty).unwrap_or(false) {
                    data.take("tag");
                }
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::registry::convert_item_stack_reverse;

    #[test]
    fn reverse_water_potion_prunes_empty_tag() {
        let mut tag = NbtMap::new();
        tag.set_string("Potion", "minecraft:water");
        let mut item = NbtMap::new();
        item.set_string("id", "minecraft:potion");
        item.set_map("tag", tag);

        let report = convert_item_stack_reverse(&mut item, 806, 805);

        assert!(!item.has_key("tag"));
        assert_eq!(report.len(), 1);
        assert_eq!(report.loss_count(), 0);
    }
}
