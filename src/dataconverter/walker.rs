//! Walker descent primitives (`WalkerUtils`) and the reusable `DataWalker*`
//! constructors. Walkers do not mutate their own node — they locate nested
//! typed sub-structures and recurse by calling another type's `convert`.

use std::sync::Arc;

use crate::nbt::NbtMap;

use super::engine::{DataType, MCValueType, Walker};
use super::loss;
use super::registry::Registry;
use super::types::{MapExt, ValueExt};
use super::version::EncodedVersion;

// --- compound-type descent (WalkerUtils, MCDataType overloads) --------------

/// Convert the single compound at `data[path]` — `WalkerUtils.convert`.
pub fn convert(
    reg: &Registry,
    ty: &DataType,
    data: &mut NbtMap,
    path: &str,
    from: EncodedVersion,
    to: EncodedVersion,
) {
    if let Some(map) = data.get_map_mut(path) {
        let _scope = loss::path_scope(path);
        ty.convert(reg, map, from, to);
    }
}

/// Convert every compound in the list `data[path]` — `WalkerUtils.convertList`.
pub fn convert_list(
    reg: &Registry,
    ty: &DataType,
    data: &mut NbtMap,
    path: &str,
    from: EncodedVersion,
    to: EncodedVersion,
) {
    if let Some(list) = data.get_list_mut(path) {
        for (i, el) in list.iter_mut().enumerate() {
            if let Some(map) = el.as_compound_mut() {
                let _scope = loss::path_scope(format!("{path}[{i}]"));
                ty.convert(reg, map, from, to);
            }
        }
    }
}

/// For each compound in `data[list_path]`, convert its `[element_path]` child —
/// `WalkerUtils.convertListPath` (two-arg form). This is how the STRUCTURE root
/// reaches `entities[].nbt` / `blocks[].nbt`.
pub fn convert_list_path(
    reg: &Registry,
    ty: &DataType,
    data: &mut NbtMap,
    list_path: &str,
    element_path: &str,
    from: EncodedVersion,
    to: EncodedVersion,
) {
    if let Some(list) = data.get_list_mut(list_path) {
        for (i, el) in list.iter_mut().enumerate() {
            if let Some(map) = el.as_compound_mut() {
                if let Some(child) = map.get_map_mut(element_path) {
                    let _scope = loss::path_scope(format!("{list_path}[{i}].{element_path}"));
                    ty.convert(reg, child, from, to);
                }
            }
        }
    }
}

// --- value-type descent (WalkerUtils, DataType<Object,Object> overloads) ----

/// Convert the single value at `data[path]` through a value type.
pub fn convert_value(
    ty: &MCValueType,
    data: &mut NbtMap,
    path: &str,
    from: EncodedVersion,
    to: EncodedVersion,
) {
    if let Some(v) = data.get_mut(path) {
        ty.convert(v, from, to);
    }
}

/// Convert every element of the list `data[path]` through a value type.
pub fn convert_value_list(
    ty: &MCValueType,
    data: &mut NbtMap,
    path: &str,
    from: EncodedVersion,
    to: EncodedVersion,
) {
    if let Some(list) = data.get_list_mut(path) {
        for el in list.iter_mut() {
            ty.convert(el, from, to);
        }
    }
}

// --- reusable DataWalker* constructors --------------------------------------
// These build a `Walker` closure that recurses a fixed type over fixed paths,
// mirroring DataWalkerItemLists / DataWalkerItems / DataWalkerTileEntities /
// DataWalkerBlockNames / DataWalkerItemNames.

/// `DataWalkerItemLists(paths…)` — each path is a *list* of itemstacks.
pub fn item_lists(paths: &'static [&'static str]) -> Walker {
    Arc::new(move |reg, data, from, to| {
        for p in paths {
            convert_list(reg, &reg.item_stack, data, p, from, to);
        }
    })
}

/// `DataWalkerItems(paths…)` — each path is a *single* itemstack.
pub fn items(paths: &'static [&'static str]) -> Walker {
    Arc::new(move |reg, data, from, to| {
        for p in paths {
            convert(reg, &reg.item_stack, data, p, from, to);
        }
    })
}

/// `DataWalkerTileEntities(paths…)` — each path is a single block entity.
pub fn tile_entities(paths: &'static [&'static str]) -> Walker {
    Arc::new(move |reg, data, from, to| {
        for p in paths {
            convert(reg, &reg.tile_entity, data, p, from, to);
        }
    })
}

/// `DataWalkerBlockNames(paths…)` — each path is a block-id string.
pub fn block_names(paths: &'static [&'static str]) -> Walker {
    Arc::new(move |reg, data, from, to| {
        for p in paths {
            convert_value(&reg.block_name, data, p, from, to);
        }
    })
}

/// `DataWalkerItemNames(paths…)` — each path is an item-id string.
pub fn item_names(paths: &'static [&'static str]) -> Walker {
    Arc::new(move |reg, data, from, to| {
        for p in paths {
            convert_value(&reg.item_name, data, p, from, to);
        }
    })
}
