//! V3814 (24w05b + 3) — schematic-relevant subset of `V3814.java`.
//!
//! The "old attributes" rename `minecraft:horse.jump_strength` ->
//! `minecraft:generic.jump_strength`, applied via
//! `ConverterAbstractOldAttributesRename` (V3814.java:14-18).
//!
//! That helper renames the `Name` field of each entry in an entity's
//! `Attributes` list and the `AttributeName` field of each entry in an item's
//! `AttributeModifiers` list. (The PLAYER registration it also performs is
//! non-schematic and skipped.) The lookup is exact — no namespace correction.
//!
//! VERSION = MCVersions.V24W05B (3811) + 3 = 3814.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 3814;

/// `(old, new)` attribute renames (V3814.java:14-16).
const ATTRIBUTE_RENAMES: &[(&str, &str)] =
    &[("minecraft:horse.jump_strength", "minecraft:generic.jump_strength")];

fn rename(name: &str) -> Option<&'static str> {
    ATTRIBUTE_RENAMES.iter().find(|(old, _)| *old == name).map(|(_, new)| *new)
}

/// Inverse lookup: map the new name back to the old name. The forward rename is
/// one-to-one, so this is exact (lossless).
fn rename_inverse(name: &str) -> Option<&'static str> {
    ATTRIBUTE_RENAMES.iter().find(|(_, new)| *new == name).map(|(old, _)| *old)
}

/// Inverse of `rename_list_field`: rename `data[list][].key` new -> old.
fn rename_list_field_inverse(data: &mut NbtMap, list: &str, key: &str) {
    if let Some(list) = data.get_list_mut(list) {
        for el in list.iter_mut() {
            if let Some(map) = el.as_compound_mut() {
                if let Some(value) = map.get_string(key) {
                    if let Some(old) = rename_inverse(value) {
                        map.set_string(key, old);
                    }
                }
            }
        }
    }
}

/// `RenameHelper.renameString(map, key, renamer)` over every map in `data[list]`.
fn rename_list_field(data: &mut NbtMap, list: &str, key: &str) {
    if let Some(list) = data.get_list_mut(list) {
        for el in list.iter_mut() {
            if let Some(map) = el.as_compound_mut() {
                if let Some(value) = map.get_string(key) {
                    if let Some(new) = rename(value) {
                        map.set_string(key, new);
                    }
                }
            }
        }
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    // ENTITY: Attributes[].Name. (PLAYER registration is non-schematic, skipped.)
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| rename_list_field(data, "Attributes", "Name")),
    );
    // REVERSE: rename Attributes[].Name new -> old (one-to-one, lossless).
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            rename_list_field_inverse(data, "Attributes", "Name")
        }),
    );

    // ITEM_STACK: AttributeModifiers[].AttributeName.
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            rename_list_field(data, "AttributeModifiers", "AttributeName")
        }),
    );
    // REVERSE: rename AttributeModifiers[].AttributeName new -> old (lossless).
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            rename_list_field_inverse(data, "AttributeModifiers", "AttributeName")
        }),
    );
}
