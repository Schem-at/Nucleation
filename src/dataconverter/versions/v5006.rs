//! V5006 (26.3 snapshot) — block-state NBT field names.
//!
//! Upstream DataConverter has no step past V4892 yet; this ports Mojang's 26.3
//! `net.minecraft.util.datafix.fixes.BlockStateFieldNamesFix` (registered at
//! schema 5006 in `DataFixers`): every BLOCK_STATE compound renames
//! `Name` -> `id` and `Properties` -> `properties` (DFU `renameField`, a no-op
//! for an absent field). 26.3's `NbtUtils.readBlockState` / `writeBlockState`
//! and the `StateHolder` codec use only the new names, so this is
//! schematic-relevant: it reaches palette entries and every block state nested
//! in entities / block entities (falling blocks, endermen, block displays, …).
//! Mojang renamed every earlier block fixer to `Legacy*` (`LegacyBlockRenameFix`,
//! `LegacyBlockPropertyRenameAndFix`, …) because they run before this step and
//! keep reading `Name` — exactly the order the converter chain here already
//! has, so the older `Name`-keyed BLOCK_STATE converters are unaffected.
//!
//! Callers that bridge a block state to/from NBT (the palette bridge in
//! `apply.rs`, the format readers/writers) pick the field names for a data
//! version with [`block_state_keys`].
//!
//! Reverse: the exact inverse rename (bucket A, lossless).
//!
//! VERSION = 5006 (Mojang schema id).

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 5006;

/// The first data version whose BLOCK_STATE compounds use `id` / `properties`
/// instead of `Name` / `Properties` (Mojang `BlockStateFieldNamesFix`; also the
/// `version >= 5006` switch in 26.3 `NbtUtils.packBlockState`).
pub const BLOCK_STATE_FIELD_NAMES_VERSION: i32 = VERSION;

/// The `(name, properties)` field names a block-state compound uses at
/// `data_version`: `("Name", "Properties")` before 5006, `("id", "properties")`
/// from 5006 on.
pub const fn block_state_keys(data_version: i32) -> (&'static str, &'static str) {
    if data_version >= BLOCK_STATE_FIELD_NAMES_VERSION {
        ("id", "properties")
    } else {
        ("Name", "Properties")
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.block_state.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            data.rename_key("Name", "id");
            data.rename_key("Properties", "properties");
        }),
    );
    // Reverse (lossless): id -> Name, properties -> Properties.
    reg.block_state.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            data.rename_key("id", "Name");
            data.rename_key("properties", "Properties");
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::{
        convert_block_state, convert_block_state_reverse, convert_entity, convert_entity_reverse,
    };

    fn legacy_state() -> NbtMap {
        let mut props = NbtMap::new();
        props.set_string("facing", "north");
        let mut bs = NbtMap::new();
        bs.set_string("Name", "minecraft:furnace");
        bs.set_map("Properties", props);
        bs
    }

    #[test]
    fn keys_switch_at_5006() {
        assert_eq!(block_state_keys(VERSION - 1), ("Name", "Properties"));
        assert_eq!(block_state_keys(VERSION), ("id", "properties"));
    }

    #[test]
    fn block_state_fields_renamed_and_reversed() {
        let mut bs = legacy_state();
        convert_block_state(&mut bs, VERSION - 1, VERSION);
        assert_eq!(bs.get_string("id"), Some("minecraft:furnace"));
        assert!(!bs.has_key("Name") && !bs.has_key("Properties"));
        assert_eq!(
            bs.get_map("properties").unwrap().get_string("facing"),
            Some("north")
        );

        let report = convert_block_state_reverse(&mut bs, VERSION, VERSION - 1);
        assert!(report.is_empty());
        assert_eq!(bs, legacy_state());
    }

    #[test]
    fn older_block_renames_still_apply_across_the_rename() {
        // V2680 grass_path -> dirt_path runs on `Name` before V5006 renames it.
        let mut bs = NbtMap::new();
        bs.set_string("Name", "minecraft:grass_path");
        convert_block_state(&mut bs, 2679, VERSION);
        assert_eq!(bs.get_string("id"), Some("minecraft:dirt_path"));

        let report = convert_block_state_reverse(&mut bs, VERSION, 2679);
        assert!(report.is_empty());
        assert_eq!(bs.get_string("Name"), Some("minecraft:grass_path"));
    }

    #[test]
    fn structure_palette_and_palettes_renamed() {
        use crate::dataconverter::types::ValueExt;
        use crate::dataconverter::{convert_structure, convert_structure_reverse};
        use crate::nbt::NbtValue;

        let mut root = NbtMap::new();
        root.set_list("palette", vec![NbtValue::Compound(legacy_state())]);
        root.set_list(
            "palettes",
            vec![NbtValue::List(vec![NbtValue::Compound(legacy_state())])],
        );
        convert_structure(&mut root, VERSION - 1, VERSION);
        let first = root.get_list("palette").unwrap()[0]
            .as_compound_ref()
            .unwrap();
        assert_eq!(first.get_string("id"), Some("minecraft:furnace"));
        // `palettes` (walked since Mojang's 26.3 STRUCTURE schema) too.
        let alt = root.get_list("palettes").unwrap()[0].as_list_ref().unwrap()[0]
            .as_compound_ref()
            .unwrap();
        assert_eq!(alt.get_string("id"), Some("minecraft:furnace"));

        let report = convert_structure_reverse(&mut root, VERSION, VERSION - 1);
        assert!(report.is_empty());
        let alt = root.get_list("palettes").unwrap()[0].as_list_ref().unwrap()[0]
            .as_compound_ref()
            .unwrap();
        assert_eq!(alt, &legacy_state());
    }

    #[test]
    fn nested_entity_block_state_renamed() {
        let mut e = NbtMap::new();
        e.set_string("id", "minecraft:falling_block");
        e.set_map("BlockState", legacy_state());
        convert_entity(&mut e, VERSION - 1, VERSION);
        assert_eq!(
            e.get_map("BlockState").unwrap().get_string("id"),
            Some("minecraft:furnace")
        );
        let report = convert_entity_reverse(&mut e, VERSION, VERSION - 1);
        assert!(report.is_empty());
        assert_eq!(e.get_map("BlockState").unwrap(), &legacy_state());
    }
}
