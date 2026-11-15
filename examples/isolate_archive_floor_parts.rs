//! Split the three retained WorldRecordPlot infrastructure components with
//! an exact platform mask, then restore bounded original platform context.
//! The canonical first-pass partition remains untouched.
#[allow(dead_code)]
#[path = "isolate_archive_plot.rs"]
mod plot;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use nucleation::block_position::BlockPosition;
use nucleation::formats::litematic::{from_litematic, to_litematic};
use nucleation::{BlockState, BoundingBox, Connectivity, UniversalSchematic};
use plot::{Part, Pos, Result};
use serde_json::{json, Value};

const CONFIG: [(&str, i32); 3] = [("part0004", 50), ("part0005", 101), ("part0006", 152)];
const MATERIALS: [&str; 3] = ["minecraft:black_stained_glass", "minecraft:white_stained_glass", "minecraft:birch_planks"];
const HALO: i32 = 1;
const BE_NORMALIZATION: &str = "authoritative_id_and_absolute_position;relative_Id_id_Pos_x_y_z_wrappers_normalized;all_other_NBT_bit_exact";

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: isolate_archive_floor_parts FIRST_PASS_EXTRACTION_DIRECTORY NEW_OUTPUT_DIRECTORY".into());
    }
    let input = PathBuf::from(&args[0]).canonicalize()?;
    let output = PathBuf::from(&args[1]);
    if output.exists() { return Err("output directory must not exist".into()); }
    if input.join("INCOMPLETE").exists() { return Err("first-pass extraction is incomplete".into()); }
    let original_manifest_bytes = fs::read(input.join("manifest.json"))?;
    let original_manifest: Value = serde_json::from_slice(&original_manifest_bytes)?;
    if original_manifest["verification"]["status"] != "passed"
        || original_manifest["verification"]["source_block_states_exactly_once"] != true
        || original_manifest["source"]["data_version"] != 4671
    { return Err("first-pass source verification/data version mismatch".into()); }
    let selection: BoundingBox = serde_json::from_value(original_manifest["source"]["selection_bounds"].clone())?;
    fs::create_dir(&output)?;
    fs::write(output.join("INCOMPLETE"), b"Floor extraction has not passed every read-back check.\n")?;
    let mut parents = Vec::new();
    let mut records = Vec::new();
    for (parent_id, floor_y) in CONFIG {
        let parent_record = original_manifest["parts"].as_array().ok_or("missing original parts")?
            .iter().find(|part| part["id"] == parent_id).ok_or("missing expected parent part")?;
        let filename = format!("{parent_id}.litematic");
        if parent_record["file"] != filename { return Err("unexpected parent filename".into()); }
        let bytes = fs::read(input.join(&filename))?;
        let file_blake3 = blake3::hash(&bytes).to_hex().to_string();
        if parent_record["file_blake3"] != file_blake3 { return Err(format!("parent {parent_id} file hash mismatch").into()); }
        let source = from_litematic(&bytes)?;
        if source.metadata.mc_version != Some(4671) { return Err("parent data version changed".into()); }
        let mut source_blocks = HashMap::new();
        for (p, state) in source.iter_blocks() {
            if state.name != "minecraft:air" && source_blocks.insert((p.x, p.y, p.z), state).is_some() {
                return Err("overlapping parent source positions".into());
            }
        }
        let source_bes = plot::block_entities(&source)?;
        let platform: HashSet<Pos> = source_blocks.iter()
            .filter(|(pos, state)| pos.1 == floor_y && MATERIALS.contains(&state.name.as_str()))
            .map(|(&pos, _)| pos).collect();
        if platform.is_empty() { return Err(format!("empty expected platform mask for {parent_id}").into()); }
        if source_bes.keys().any(|pos| platform.contains(pos)) {
            return Err("masked platform contains a block entity; requires explicit ownership review".into());
        }
        let mut masked = source.clone();
        let air = BlockState::new("minecraft:air");
        for &(x, y, z) in &platform { masked.set_block(x, y, z, &air); }
        let mut groups: Vec<Part> = masked.split_connected_attach_nearby(Connectivity::Corner, 16, 3)
            .into_iter().map(|mut schematic| {
                schematic.metadata = source.metadata.clone();
                let mut positions: Vec<_> = schematic.iter_blocks().filter(|(_, state)| state.name != "minecraft:air")
                    .map(|(p, _)| (p.x, p.y, p.z)).collect();
                positions.sort_unstable();
                Part { block_bounds: schematic.get_tight_bounds(), schematic, positions, entity_indices: Vec::new(), quarantine: false }
            }).collect();
        groups.sort_by(|a, b| plot::bounds_key(a.block_bounds.as_ref()).cmp(&plot::bounds_key(b.block_bounds.as_ref())).then_with(|| a.positions.cmp(&b.positions)));
        let component_count = groups.len();
        let mut owners = HashMap::new();
        let mut seen_bes = HashSet::new();
        for (index, group) in groups.iter().enumerate() {
            for (p, state) in group.schematic.iter_blocks() {
                if state.name == "minecraft:air" { continue; }
                let pos = (p.x, p.y, p.z);
                if platform.contains(&pos) || source_blocks.get(&pos).copied() != Some(state) || owners.insert(pos, index).is_some() {
                    return Err(format!("{parent_id}: nonplatform partition mismatch at {pos:?}").into());
                }
            }
            for (pos, nbt) in plot::block_entities(&group.schematic)? {
                if source_bes.get(&pos) != Some(&nbt) || !seen_bes.insert(pos) { return Err("nonplatform block entity mismatch".into()); }
            }
        }
        let mut omitted: Vec<_> = source_blocks.keys().filter(|pos| !platform.contains(*pos) && !owners.contains_key(*pos)).copied().collect();
        omitted.sort_unstable();
        let mut air_assignments = Vec::new();
        for pos in omitted {
            if !matches!(source_blocks[&pos].name.as_str(), "minecraft:cave_air" | "minecraft:void_air") {
                return Err(format!("{parent_id}: nonplatform state lost at {pos:?}").into());
            }
            let center = (f64::from(pos.0) + 0.5, f64::from(pos.1) + 0.5, f64::from(pos.2) + 0.5);
            let index = plot::nearest_physical_part(&groups, center, &source_blocks, true).ok_or("no physical part for explicit air")?;
            air_assignments.push((pos, index));
        }
        for (pos, index) in air_assignments {
            groups[index].schematic.set_block(pos.0, pos.1, pos.2, source_blocks[&pos]);
            groups[index].positions.push(pos);
            owners.insert(pos, index);
            let bp = BlockPosition { x: pos.0, y: pos.1, z: pos.2 };
            if let Some(be) = source.get_block_entity_owned(bp) {
                if !seen_bes.insert(pos) { return Err("duplicate explicit-air block entity".into()); }
                groups[index].schematic.set_block_entity(bp, be);
            }
        }
        if owners.len() + platform.len() != source_blocks.len() || seen_bes.len() != source_bes.len() {
            return Err(format!("{parent_id}: incomplete nonplatform partition").into());
        }
        let mut entities = source.get_entities_as_list();
        entities.sort_by_cached_key(plot::entity_bytes);
        let source_entity_bytes: Vec<_> = entities.iter().map(plot::entity_bytes).collect();
        let global_entity_indices: Vec<usize> = serde_json::from_value(parent_record["entity_source_indices"].clone())?;
        if global_entity_indices.len() != entities.len() { return Err("parent entity index count mismatch".into()); }
        let mut unassigned = Vec::new();
        for (entity_index, entity) in entities.iter().enumerate() {
            if let Some(index) = plot::nearest_physical_part(&groups, entity.position, &source_blocks, false) {
                groups[index].schematic.add_entity(entity.clone());
                groups[index].entity_indices.push(entity_index);
            } else { unassigned.push(entity_index); }
        }
        if !unassigned.is_empty() {
            let mut schematic = UniversalSchematic::new("Unassigned floor plot entities".into());
            schematic.metadata = source.metadata.clone();
            for &index in &unassigned { schematic.add_entity(entities[index].clone()); }
            groups.push(Part { schematic, positions: Vec::new(), block_bounds: None, entity_indices: unassigned, quarantine: true });
        }
        eprintln!("{parent_id}: {} masked platform cells, {component_count} physical components, {} entity quarantine", platform.len(), groups.len() - component_count);
        let source_bounds = source.get_bounding_box();
        let mut used_platform = HashSet::new();
        let mut context_total = 0;
        let mut verified_nonplatform = HashSet::new();
        let mut verified_bes = HashSet::new();
        let mut verified_entities = Vec::new();
        let mut parent_part_ids = Vec::new();
        let mut piston_candidates = 0;
        for (index, group) in groups.iter_mut().enumerate() {
            group.positions.sort_unstable();
            let nonplatform_positions = group.positions.clone();
            let nonplatform_non_air = nonplatform_positions.iter().filter(|pos| !plot::is_air_state(source_blocks[*pos])).count();
            let nonplatform_explicit_air = nonplatform_positions.len() - nonplatform_non_air;
            let mut context = Vec::new();
            if let Some(bounds) = &group.block_bounds {
                let min_x = bounds.min.0.saturating_sub(HALO).max(source_bounds.min.0);
                let max_x = bounds.max.0.saturating_add(HALO).min(source_bounds.max.0);
                let min_z = bounds.min.2.saturating_sub(HALO).max(source_bounds.min.2);
                let max_z = bounds.max.2.saturating_add(HALO).min(source_bounds.max.2);
                for x in min_x..=max_x {
                    for z in min_z..=max_z {
                        let pos = (x, floor_y, z);
                        if platform.contains(&pos) {
                            group.schematic.set_block(x, floor_y, z, source_blocks[&pos]);
                            group.positions.push(pos);
                            context.push(pos);
                            used_platform.insert(pos);
                        }
                    }
                }
            }
            context.sort_unstable();
            group.positions.sort_unstable();
            let context_set: HashSet<_> = context.iter().copied().collect();
            context_total += context.len();
            let id = format!("{parent_id}-floorpart{:04}", index + 1);
            let filename = format!("{id}.litematic");
            let data = to_litematic(&group.schematic)?;
            fs::write(output.join(&filename), &data)?;
            let disk_bytes = fs::read(output.join(&filename))?;
            if disk_bytes != data { return Err("file write changed bytes".into()); }
            let readback = from_litematic(&disk_bytes)?;
            if readback.metadata != source.metadata { return Err(format!("{id}: metadata changed").into()); }
            let mut positions = Vec::new();
            for (p, state) in readback.iter_blocks() {
                if state.name == "minecraft:air" { continue; }
                let pos = (p.x, p.y, p.z);
                if source_blocks.get(&pos).copied() != Some(state) { return Err(format!("{id}: block state changed at {pos:?}").into()); }
                if context_set.contains(&pos) {
                    if !platform.contains(&pos) { return Err("context contains nonplatform block".into()); }
                } else if owners.get(&pos) != Some(&index) || !verified_nonplatform.insert(pos) {
                    return Err(format!("{id}: nonplatform ownership mismatch at {pos:?}").into());
                }
                positions.push(pos);
            }
            positions.sort_unstable();
            if positions != group.positions { return Err(format!("{id}: readback lost/duplicated positions").into()); }
            let readback_bes = plot::block_entities(&readback)?;
            if readback_bes != plot::block_entities(&group.schematic)? {
                return Err(format!("{id}: {}", plot::block_entity_difference(&group.schematic, &readback)?).into());
            }
            for (pos, nbt) in &readback_bes {
                if source_bes.get(pos) != Some(nbt) || !verified_bes.insert(*pos) { return Err("readback block entity partition mismatch".into()); }
            }
            let mut actual_entities: Vec<_> = readback.get_entities_as_list().iter().map(plot::entity_bytes).collect();
            let mut expected_entities: Vec<_> = group.entity_indices.iter().map(|&i| source_entity_bytes[i].clone()).collect();
            actual_entities.sort(); expected_entities.sort();
            if actual_entities != expected_entities { return Err(format!("{id}: entity coordinates/NBT changed").into()); }
            verified_entities.extend(actual_entities);
            let mut record = plot::describe_part(group, &source_blocks, &selection);
            let is_door = record["pistons"]["total"].as_u64().unwrap_or(0) > 0;
            piston_candidates += usize::from(is_door);
            let object = record.as_object_mut().ok_or("description object missing")?;
            object.insert("id".into(), json!(id));
            object.insert("file".into(), json!(filename));
            object.insert("parent_part".into(), json!(parent_id));
            object.insert("parent_file_blake3".into(), json!(file_blake3));
            object.insert("classification".into(), json!(if is_door { "door_candidate" } else { "other_component" }));
            object.insert("entity_quarantine".into(), json!(group.quarantine));
            object.insert("nonplatform_bounds".into(), json!(group.block_bounds));
            object.insert("nonplatform_non_air_blocks".into(), json!(nonplatform_non_air));
            object.insert("nonplatform_preserved_block_states".into(), json!(nonplatform_positions.len()));
            object.insert("nonplatform_explicit_air_states".into(), json!(nonplatform_explicit_air));
            object.insert("nonplatform_content_blake3".into(), json!(plot::blocks_digest(&nonplatform_positions, &source_blocks)));
            object.insert("entity_source_indices".into(), json!(group.entity_indices.iter().map(|&i| global_entity_indices[i]).collect::<Vec<_>>()));
            object.insert("parent_entity_indices".into(), json!(group.entity_indices));
            object.insert("cutoff_face_contacts".into(), json!(face_contacts(&nonplatform_positions, &selection)));
            object.insert("platform_context_face_contacts".into(), json!(face_contacts(&context, &selection)));
            object.insert("restored_platform_context".into(), json!({
                "floor_y": floor_y, "horizontal_halo": HALO, "positions": context,
                "count": context.len(), "block_entities": 0,
                "content_blake3": plot::blocks_digest(&context, &source_blocks),
                "exact_source_cells_only": true, "overlap_with_other_children_allowed": true,
                "remaining_platform_retained_in_canonical_parent": true
            }));
            object.insert("mask".into(), json!({"floor_y": floor_y, "materials": MATERIALS, "applies_to_connectivity_only": true}));
            object.insert("bytes".into(), json!(data.len()));
            object.insert("file_blake3".into(), json!(blake3::hash(&data).to_hex().to_string()));
            object.insert("content_blake3".into(), json!(plot::blocks_digest(&group.positions, &source_blocks)));
            object.insert("block_entities_blake3".into(), json!(plot::digest_positioned(&readback_bes)));
            object.insert("entities_blake3".into(), json!(plot::digest_records(&expected_entities)));
            object.insert("verification".into(), json!({
                "readback": "passed", "all_states_exact_subset_of_parent": true,
                "nonplatform_ownership": "exactly_once_across_children",
                "platform_context": "exact_source_cells;overlap_allowed;not_a_partition",
                "block_entities": "exact_nbt_and_positions", "block_entity_coordinate_normalization": BE_NORMALIZATION,
                "entities": "exact_nbt_and_positions_including_passengers", "metadata": "equal_to_parent",
                "float_comparison": "IEEE_bits_including_NaN", "functional_status": "unverified", "simulation_status": "not_run"
            }));
            let tags = object.get_mut("review_tags").and_then(Value::as_array_mut).ok_or("review tags missing")?;
            tags.retain(|tag| tag != "plot_boundary" && tag != "infrastructure_review");
            if !face_contacts(&nonplatform_positions, &selection).is_empty() { tags.push(json!("plot_boundary")); }
            if !context.is_empty() { tags.push(json!("restored_platform_context")); tags.push(json!("platform_context_requires_motion_review")); }
            tags.push(json!("floor_mask_derivative"));
            tags.push(json!("adjacent_component_dependencies_unverified"));
            records.push(record);
            parent_part_ids.push(id);
        }
        verified_entities.sort();
        if verified_nonplatform.len() + platform.len() != source_blocks.len()
            || verified_bes.len() != source_bes.len() || verified_entities != source_entity_bytes
        { return Err(format!("{parent_id}: aggregate verification failed").into()); }
        let mut nonplatform_positions: Vec<_> = owners.keys().copied().collect(); nonplatform_positions.sort_unstable();
        let mut platform_positions: Vec<_> = platform.iter().copied().collect(); platform_positions.sort_unstable();
        parents.push(json!({
            "id": parent_id, "file": format!("{parent_id}.litematic"), "file_blake3": file_blake3, "bytes": bytes.len(),
            "data_version": 4671, "bounds": source_bounds, "metadata": source.metadata,
            "mask": {"floor_y": floor_y, "materials": MATERIALS, "masked_platform_cells": platform.len(), "content_blake3": plot::blocks_digest(&platform_positions, &source_blocks)},
            "nonplatform_non_air_blocks": nonplatform_positions.iter().filter(|pos| !plot::is_air_state(source_blocks[*pos])).count(),
            "nonplatform_preserved_block_states": nonplatform_positions.len(),
            "nonplatform_explicit_air_states": nonplatform_positions.iter().filter(|pos| plot::is_air_state(source_blocks[*pos])).count(),
            "nonplatform_content_blake3": plot::blocks_digest(&nonplatform_positions, &source_blocks),
            "block_entities": source_bes.len(), "entities": entities.len(), "entity_source_indices": global_entity_indices,
            "physical_components": component_count, "files": groups.len(), "entity_quarantine_files": groups.len() - component_count,
            "door_candidates": piston_candidates, "part_ids": parent_part_ids,
            "platform_context_instances": context_total, "unique_platform_context_cells": used_platform.len(),
            "platform_cells_retained_only_in_canonical_parent": platform.len() - used_platform.len(),
            "verification": {"status": "passed", "nonplatform_states_exactly_once": true,
                "block_entities_exactly_once": true, "entities_exactly_once": true,
                "context_is_exact_source_subset": true, "all_exported_files_read_back": true}
        }));
        eprintln!("{parent_id}: all {} output files verified; {piston_candidates} piston candidates", groups.len());
    }
    let manifest = json!({
        "schema_version": 1, "kind": "world_record_floor_parts",
        "original_plot": original_manifest["source"],
        "canonical_extraction_manifest_blake3": blake3::hash(&original_manifest_bytes).to_hex().to_string(),
        "canonical_partition_retained_unchanged": true,
        "isolation": {"api": "UniversalSchematic::split_connected_attach_nearby", "connectivity": "Corner",
            "min_standalone_blocks": 16, "max_air_gap": 3, "context_horizontal_halo": HALO,
            "context_rule": "original_masked_platform_cells_in_frozen_nonplatform_bbox_XZ_plus_halo;no_hole_filling",
            "coordinates": "original_source_coordinates_without_rebasing", "inventory_repairs": "none",
            "entity_assignment": "physical_bbox_containment_then_nearest_physical_block;outside_entities_quarantined",
            "platform_overlap": "allowed_and_recorded;children_are_not_a_partition_of_platform"},
        "parents": parents, "parts": records,
        "verification": {"status": "passed", "all_exported_files_read_back": true,
            "all_nonplatform_states_exactly_once": true, "all_parent_block_entities_exactly_once": true,
            "all_parent_entities_exactly_once": true, "all_context_cells_exact_parent_states": true,
            "metadata_preserved": true, "block_entity_coordinate_normalization": BE_NORMALIZATION,
            "functional_status": "unverified", "simulation_status": "not_run"},
        "warnings": ["These floor-mask derivatives are physical candidates, not validated complete doors.",
            "Platform context may overlap between children; never sum it as a disjoint source partition.",
            "The full original platform remains in the unchanged canonical first-pass parent.",
            "A one-block horizontal platform halo is source preview context, not proof of complete support or piston travel space.",
            "Nearby unowned components may be dependencies; inventories, controls and outdated mechanics are not repaired."]
    });
    fs::write(output.join("manifest.json"), serde_json::to_vec_pretty(&manifest)?)?;
    fs::remove_file(output.join("INCOMPLETE"))?;
    println!("{}", output.join("manifest.json").display());
    Ok(())
}

fn face_contacts(positions: &[Pos], bounds: &BoundingBox) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for pos in positions {
        for (touches, face) in [(pos.0 == bounds.min.0, "min_x"), (pos.0 == bounds.max.0, "max_x"),
            (pos.1 == bounds.min.1, "min_y"), (pos.1 == bounds.max.1, "max_y"),
            (pos.2 == bounds.min.2, "min_z"), (pos.2 == bounds.max.2, "max_z")] {
            if touches { *counts.entry(face).or_default() += 1; }
        }
    }
    counts
}
