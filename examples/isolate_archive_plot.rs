//! Extract a plot with Nucleation's existing part isolator, without rebasing or
//! repairing the source. Run with INPUT.litematic NEW_OUTPUT_DIRECTORY.
//!
//! A successful manifest is written only after every exported file has been
//! read back and its blocks, block entities, metadata and entities checked.
//! Connectivity is a curation aid, not evidence that a candidate is a whole or
//! working door. The original selection bounds remain in the manifest.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use nucleation::block_entity::BlockEntity;
use nucleation::block_position::BlockPosition;
use nucleation::formats::litematic::{from_litematic, to_litematic};
use nucleation::{BlockState, BoundingBox, Connectivity, Entity, UniversalSchematic};
use quartz_nbt::{NbtCompound, NbtTag};
use serde_json::{json, Value};

pub(crate) type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub(crate) type Pos = (i32, i32, i32);

const MIN_STANDALONE_BLOCKS: usize = 16;
const MAX_AIR_GAP: u32 = 3;

pub(crate) struct Part {
    pub(crate) schematic: UniversalSchematic,
    pub(crate) positions: Vec<Pos>,
    pub(crate) block_bounds: Option<BoundingBox>,
    pub(crate) entity_indices: Vec<usize>,
    pub(crate) quarantine: bool,
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: isolate_archive_plot INPUT.litematic NEW_OUTPUT_DIRECTORY".into());
    }
    let input = PathBuf::from(&args[0]).canonicalize()?;
    let output = PathBuf::from(&args[1]);
    if output.exists() {
        return Err(format!("output directory must not exist: {}", output.display()).into());
    }
    let bytes = fs::read(&input)?;
    let source_file_hash = blake3::hash(&bytes).to_hex().to_string();
    let source = from_litematic(&bytes)?;
    let data_version = source.metadata.mc_version.ok_or("source has no MinecraftDataVersion")?;
    let selection_bounds = source.get_bounding_box();

    // Keep one source lookup, rather than rescanning the entire plot for every
    // candidate. Ordinary air is implicit. Cave/void air are explicit source
    // states: the physical isolator omits them, so attach them losslessly below.
    let mut source_blocks = HashMap::new();
    for (position, state) in source.iter_blocks() {
        if state.name != "minecraft:air" {
            let pos = (position.x, position.y, position.z);
            if source_blocks.insert(pos, state).is_some() {
                return Err(format!("overlapping source blocks at {pos:?}; cannot partition losslessly").into());
            }
        }
    }
    let source_non_air = source_blocks.values().filter(|state| !is_air_state(state)).count();
    let source_explicit_air = source_blocks.len() - source_non_air;
    let source_bes = block_entities(&source)?;
    let mut source_entities = source.get_entities_as_list();
    // Stable source indices, independent of the order of named-region HashMaps.
    source_entities.sort_by_cached_key(entity_bytes);
    for entity in &source_entities {
        if !entity.position.0.is_finite()
            || !entity.position.1.is_finite()
            || !entity.position.2.is_finite()
        {
            return Err("nonfinite entity position cannot be represented in litematic region bounds".into());
        }
    }
    let source_entity_bytes: Vec<_> = source_entities.iter().map(entity_bytes).collect();
    let source_entities_hash = digest_records(&source_entity_bytes);

    eprintln!(
        "Isolating {} non-air blocks plus {} explicit air states, {} block entities and {} entities at data version {data_version}",
        source_non_air, source_explicit_air, source_bes.len(), source_entities.len()
    );
    let mut parts: Vec<_> = source
        .split_connected_attach_nearby(Connectivity::Corner, MIN_STANDALONE_BLOCKS, MAX_AIR_GAP)
        .into_iter()
        .map(|mut schematic| {
            // Preserve the original metadata exactly; the archive's individual
            // display titles are separate manifest fields.
            schematic.metadata = source.metadata.clone();
            let mut positions: Vec<_> = schematic
                .iter_blocks()
                .filter(|(_, state)| state.name != "minecraft:air")
                .map(|(p, _)| (p.x, p.y, p.z))
                .collect();
            positions.sort_unstable();
            Part {
                block_bounds: schematic.get_tight_bounds(),
                schematic,
                positions,
                entity_indices: Vec::new(),
                quarantine: false,
            }
        })
        .collect();
    parts.sort_by(|a, b| {
        bounds_key(a.block_bounds.as_ref())
            .cmp(&bounds_key(b.block_bounds.as_ref()))
            .then_with(|| a.positions.cmp(&b.positions))
    });

    let mut owners = HashMap::with_capacity(source_blocks.len());
    let mut seen_bes = HashSet::with_capacity(source_bes.len());
    for (index, part) in parts.iter().enumerate() {
        for (p, state) in part.schematic.iter_blocks() {
            if state.name == "minecraft:air" {
                continue;
            }
            let pos = (p.x, p.y, p.z);
            if source_blocks.get(&pos).copied() != Some(state) {
                return Err(format!("isolator changed block at {pos:?}").into());
            }
            if owners.insert(pos, index).is_some() {
                return Err(format!("isolator duplicated block at {pos:?}").into());
            }
        }
        for (pos, value) in block_entities(&part.schematic)? {
            if source_bes.get(&pos) != Some(&value) || !seen_bes.insert(pos) {
                return Err(format!("isolator changed or duplicated block entity at {pos:?}").into());
            }
        }
        if !part.schematic.get_entities_as_list().is_empty() {
            return Err("part isolator now copies entities; update the explicit assignment path".into());
        }
    }
    // The core algorithm deliberately treats cave_air and void_air as empty
    // geometry. Preserve their distinct states without changing that algorithm
    // or making them artificial bridges between physical components. Compute
    // every assignment against the original physical blocks before inserting.
    let mut omitted: Vec<_> = source_blocks.keys().filter(|pos| !owners.contains_key(*pos)).copied().collect();
    omitted.sort_unstable();
    let mut air_assignments = Vec::with_capacity(omitted.len());
    for pos in omitted {
        let state = source_blocks[&pos];
        if !matches!(state.name.as_str(), "minecraft:cave_air" | "minecraft:void_air") {
            return Err(format!("isolator lost non-air state {} at {pos:?}", state.name).into());
        }
        let center = (f64::from(pos.0) + 0.5, f64::from(pos.1) + 0.5, f64::from(pos.2) + 0.5);
        let index = nearest_physical_part(&parts, center, &source_blocks, true)
            .ok_or("cannot attach explicit air states without a physical component")?;
        air_assignments.push((pos, index));
    }
    for (pos, index) in air_assignments {
        let state = source_blocks[&pos];
        let part = &mut parts[index];
        part.schematic.set_block(pos.0, pos.1, pos.2, state);
        if part.schematic.get_block(pos.0, pos.1, pos.2) != Some(state) {
            return Err(format!("failed to preserve explicit air state at {pos:?}").into());
        }
        if owners.insert(pos, index).is_some() {
            return Err(format!("explicit air assignment duplicated position {pos:?}").into());
        }
        part.positions.push(pos);
        if let Some(be) = source.get_block_entity_owned(BlockPosition { x: pos.0, y: pos.1, z: pos.2 }) {
            let encoded = nbt_bytes(&canonical_block_entity(&be)?);
            if source_bes.get(&pos) != Some(&encoded) || !seen_bes.insert(pos) {
                return Err(format!("explicit air block entity mismatch at {pos:?}").into());
            }
            part.schematic.set_block_entity(BlockPosition { x: pos.0, y: pos.1, z: pos.2 }, be);
        }
    }
    for part in &mut parts {
        part.positions.sort_unstable();
    }
    if owners.len() != source_blocks.len() || seen_bes.len() != source_bes.len() {
        return Err(format!(
            "isolator lost content: blocks {}/{}, block entities {}/{}",
            owners.len(), source_blocks.len(), seen_bes.len(), source_bes.len()
        ).into());
    }

    let isolated_parts = parts.len();
    let mut unassigned = Vec::new();
    for (entity_index, entity) in source_entities.iter().enumerate() {
        let target = nearest_physical_part(&parts, entity.position, &source_blocks, false);
        if let Some(index) = target {
            parts[index].schematic.add_entity(entity.clone());
            parts[index].entity_indices.push(entity_index);
        } else {
            unassigned.push(entity_index);
        }
    }
    if !unassigned.is_empty() {
        let mut schematic = UniversalSchematic::new("entity quarantine".into());
        schematic.metadata = source.metadata.clone();
        for &index in &unassigned {
            schematic.add_entity(source_entities[index].clone());
        }
        parts.push(Part {
            schematic,
            positions: Vec::new(),
            block_bounds: None,
            entity_indices: unassigned,
            quarantine: true,
        });
    }

    let assigned: Vec<_> = parts.iter().flat_map(|part| &part.entity_indices).copied().collect();
    let assigned_set: HashSet<_> = assigned.iter().copied().collect();
    if assigned.len() != source_entities.len() || assigned_set.len() != source_entities.len() {
        return Err("entity assignment failed exact-once coverage".into());
    }
    eprintln!("Exporting {isolated_parts} components and {} quarantine part(s)", parts.len() - isolated_parts);

    // Never overwrite an earlier run. A failed run remains visibly incomplete
    // and contains no success manifest; the original file is read-only.
    fs::create_dir(&output)?;
    fs::write(output.join("INCOMPLETE"), b"Extraction has not passed all read-back checks.\n")?;
    let mut manifest_parts = Vec::with_capacity(parts.len());
    let mut roundtrip_blocks = HashSet::with_capacity(source_blocks.len());
    let mut roundtrip_bes = HashSet::with_capacity(source_bes.len());
    let mut roundtrip_entities = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        let id = format!("part{:04}", index + 1);
        let file = format!("{id}.litematic");
        let expected_entities: Vec<_> = part.entity_indices.iter()
            .map(|&i| source_entity_bytes[i].clone()).collect();
        let actual_entities: Vec<_> = part.schematic.get_entities_as_list().iter().map(entity_bytes).collect();
        if actual_entities != expected_entities {
            return Err(format!("pre-export entity mismatch in {id}").into());
        }
        let encoded = to_litematic(&part.schematic)?;
        let path = output.join(&file);
        fs::write(&path, &encoded)?;
        // Read the bytes from disk, not the in-memory encoder buffer.
        let disk_bytes = fs::read(&path)?;
        if disk_bytes != encoded {
            return Err(format!("file write mismatch in {id}").into());
        }
        let readback = from_litematic(&disk_bytes)?;
        if readback.metadata != source.metadata {
            return Err(format!("metadata changed on read-back of {id}: {:?} != {:?}", readback.metadata, source.metadata).into());
        }
        let mut readback_positions = Vec::new();
        for (p, state) in readback.iter_blocks() {
            if state.name == "minecraft:air" {
                continue;
            }
            let pos = (p.x, p.y, p.z);
            if owners.get(&pos) != Some(&index)
                || source_blocks.get(&pos).copied() != Some(state)
                || !roundtrip_blocks.insert(pos)
            {
                return Err(format!("read-back block partition mismatch at {pos:?} in {id}").into());
            }
            readback_positions.push(pos);
        }
        readback_positions.sort_unstable();
        if readback_positions != part.positions {
            return Err(format!("read-back block count/coordinates changed in {id}").into());
        }
        let readback_block_entities = block_entities(&readback)?;
        let expected_bes = block_entities(&part.schematic)?;
        if readback_block_entities != expected_bes {
            return Err(format!("read-back block entity NBT changed in {id}: {}", block_entity_difference(&part.schematic, &readback)?).into());
        }
        for (pos, value) in &readback_block_entities {
            if source_bes.get(pos) != Some(value) || !roundtrip_bes.insert(*pos) {
                return Err(format!("read-back block entity partition mismatch at {pos:?}").into());
            }
        }
        let mut actual_entities: Vec<_> = readback.get_entities_as_list().iter().map(entity_bytes).collect();
        let mut expected_entities = expected_entities;
        actual_entities.sort();
        expected_entities.sort();
        if actual_entities != expected_entities {
            return Err(format!("read-back entity NBT or coordinates changed in {id}").into());
        }
        roundtrip_entities.extend(actual_entities);
        let mut record = describe_part(part, &source_blocks, &selection_bounds);
        let object = record.as_object_mut().expect("part description object");
        object.insert("id".into(), json!(id));
        object.insert("file".into(), json!(file));
        object.insert("display_name".into(), json!(format!("{} · {id}", source.metadata.name.as_deref().unwrap_or("World record plot"))));
        object.insert("file_blake3".into(), json!(blake3::hash(&disk_bytes).to_hex().to_string()));
        object.insert("content_blake3".into(), json!(blocks_digest(&part.positions, &source_blocks)));
        object.insert("block_entities_blake3".into(), json!(digest_positioned(&readback_block_entities)));
        object.insert("entities_blake3".into(), json!(digest_records(&expected_entities)));
        object.insert("bytes".into(), json!(disk_bytes.len()));
        object.insert("verification".into(), json!({
            "blocks": "exact_source_partition",
            "block_entities": "exact_nbt_and_positions",
            "block_entity_coordinate_normalization": "authoritative_id_and_absolute_position;relative_Id_id_Pos_x_y_z_wrappers_normalized;all_other_NBT_bit_exact",
            "entities": "exact_nbt_and_positions_including_passengers",
            "metadata": "equal_to_source",
            "readback": "passed",
            "float_comparison": "IEEE_bits_including_NaN",
            "functional_status": "unverified",
            "simulation_status": "not_run"
        }));
        manifest_parts.push(record);
        if (index + 1) % 50 == 0 || index + 1 == parts.len() {
            eprintln!("Verified {}/{} files", index + 1, parts.len());
        }
    }
    roundtrip_entities.sort();
    if roundtrip_blocks.len() != source_blocks.len()
        || roundtrip_bes.len() != source_bes.len()
        || roundtrip_entities != source_entity_bytes
    {
        return Err("aggregate exported read-back coverage is not equal to source".into());
    }
    let mut source_positions: Vec<_> = source_blocks.keys().copied().collect();
    source_positions.sort_unstable();
    let manifest = json!({
        "schema_version": 1,
        "source": {
            "path": input,
            "file_blake3": source_file_hash,
            "bytes": bytes.len(),
            "data_version": data_version,
            "source_data_version": source.metadata.source_data_version,
            "selection_bounds": selection_bounds,
            "metadata": source.metadata,
            "embedded_test": source.metadata.embedded_test,
            "non_air_blocks": source_non_air,
            "preserved_block_states": source_blocks.len(),
            "explicit_air_states": source_explicit_air,
            "block_entities": source_bes.len(),
            "entities": source_entities.len(),
            "blocks_blake3": blocks_digest(&source_positions, &source_blocks),
            "block_entities_blake3": digest_positioned(&source_bes),
            "entities_blake3": source_entities_hash
        },
        "isolation": {
            "api": "UniversalSchematic::split_connected_attach_nearby",
            "connectivity": "Corner",
            "min_standalone_blocks": MIN_STANDALONE_BLOCKS,
            "max_air_gap": MAX_AIR_GAP,
            "coordinates": "original_source_coordinates_without_rebasing",
            "part_order": "block_bounds_min_then_max_then_sorted_block_positions",
            "entity_source_index": "canonical_NBT_lexicographic_order_zero_based",
            "entity_assignment": "bounding_box_containment_then_nearest_closed_block_cube_then_part_index",
            "entity_bbox_convention": "min_inclusive_max_plus_one_exclusive",
            "entities_outside_all_block_bounds": "entity_only_quarantine_part",
            "explicit_air_assignment": "prefer_containing_physical_bbox_then_nearest_physical_block_cube_then_part_index",
            "explicit_air_states": "cave_air_and_void_air_preserved_without_affecting_physical_connectivity",
            "non_air_definition": "excludes_minecraft_air_cave_air_and_void_air",
            "inventory_repairs": "none"
        },
        "verification": {
            "status": "passed",
            "all_exported_files_read_back": true,
            "source_block_states_exactly_once": true,
            "source_block_entities_exactly_once": true,
            "source_entities_exactly_once": true,
            "metadata_preserved": true,
            "block_entity_coordinate_normalization": "authoritative_id_and_absolute_position;relative_Id_id_Pos_x_y_z_wrappers_normalized;all_other_NBT_bit_exact",
            "non_air_blocks": roundtrip_blocks.iter().filter(|pos| !is_air_state(source_blocks[*pos])).count(),
            "preserved_block_states": roundtrip_blocks.len(),
            "explicit_air_states": roundtrip_blocks.iter().filter(|pos| is_air_state(source_blocks[*pos])).count(),
            "block_entities": roundtrip_bes.len(),
            "entities": roundtrip_entities.len(),
            "hash_encoding": "typed_big_endian_length_delimited_NBT_with_sorted_compound_keys_and_bitwise_floats",
            "functional_status": "unverified"
        },
        "warnings": [
            "Geometric components are candidates, not certified complete or functional doors.",
            "The source selection may cut builds at its edges; boundary contact is a review signal, not proof of damage.",
            "Empty or absent inventories and missing controls are preserved, not repaired.",
            "A component can contain shared infrastructure or multiple mechanisms connected by a floor.",
            "Air padding, derived Litematica volume statistics and writer Software headers are regenerated; original selection bounds are recorded here.",
            "Block entity Id/id and Pos/x/y/z transport fields are normalized to authoritative identity and absolute source coordinates for comparison; all other NBT is bit-exact. Exported relative coordinates change with the new region origin.",
            "Native import/export does not preserve pending tick lists or unknown file tags; retain the untouched source file as the archival original."
        ],
        "component_count": isolated_parts,
        "quarantine_count": parts.len() - isolated_parts,
        "parts": manifest_parts
    });
    fs::write(output.join("manifest.json"), serde_json::to_vec_pretty(&manifest)?)?;
    fs::remove_file(output.join("INCOMPLETE"))?;
    println!("{}", output.join("manifest.json").display());
    Ok(())
}

pub(crate) fn bounds_key(bounds: Option<&BoundingBox>) -> (Pos, Pos) {
    bounds.map(|b| (b.min, b.max)).unwrap_or(((i32::MAX, i32::MAX, i32::MAX), (i32::MAX, i32::MAX, i32::MAX)))
}

pub(crate) fn is_air_state(state: &BlockState) -> bool {
    matches!(state.name.as_str(), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air")
}

pub(crate) fn nearest_physical_part(parts: &[Part], position: (f64, f64, f64), source: &HashMap<Pos, &BlockState>, allow_outside: bool) -> Option<usize> {
    let contains = |part: &Part| part.block_bounds.as_ref().is_some_and(|bounds| contains_entity(bounds, position));
    let has_containing = parts.iter().any(contains);
    if !has_containing && !allow_outside {
        return None;
    }
    parts.iter().enumerate().filter_map(|(index, part)| {
        if has_containing && !contains(part) {
            return None;
        }
        let distance = part.positions.iter()
            .filter(|pos| !is_air_state(source[*pos]))
            .map(|&pos| distance_to_block(position, pos))
            .fold(f64::INFINITY, f64::min);
        distance.is_finite().then_some((index, distance))
    }).min_by(|(a, da), (b, db)| da.total_cmp(db).then_with(|| a.cmp(b))).map(|(index, _)| index)
}

pub(crate) fn contains_entity(bounds: &BoundingBox, p: (f64, f64, f64)) -> bool {
    p.0 >= f64::from(bounds.min.0) && p.0 < f64::from(bounds.max.0) + 1.0
        && p.1 >= f64::from(bounds.min.1) && p.1 < f64::from(bounds.max.1) + 1.0
        && p.2 >= f64::from(bounds.min.2) && p.2 < f64::from(bounds.max.2) + 1.0
}

pub(crate) fn distance_to_block(p: (f64, f64, f64), block: Pos) -> f64 {
    [(p.0, block.0), (p.1, block.1), (p.2, block.2)].into_iter().map(|(value, min)| {
        let min = f64::from(min);
        let delta = if value < min { min - value } else if value > min + 1.0 { value - min - 1.0 } else { 0.0 };
        delta * delta
    }).sum()
}

pub(crate) fn block_entities(schematic: &UniversalSchematic) -> Result<BTreeMap<Pos, Vec<u8>>> {
    let mut map = BTreeMap::new();
    // This API materializes the correct position of deduplicated BE templates.
    for be in schematic.get_block_entities_as_list() {
        let pos = be.position;
        if map.insert(pos, nbt_bytes(&canonical_block_entity(&be)?)).is_some() {
            return Err(format!("duplicate block entity at {pos:?}").into());
        }
    }
    Ok(map)
}

pub(crate) fn canonical_block_entity(be: &BlockEntity) -> Result<NbtTag> {
    let mut compound = be.to_nbt();
    // BlockEntity retains the transport NBT from import, including relative
    // positions. It also exposes the authoritative, materialized absolute
    // position. New regions necessarily use different relative coordinates;
    // compare the semantic record, not these duplicated placement wrappers.
    for key in ["Id", "id"] {
        if let Some(value) = be.nbt.get(key) {
            if value.as_string().map(String::as_str) != Some(be.id.as_str()) {
                return Err(format!("conflicting block entity {key} at {:?}: {:?}", be.position, value).into());
            }
        }
    }
    let raw = be.to_nbt();
    let relative_pos = match be.nbt.get("Pos") {
        None => None,
        Some(value) => {
            let values = value.as_int_array().filter(|values| values.len() == 3)
                .ok_or_else(|| format!("invalid block entity Pos wrapper at {:?}", be.position))?;
            Some([values[0], values[1], values[2]])
        }
    };
    let xyz = ["x", "y", "z"].map(|key| raw.get::<_, i32>(key).ok());
    if ["x", "y", "z"].iter().any(|key| be.nbt.get(key).is_some()) {
        let [Some(x), Some(y), Some(z)] = xyz else {
            return Err(format!("partial block entity xyz wrappers at {:?}", be.position).into());
        };
        if relative_pos.is_some_and(|p| p != [x, y, z]) {
            return Err(format!("conflicting block entity Pos/xyz wrappers at {:?}", be.position).into());
        }
    }
    for key in ["Id", "id", "Pos", "x", "y", "z"] {
        compound.inner_mut().remove(key);
    }
    compound.insert("Id", NbtTag::String(be.id.to_string()));
    compound.insert("Pos", NbtTag::IntArray(vec![be.position.0, be.position.1, be.position.2]));
    Ok(NbtTag::Compound(compound))
}

pub(crate) fn block_entity_difference(expected: &UniversalSchematic, actual: &UniversalSchematic) -> Result<String> {
    let collect = |schematic: &UniversalSchematic| -> Result<BTreeMap<Pos, NbtTag>> {
        schematic.get_block_entities_as_list().iter().map(|be| Ok((be.position, canonical_block_entity(be)?))).collect()
    };
    let expected = collect(expected)?;
    let actual = collect(actual)?;
    let positions: BTreeSet<_> = expected.keys().chain(actual.keys()).copied().collect();
    for pos in positions {
        match (expected.get(&pos), actual.get(&pos)) {
            (Some(a), Some(b)) => {
                if let Some(difference) = first_nbt_difference(a, b, "NBT") {
                    return Ok(format!("position {pos:?}, {difference}"));
                }
            }
            (Some(_), None) => return Ok(format!("position {pos:?}: missing block entity")),
            (None, Some(_)) => return Ok(format!("position {pos:?}: unexpected block entity")),
            _ => unreachable!(),
        }
    }
    Ok("no canonical difference found".into())
}

fn first_nbt_difference(expected: &NbtTag, actual: &NbtTag, path: &str) -> Option<String> {
    if nbt_bytes(expected) == nbt_bytes(actual) {
        return None;
    }
    match (expected, actual) {
        (NbtTag::Compound(a), NbtTag::Compound(b)) => {
            let keys: BTreeSet<_> = a.inner().keys().chain(b.inner().keys()).collect();
            for key in keys {
                let nested = format!("{path}.{key}");
                match (a.inner().get(key), b.inner().get(key)) {
                    (Some(a), Some(b)) => if let Some(difference) = first_nbt_difference(a, b, &nested) { return Some(difference); },
                    (Some(value), None) => return Some(format!("{nested}: expected {}, actual missing", describe_nbt(value))),
                    (None, Some(value)) => return Some(format!("{nested}: expected missing, actual {}", describe_nbt(value))),
                    _ => unreachable!(),
                }
            }
        }
        (NbtTag::List(a), NbtTag::List(b)) => {
            for (index, (a, b)) in a.iter().zip(b.iter()).enumerate() {
                if let Some(difference) = first_nbt_difference(a, b, &format!("{path}[{index}]")) { return Some(difference); }
            }
            return Some(format!("{path}: expected List length {}, actual List length {}", a.len(), b.len()));
        }
        _ => {}
    }
    Some(format!("{path}: expected {}, actual {}", describe_nbt(expected), describe_nbt(actual)))
}

fn describe_nbt(tag: &NbtTag) -> String {
    match tag {
        NbtTag::Float(v) => format!("Float({v:?}, bits=0x{:08x})", v.to_bits()),
        NbtTag::Double(v) => format!("Double({v:?}, bits=0x{:016x})", v.to_bits()),
        _ => format!("{tag:?}").chars().take(240).collect(),
    }
}

pub(crate) fn entity_bytes(entity: &Entity) -> Vec<u8> {
    nbt_bytes(&entity.to_nbt())
}

fn nbt_bytes(tag: &NbtTag) -> Vec<u8> {
    let mut out = Vec::new();
    encode_nbt(tag, &mut out);
    out
}

fn write_len(value: usize, out: &mut Vec<u8>) {
    out.extend_from_slice(&(value as u64).to_be_bytes());
}

// Canonical typed binary representation for equality and fingerprints. JSON
// would turn nonfinite floats into null; PartialEq treats NaN as unequal.
fn encode_nbt(tag: &NbtTag, out: &mut Vec<u8>) {
    match tag {
        NbtTag::Byte(v) => { out.push(1); out.push(*v as u8); }
        NbtTag::Short(v) => { out.push(2); out.extend_from_slice(&v.to_be_bytes()); }
        NbtTag::Int(v) => { out.push(3); out.extend_from_slice(&v.to_be_bytes()); }
        NbtTag::Long(v) => { out.push(4); out.extend_from_slice(&v.to_be_bytes()); }
        NbtTag::Float(v) => { out.push(5); out.extend_from_slice(&v.to_bits().to_be_bytes()); }
        NbtTag::Double(v) => { out.push(6); out.extend_from_slice(&v.to_bits().to_be_bytes()); }
        NbtTag::ByteArray(v) => { out.push(7); write_len(v.len(), out); out.extend(v.iter().map(|v| *v as u8)); }
        NbtTag::String(v) => { out.push(8); write_len(v.len(), out); out.extend_from_slice(v.as_bytes()); }
        NbtTag::List(v) => {
            out.push(9); write_len(v.len(), out);
            for item in v.iter() { encode_nbt(item, out); }
        }
        NbtTag::Compound(v) => {
            out.push(10); write_len(v.inner().len(), out);
            let mut fields: Vec<_> = v.inner().iter().collect();
            fields.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
            for (key, value) in fields {
                write_len(key.len(), out); out.extend_from_slice(key.as_bytes());
                encode_nbt(value, out);
            }
        }
        NbtTag::IntArray(v) => {
            out.push(11); write_len(v.len(), out);
            for value in v { out.extend_from_slice(&value.to_be_bytes()); }
        }
        NbtTag::LongArray(v) => {
            out.push(12); write_len(v.len(), out);
            for value in v { out.extend_from_slice(&value.to_be_bytes()); }
        }
    }
}

pub(crate) fn digest_records(records: &[Vec<u8>]) -> String {
    let mut hasher = blake3::Hasher::new();
    for record in records {
        hasher.update(&(record.len() as u64).to_be_bytes());
        hasher.update(record);
    }
    hasher.finalize().to_hex().to_string()
}

pub(crate) fn digest_positioned(records: &BTreeMap<Pos, Vec<u8>>) -> String {
    let mut hasher = blake3::Hasher::new();
    for (pos, value) in records {
        for coordinate in [pos.0, pos.1, pos.2] { hasher.update(&coordinate.to_be_bytes()); }
        hasher.update(&(value.len() as u64).to_be_bytes());
        hasher.update(value);
    }
    hasher.finalize().to_hex().to_string()
}

pub(crate) fn blocks_digest(positions: &[Pos], source: &HashMap<Pos, &BlockState>) -> String {
    let mut hasher = blake3::Hasher::new();
    for pos in positions {
        for coordinate in [pos.0, pos.1, pos.2] { hasher.update(&coordinate.to_be_bytes()); }
        let state = nbt_bytes(&source[pos].to_nbt());
        hasher.update(&(state.len() as u64).to_be_bytes());
        hasher.update(&state);
    }
    hasher.finalize().to_hex().to_string()
}

fn container_name(name: &str) -> bool {
    let name = name.strip_prefix("minecraft:").unwrap_or(name);
    matches!(name, "hopper" | "dropper" | "dispenser" | "chest" | "trapped_chest"
        | "barrel" | "furnace" | "blast_furnace" | "smoker" | "brewing_stand"
        | "crafter" | "shulker_box" | "chiseled_bookshelf" | "decorated_pot"
        | "jukebox" | "campfire" | "soul_campfire") || name.ends_with("_shulker_box")
        || name.ends_with("_shelf")
}

fn control_name(name: &str) -> bool {
    name == "minecraft:lever" || name == "minecraft:tripwire_hook"
        || name.ends_with("_button") || name.ends_with("_pressure_plate")
}

fn inventory_items(be: &BlockEntity) -> Option<Vec<NbtTag>> {
    let nbt = be.to_nbt();
    for key in ["Items", "items"] {
        if let Ok(items) = nbt.get::<_, &quartz_nbt::NbtList>(key) {
            return Some(items.iter().cloned().collect());
        }
    }
    // Single-slot containers can store a compound rather than an Items list.
    for key in ["item", "Item", "RecordItem"] {
        if let Ok(item) = nbt.get::<_, &NbtCompound>(key) {
            return Some(if item.inner().is_empty() { Vec::new() } else { vec![NbtTag::Compound(item.clone())] });
        }
    }
    None
}

fn stack_count(tag: &NbtTag) -> u64 {
    let NbtTag::Compound(item) = tag else { return 0; };
    for key in ["count", "Count"] {
        if let Some(value) = item.inner().get(key) {
            let count = match value {
                NbtTag::Byte(v) => i64::from(*v),
                NbtTag::Short(v) => i64::from(*v),
                NbtTag::Int(v) => i64::from(*v),
                NbtTag::Long(v) => *v,
                _ => continue,
            };
            return count.max(0) as u64;
        }
    }
    0
}

fn sign_text(nbt: &NbtCompound) -> Value {
    let mut sides = BTreeMap::new();
    for side in ["front_text", "back_text"] {
        if let Ok(text) = nbt.get::<_, &NbtCompound>(side) {
            if let Ok(messages) = text.get::<_, &quartz_nbt::NbtList>("messages") {
                sides.insert(side.to_string(), messages.iter().filter_map(|tag| {
                    if let NbtTag::String(value) = tag { Some(value.clone()) } else { None }
                }).collect::<Vec<_>>());
            }
        }
    }
    let legacy: Vec<_> = ["Text1", "Text2", "Text3", "Text4"].iter()
        .filter_map(|key| nbt.get::<_, &str>(*key).ok().map(str::to_string)).collect();
    if !legacy.is_empty() { sides.insert("legacy".into(), legacy); }
    json!(sides)
}

pub(crate) fn describe_part(part: &Part, source: &HashMap<Pos, &BlockState>, selection: &BoundingBox) -> Value {
    let bes: BTreeMap<_, _> = part.schematic.get_block_entities_as_list().into_iter().map(|be| (be.position, be)).collect();
    let mut histogram = BTreeMap::<String, usize>::new();
    let mut pistons = BTreeMap::<String, usize>::new();
    let mut controls = Vec::new();
    let mut containers = Vec::new();
    let mut signs = Vec::new();
    let mut cutoff = BTreeMap::<&str, usize>::new();
    let mut observers = 0;
    let mut missing_be = 0;
    let mut inventory_present = 0;
    let mut inventory_empty = 0;
    let mut inventory_nonempty = 0;
    let mut inventory_absent = 0;
    let mut occupied_entries = 0;
    let mut item_count = 0u64;
    let mut non_air_blocks = 0;
    let mut explicit_air_states = 0;
    for pos in &part.positions {
        let state = source[pos];
        let name = state.name.as_str();
        *histogram.entry(name.to_string()).or_default() += 1;
        if is_air_state(state) {
            explicit_air_states += 1;
            continue;
        }
        non_air_blocks += 1;
        if matches!(name, "minecraft:piston" | "minecraft:sticky_piston" | "minecraft:piston_head" | "minecraft:moving_piston") {
            *pistons.entry(name.to_string()).or_default() += 1;
        }
        if name == "minecraft:observer" { observers += 1; }
        if control_name(name) { controls.push(json!({"position": pos, "state": state.to_string()})); }
        for (touches, face) in [
            (pos.0 == selection.min.0, "min_x"), (pos.0 == selection.max.0, "max_x"),
            (pos.1 == selection.min.1, "min_y"), (pos.1 == selection.max.1, "max_y"),
            (pos.2 == selection.min.2, "min_z"), (pos.2 == selection.max.2, "max_z"),
        ] {
            if touches { *cutoff.entry(face).or_default() += 1; }
        }
        if container_name(name) {
            let be = bes.get(pos);
            if be.is_none() { missing_be += 1; }
            let items = be.and_then(inventory_items);
            let (status, entries, count) = match items {
                Some(items) => {
                    inventory_present += 1;
                    if items.is_empty() { inventory_empty += 1; } else { inventory_nonempty += 1; }
                    let count: u64 = items.iter().map(stack_count).sum();
                    let entries = items.len();
                    occupied_entries += entries;
                    item_count += count;
                    (if entries == 0 { "saved_empty" } else { "saved_nonempty" }, entries, count)
                }
                None => { inventory_absent += 1; ("inventory_tag_absent", 0, 0) }
            };
            containers.push(json!({"position": pos, "state": state.to_string(), "block_entity_present": be.is_some(), "inventory_state": status, "saved_entries": entries, "saved_item_count": count}));
        }
    }
    for be in bes.values() {
        if be.id.contains("sign") {
            signs.push(json!({"position": be.position, "id": be.id.to_string(), "text": sign_text(&be.to_nbt())}));
        }
    }
    let piston_count: usize = pistons.values().sum();
    let classification = if part.quarantine { "entity_quarantine" } else if piston_count > 0 { "door_candidate" } else { "other_component" };
    let mut tags = Vec::new();
    if !cutoff.is_empty() { tags.push("plot_boundary"); }
    if part.quarantine { tags.push("unassigned_entities"); }
    if piston_count > 0 && controls.is_empty() { tags.push("no_saved_activation_control"); }
    if inventory_empty + inventory_absent > 0 { tags.push("empty_or_absent_saved_inventory"); }
    if missing_be > 0 { tags.push("missing_container_block_entity"); }
    if non_air_blocks > 20_000 || (piston_count == 0 && non_air_blocks >= 1_000) { tags.push("infrastructure_review"); }
    let moving_without_be = part.positions.iter().filter(|pos| source[*pos].name == "minecraft:moving_piston" && !bes.contains_key(*pos)).count();
    if moving_without_be > 0 { tags.push("moving_piston_without_block_entity"); }
    json!({
        "classification": classification,
        "bounds": part.schematic.get_content_bounds(),
        "block_bounds": part.block_bounds,
        "non_air_blocks": non_air_blocks,
        "preserved_block_states": part.positions.len(),
        "explicit_air_states": explicit_air_states,
        "block_entities": bes.len(),
        "entities": part.entity_indices.len(),
        "entity_source_indices": part.entity_indices,
        "block_histogram": histogram,
        "pistons": {"total": piston_count, "by_block": pistons, "moving_without_block_entity": moving_without_be},
        "observers": observers,
        "controls": {"count": controls.len(), "locations": controls},
        "containers": {"count": containers.len(), "missing_block_entities": missing_be, "locations": containers},
        "inventory": {"tags_present": inventory_present, "saved_empty": inventory_empty, "saved_nonempty": inventory_nonempty, "tag_absent": inventory_absent, "saved_entries": occupied_entries, "saved_item_count": item_count},
        "cutoff_face_contacts": cutoff,
        "review_tags": tags,
        "signs": signs,
        "functional_status": "unverified",
        "simulation_status": "not_run"
    })
}
