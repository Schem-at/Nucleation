//! V1451 (V17W47A = 1451) — the schematic-relevant subset of `V1451.java`, the
//! main body of the 1.13 Flattening.
//!
//! Ported steps (Java step in parens):
//!   * (0) TILE_ENTITY `trapped_chest` Items walker.
//!   * (2) TILE_ENTITY `piston` blockId/blockData -> flattened `blockState`
//!         (`HelperBlockFlatteningV1450.getNBTForId`) + its BLOCK_STATE walker.
//!   * (3) `ConverterFlattenEntity` (falling_block + arrows/minecarts/… legacy
//!         numeric block fields -> flattened state), ITEM_STACK `filled_map`
//!         Damage->tag.map, and the V3 ENTITY walkers (block states / items /
//!         spawner / command output in vehicles & projectiles).
//!   * (4) BLOCK_NAME numeric/string -> flattened name, ITEM_STACK
//!         `ConverterFlattenItemStack` (id+Damage subtype -> flattened id,
//!         durability Damage migrated to tag).
//!   * (5) ITEM_STACK `spawn_egg` (EntityTag.id -> egg id), TILE_ENTITY `banner`
//!         colour inversion (Base/Patterns 15-x).
//!   * (6) TILE_ENTITY `jukebox` numeric `Record` -> `RecordItem`.
//!   * (7) ENTITY `villager` trade pumpkin fix (carved_pumpkin -> pumpkin).
//!
//! Skipped (non-schematic): the V1 CHUNK flatten + walker, the V5 LEVEL flat-
//! generator-settings fix, the V6 STATS/OBJECTIVE flatten, and the V7
//! STRUCTURE_FEATURE block-id flatten — none occur in a schematic file.

use std::sync::Arc;

use crate::nbt::{NbtMap, NbtValue};

use super::super::engine::{Converter, Walker};
use super::super::flattening::{
    entity_block_id, entity_for_spawn_egg, flat_to_numeric, flatten_item, flatten_nbt,
    get_name_for_id, get_name_from_id, get_nbt_for_id, get_new_block_name, get_old_block_name,
    id_from_item_name, item_has_damage, spawn_egg_for_entity, unflatten_item, unflatten_nbt,
    Unflatten,
};
use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};
use super::super::walker::{convert, item_lists, items, tile_entities};

const VERSION: i32 = 1451;

/// `ConverterFlattenEntity` per-id converter: a legacy numeric block (id at
/// `paths[0]`, data at `paths[1]`) becomes a flattened block state at
/// `paths[2]`. The one-path form simply drops a now-unused legacy field.
fn flatten_entity_state(paths: &'static [&'static str]) -> Converter {
    Box::new(move |data, _from, _to| {
        if paths.len() == 1 {
            data.take(paths[0]);
            return;
        }
        let (id_path, data_path, out_path) = (paths[0], paths[1], paths[2]);

        // Number at id_path, else getBlockId(string).
        let block_id = match data.get_i32(id_path) {
            Some(n) => n,
            None => entity_block_id(data.get_string(id_path).unwrap_or("")),
        };
        let block_data = data.get_i32(data_path).unwrap_or(0) & 15;

        data.take(id_path);
        data.take(data_path);
        data.set_map(out_path, get_nbt_for_id((block_id << 4) | block_data));
    })
}

/// Reverse of [`flatten_entity_state`]: a flattened block state at `paths[2]`
/// becomes the legacy numeric `(id, data)` at `paths[0]`/`paths[1]`. The one-path
/// form had no value to restore (the forward merely dropped a vestigial legacy
/// `inTile`), so its inverse is a no-op.
fn unflatten_entity_state(paths: &'static [&'static str]) -> Converter {
    Box::new(move |data, _from, _to| {
        if paths.len() == 1 {
            return;
        }
        let (id_path, data_path, out_path) = (paths[0], paths[1], paths[2]);
        let Some(state) = data.get_map(out_path).cloned() else {
            return;
        };
        data.take(out_path);
        match numeric_from_flat_or_legacy_state(&state) {
            Some(idx) => {
                data.set_i32(id_path, idx >> 4);
                data.set_i32(data_path, idx & 15);
            }
            None => report_loss(
                VERSION,
                LossKind::FlatteningUnknownBlock,
                Severity::Loss,
                format!(
                    "{out_path} block state has no pre-1.13 numeric id; legacy {id_path} not restored"
                ),
            ),
        }
    })
}

fn numeric_from_flat_or_legacy_state(state: &NbtMap) -> Option<i32> {
    flat_to_numeric(state).or_else(|| {
        let flattened = flatten_nbt(state)?;
        flat_to_numeric(&flattened)
    })
}

fn legacy_state_from_flat_or_legacy_state(state: &NbtMap) -> Option<NbtMap> {
    match unflatten_nbt(state) {
        Unflatten::Exact(o) | Unflatten::Approximated(o) => Some(o),
        Unflatten::Unknown => {
            if flatten_nbt(state).is_some() {
                Some(state.clone())
            } else {
                None
            }
        }
    }
}

/// A walker that recurses the single block state at `data[path]`.
fn block_state_path(path: &'static str) -> Walker {
    Arc::new(move |reg, data, from, to| convert(reg, &reg.block_state, data, path, from, to))
}

/// A walker that recurses the single text component at `data[path]`.
fn text_component_path(path: &'static str) -> Walker {
    Arc::new(move |reg, data, from, to| convert(reg, &reg.text_component, data, path, from, to))
}

pub fn register(reg: &mut RegistryBuilder) {
    // --- step 0 -----------------------------------------------------------
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "minecraft:trapped_chest",
        item_lists(&["Items"]),
    );

    // --- step 2: piston ---------------------------------------------------
    reg.tile_entity.add_converter_for_id(
        "minecraft:piston",
        VERSION,
        2,
        Box::new(|data, _from, _to| {
            let block_id = data.get_i32("blockId").unwrap_or(0);
            let block_data = data.get_i32("blockData").unwrap_or(0) & 15;
            data.take("blockId");
            data.take("blockData");
            data.set_map("blockState", get_nbt_for_id((block_id << 4) | block_data));
        }),
    );
    reg.tile_entity.add_walker(
        VERSION,
        2,
        "minecraft:piston",
        block_state_path("blockState"),
    );

    // --- step 3: ConverterFlattenEntity -----------------------------------
    // falling_block has its own id resolution (Block / TileID / Tile).
    reg.entity.add_converter_for_id(
        "minecraft:falling_block",
        VERSION,
        3,
        Box::new(|data, _from, _to| {
            let block_id = if data.has_key("Block") {
                match data.get_i32("Block") {
                    Some(n) => n,
                    None => entity_block_id(data.get_string("Block").unwrap_or("")),
                }
            } else if let Some(tile_id) = data.get_i32("TileID") {
                tile_id
            } else {
                data.get_i32("Tile").unwrap_or(0) & 255
            };
            let block_data = data.get_i32("Data").unwrap_or(0) & 15;

            data.take("Block");
            data.take("Data");
            data.take("TileID");
            data.take("Tile");

            data.set_map("BlockState", get_nbt_for_id((block_id << 4) | block_data));
        }),
    );

    // Three-path vehicles / projectiles.
    for (id, paths) in [
        (
            "minecraft:enderman",
            &["carried", "carriedData", "carriedBlockState"][..],
        ),
        ("minecraft:arrow", &["inTile", "inData", "inBlockState"][..]),
        (
            "minecraft:spectral_arrow",
            &["inTile", "inData", "inBlockState"][..],
        ),
        (
            "minecraft:commandblock_minecart",
            &["DisplayTile", "DisplayData", "DisplayState"][..],
        ),
        (
            "minecraft:minecart",
            &["DisplayTile", "DisplayData", "DisplayState"][..],
        ),
        (
            "minecraft:chest_minecart",
            &["DisplayTile", "DisplayData", "DisplayState"][..],
        ),
        (
            "minecraft:furnace_minecart",
            &["DisplayTile", "DisplayData", "DisplayState"][..],
        ),
        (
            "minecraft:tnt_minecart",
            &["DisplayTile", "DisplayData", "DisplayState"][..],
        ),
        (
            "minecraft:hopper_minecart",
            &["DisplayTile", "DisplayData", "DisplayState"][..],
        ),
        (
            "minecraft:spawner_minecart",
            &["DisplayTile", "DisplayData", "DisplayState"][..],
        ),
    ] {
        reg.entity
            .add_converter_for_id(id, VERSION, 3, flatten_entity_state(paths));
    }

    // One-path projectiles (just drop the legacy `inTile`).
    for id in [
        "minecraft:egg",
        "minecraft:ender_pearl",
        "minecraft:fireball",
        "minecraft:potion",
        "minecraft:small_fireball",
        "minecraft:snowball",
        "minecraft:wither_skull",
        "minecraft:xp_bottle",
    ] {
        reg.entity
            .add_converter_for_id(id, VERSION, 3, flatten_entity_state(&["inTile"]));
    }

    // ITEM_STACK filled_map: copy base Damage into tag.map if absent.
    reg.item_stack.add_converter_for_id(
        "minecraft:filled_map",
        VERSION,
        3,
        Box::new(|data, _from, _to| {
            let damage = data.get_i32("Damage").unwrap_or(0);
            if data.get_map("tag").is_none() {
                data.set_map("tag", NbtMap::new());
            }
            let tag = data.get_map_mut("tag").expect("just inserted");
            if tag.get_i32("map").is_none() {
                tag.set_i32("map", damage);
            }
        }),
    );

    // V3 ENTITY walkers.
    reg.entity
        .add_walker(VERSION, 3, "minecraft:potion", items(&["Potion"]));
    reg.entity.add_walker(
        VERSION,
        3,
        "minecraft:arrow",
        block_state_path("inBlockState"),
    );
    reg.entity.add_walker(
        VERSION,
        3,
        "minecraft:enderman",
        block_state_path("carriedBlockState"),
    );
    reg.entity.add_walker(
        VERSION,
        3,
        "minecraft:falling_block",
        block_state_path("BlockState"),
    );
    reg.entity.add_walker(
        VERSION,
        3,
        "minecraft:falling_block",
        tile_entities(&["TileEntityData"]),
    );
    reg.entity.add_walker(
        VERSION,
        3,
        "minecraft:spectral_arrow",
        block_state_path("inBlockState"),
    );
    reg.entity.add_walker(
        VERSION,
        3,
        "minecraft:chest_minecart",
        block_state_path("DisplayState"),
    );
    reg.entity.add_walker(
        VERSION,
        3,
        "minecraft:chest_minecart",
        item_lists(&["Items"]),
    );
    reg.entity.add_walker(
        VERSION,
        3,
        "minecraft:commandblock_minecart",
        block_state_path("DisplayState"),
    );
    reg.entity.add_walker(
        VERSION,
        3,
        "minecraft:commandblock_minecart",
        text_component_path("LastOutput"),
    );
    reg.entity.add_walker(
        VERSION,
        3,
        "minecraft:furnace_minecart",
        block_state_path("DisplayState"),
    );
    reg.entity.add_walker(
        VERSION,
        3,
        "minecraft:hopper_minecart",
        block_state_path("DisplayState"),
    );
    reg.entity.add_walker(
        VERSION,
        3,
        "minecraft:hopper_minecart",
        item_lists(&["Items"]),
    );
    reg.entity.add_walker(
        VERSION,
        3,
        "minecraft:minecart",
        block_state_path("DisplayState"),
    );
    reg.entity.add_walker(
        VERSION,
        3,
        "minecraft:spawner_minecart",
        block_state_path("DisplayState"),
    );
    reg.entity.add_walker(
        VERSION,
        3,
        "minecraft:spawner_minecart",
        Arc::new(|reg, data, from, to| reg.untagged_spawner.convert(reg, data, from, to)),
    );
    reg.entity.add_walker(
        VERSION,
        3,
        "minecraft:tnt_minecart",
        block_state_path("DisplayState"),
    );

    // --- step 4: BLOCK_NAME + item flatten --------------------------------
    reg.block_name.add_converter(
        VERSION,
        4,
        Box::new(|val, _from, _to| {
            if let NbtValue::String(s) = val {
                let new = get_new_block_name(s);
                *s = new;
            } else if let Some(n) = val.as_number_i64() {
                *val = NbtValue::String(get_name_for_id(n as i32));
            }
        }),
    );

    reg.item_stack.add_structure_converter(
        VERSION,
        4,
        Box::new(|data, _from, _to| {
            let id = match data.get_string("id") {
                Some(s) => s.to_string(),
                None => return,
            };
            let damage = data.get_i32("Damage").unwrap_or(0);
            data.take("Damage");

            if let Some(remap) = flatten_item(&id, damage) {
                data.set_string("id", remap);
            }

            if damage != 0 && item_has_damage(&id) {
                if data.get_map("tag").is_none() {
                    data.set_map("tag", NbtMap::new());
                }
                let tag = data.get_map_mut("tag").expect("just inserted");
                tag.set_i32("Damage", damage);
            }
        }),
    );

    // --- step 5: spawn egg + banner ---------------------------------------
    reg.item_stack.add_converter_for_id(
        "minecraft:spawn_egg",
        VERSION,
        5,
        Box::new(|data, _from, _to| {
            let new_id = data
                .get_map("tag")
                .and_then(|tag| tag.get_map("EntityTag"))
                .and_then(|et| et.get_string("id"))
                .map(spawn_egg_for_entity);
            if let Some(nid) = new_id {
                data.set_string("id", nid);
            }
        }),
    );

    reg.tile_entity.add_converter_for_id(
        "minecraft:banner",
        VERSION,
        5,
        Box::new(|data, _from, _to| {
            if let Some(base) = data.get_i32("Base") {
                data.set_i32("Base", 15 - base);
            }
            if let Some(patterns) = data.get_list_mut("Patterns") {
                for p in patterns.iter_mut() {
                    if let Some(pm) = p.as_compound_mut() {
                        if let Some(c) = pm.get_i32("Color") {
                            pm.set_i32("Color", 15 - c);
                        }
                    }
                }
            }
        }),
    );

    // --- step 6: jukebox --------------------------------------------------
    reg.tile_entity.add_converter_for_id(
        "minecraft:jukebox",
        VERSION,
        6,
        Box::new(|data, _from, _to| {
            let record = data.get_i32("Record").unwrap_or(0);
            if record <= 0 {
                return;
            }
            data.take("Record");

            let new_item_id = get_name_from_id(record).and_then(|name| flatten_item(name, 0));
            let new_item_id = match new_item_id {
                Some(id) => id,
                None => return,
            };

            let mut record_item = NbtMap::new();
            record_item.set_string("id", new_item_id);
            record_item.set_byte("Count", 1);
            data.set_map("RecordItem", record_item);
        }),
    );

    // --- step 7: villager trade pumpkin -----------------------------------
    reg.entity.add_converter_for_id(
        "minecraft:villager",
        VERSION,
        7,
        Box::new(|data, _from, _to| {
            let Some(offers) = data.get_map_mut("Offers") else {
                return;
            };
            let Some(recipes) = offers.get_list_mut("Recipes") else {
                return;
            };
            for recipe in recipes.iter_mut() {
                let Some(rm) = recipe.as_compound_mut() else {
                    continue;
                };
                for path in ["buy", "buyB", "sell"] {
                    if let Some(item) = rm.get_map_mut(path) {
                        if item.get_string("id") == Some("minecraft:carved_pumpkin") {
                            item.set_string("id", "minecraft:pumpkin");
                        }
                    }
                }
            }
        }),
    );

    // ======================================================================
    // REVERSE (new -> old): inverses of the steps above, registered at the
    // same (VERSION, step). Renames are NOT here (none use map_renamer); these
    // are all bespoke structural/lossy inverses. Walkers are direction-neutral.
    // ======================================================================

    // step 2 reverse: piston flattened blockState -> numeric blockId/blockData.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:piston",
        VERSION,
        2,
        Box::new(|data, _from, _to| {
            let Some(state) = data.get_map("blockState").cloned() else {
                return;
            };
            data.take("blockState");
            match numeric_from_flat_or_legacy_state(&state) {
                Some(idx) => {
                    data.set_i32("blockId", idx >> 4);
                    data.set_i32("blockData", idx & 15);
                }
                None => report_loss(
                    VERSION,
                    LossKind::FlatteningUnknownBlock,
                    Severity::Loss,
                    "piston blockState has no pre-1.13 numeric id; blockId/blockData not restored",
                ),
            }
        }),
    );

    // step 3 reverse: falling_block flattened BlockState -> legacy Block (name) + Data.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:falling_block",
        VERSION,
        3,
        Box::new(|data, _from, _to| {
            let Some(state) = data.get_map("BlockState").cloned() else {
                return;
            };
            data.take("BlockState");
            let meta = numeric_from_flat_or_legacy_state(&state).map(|idx| idx & 15);
            let name = legacy_state_from_flat_or_legacy_state(&state)
                .and_then(|o| o.get_string("Name").map(str::to_string));
            match (name, meta) {
                (Some(name), Some(meta)) => {
                    // 1.12 falling_block uses a string `Block` id + numeric `Data`.
                    data.set_string("Block", name);
                    data.set_i32("Data", meta);
                }
                _ => report_loss(
                    VERSION,
                    LossKind::FlatteningUnknownBlock,
                    Severity::Loss,
                    "falling_block BlockState has no pre-1.13 form; Block/Data not restored",
                ),
            }
        }),
    );

    // step 3 reverse: 3-path vehicles/projectiles (numeric id+data).
    for (id, paths) in [
        (
            "minecraft:enderman",
            &["carried", "carriedData", "carriedBlockState"][..],
        ),
        ("minecraft:arrow", &["inTile", "inData", "inBlockState"][..]),
        (
            "minecraft:spectral_arrow",
            &["inTile", "inData", "inBlockState"][..],
        ),
        (
            "minecraft:commandblock_minecart",
            &["DisplayTile", "DisplayData", "DisplayState"][..],
        ),
        (
            "minecraft:minecart",
            &["DisplayTile", "DisplayData", "DisplayState"][..],
        ),
        (
            "minecraft:chest_minecart",
            &["DisplayTile", "DisplayData", "DisplayState"][..],
        ),
        (
            "minecraft:furnace_minecart",
            &["DisplayTile", "DisplayData", "DisplayState"][..],
        ),
        (
            "minecraft:tnt_minecart",
            &["DisplayTile", "DisplayData", "DisplayState"][..],
        ),
        (
            "minecraft:hopper_minecart",
            &["DisplayTile", "DisplayData", "DisplayState"][..],
        ),
        (
            "minecraft:spawner_minecart",
            &["DisplayTile", "DisplayData", "DisplayState"][..],
        ),
    ] {
        reg.entity
            .add_reverse_converter_for_id(id, VERSION, 3, unflatten_entity_state(paths));
    }
    // One-path projectiles: forward dropped a vestigial inTile — nothing to restore.

    // step 3 reverse: filled_map tag.map -> base Damage (the forward copied
    // Damage into tag.map). Runs after step-4 reverse re-added Damage=0, so this
    // overwrites it with the real map number.
    reg.item_stack.add_reverse_converter_for_id(
        "minecraft:filled_map",
        VERSION,
        3,
        Box::new(|data, _from, _to| {
            let map_no = data.get_map("tag").and_then(|t| t.get_i32("map"));
            if let Some(map_no) = map_no {
                if let Some(tag) = data.get_map_mut("tag") {
                    tag.take("map");
                }
                data.set_short("Damage", map_no as i16);
            }
        }),
    );

    // step 4 reverse: BLOCK_NAME flattened name -> pre-1.13 name (string only; a
    // forward numeric input cannot be recovered, the string form is canonical).
    reg.block_name.add_reverse_converter(
        VERSION,
        4,
        Box::new(|val, _from, _to| {
            if let NbtValue::String(s) = val {
                *s = get_old_block_name(s);
            }
        }),
    );

    // step 4 reverse: ITEM_STACK un-flatten — the chain's main item downgrade.
    //   * subtype items (wool/dye/…): flattened id -> legacy id + Damage subtype.
    //   * durability items (tools/armor): tag.Damage -> base Damage.
    //   * everything else: pre-1.13 items always carried Damage (default 0).
    reg.item_stack.add_reverse_converter(
        VERSION,
        4,
        Box::new(|data, _from, _to| {
            let id = match data.get_string("id") {
                Some(s) => s.to_string(),
                None => return,
            };

            if let Some((old_id, subtype)) = unflatten_item(&id) {
                data.set_string("id", old_id);
                data.set_short("Damage", subtype as i16);
                return;
            }

            if item_has_damage(&id) {
                let dmg = data.get_map("tag").and_then(|t| t.get_i32("Damage"));
                if let Some(dmg) = dmg {
                    if let Some(tag) = data.get_map_mut("tag") {
                        tag.take("Damage");
                    }
                    data.set_short("Damage", dmg as i16);
                    return;
                }
            }

            // No subtype, no migrated durability: restore the default Damage.
            if data.get_i64("Damage").is_none() {
                report_loss(
                    VERSION,
                    LossKind::ItemFlatteningDamage,
                    Severity::Approximated,
                    format!(
                        "{id}: restored legacy Damage=0; any nonzero pre-1.13 Damage value is unrecoverable"
                    ),
                );
                data.set_short("Damage", 0);
            }
        }),
    );

    // step 5 reverse: spawn egg typed id -> minecraft:spawn_egg + EntityTag.id.
    reg.item_stack.add_reverse_converter(
        VERSION,
        5,
        Box::new(|data, _from, _to| {
            let id = match data.get_string("id") {
                Some(s) => s.to_string(),
                None => return,
            };
            let Some(entity_id) = entity_for_spawn_egg(&id) else {
                if id.ends_with("_spawn_egg") {
                    report_loss(
                        VERSION,
                        LossKind::UnsupportedInTarget,
                        Severity::Loss,
                        format!(
                            "spawn egg item '{id}' has no pre-1.13 EntityTag mapping; leaving typed id unchanged"
                        ),
                    );
                }
                return;
            };
            data.set_string("id", "minecraft:spawn_egg");
            if data.get_map("tag").is_none() {
                data.set_map("tag", NbtMap::new());
            }
            let tag = data.get_map_mut("tag").expect("just inserted");
            if tag.get_map("EntityTag").is_none() {
                tag.set_map("EntityTag", NbtMap::new());
            }
            let entity_tag = tag.get_map_mut("EntityTag").expect("just inserted");
            if let Some(existing) = entity_tag.get_string("id") {
                if existing != entity_id {
                    report_loss(
                        VERSION,
                        LossKind::EntityMergeAmbiguous,
                        Severity::Loss,
                        format!(
                            "spawn egg typed id '{id}' conflicts with existing EntityTag.id '{existing}'; using typed id entity '{entity_id}'"
                        ),
                    );
                }
            }
            entity_tag.set_string("id", entity_id);
        }),
    );

    // step 5 reverse: banner colour inversion is self-inverse (15 - x again).
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:banner",
        VERSION,
        5,
        Box::new(|data, _from, _to| {
            if let Some(base) = data.get_i32("Base") {
                data.set_i32("Base", 15 - base);
            }
            if let Some(patterns) = data.get_list_mut("Patterns") {
                for p in patterns.iter_mut() {
                    if let Some(pm) = p.as_compound_mut() {
                        if let Some(c) = pm.get_i32("Color") {
                            pm.set_i32("Color", 15 - c);
                        }
                    }
                }
            }
        }),
    );

    // step 6 reverse: jukebox RecordItem -> numeric Record.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:jukebox",
        VERSION,
        6,
        Box::new(|data, _from, _to| {
            let item_id = data
                .get_map("RecordItem")
                .and_then(|ri| ri.get_string("id"))
                .map(str::to_string);
            let Some(item_id) = item_id else { return };
            // Forward did get_name_from_id(record) -> flatten_item(.,0). The
            // record ids did not gain a subtype, so the flattened id equals the
            // name's flat form; invert via the item-name table.
            let old_name = unflatten_item(&item_id)
                .map(|(old, _)| old)
                .unwrap_or(item_id.as_str());
            match id_from_item_name(old_name) {
                Some(record) => {
                    data.take("RecordItem");
                    data.set_i32("Record", record);
                }
                None => report_loss(
                    VERSION,
                    LossKind::Other,
                    Severity::Loss,
                    format!("jukebox RecordItem '{item_id}' has no numeric Record id"),
                ),
            }
        }),
    );

    // step 7 reverse: villager trade pumpkin -> carved_pumpkin (inverse of the
    // forward carved_pumpkin -> pumpkin). Pre-1.13 trades used the carved id for
    // what 1.13 calls a plain pumpkin; a genuine uncarved-pumpkin trade is
    // indistinguishable, so this is a best-effort substitution.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:villager",
        VERSION,
        7,
        Box::new(|data, _from, _to| {
            let Some(offers) = data.get_map_mut("Offers") else { return };
            let Some(recipes) = offers.get_list_mut("Recipes") else { return };
            let mut touched = false;
            for recipe in recipes.iter_mut() {
                let Some(rm) = recipe.as_compound_mut() else { continue };
                for path in ["buy", "buyB", "sell"] {
                    if let Some(item) = rm.get_map_mut(path) {
                        if item.get_string("id") == Some("minecraft:pumpkin") {
                            item.set_string("id", "minecraft:carved_pumpkin");
                            touched = true;
                        }
                    }
                }
            }
            if touched {
                report_loss(
                    VERSION,
                    LossKind::Other,
                    Severity::Approximated,
                    "villager trade minecraft:pumpkin mapped to carved_pumpkin (pre-1.13 had no uncarved-pumpkin item)",
                );
            }
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::registry::{convert_block_entity_reverse, convert_entity_reverse};

    #[test]
    fn reverse_piston_accepts_already_unflattened_block_state() {
        let mut props = NbtMap::new();
        props.set_string("variant", "granite");
        let mut state = NbtMap::new();
        state.set_string("Name", "minecraft:stone");
        state.set_map("Properties", props);
        let mut piston = NbtMap::new();
        piston.set_string("id", "minecraft:piston");
        piston.set_map("blockState", state);

        let report = convert_block_entity_reverse(&mut piston, 1451, 1450);

        assert!(report.is_empty());
        assert!(piston.has_key("blockId"));
        assert!(piston.has_key("blockData"));
        assert!(!piston.has_key("blockState"));
    }

    #[test]
    fn reverse_unknown_typed_spawn_egg_reports_loss() {
        let mut egg = NbtMap::new();
        egg.set_string("id", "minecraft:camel_spawn_egg");

        let report =
            crate::dataconverter::registry::convert_item_stack_reverse(&mut egg, 1451, 1450);

        assert_eq!(egg.get_string("id"), Some("minecraft:camel_spawn_egg"));
        assert_eq!(report.loss_count(), 1);
    }

    #[test]
    fn reverse_falling_block_accepts_already_unflattened_state() {
        let mut state = NbtMap::new();
        state.set_string("Name", "minecraft:stone");
        let mut falling = NbtMap::new();
        falling.set_string("id", "minecraft:falling_block");
        falling.set_map("BlockState", state);

        let report = convert_entity_reverse(&mut falling, 1451, 1450);

        assert!(report.is_empty());
        assert_eq!(falling.get_string("Block"), Some("minecraft:stone"));
        assert_eq!(falling.get_i32("Data"), Some(0));
        assert!(!falling.has_key("BlockState"));
    }
}
