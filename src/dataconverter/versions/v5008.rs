//! V5008 (26.3 snapshot) — explorer maps become their own items; `map_color`
//! is removed.
//!
//! Upstream DataConverter has no step past V4892 yet; this ports the two
//! Mojang 26.3 fixes registered at schema 5008 in
//! `net.minecraft.util.datafix.DataFixers`, both schematic-relevant (item
//! stacks):
//!
//! * `ExplorerMapItemFix` (ITEM_STACK): a `minecraft:filled_map` whose
//!   `minecraft:map_decorations` holds an explorer target under key `"+"` is
//!   re-identified by that decoration's `type` (namespaced) as one of the new
//!   map items (`minecraft:mansion` -> `minecraft:woodland_explorer_map`, …,
//!   `minecraft:red_x` -> `minecraft:buried_treasure_map`). Its
//!   `minecraft:item_name` is removed when it is exactly the legacy plain
//!   translation (`ExtraDataFixUtils.getPlainTranslationKey`: a one-field
//!   `{translate}` component) for that decoration, e.g. `filled_map.mansion`.
//!   Five of the new ids are renamed again at V5012 (`v5012.rs`).
//! * `MapColorComponentRemovalFix` (DATA_COMPONENTS,
//!   `DataComponentRemainderFix` returning null): `minecraft:map_color` is
//!   dropped from every component map.
//!
//! Reverse (`ExplorerMapItemFix` and the `map_color` removal together): a new
//! map item turns back into `minecraft:filled_map`, and — when absent — gets
//! back the `minecraft:item_name` `{translate: <legacy key>}` and the
//! `minecraft:map_color` that 26.2 always gave an explorer map
//! (`MapItemSavedData.addTargetDecoration` sets `map_color` from the decoration
//! type's colour, 26.2 `MapDecorationTypes`; trades and loot set the name).
//! These are fixed per-type defaults, so restoring them is exact (cheatsheet
//! rule 11). A custom `map_color` on any other item is dropped by the forward
//! with no trace in the newer data, so there is nothing to restore for it.
//!
//! VERSION = 5008 (Mojang schema id).

use crate::nbt::{NbtMap, NbtValue};

use super::super::helpers::correct_namespace;
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 5008;

const FILLED_MAP: &str = "minecraft:filled_map";

/// 26.2 `MapColor.COLOR_LIGHT_GRAY.col`.
const LIGHT_GRAY: i32 = 10066329;

/// One explorer-map kind: `(decoration type, new item id, V5012 item id,
/// legacy item_name translation key, 26.2 map_color)`.
///
/// Decoration type -> item id and -> legacy name key come from
/// `ExplorerMapItemFix.DECORATION_TYPE_TO_ITEM_ID` /
/// `DECORATION_TYPE_TO_LEGACY_NAME_KEY`; the V5012 id from the
/// "Rename explorer map items" `ItemRenameFix` (the same id when not renamed);
/// the colour from 26.2 `MapDecorationTypes` (`None` = `hasMapColor()` false).
type ExplorerMap = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    Option<i32>,
);

const EXPLORER_MAPS: &[ExplorerMap] = &[
    (
        "minecraft:mansion",
        "minecraft:woodland_explorer_map",
        "minecraft:woodland_mansion_map",
        "filled_map.mansion",
        Some(5393476),
    ),
    (
        "minecraft:monument",
        "minecraft:ocean_explorer_map",
        "minecraft:ocean_monument_map",
        "filled_map.monument",
        Some(3830373),
    ),
    (
        "minecraft:trial_chambers",
        "minecraft:trial_explorer_map",
        "minecraft:buried_trial_chambers_map",
        "filled_map.trial_chambers",
        Some(12741452),
    ),
    (
        "minecraft:jungle_temple",
        "minecraft:jungle_explorer_map",
        "minecraft:jungle_pyramid_map",
        "filled_map.explorer_jungle",
        Some(LIGHT_GRAY),
    ),
    (
        "minecraft:swamp_hut",
        "minecraft:swamp_explorer_map",
        "minecraft:swamp_hut_map",
        "filled_map.explorer_swamp",
        Some(LIGHT_GRAY),
    ),
    (
        "minecraft:village_desert",
        "minecraft:desert_village_map",
        "minecraft:desert_village_map",
        "filled_map.village_desert",
        Some(LIGHT_GRAY),
    ),
    (
        "minecraft:village_plains",
        "minecraft:plains_village_map",
        "minecraft:plains_village_map",
        "filled_map.village_plains",
        Some(LIGHT_GRAY),
    ),
    (
        "minecraft:village_savanna",
        "minecraft:savanna_village_map",
        "minecraft:savanna_village_map",
        "filled_map.village_savanna",
        Some(LIGHT_GRAY),
    ),
    (
        "minecraft:village_snowy",
        "minecraft:snowy_village_map",
        "minecraft:snowy_village_map",
        "filled_map.village_snowy",
        Some(LIGHT_GRAY),
    ),
    (
        "minecraft:village_taiga",
        "minecraft:taiga_village_map",
        "minecraft:taiga_village_map",
        "filled_map.village_taiga",
        Some(LIGHT_GRAY),
    ),
    (
        "minecraft:red_x",
        "minecraft:buried_treasure_map",
        "minecraft:buried_treasure_map",
        "filled_map.buried_treasure",
        None,
    ),
];

/// `ExtraDataFixUtils.getPlainTranslationKey`: the `translate` key of a text
/// component that is a map with exactly one field.
pub(super) fn plain_translation_key(component: Option<&NbtValue>) -> Option<&str> {
    let map = component?.as_compound_ref()?;
    if map.len() != 1 {
        return None;
    }
    map.get_string("translate")
}

/// The explorer-map kind of a filled map, from its `"+"` decoration type.
fn explorer_map_for(item: &NbtMap) -> Option<&'static ExplorerMap> {
    let decoration = item
        .get_map("components")?
        .get_map("minecraft:map_decorations")?
        .get_map("+")?;
    let ty = correct_namespace(decoration.get_string("type").unwrap_or(""));
    EXPLORER_MAPS.iter().find(|m| m.0 == ty)
}

pub fn register(reg: &mut RegistryBuilder) {
    // ExplorerMapItemFix (ITEM_STACK).
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_string("id") != Some(FILLED_MAP) {
                return;
            }
            let Some(&(_, new_id, _, legacy_key, _)) = explorer_map_for(data) else {
                return;
            };
            data.set_string("id", new_id);
            if let Some(components) = data.get_map_mut("components") {
                if plain_translation_key(components.get("minecraft:item_name")) == Some(legacy_key)
                {
                    components.take("minecraft:item_name");
                }
            }
        }),
    );
    // Reverse: new map item -> filled_map, restoring the 26.2 per-type defaults.
    // At this point any V5012 rename has already been undone (item ids are
    // reverse-renamed during the ITEM_STACK walker descent), so the V4996-era
    // id is expected; the V5012 id is accepted too for robustness.
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let Some(id) = data.get_string("id") else {
                return;
            };
            let Some(&(_, _, _, legacy_key, color)) =
                EXPLORER_MAPS.iter().find(|m| m.1 == id || m.2 == id)
            else {
                return;
            };
            data.set_string("id", FILLED_MAP);
            if data.get_map("components").is_none() {
                data.set_map("components", NbtMap::new());
            }
            let components = data.get_map_mut("components").expect("just ensured");
            if !components.has_key("minecraft:item_name") {
                let mut name = NbtMap::new();
                name.set_string("translate", legacy_key);
                components.set_map("minecraft:item_name", name);
            }
            if let Some(color) = color {
                if !components.has_key("minecraft:map_color") {
                    components.set_i32("minecraft:map_color", color);
                }
            }
        }),
    );

    // MapColorComponentRemovalFix (DATA_COMPONENTS). Its inverse is folded into
    // the ExplorerMapItemFix reverse above (the only items whose map_color is
    // determined by the newer data).
    reg.data_components.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            data.take("minecraft:map_color");
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::{convert_item_stack, convert_item_stack_reverse};

    fn explorer_map(decoration_type: &str, with_name: Option<&str>) -> NbtMap {
        let mut plus = NbtMap::new();
        plus.set_string("type", decoration_type);
        plus.set_f64("x", 10.0);
        plus.set_f64("z", 20.0);
        plus.set_f32("rotation", 180.0);
        let mut decorations = NbtMap::new();
        decorations.set_map("+", plus);
        let mut components = NbtMap::new();
        components.set_map("minecraft:map_decorations", decorations);
        components.set_i32("minecraft:map_id", 7);
        components.set_i32("minecraft:map_color", 5393476);
        if let Some(key) = with_name {
            let mut name = NbtMap::new();
            name.set_string("translate", key);
            components.set_map("minecraft:item_name", name);
        }
        let mut item = NbtMap::new();
        item.set_string("id", FILLED_MAP);
        item.set_i32("count", 1);
        item.set_map("components", components);
        item
    }

    #[test]
    fn mansion_map_becomes_explorer_item_and_back() {
        let original = explorer_map("mansion", Some("filled_map.mansion"));
        let mut item = original.clone();
        convert_item_stack(&mut item, VERSION - 1, VERSION);
        assert_eq!(
            item.get_string("id"),
            Some("minecraft:woodland_explorer_map")
        );
        let c = item.get_map("components").unwrap();
        assert!(!c.has_key("minecraft:item_name"));
        assert!(!c.has_key("minecraft:map_color"));
        assert_eq!(c.get_i32("minecraft:map_id"), Some(7));

        let report = convert_item_stack_reverse(&mut item, VERSION, VERSION - 1);
        assert!(report.is_empty());
        assert_eq!(item, original);
    }

    #[test]
    fn custom_name_is_kept_and_unknown_decoration_ignored() {
        let mut item = explorer_map("minecraft:monument", Some("my.custom.key"));
        convert_item_stack(&mut item, VERSION - 1, VERSION);
        assert_eq!(item.get_string("id"), Some("minecraft:ocean_explorer_map"));
        assert!(item
            .get_map("components")
            .unwrap()
            .has_key("minecraft:item_name"));

        let mut other = explorer_map("minecraft:target_x", None);
        convert_item_stack(&mut other, VERSION - 1, VERSION);
        assert_eq!(other.get_string("id"), Some(FILLED_MAP));
        // map_color is removed from every item.
        assert!(!other
            .get_map("components")
            .unwrap()
            .has_key("minecraft:map_color"));
    }

    #[test]
    fn buried_treasure_map_reverse_has_no_color() {
        let mut item = explorer_map("minecraft:red_x", Some("filled_map.buried_treasure"));
        convert_item_stack(&mut item, VERSION - 1, VERSION);
        assert_eq!(item.get_string("id"), Some("minecraft:buried_treasure_map"));
        convert_item_stack_reverse(&mut item, VERSION, VERSION - 1);
        let c = item.get_map("components").unwrap();
        assert_eq!(item.get_string("id"), Some(FILLED_MAP));
        assert!(!c.has_key("minecraft:map_color"));
        assert_eq!(
            plain_translation_key(c.get("minecraft:item_name")),
            Some("filled_map.buried_treasure")
        );
    }
}
