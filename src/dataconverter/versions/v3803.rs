//! V3803 (23w51b + 1) — schematic-relevant subset of `V3803.java`.
//!
//! A single enchantment id rename (`minecraft:sweeping` ->
//! `minecraft:sweeping_edge`) applied to ITEM_STACK via
//! `ConverterEnchantmentsRename` (V3803.java:16-21).
//!
//! `ConverterEnchantmentsRename` renames the `id` field of each map in the
//! `tag.Enchantments` and `tag.StoredEnchantments` lists. Critically, the input
//! id is first passed through `NamespaceUtil.correctNamespace` before the lookup,
//! so a legacy unnamespaced `sweeping` still matches. The engine has no exported
//! `correctNamespace`, so the parse rule is inlined here (mirrors
//! `helpers::correct_namespace_or_null`).
//!
//! VERSION = MCVersions.V23W51B (3802) + 1 = 3803.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 3803;

/// `(old, new)` enchantment renames (V3803.java:16-18). Matched against the
/// namespace-corrected id.
const ENCHANTMENT_RENAMES: &[(&str, &str)] = &[("minecraft:sweeping", "minecraft:sweeping_edge")];

/// Port of `NamespaceUtil.correctNamespace`: default-namespace an unnamespaced,
/// parseable resource location; otherwise return the input unchanged.
fn correct_namespace(value: &str) -> String {
    if value.contains(':') {
        return value.to_string();
    }
    // Only a valid path (no illegal chars) gets the implicit `minecraft:`
    // namespace; anything else parses as-null and is returned unchanged.
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

/// `renamer.apply(correctNamespace(input))` (ConverterEnchantmentsRename:19-22).
fn rename(id: &str) -> Option<&'static str> {
    let corrected = correct_namespace(id);
    ENCHANTMENT_RENAMES
        .iter()
        .find(|(old, _)| *old == corrected)
        .map(|(_, new)| *new)
}

/// Inverse of `rename`: map the new id back to the old id. The new id
/// (`minecraft:sweeping_edge`) uniquely encodes the old id, so this is exact.
fn rename_reverse(id: &str) -> Option<&'static str> {
    let corrected = correct_namespace(id);
    ENCHANTMENT_RENAMES
        .iter()
        .find(|(_, new)| *new == corrected)
        .map(|(old, _)| *old)
}

/// `RenameHelper.renameListMapItems(tag, listPath, "id", renamer)`.
fn rename_list_map_ids(tag: &mut NbtMap, list_path: &str) {
    if let Some(list) = tag.get_list_mut(list_path) {
        for el in list.iter_mut() {
            if let Some(map) = el.as_compound_mut() {
                if let Some(id) = map.get_string("id") {
                    if let Some(new) = rename(id) {
                        map.set_string("id", new);
                    }
                }
            }
        }
    }
}

/// Reverse of `rename_list_map_ids`: rename `id` fields back to their old ids.
fn rename_list_map_ids_reverse(tag: &mut NbtMap, list_path: &str) {
    if let Some(list) = tag.get_list_mut(list_path) {
        for el in list.iter_mut() {
            if let Some(map) = el.as_compound_mut() {
                if let Some(id) = map.get_string("id") {
                    if let Some(old) = rename_reverse(id) {
                        map.set_string("id", old);
                    }
                }
            }
        }
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let tag = match data.get_map_mut("tag") {
                Some(t) => t,
                None => return,
            };
            rename_list_map_ids(tag, "Enchantments");
            rename_list_map_ids(tag, "StoredEnchantments");
        }),
    );

    // Reverse: `minecraft:sweeping_edge` -> `minecraft:sweeping`. Lossless
    // rename — the new id uniquely encodes the old one (bucket A).
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let tag = match data.get_map_mut("tag") {
                Some(t) => t,
                None => return,
            };
            rename_list_map_ids_reverse(tag, "Enchantments");
            rename_list_map_ids_reverse(tag, "StoredEnchantments");
        }),
    );
}
