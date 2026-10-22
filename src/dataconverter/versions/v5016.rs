//! V5016 (26.3 snapshot) — unfilled buried-treasure maps get the new name key.
//!
//! Upstream DataConverter has no step past V4892 yet; this ports Mojang's 26.3
//! `net.minecraft.util.datafix.fixes.UnfilledBuriedTreasureMapFix` (registered
//! at schema 5016 in `net.minecraft.util.datafix.DataFixers`): on a
//! `minecraft:map` item stack whose `minecraft:item_name` is the plain
//! translation `filled_map.buried_treasure` (one-field `{translate}`
//! component, `ExtraDataFixUtils.getPlainTranslationKey`), the translation key
//! becomes `item.minecraft.buried_treasure_map`. ITEM_STACK is
//! schematic-relevant, so this is ported in full.
//!
//! Reverse: the inverse key swap on the same shape (bucket A, lossless — the
//! new key did not exist before this version).
//!
//! VERSION = 5016 (Mojang schema id).

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::v5008::plain_translation_key;

const VERSION: i32 = 5016;

const OLD_TRANSLATION: &str = "filled_map.buried_treasure";
const NEW_TRANSLATION: &str = "item.minecraft.buried_treasure_map";

fn swap_name(data: &mut NbtMap, from_key: &str, to_key: &str) {
    if data.get_string("id") != Some("minecraft:map") {
        return;
    }
    let Some(components) = data.get_map_mut("components") else {
        return;
    };
    if plain_translation_key(components.get("minecraft:item_name")) == Some(from_key) {
        if let Some(name) = components.get_map_mut("minecraft:item_name") {
            name.set_string("translate", to_key);
        }
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            swap_name(data, OLD_TRANSLATION, NEW_TRANSLATION);
        }),
    );
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            swap_name(data, NEW_TRANSLATION, OLD_TRANSLATION);
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::{convert_item_stack, convert_item_stack_reverse};

    fn named_item(id: &str, key: &str) -> NbtMap {
        let mut name = NbtMap::new();
        name.set_string("translate", key);
        let mut components = NbtMap::new();
        components.set_map("minecraft:item_name", name);
        let mut item = NbtMap::new();
        item.set_string("id", id);
        item.set_i32("count", 1);
        item.set_map("components", components);
        item
    }

    fn key(item: &NbtMap) -> Option<&str> {
        item.get_map("components")?
            .get_map("minecraft:item_name")?
            .get_string("translate")
    }

    #[test]
    fn unfilled_map_name_swapped_and_reversed() {
        let mut item = named_item("minecraft:map", OLD_TRANSLATION);
        convert_item_stack(&mut item, VERSION - 1, VERSION);
        assert_eq!(key(&item), Some(NEW_TRANSLATION));
        let report = convert_item_stack_reverse(&mut item, VERSION, VERSION - 1);
        assert!(report.is_empty());
        assert_eq!(key(&item), Some(OLD_TRANSLATION));
    }

    #[test]
    fn other_items_untouched() {
        let mut item = named_item("minecraft:paper", OLD_TRANSLATION);
        convert_item_stack(&mut item, VERSION - 1, VERSION);
        assert_eq!(key(&item), Some(OLD_TRANSLATION));
    }
}
