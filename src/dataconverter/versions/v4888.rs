//! V4888 (26.2-snapshot-4 + 1) — schematic-relevant subset of `V4888.java`.
//!
//! The `minecraft:nameplate_distance` attribute is renamed to
//! `minecraft:name_tag_distance` (Mojang `RenameNameplateToNameTagFix`),
//! registered upstream through
//! `ConverterAbstractPostV4307AttributesRename.register(VERSION, V4888::rename)`
//! (`converters/attributes/ConverterAbstractPostV4307AttributesRename.java`).
//! Ported (PLAYER is skipped, as in `v4055.rs`):
//!   * DATA_COMPONENTS converter: rename `type` on each entry of the
//!     `minecraft:attribute_modifiers` list (post-V4307 the component *is* the
//!     list) — ConverterAbstractPostV4307AttributesRename.java:18-31.
//!   * ENTITY converter: rename `id` on each `attributes[]` entry —
//!     ConverterAbstractPostV4307AttributesRename.java:34-47.
//!
//! Reverse: the rename is one-to-one (`name_tag_distance` did not exist before
//! this version), so the inverse is exact and lossless (bucket A).
//!
//! VERSION = MCVersions.V26_2_SNAPSHOT4 (4887) + 1 = 4888.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 4888;

const OLD: &str = "minecraft:nameplate_distance";
const NEW: &str = "minecraft:name_tag_distance";

/// `RenameHelper.renameString(map, key, renamer)` with a single-pair renamer.
fn rename_in_list(list: Option<&mut Vec<crate::nbt::NbtValue>>, key: &str, from: &str, to: &str) {
    let Some(list) = list else { return };
    for el in list.iter_mut() {
        if let Some(m) = el.as_compound_mut() {
            if m.get_string(key) == Some(from) {
                m.set_string(key, to);
            }
        }
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    // DATA_COMPONENTS: attribute_modifiers[].type.
    reg.data_components.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            rename_in_list(
                data.get_list_mut("minecraft:attribute_modifiers"),
                "type",
                OLD,
                NEW,
            );
        }),
    );
    // Reverse (lossless): name_tag_distance -> nameplate_distance.
    reg.data_components.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            rename_in_list(
                data.get_list_mut("minecraft:attribute_modifiers"),
                "type",
                NEW,
                OLD,
            );
        }),
    );

    // ENTITY: attributes[].id.
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            rename_in_list(data.get_list_mut("attributes"), "id", OLD, NEW);
        }),
    );
    // Reverse (lossless).
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            rename_in_list(data.get_list_mut("attributes"), "id", NEW, OLD);
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::{
        convert_entity, convert_entity_reverse, convert_item_stack, convert_item_stack_reverse,
    };
    use crate::nbt::NbtValue;

    fn attr(key: &str, id: &str) -> NbtValue {
        let mut m = NbtMap::new();
        m.set_string(key, id);
        m.set_f64("amount", 1.0);
        NbtValue::Compound(m)
    }

    #[test]
    fn entity_attribute_renamed_and_reversed() {
        let mut e = NbtMap::new();
        e.set_string("id", "minecraft:zombie");
        e.set_list(
            "attributes",
            vec![attr("id", OLD), attr("id", "minecraft:max_health")],
        );

        convert_entity(&mut e, VERSION - 1, VERSION);
        let attrs = e.get_list("attributes").unwrap();
        assert_eq!(
            attrs[0].as_compound_ref().unwrap().get_string("id"),
            Some(NEW)
        );
        assert_eq!(
            attrs[1].as_compound_ref().unwrap().get_string("id"),
            Some("minecraft:max_health")
        );

        let report = convert_entity_reverse(&mut e, VERSION, VERSION - 1);
        assert!(report.is_empty());
        let attrs = e.get_list("attributes").unwrap();
        assert_eq!(
            attrs[0].as_compound_ref().unwrap().get_string("id"),
            Some(OLD)
        );
    }

    #[test]
    fn item_attribute_modifier_renamed_and_reversed() {
        let mut components = NbtMap::new();
        components.set_list("minecraft:attribute_modifiers", vec![attr("type", OLD)]);
        let mut item = NbtMap::new();
        item.set_string("id", "minecraft:diamond_helmet");
        item.set_i32("count", 1);
        item.set_map("components", components);

        convert_item_stack(&mut item, VERSION - 1, VERSION);
        let modifiers = item
            .get_map("components")
            .unwrap()
            .get_list("minecraft:attribute_modifiers")
            .unwrap();
        assert_eq!(
            modifiers[0].as_compound_ref().unwrap().get_string("type"),
            Some(NEW)
        );

        let report = convert_item_stack_reverse(&mut item, VERSION, VERSION - 1);
        assert!(report.is_empty());
        let modifiers = item
            .get_map("components")
            .unwrap()
            .get_list("minecraft:attribute_modifiers")
            .unwrap();
        assert_eq!(
            modifiers[0].as_compound_ref().unwrap().get_string("type"),
            Some(OLD)
        );
    }
}
