use crate::definition_region::DefinitionRegion;
use crate::formats::error::Result;
use crate::formats::manager::{SchematicExporter, SchematicImporter};
use crate::metadata::{Metadata, SchematicProvenance, TransformationRecord};
use crate::universal_schematic::UniversalSchematic;
use crate::Region;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const MAGIC: &[u8; 4] = b"NUSN";
const VERSION: u32 = 3;

/// The metadata layout written by snapshot version 1. Keep this separate from
/// [`Metadata`]: bincode structs are positional, so adding a field to the live
/// type otherwise makes every existing snapshot unreadable.
#[derive(Serialize, Deserialize)]
struct SnapshotMetadataV1 {
    name: Option<String>,
    author: Option<String>,
    description: Option<String>,
    created: Option<u64>,
    modified: Option<u64>,
    lm_version: Option<i32>,
    mc_version: Option<i32>,
    we_version: Option<i32>,
}

#[derive(Serialize, Deserialize)]
struct SnapshotV1 {
    metadata: SnapshotMetadataV1,
    default_region: Region,
    other_regions: HashMap<String, Region>,
    default_region_name: String,
    definition_regions: HashMap<String, DefinitionRegion>,
}

/// Version 2 owns its wire schema instead of serializing
/// `UniversalSchematic` directly. Provenance is JSON inside the binary
/// envelope so its extensible, map-like schema is not coupled to bincode's
/// positional struct encoding.
#[derive(Serialize, Deserialize)]
struct SnapshotMetadataV2 {
    name: Option<String>,
    author: Option<String>,
    description: Option<String>,
    created: Option<u64>,
    modified: Option<u64>,
    lm_version: Option<i32>,
    mc_version: Option<i32>,
    we_version: Option<i32>,
    provenance_json: Option<String>,
}

#[derive(Deserialize)]
struct SnapshotV2 {
    metadata: SnapshotMetadataV2,
    default_region: Region,
    other_regions: HashMap<String, Region>,
    default_region_name: String,
    definition_regions: HashMap<String, DefinitionRegion>,
}

/// Version 3 adds transformation history without changing the positional v2
/// wire type, so existing snapshots remain readable.
#[derive(Serialize, Deserialize)]
struct SnapshotMetadataV3 {
    name: Option<String>,
    author: Option<String>,
    description: Option<String>,
    created: Option<u64>,
    modified: Option<u64>,
    lm_version: Option<i32>,
    mc_version: Option<i32>,
    we_version: Option<i32>,
    provenance_json: Option<String>,
    transformation_history_json: Option<String>,
}

#[derive(Serialize)]
struct SnapshotV3Ref<'a> {
    metadata: SnapshotMetadataV3,
    default_region: &'a Region,
    other_regions: &'a HashMap<String, Region>,
    default_region_name: &'a str,
    definition_regions: &'a HashMap<String, DefinitionRegion>,
}

#[derive(Deserialize)]
struct SnapshotV3 {
    metadata: SnapshotMetadataV3,
    default_region: Region,
    other_regions: HashMap<String, Region>,
    default_region_name: String,
    definition_regions: HashMap<String, DefinitionRegion>,
}

pub struct SnapshotFormat;

impl SchematicImporter for SnapshotFormat {
    fn name(&self) -> String {
        "snapshot".to_string()
    }

    fn detect(&self, data: &[u8]) -> bool {
        data.len() >= 4 && &data[0..4] == MAGIC
    }

    fn read(&self, data: &[u8]) -> Result<UniversalSchematic> {
        from_snapshot(data)
    }

    fn read_bounded(
        &self,
        data: &[u8],
        limits: &crate::formats::limits::DecodeLimits,
    ) -> Result<UniversalSchematic> {
        from_snapshot_bounded(data, limits)
    }
}

impl SchematicExporter for SnapshotFormat {
    fn name(&self) -> String {
        "snapshot".to_string()
    }

    fn extensions(&self) -> Vec<String> {
        vec!["nusn".to_string()]
    }

    fn available_versions(&self) -> Vec<String> {
        vec![VERSION.to_string()]
    }

    fn default_version(&self) -> String {
        VERSION.to_string()
    }

    fn write(&self, schematic: &UniversalSchematic, _version: Option<&str>) -> Result<Vec<u8>> {
        to_snapshot(schematic)
    }
}

pub fn to_snapshot(schematic: &UniversalSchematic) -> Result<Vec<u8>> {
    let provenance_json = schematic
        .metadata
        .provenance
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;
    let transformation_history_json = if schematic.metadata.transformation_history.is_empty() {
        None
    } else {
        Some(serde_json::to_string(
            &schematic.metadata.transformation_history,
        )?)
    };
    let snapshot = SnapshotV3Ref {
        metadata: SnapshotMetadataV3 {
            name: schematic.metadata.name.clone(),
            author: schematic.metadata.author.clone(),
            description: schematic.metadata.description.clone(),
            created: schematic.metadata.created,
            modified: schematic.metadata.modified,
            lm_version: schematic.metadata.lm_version,
            mc_version: schematic.metadata.mc_version,
            we_version: schematic.metadata.we_version,
            provenance_json,
            transformation_history_json,
        },
        default_region: &schematic.default_region,
        other_regions: &schematic.other_regions,
        default_region_name: &schematic.default_region_name,
        definition_regions: &schematic.definition_regions,
    };
    let payload = bincode::serialize(&snapshot)?;
    let mut buf = Vec::with_capacity(8 + payload.len());
    buf.extend_from_slice(MAGIC);
    buf.extend_from_slice(&VERSION.to_le_bytes());
    buf.extend_from_slice(&payload);
    Ok(buf)
}

pub fn from_snapshot(data: &[u8]) -> Result<UniversalSchematic> {
    from_snapshot_bounded(data, &crate::formats::limits::DecodeLimits::default())
}

pub fn from_snapshot_bounded(
    data: &[u8],
    limits: &crate::formats::limits::DecodeLimits,
) -> Result<UniversalSchematic> {
    use bincode::Options;

    limits.check_input(data)?;
    if data.len() < 8 {
        return Err("Snapshot data too short".into());
    }
    if &data[0..4] != MAGIC {
        return Err("Invalid snapshot magic bytes".into());
    }
    let version = u32::from_le_bytes(data[4..8].try_into()?);
    let options = || {
        bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes()
            .with_limit(limits.max_decompressed_bytes as u64)
    };
    let mut schematic = match version {
        1 => schematic_from_v1(options().deserialize(&data[8..])?),
        2 => schematic_from_v2(options().deserialize(&data[8..])?)?,
        VERSION => schematic_from_v3(options().deserialize(&data[8..])?)?,
        _ => return Err(format!("Unsupported snapshot version: {}", version).into()),
    };

    // Rebuild cached fields that are #[serde(skip)] on Region
    rebuild_region(&mut schematic.default_region);
    for region in schematic.other_regions.values_mut() {
        rebuild_region(region);
    }

    limits.validate_schematic(&schematic)?;
    Ok(schematic)
}

fn metadata_from_v1(metadata: SnapshotMetadataV1) -> Metadata {
    Metadata {
        name: metadata.name,
        author: metadata.author,
        description: metadata.description,
        created: metadata.created,
        modified: metadata.modified,
        lm_version: metadata.lm_version,
        mc_version: metadata.mc_version,
        we_version: metadata.we_version,
        provenance: None,
        transformation_history: Vec::new(),
        source_data_version: None,
        embedded_test: None,
        cell_contract: None,
    }
}

fn schematic_from_parts(
    metadata: Metadata,
    default_region: Region,
    other_regions: HashMap<String, Region>,
    default_region_name: String,
    definition_regions: HashMap<String, DefinitionRegion>,
) -> UniversalSchematic {
    let mut schematic = UniversalSchematic::new(default_region_name.clone());
    schematic.metadata = metadata;
    schematic.default_region = default_region;
    schematic.other_regions = other_regions;
    schematic.default_region_name = default_region_name;
    schematic.definition_regions = definition_regions;
    schematic
}

fn schematic_from_v1(snapshot: SnapshotV1) -> UniversalSchematic {
    schematic_from_parts(
        metadata_from_v1(snapshot.metadata),
        snapshot.default_region,
        snapshot.other_regions,
        snapshot.default_region_name,
        snapshot.definition_regions,
    )
}

fn schematic_from_v2(snapshot: SnapshotV2) -> Result<UniversalSchematic> {
    let provenance = snapshot
        .metadata
        .provenance_json
        .as_deref()
        .map(serde_json::from_str::<SchematicProvenance>)
        .transpose()?;
    let metadata = Metadata {
        name: snapshot.metadata.name,
        author: snapshot.metadata.author,
        description: snapshot.metadata.description,
        created: snapshot.metadata.created,
        modified: snapshot.metadata.modified,
        lm_version: snapshot.metadata.lm_version,
        mc_version: snapshot.metadata.mc_version,
        we_version: snapshot.metadata.we_version,
        provenance,
        transformation_history: Vec::new(),
        source_data_version: None,
        embedded_test: None,
        cell_contract: None,
    };
    Ok(schematic_from_parts(
        metadata,
        snapshot.default_region,
        snapshot.other_regions,
        snapshot.default_region_name,
        snapshot.definition_regions,
    ))
}

fn schematic_from_v3(snapshot: SnapshotV3) -> Result<UniversalSchematic> {
    let provenance = snapshot
        .metadata
        .provenance_json
        .as_deref()
        .map(serde_json::from_str::<SchematicProvenance>)
        .transpose()?;
    let transformation_history = snapshot
        .metadata
        .transformation_history_json
        .as_deref()
        .map(serde_json::from_str::<Vec<TransformationRecord>>)
        .transpose()?
        .unwrap_or_default();
    let metadata = Metadata {
        name: snapshot.metadata.name,
        author: snapshot.metadata.author,
        description: snapshot.metadata.description,
        created: snapshot.metadata.created,
        modified: snapshot.metadata.modified,
        lm_version: snapshot.metadata.lm_version,
        mc_version: snapshot.metadata.mc_version,
        we_version: snapshot.metadata.we_version,
        provenance,
        transformation_history,
        source_data_version: None,
        embedded_test: None,
        cell_contract: None,
    };
    Ok(schematic_from_parts(
        metadata,
        snapshot.default_region,
        snapshot.other_regions,
        snapshot.default_region_name,
        snapshot.definition_regions,
    ))
}

fn rebuild_region(region: &mut crate::region::Region) {
    region.rebuild_bbox();
    region.rebuild_palette_index();
    region.rebuild_air_index();
    region.rebuild_non_air_count();
    region.rebuild_tight_bounds();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockState;

    #[test]
    fn reads_version_1_snapshots_after_the_metadata_schema_change() {
        let mut schematic = UniversalSchematic::new("legacy".into());
        schematic.metadata.author = Some("snapshot-v1".into());
        schematic.set_block(4, 5, 6, &BlockState::new("minecraft:stone"));
        let legacy = SnapshotV1 {
            metadata: SnapshotMetadataV1 {
                name: schematic.metadata.name.clone(),
                author: schematic.metadata.author.clone(),
                description: schematic.metadata.description.clone(),
                created: schematic.metadata.created,
                modified: schematic.metadata.modified,
                lm_version: schematic.metadata.lm_version,
                mc_version: schematic.metadata.mc_version,
                we_version: schematic.metadata.we_version,
            },
            default_region: schematic.default_region.clone(),
            other_regions: schematic.other_regions.clone(),
            default_region_name: schematic.default_region_name.clone(),
            definition_regions: schematic.definition_regions.clone(),
        };
        let payload = bincode::serialize(&legacy).unwrap();
        let mut bytes = Vec::with_capacity(8 + payload.len());
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&payload);

        let restored = from_snapshot(&bytes).unwrap();
        assert_eq!(restored.metadata.author.as_deref(), Some("snapshot-v1"));
        assert_eq!(restored.get_block(4, 5, 6).unwrap().name, "minecraft:stone");
        assert!(restored.metadata.provenance.is_none());
    }
}
