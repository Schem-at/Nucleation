//! Applying the conversion engine to nucleation's in-memory schematic types.
//!
//! These bridge the engine (which operates on [`NbtMap`]) to the structs the
//! rest of the crate uses. Block-state palette conversion is unambiguous and
//! lives here now; block-entity and entity conversion (which need id-key and
//! entity-enum bridging) land alongside the load/save wiring.

use smol_str::SmolStr;
use std::collections::HashMap;

use crate::block_entity::BlockEntity;
use crate::entity::{Entity, NbtValue as EntityNbtValue};
use crate::nbt::{NbtMap, NbtValue};
use crate::{BlockState, Region, UniversalSchematic};

use super::loss::{self, LossReport};
use super::registry::{
    convert_block_entity, convert_block_state, convert_entity, convert_reverse_under_session,
    registry,
};
use super::types::{MapExt, ValueExt};

/// Convert one palette [`BlockState`] in place through the BLOCK_STATE converter
/// chain (block renames + per-block property fixers). Round-trips the struct
/// through the `{Name, Properties}` NBT shape the engine expects.
pub fn convert_block_state_struct(bs: &mut BlockState, from: i32, to: i32) {
    let mut map = NbtMap::new();
    map.set_string("Name", bs.get_name());
    if !bs.properties.is_empty() {
        let mut props = NbtMap::new();
        for (k, v) in &bs.properties {
            props.set_string(k.as_str(), v.as_str());
        }
        map.set_map("Properties", props);
    }

    convert_block_state(&mut map, from, to);

    if let Some(name) = map.get_string("Name") {
        bs.name = SmolStr::from(name);
    }
    let mut entries: Vec<(SmolStr, SmolStr)> = match map.get_map("Properties") {
        Some(props) => props
            .iter()
            .filter_map(|(k, v)| {
                v.as_str()
                    .map(|s| (SmolStr::from(k.as_str()), SmolStr::from(s)))
            })
            .collect(),
        None => Vec::new(),
    };
    // BlockState keeps properties sorted by key (see `with_properties`).
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    bs.properties = entries;
}

/// Convert every block state in a palette.
pub fn convert_palette(palette: &mut [BlockState], from: i32, to: i32) {
    for bs in palette.iter_mut() {
        convert_block_state_struct(bs, from, to);
    }
}

/// Convert one [`BlockEntity`] in place through the TILE_ENTITY chain (id
/// rename + namespacing + inventory/item recursion + components walk).
///
/// `BlockEntity` keeps its type id in the `id` field, separate from `nbt`. We
/// present the engine the canonical `{id, …nbt}` shape (dropping any stale
/// `id`/`Id` key in the data so the struct's `id` is authoritative), convert,
/// then split the id back out.
pub fn convert_block_entity_struct(be: &mut BlockEntity, from: i32, to: i32) {
    let mut map: NbtMap = (*be.nbt).clone();
    map.take("id");
    map.take("Id");
    map.set_string("id", be.id.as_str());

    convert_block_entity(&mut map, from, to);

    if let Some(new_id) = map.get_string("id").map(str::to_string) {
        be.id = new_id;
    }
    map.take("id");
    be.set_nbt(map);
}

fn entity_value_to_nbt(value: &EntityNbtValue) -> NbtValue {
    match value {
        EntityNbtValue::String(v) => NbtValue::String(v.clone()),
        EntityNbtValue::Int(v) => NbtValue::Int(*v),
        EntityNbtValue::Long(v) => NbtValue::Long(*v),
        EntityNbtValue::Float(v) => NbtValue::Float(*v),
        EntityNbtValue::Double(v) => NbtValue::Double(*v),
        EntityNbtValue::Byte(v) => NbtValue::Byte(*v),
        EntityNbtValue::Short(v) => NbtValue::Short(*v),
        EntityNbtValue::Boolean(v) => NbtValue::Byte(if *v { 1 } else { 0 }),
        EntityNbtValue::IntArray(v) => NbtValue::IntArray(v.clone()),
        EntityNbtValue::LongArray(v) => NbtValue::LongArray(v.clone()),
        EntityNbtValue::ByteArray(v) => NbtValue::ByteArray(v.clone()),
        EntityNbtValue::List(v) => NbtValue::List(v.iter().map(entity_value_to_nbt).collect()),
        EntityNbtValue::Compound(v) => NbtValue::Compound(entity_map_to_nbt(v)),
    }
}

fn nbt_value_to_entity(value: &NbtValue) -> EntityNbtValue {
    match value {
        NbtValue::String(v) => EntityNbtValue::String(v.clone()),
        NbtValue::Int(v) => EntityNbtValue::Int(*v),
        NbtValue::Long(v) => EntityNbtValue::Long(*v),
        NbtValue::Float(v) => EntityNbtValue::Float(*v),
        NbtValue::Double(v) => EntityNbtValue::Double(*v),
        NbtValue::Byte(v) => EntityNbtValue::Byte(*v),
        NbtValue::Short(v) => EntityNbtValue::Short(*v),
        NbtValue::IntArray(v) => EntityNbtValue::IntArray(v.clone()),
        NbtValue::LongArray(v) => EntityNbtValue::LongArray(v.clone()),
        NbtValue::ByteArray(v) => EntityNbtValue::ByteArray(v.clone()),
        NbtValue::List(v) => EntityNbtValue::List(v.iter().map(nbt_value_to_entity).collect()),
        NbtValue::Compound(v) => EntityNbtValue::Compound(nbt_map_to_entity(v)),
    }
}

fn entity_map_to_nbt(map: &HashMap<String, EntityNbtValue>) -> NbtMap {
    let mut out = NbtMap::new();
    for (key, value) in map {
        out.set_generic(key, entity_value_to_nbt(value));
    }
    out
}

fn nbt_map_to_entity(map: &NbtMap) -> HashMap<String, EntityNbtValue> {
    let mut out = HashMap::new();
    for (key, value) in map.iter() {
        out.insert(key.clone(), nbt_value_to_entity(value));
    }
    out
}

/// Convert one mobile [`Entity`] in place through the ENTITY chain.
pub fn convert_entity_struct(entity: &mut Entity, from: i32, to: i32) {
    let mut map = entity_map_to_nbt(&entity.nbt);
    map.take("id");
    map.take("Id");
    map.take("Pos");
    map.set_string("id", entity.id.as_str());

    convert_entity(&mut map, from, to);

    if let Some(new_id) = map.get_string("id").map(str::to_string) {
        entity.id = new_id;
    }
    map.take("id");
    entity.nbt = nbt_map_to_entity(&map);
}

/// Convert a region's block-state palette and every block entity. (Mobile
/// entities use their own NBT enum, so bridge them through the converter NBT
/// shape here.)
pub fn convert_region(region: &mut Region, from: i32, to: i32) {
    convert_palette(&mut region.palette, from, to);

    let entries = region.block_entities.drain();
    for (pos, mut be) in entries {
        convert_block_entity_struct(&mut be, from, to);
        region.block_entities.insert(pos, be);
    }

    for entity in &mut region.entities {
        convert_entity_struct(entity, from, to);
    }
}

/// Convert an entire schematic's blocks + block entities from `from` to `to`.
pub fn convert_schematic(schematic: &mut UniversalSchematic, from: i32, to: i32) {
    if from == to {
        return;
    }
    convert_region(&mut schematic.default_region, from, to);
    for region in schematic.other_regions.values_mut() {
        convert_region(region, from, to);
    }
}

// --- reverse (new -> old) -----------------------------------------------------

/// Reverse-convert one palette [`BlockState`] in place (new -> old). Runs under
/// the caller's reverse session (does not open its own).
fn convert_block_state_struct_reverse(bs: &mut BlockState, from: i32, to: i32) {
    let reg = registry();
    let mut map = NbtMap::new();
    map.set_string("Name", bs.get_name());
    if !bs.properties.is_empty() {
        let mut props = NbtMap::new();
        for (k, v) in &bs.properties {
            props.set_string(k.as_str(), v.as_str());
        }
        map.set_map("Properties", props);
    }

    convert_reverse_under_session(&reg.block_state, &mut map, from, to);

    if let Some(name) = map.get_string("Name") {
        bs.name = SmolStr::from(name);
    }
    let mut entries: Vec<(SmolStr, SmolStr)> = match map.get_map("Properties") {
        Some(props) => props
            .iter()
            .filter_map(|(k, v)| {
                v.as_str()
                    .map(|s| (SmolStr::from(k.as_str()), SmolStr::from(s)))
            })
            .collect(),
        None => Vec::new(),
    };
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    bs.properties = entries;
}

/// Reverse-convert one [`BlockEntity`] in place (new -> old), under the caller's
/// reverse session.
fn convert_block_entity_struct_reverse(be: &mut BlockEntity, from: i32, to: i32) {
    let reg = registry();
    let mut map: NbtMap = (*be.nbt).clone();
    map.take("id");
    map.take("Id");
    map.set_string("id", be.id.as_str());

    convert_reverse_under_session(&reg.tile_entity, &mut map, from, to);

    if let Some(new_id) = map.get_string("id").map(str::to_string) {
        be.id = new_id;
    }
    map.take("id");
    be.set_nbt(map);
}

/// Reverse-convert one mobile [`Entity`] in place (new -> old), under the
/// caller's reverse session.
fn convert_entity_struct_reverse(entity: &mut Entity, from: i32, to: i32) {
    let reg = registry();
    let mut map = entity_map_to_nbt(&entity.nbt);
    map.take("id");
    map.take("Id");
    map.take("Pos");
    map.set_string("id", entity.id.as_str());

    convert_reverse_under_session(&reg.entity, &mut map, from, to);

    if let Some(new_id) = map.get_string("id").map(str::to_string) {
        entity.id = new_id;
    }
    map.take("id");
    entity.nbt = nbt_map_to_entity(&map);
}

/// Reverse-convert a region's palette + block entities, seeding a human path for
/// each so the loss report can point at the offending block / block entity.
fn convert_region_reverse(region: &mut Region, from: i32, to: i32) {
    for (i, bs) in region.palette.iter_mut().enumerate() {
        let _scope = loss::path_scope(format!("palette[{i}] {}", bs.get_name()));
        convert_block_state_struct_reverse(bs, from, to);
    }

    let entries = region.block_entities.drain();
    for (pos, mut be) in entries {
        let _scope = loss::path_scope(format!("block_entity {} @ {:?}", be.id, pos));
        convert_block_entity_struct_reverse(&mut be, from, to);
        region.block_entities.insert(pos, be);
    }

    for (i, entity) in region.entities.iter_mut().enumerate() {
        let _scope = loss::path_scope(format!("entity[{i}] {}", entity.id));
        convert_entity_struct_reverse(entity, from, to);
    }
}

/// Reverse-convert an entire schematic from the newer `from` down to the older
/// `to`, returning the [`LossReport`] describing every approximation/data loss.
///
/// This is the inverse of [`convert_schematic`]: it is used on **save** to write
/// a schematic targeting an older Minecraft version. Unlike the forward path it
/// can be lossy (the 1.13 Flattening, 1.20.5 components, item `Damage`, …), so
/// it always returns a report — empty when the downgrade was lossless. The
/// caller (the developer, and via WASM the tool's user) is expected to surface a
/// non-empty report rather than save silently.
pub fn convert_schematic_reverse(
    schematic: &mut UniversalSchematic,
    from: i32,
    to: i32,
) -> LossReport {
    if from == to {
        return LossReport::default();
    }
    let (_, report) = loss::run_reverse(|| {
        convert_region_reverse(&mut schematic.default_region, from, to);
        for region in schematic.other_regions.values_mut() {
            convert_region_reverse(region, from, to);
        }
    });
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nbt::NbtValue;

    #[test]
    fn convert_entity_struct_forward_and_reverse_round_trip() {
        // V107 splits Minecart + Type=2 -> "MinecartFurnace"; bridge the entity
        // enum across the converter and confirm forward + (schematic) reverse.
        let mut e = Entity::new("Minecart".to_string(), (0.0, 0.0, 0.0));
        e.nbt.insert("Type".to_string(), EntityNbtValue::Int(2));

        convert_entity_struct(&mut e, 106, 107);
        assert_eq!(e.id, "MinecartFurnace");
        assert!(!e.nbt.contains_key("Type"), "Type folded into the id");

        // Reverse via the schematic entry (opens a loss session). Put the entity
        // in a schematic and downgrade back across V107.
        let mut schem = UniversalSchematic::new("e".to_string());
        schem.default_region.entities.push(e);
        let report = convert_schematic_reverse(&mut schem, 107, 106);
        let back = &schem.default_region.entities[0];
        assert_eq!(back.id, "Minecart");
        assert_eq!(back.nbt.get("Type"), Some(&EntityNbtValue::Int(2)));
        assert!(report.is_empty(), "minecart type split is a lossless inverse");
    }

    #[test]
    fn convert_block_entity_namespaces_id_and_recurses_into_items() {
        let mut be = BlockEntity::new("Chest".to_string(), (0, 0, 0));
        let mut item = NbtMap::new();
        item.set_string("id", "minecraft:pottery_shard_archer");
        item.set_byte("Count", 1);
        be.nbt_mut()
            .set_list("Items", vec![NbtValue::Compound(item)]);

        convert_block_entity_struct(&mut be, 703, 3438);

        assert_eq!(be.id, "minecraft:chest"); // V704 legacy id -> namespaced
        let items = be.nbt.get_list("Items").expect("Items");
        assert_eq!(
            items[0].as_compound_ref().unwrap().get_string("id"),
            Some("minecraft:archer_pottery_shard") // V3438 nested item rename
        );
    }

    #[test]
    fn convert_palette_applies_block_renames_across_versions() {
        let mut palette = vec![
            BlockState::new("minecraft:melon_block"), // -> melon at V1490
            BlockState::new("minecraft:grass_path"),  // -> dirt_path at V2680
        ];
        convert_palette(&mut palette, 1489, 2680);
        assert_eq!(palette[0].get_name(), "minecraft:melon");
        assert_eq!(palette[1].get_name(), "minecraft:dirt_path");
    }

    #[test]
    fn convert_block_state_preserves_properties() {
        let mut bs = BlockState::new("minecraft:melon_block").with_property("foo", "bar");
        convert_block_state_struct(&mut bs, 1489, 1490);
        assert_eq!(bs.get_name(), "minecraft:melon");
        assert_eq!(bs.get_property("foo").map(|s| s.as_str()), Some("bar"));
    }
}
