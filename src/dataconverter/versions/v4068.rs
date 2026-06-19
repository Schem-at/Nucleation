//! V4068 (24w38a + 2) — schematic-relevant subset of `V4068.java`.
//!
//! The `Lock` string (a raw key item name) becomes a predicate compound:
//!   `{ components: { "minecraft:custom_name": "<escaped lock>" } }`
//! where the lock text is escaped for embedding in a string-quoted component
//! (`"` -> `\"`, `\` -> `\\`). An empty lock string drops the key entirely.
//!
//!   * ITEM_STACK structure converter: `components["minecraft:lock"]` (string)
//!     -> `components["minecraft:lock"]` (predicate compound).
//!   * TILE_ENTITY structure converter: `Lock` (string) -> `lock` (predicate
//!     compound).
//!
//! Nothing non-schematic is present in this version.
//!
//! VERSION = MCVersions.V24W38A (4066) + 2 = 4068.

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 4068;

/// Escaper(`"` -> `\"`, `\` -> `\\`) — Guava `Escapers` used by V4068.java.
fn escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(c),
        }
    }
    out
}

/// `convertLock`: move the (string) lock at `src_path` to a predicate compound at
/// `dst_path`. A non-string or empty value is removed without a replacement.
fn convert_lock(root: &mut NbtMap, src_path: &str, dst_path: &str) {
    let lock_generic = match root.take(src_path) {
        Some(v) => v,
        None => return,
    };

    if let Some(lock) = lock_generic.as_str() {
        if !lock.is_empty() {
            let mut lock_components = NbtMap::new();
            lock_components.set_string("minecraft:custom_name", escape(lock));

            let mut new_lock = NbtMap::new();
            new_lock.set_map("components", lock_components);

            root.set_map(dst_path, new_lock);
        }
    }
}

/// Inverse of `escape`: undo the embedding escapes (`\"` -> `"`, `\\` -> `\`).
/// `escape` only ever produces these two two-char sequences, so this is an exact
/// left inverse of every string it could have written.
fn unescape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                // A lone/foreign backslash never appears in forward output; keep
                // it verbatim (with whatever followed) so we lose nothing.
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Reverse of `convertLock`: collapse the predicate compound at `dst_path` back to
/// the raw (string) lock item name at `src_path`.
///
/// The forward step is a lossless structural wrap: the lock name is escaped and
/// stored at `components.minecraft:custom_name`, and `escape`/`unescape` are exact
/// inverses, so for any forward-produced lock this restores the original string.
/// Anything that is not the expected `{components:{minecraft:custom_name:<str>}}`
/// shape (e.g. an empty or absent predicate) leaves nothing behind — matching the
/// forward step, which drops empty/non-string locks entirely.
fn reverse_convert_lock(root: &mut NbtMap, src_path: &str, dst_path: &str) {
    let new_lock = match root.take(dst_path) {
        Some(v) => v,
        None => return,
    };

    let Some(lock_map) = new_lock.as_compound_ref() else {
        report_loss(
            VERSION,
            LossKind::ComponentDropped,
            Severity::Loss,
            "lock predicate was not a compound and cannot be represented as a legacy lock string",
        );
        return;
    };
    let Some(components) = lock_map.get_map("components") else {
        report_loss(
            VERSION,
            LossKind::ComponentDropped,
            Severity::Loss,
            "lock predicate had no components.minecraft:custom_name string; dropped unsupported lock predicate",
        );
        return;
    };
    let Some(lock_name) = components.get_string("minecraft:custom_name") else {
        report_loss(
            VERSION,
            LossKind::ComponentDropped,
            Severity::Loss,
            "lock predicate had no components.minecraft:custom_name string; dropped unsupported lock predicate",
        );
        return;
    };

    if lock_map.inner().len() != 1 || components.inner().len() != 1 {
        report_loss(
            VERSION,
            LossKind::ComponentDropped,
            Severity::Loss,
            "lock predicate had fields beyond components.minecraft:custom_name; dropped extras while restoring legacy lock string",
        );
    }
    root.set_string(src_path, unescape(lock_name));
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(components) = data.get_map_mut("components") {
                convert_lock(components, "minecraft:lock", "minecraft:lock");
            }
        }),
    );
    // Reverse of the ITEM_STACK lock wrap (V4068.java:42-54): predicate compound
    // at components["minecraft:lock"] -> raw string at components["minecraft:lock"].
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(components) = data.get_map_mut("components") {
                reverse_convert_lock(components, "minecraft:lock", "minecraft:lock");
            }
        }),
    );

    reg.tile_entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            convert_lock(data, "Lock", "lock");
        }),
    );
    // Reverse of the TILE_ENTITY lock wrap (V4068.java:55-61): predicate compound
    // at `lock` -> raw string at `Lock`.
    reg.tile_entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            reverse_convert_lock(data, "Lock", "lock");
        }),
    );
}
