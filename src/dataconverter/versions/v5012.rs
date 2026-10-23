//! V5012 (26.3 snapshot) — explorer map item renames.
//!
//! Upstream DataConverter has no step past V4892 yet; this ports Mojang's 26.3
//! "Rename explorer map items" `ItemRenameFix` (registered at schema 5012 in
//! `net.minecraft.util.datafix.DataFixers`): five of the map items introduced
//! at V5008 (`v5008.rs`) get their final names. ITEM_NAME is
//! schematic-relevant (item stacks), so this is ported in full.
//!
//! Reverse: the table is injective, so the engine's automatic inverse of the
//! `map_renamer` table is exact (cheatsheet rule 1).
//!
//! VERSION = 5012 (Mojang schema id).

use super::super::helpers::{map_renamer, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 5012;

/// `(old, new)` from the `ImmutableMap.of(...)` in `DataFixers` (schema 5012).
pub const EXPLORER_MAP_RENAMES: &[(&str, &str)] = &[
    (
        "minecraft:ocean_explorer_map",
        "minecraft:ocean_monument_map",
    ),
    ("minecraft:swamp_explorer_map", "minecraft:swamp_hut_map"),
    (
        "minecraft:trial_explorer_map",
        "minecraft:buried_trial_chambers_map",
    ),
    (
        "minecraft:woodland_explorer_map",
        "minecraft:woodland_mansion_map",
    ),
    (
        "minecraft:jungle_explorer_map",
        "minecraft:jungle_pyramid_map",
    ),
];

pub fn register(reg: &mut RegistryBuilder) {
    register_item_rename(reg, VERSION, map_renamer(EXPLORER_MAP_RENAMES));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::types::MapExt;
    use crate::dataconverter::{convert_item_stack, convert_item_stack_reverse};
    use crate::nbt::{NbtMap, NbtValue};

    #[test]
    fn explorer_maps_renamed_and_reversed() {
        for (old, new) in EXPLORER_MAP_RENAMES {
            let mut item = NbtMap::new();
            item.set_string("id", *old);
            item.set_i32("count", 1);
            convert_item_stack(&mut item, VERSION - 1, VERSION);
            assert_eq!(item.get_string("id"), Some(*new));
            let report = convert_item_stack_reverse(&mut item, VERSION, VERSION - 1);
            assert!(report.is_empty());
            assert_eq!(item.get_string("id"), Some(*old));
        }
    }

    #[test]
    fn filled_explorer_map_goes_through_both_steps() {
        // 26.2 mansion map -> V5008 woodland_explorer_map -> V5012 woodland_mansion_map,
        // and all the way back.
        let mut plus = NbtMap::new();
        plus.set_string("type", "minecraft:mansion");
        let mut decorations = NbtMap::new();
        decorations.set_map("+", plus);
        let mut name = NbtMap::new();
        name.set_string("translate", "filled_map.mansion");
        let mut components = NbtMap::new();
        components.set_map("minecraft:map_decorations", decorations);
        components.set_map("minecraft:item_name", name);
        components.set_generic("minecraft:map_color", NbtValue::Int(5393476));
        let mut item = NbtMap::new();
        item.set_string("id", "minecraft:filled_map");
        item.set_i32("count", 1);
        item.set_map("components", components);
        let original = item.clone();

        convert_item_stack(&mut item, 4903, VERSION);
        assert_eq!(
            item.get_string("id"),
            Some("minecraft:woodland_mansion_map")
        );

        let report = convert_item_stack_reverse(&mut item, VERSION, 4903);
        assert!(report.is_empty());
        assert_eq!(item, original);
    }
}
