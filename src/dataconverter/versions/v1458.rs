//! V1458 (17w50a+1) — schematic-relevant subset of `V1458.java`.
//!
//! VERSION = MCVersions.V17W50A + 1 = 1457 + 1 = 1458.
//!
//! Ported:
//!   * ENTITY structure converter (V1458.java:46-55): for every entity *except*
//!     `minecraft:commandblock_minecart`, run `updateCustomName` — a non-empty
//!     plain-string `CustomName` becomes a `{"text":…}` text component, an
//!     empty/absent one is removed.
//!   * ITEM_STACK structure converter (V1458.java:57-83): a plain-string
//!     `tag.display.Name` is wrapped into a `{"text":…}` text component. The
//!     `LocName` branch is commented out in Java (1.20.5 removed
//!     ItemCustomNameToComponentFix), so it is intentionally not ported.
//!   * TILE_ENTITY structure converter (V1458.java:85-94): for every tile entity
//!     *except* `minecraft:command_block`, run `updateCustomName` on `CustomName`.
//!   * ENTITY structure walker (V1458.java:96-101): walk the `Passengers` list as
//!     ENTITY, walk `CustomName` as TEXT_COMPONENT, then run
//!     ENTITY_EQUIPMENT.convert. This supersedes the V135 entity structure walker
//!     by adding the CustomName TEXT_COMPONENT descent.
//!   * `named` / `namedInventory` TILE_ENTITY walkers (V1458.java:28-35,128-138)
//!     for beacon, banner, brewing_stand, chest, trapped_chest, dispenser,
//!     dropper, enchanting_table, furnace, hopper, shulker_box. `named` routes
//!     `CustomName` through TEXT_COMPONENT; `namedInventory` also walks `Items`.
//!
//! Skipped (non-schematic): the PLAYER structure converter (V1458.java:39-44) and
//! the PLAYER structure walker (V1458.java:105-126) — PLAYER never appears in a
//! schematic file. The command-block / commandblock_minecart cases that V1458
//! excludes here are handled at V1488 (see `v1488.rs`).

use std::sync::Arc;

use crate::nbt::NbtMap;

use super::super::helpers::create_plain_text_component;
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::{convert, convert_list, item_lists};

const VERSION: i32 = 1458;

/// `V1458.updateCustomName` (V1458.java:16-26): a non-empty plain-string
/// `CustomName` becomes a JSON text component `{"text":"<name>"}`; an
/// empty/absent one is removed.
fn update_custom_name(data: &mut NbtMap) {
    let name = data.get_string("CustomName").unwrap_or("").to_string();
    if name.is_empty() {
        data.take("CustomName");
    } else {
        data.set_string("CustomName", create_plain_text_component(&name));
    }
}

/// Inverse of `update_custom_name` (and the item `tag.display.Name` wrap): unwrap
/// a single-key plain-text component `{"text": s}` back to the raw legacy string
/// `s`.
///
/// The forward (`ComponentUtils.createPlainTextComponent`) only ever produces
/// `{"text": <raw string>}` from a non-empty legacy `CustomName`/`Name`. That
/// mapping is injective — the single `"text"` value is exactly the original raw
/// string (even if that string itself looked like JSON, it was stored as the
/// string value), so this unwrap is an exact, lossless inverse for any real
/// downgrade. The empty/absent case the forward *removed* leaves nothing to
/// invert (absence stays absence). Anything that is not a single-key `{"text":…}`
/// string component is foreign to the forward output and is left untouched.
fn revert_custom_name(data: &mut NbtMap, path: &str) {
    let text = match data.get_string(path).map(str::to_string) {
        Some(t) => t,
        None => return,
    };
    let parsed: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        // Forward only ever emits JSON here; non-JSON is foreign data — leave it.
        Err(_) => return,
    };
    if let serde_json::Value::Object(obj) = &parsed {
        if obj.len() == 1 {
            if let Some(serde_json::Value::String(s)) = obj.get("text") {
                data.set_string(path, s);
            }
        }
    }
}

/// `named(version, id)` (V1458.java:28-30): route `CustomName` through
/// TEXT_COMPONENT for the given tile-entity id.
fn named(reg: &mut RegistryBuilder, id: &'static str) {
    reg.tile_entity.add_walker(
        VERSION,
        0,
        id,
        Arc::new(|reg, data, from, to| {
            convert(reg, &reg.text_component, data, "CustomName", from, to);
        }),
    );
}

/// `namedInventory(version, id)` (V1458.java:32-35): `named` plus a
/// `DataWalkerItemLists("Items")`.
fn named_inventory(reg: &mut RegistryBuilder, id: &'static str) {
    named(reg, id);
    reg.tile_entity
        .add_walker(VERSION, 0, id, item_lists(&["Items"]));
}

pub fn register(reg: &mut RegistryBuilder) {
    // ENTITY: updateCustomName for every entity except commandblock_minecart
    // (V1458.java:46-55). The commandblock_minecart case is handled at V1488.
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_string("id") == Some("minecraft:commandblock_minecart") {
                return;
            }
            update_custom_name(data);
        }),
    );

    // Reverse of the ENTITY converter: unwrap the `{"text":…}` CustomName back to
    // a raw legacy string. Mirror the same commandblock_minecart exclusion as the
    // forward (it never wrapped that entity, so we must not unwrap it either).
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_string("id") == Some("minecraft:commandblock_minecart") {
                return;
            }
            revert_custom_name(data, "CustomName");
        }),
    );

    // ITEM_STACK: wrap a plain-string tag.display.Name into a text component
    // (V1458.java:57-83). The LocName branch is dead in Java, so it is skipped.
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let Some(tag) = data.get_map_mut("tag") else {
                return;
            };
            let Some(display) = tag.get_map_mut("display") else {
                return;
            };
            if let Some(name) = display.get_string("Name").map(str::to_string) {
                display.set_string("Name", create_plain_text_component(&name));
            }
        }),
    );

    // Reverse of the ITEM_STACK converter: unwrap the `{"text":…}` tag.display.Name
    // back to its raw legacy string (same single-key plain-text inverse).
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let Some(tag) = data.get_map_mut("tag") else {
                return;
            };
            let Some(display) = tag.get_map_mut("display") else {
                return;
            };
            revert_custom_name(display, "Name");
        }),
    );

    // TILE_ENTITY: updateCustomName for every tile entity except command_block
    // (V1458.java:85-94). The command_block case is handled at V1488.
    reg.tile_entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_string("id") == Some("minecraft:command_block") {
                return;
            }
            update_custom_name(data);
        }),
    );

    // Reverse of the TILE_ENTITY converter: unwrap the `{"text":…}` CustomName back
    // to a raw legacy string, mirroring the forward's command_block exclusion.
    reg.tile_entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_string("id") == Some("minecraft:command_block") {
                return;
            }
            revert_custom_name(data, "CustomName");
        }),
    );

    // ENTITY structure walker (V1458.java:96-101): Passengers (ENTITY) +
    // CustomName (TEXT_COMPONENT) + entity equipment. Supersedes the V135 walker.
    reg.entity.add_structure_walker(
        VERSION,
        0,
        Arc::new(|reg, data, from, to| {
            convert_list(reg, &reg.entity, data, "Passengers", from, to);
            convert(reg, &reg.text_component, data, "CustomName", from, to);
            reg.entity_equipment.convert(reg, data, from, to);
        }),
    );

    // named / namedInventory tile-entity walkers (V1458.java:128-138).
    named(reg, "minecraft:beacon");
    named(reg, "minecraft:banner");
    named_inventory(reg, "minecraft:brewing_stand");
    named_inventory(reg, "minecraft:chest");
    named_inventory(reg, "minecraft:trapped_chest");
    named_inventory(reg, "minecraft:dispenser");
    named_inventory(reg, "minecraft:dropper");
    named(reg, "minecraft:enchanting_table");
    named_inventory(reg, "minecraft:furnace");
    named_inventory(reg, "minecraft:hopper");
    named_inventory(reg, "minecraft:shulker_box");
}
