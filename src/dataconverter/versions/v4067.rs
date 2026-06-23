//! V4067 (24w38a + 1) — schematic-relevant subset of `V4067.java`.
//!
//! The single `minecraft:boat` / `minecraft:chest_boat` entities (carrying a
//! `Type` discriminator) are split into per-wood ids. Ported parts:
//!   * ENTITY walkers for the new chest-boat ids: `Items` item-list.
//!   * ITEM_STACK structure converter: for a boat *item* whose id is in the
//!     wood table, force `components["minecraft:entity_data"]["Type"]`.
//!   * ENTITY structure converter: rewrite `minecraft:boat` /
//!     `minecraft:chest_boat` -> the per-wood id from `Type` (dropping `Type`).
//!
//! Faithful-port note: as in Java, the ITEM_STACK lookup table is keyed by the
//! *un-namespaced* item id (`oak_boat`, not `minecraft:oak_boat`), and the
//! ENTITY remapping values are likewise un-namespaced (only the fallbacks carry
//! `minecraft:`). The later namespaced-id hooks correct these.
//!
//! Skipped (non-schematic): the LIGHTWEIGHT_LEVEL `bundle` feature-flag removal.
//!
//! VERSION = MCVersions.V24W38A (4066) + 1 = 4067.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::item_lists;

const VERSION: i32 = 4067;

/// `(name, suffix)` boat-wood table (V4067.java:19-29).
const BOAT_TYPES: &[(&str, &str)] = &[
    ("oak", "boat"),
    ("spruce", "boat"),
    ("birch", "boat"),
    ("jungle", "boat"),
    ("acacia", "boat"),
    ("cherry", "boat"),
    ("dark_oak", "boat"),
    ("mangrove", "boat"),
    ("bamboo", "raft"),
];

fn register_chest_boat(reg: &mut RegistryBuilder, id: &str) {
    reg.entity
        .add_walker(VERSION, 0, id, item_lists(&["Items"]));
}

/// Look up the wood name for an item id, matching the Java map's *un-namespaced*
/// keys (`<name>_<suffix>` and `<name>_chest_<suffix>`).
fn boat_type_for_item_id(id: &str) -> Option<&'static str> {
    for (name, suffix) in BOAT_TYPES {
        if id == format!("{name}_{suffix}") || id == format!("{name}_chest_{suffix}") {
            return Some(name);
        }
    }
    None
}

/// Reverse of the ENTITY split: map a per-wood boat *entity* id back to its
/// pre-V4067 `(base id, Type)` pair, where base id is `minecraft:boat` (normal)
/// or `minecraft:chest_boat` (chest). The per-wood id uniquely encodes the wood
/// discriminator the forward removed, so this is lossless (cheatsheet rule 11).
///
/// Accepts both the *un-namespaced* ids the forward converter emits for matched
/// woods (`oak_boat`, `bamboo_raft`, `oak_chest_boat`, `bamboo_chest_raft`) and
/// the canonical `minecraft:`-namespaced forms (the real ids carried by genuine
/// V4067+ data; the namespaced-id hooks fix up the un-namespaced ones forward,
/// and the inverse id-rename runs later in the descending sweep — rule 4).
fn boat_entity_to_base_and_type(id: &str) -> Option<(&'static str, &'static str)> {
    let bare = id.strip_prefix("minecraft:").unwrap_or(id);
    for (name, suffix) in BOAT_TYPES {
        // chest variants: `<name>_chest_<suffix>`
        if bare == format!("{name}_chest_{suffix}") {
            return Some(("minecraft:chest_boat", name));
        }
        // normal variants: `<name>_<suffix>`
        if bare == format!("{name}_{suffix}") {
            return Some(("minecraft:boat", name));
        }
    }
    None
}

pub fn register(reg: &mut RegistryBuilder) {
    register_chest_boat(reg, "minecraft:oak_chest_boat");
    register_chest_boat(reg, "minecraft:spruce_chest_boat");
    register_chest_boat(reg, "minecraft:birch_chest_boat");
    register_chest_boat(reg, "minecraft:jungle_chest_boat");
    register_chest_boat(reg, "minecraft:acacia_chest_boat");
    register_chest_boat(reg, "minecraft:cherry_chest_boat");
    register_chest_boat(reg, "minecraft:dark_oak_chest_boat");
    register_chest_boat(reg, "minecraft:mangrove_chest_boat");
    register_chest_boat(reg, "minecraft:bamboo_chest_raft");

    // ITEM_STACK: force entity_data.Type for boat items (un-namespaced keys).
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let id = match data.get_string("id") {
                Some(s) => s.to_string(),
                None => return,
            };
            let boat_type = match boat_type_for_item_id(&id) {
                Some(t) => t,
                None => return,
            };
            let components = match data.get_map_mut("components") {
                Some(c) => c,
                None => return,
            };
            let entity_data = match components.get_map_mut("minecraft:entity_data") {
                Some(e) => e,
                None => return,
            };
            entity_data.set_string("Type", boat_type);
        }),
    );

    // No reverse for the ITEM_STACK converter (V4067.java:60-93): it is a pure
    // normalization. Pre-V4067 a boat *item*'s `components.minecraft:entity_data`
    // already carried the `Type` string the old `minecraft:boat`/`chest_boat`
    // entity needed (boats were a single typed entity); the forward only *forces*
    // that `Type` to agree with the per-wood item id. The old format both keeps
    // `Type` in `entity_data` and uses the same per-wood item id, so the inverse
    // is identity — removing `Type` here would drop a field the old format needs
    // (cheatsheet rule 10).

    // ENTITY: split minecraft:boat / minecraft:chest_boat by Type.
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let id = match data.get_string("id") {
                Some(s) => s.to_string(),
                None => return, // wat
            };

            let normal_boat = id == "minecraft:boat";
            let chest_boat = id == "minecraft:chest_boat";
            if !normal_boat && !chest_boat {
                return;
            }

            let type_str = data.get_string("Type").map(|s| s.to_string());
            data.take("Type");

            // NORMAL_REMAPPING / CHEST_REMAPPING: Type -> "<name>_boat" /
            // "<name>_chest_<suffix>" (un-namespaced), defaulting to the
            // namespaced oak variant.
            let new_id = if normal_boat {
                type_str
                    .as_deref()
                    .and_then(|t| {
                        BOAT_TYPES
                            .iter()
                            .find(|(name, _)| *name == t)
                            .map(|(name, suffix)| format!("{name}_{suffix}"))
                    })
                    .unwrap_or_else(|| "minecraft:oak_boat".to_string())
            } else {
                type_str
                    .as_deref()
                    .and_then(|t| {
                        BOAT_TYPES
                            .iter()
                            .find(|(name, _)| *name == t)
                            .map(|(name, suffix)| format!("{name}_chest_{suffix}"))
                    })
                    .unwrap_or_else(|| "minecraft:oak_chest_boat".to_string())
            };

            data.set_string("id", new_id);
        }),
    );

    // Reverse of the ENTITY split (V4067.java:95-135): merge each per-wood boat
    // entity id back into `minecraft:boat` / `minecraft:chest_boat`, restoring
    // the `Type` string discriminator the forward removed. The per-wood id
    // uniquely encodes the wood, so this is exact — no loss (cheatsheet rule 11).
    //
    // Registered as a single id-agnostic reverse converter (rather than
    // `add_reverse_converter_for_id`, which needs `&'static str` ids) because the
    // boat ids are built dynamically from BOAT_TYPES. It matches both the
    // un-namespaced ids the forward emits and their `minecraft:`-namespaced
    // canonical forms; non-boat entities fall through untouched.
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let id = match data.get_string("id") {
                Some(s) => s.to_string(),
                None => return,
            };
            let (base_id, wood) = match boat_entity_to_base_and_type(&id) {
                Some(pair) => pair,
                None => return,
            };
            data.set_string("id", base_id);
            data.set_string("Type", wood);
        }),
    );
}
