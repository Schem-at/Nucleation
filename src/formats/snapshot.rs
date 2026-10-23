use crate::definition_region::DefinitionRegion;
use crate::formats::error::Result;
use crate::formats::manager::{SchematicExporter, SchematicImporter};
use crate::metadata::{Metadata, SchematicProvenance, TransformationRecord};
use crate::block_entity_store::BlockEntityStore;
use crate::entity::Entity;
use crate::universal_schematic::UniversalSchematic;
use crate::{BlockState, Region};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const MAGIC: &[u8; 4] = b"NUSN";
const VERSION: u32 = 4;

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

/// Writer for version 3, kept so tests can produce v3 files.
#[cfg(test)]
#[derive(Serialize)]
struct SnapshotV3Ref<'a> {
    metadata: SnapshotMetadataV3,
    default_region: &'a Region,
    other_regions: &'a HashMap<String, Region>,
    default_region_name: &'a str,
    definition_regions: &'a HashMap<String, DefinitionRegion>,
}

/// Version 4 owns its region wire type too. Blocks travel as one byte blob at
/// the narrowest width the palette allows (u8, u16 or u32 per cell) instead
/// of a u64 per cell, so writing is a single pass into a pre-sized buffer and
/// reading is a bounds-checked widen. Named regions are written sorted by
/// name, so equal schematics produce equal bytes.
#[derive(Serialize)]
struct SnapshotV4Ref<'a> {
    metadata: SnapshotMetadataV3,
    default_region_name: &'a str,
    default_region: RegionV4Ref<'a>,
    other_regions: Vec<(&'a str, RegionV4Ref<'a>)>,
    definition_regions: &'a HashMap<String, DefinitionRegion>,
}

#[derive(Deserialize)]
struct SnapshotV4 {
    metadata: SnapshotMetadataV3,
    default_region_name: String,
    default_region: RegionV4,
    other_regions: Vec<(String, RegionV4)>,
    definition_regions: HashMap<String, DefinitionRegion>,
}

#[derive(Serialize)]
struct RegionV4Ref<'a> {
    name: &'a str,
    position: (i32, i32, i32),
    size: (i32, i32, i32),
    palette: &'a [BlockState],
    blocks: PackedBlocks,
    entities: &'a [Entity],
    block_entities: &'a BlockEntityStore,
}

#[derive(Deserialize)]
struct RegionV4 {
    name: String,
    position: (i32, i32, i32),
    size: (i32, i32, i32),
    palette: Vec<BlockState>,
    blocks: PackedBlocks,
    entities: Vec<Entity>,
    block_entities: BlockEntityStore,
}

impl<'a> RegionV4Ref<'a> {
    fn new(region: &'a Region) -> Self {
        RegionV4Ref {
            name: &region.name,
            position: region.position,
            size: region.size,
            palette: &region.palette,
            blocks: PackedBlocks::encode(&region.dense_blocks(), region.palette.len()),
            entities: &region.entities,
            block_entities: &region.block_entities,
        }
    }
}

/// Palette indices as `[width, little-endian cells...]`, serialized as bytes.
struct PackedBlocks(Vec<u8>);

impl PackedBlocks {
    fn encode(blocks: &[u32], palette_len: usize) -> Self {
        let mut out;
        if palette_len <= 1 << 8 {
            out = Vec::with_capacity(1 + blocks.len());
            out.push(1);
            out.extend(blocks.iter().map(|&b| b as u8));
        } else if palette_len <= 1 << 16 {
            out = Vec::with_capacity(1 + blocks.len() * 2);
            out.push(2);
            for &b in blocks {
                out.extend_from_slice(&(b as u16).to_le_bytes());
            }
        } else {
            out = Vec::with_capacity(1 + blocks.len() * 4);
            out.push(4);
            for &b in blocks {
                out.extend_from_slice(&b.to_le_bytes());
            }
        }
        PackedBlocks(out)
    }

    fn decode(&self, volume: usize) -> Result<Vec<u32>> {
        let (&width, cells) = self.0.split_first().ok_or("Snapshot region has no block data")?;
        let width = width as usize;
        if !matches!(width, 1 | 2 | 4) || volume.checked_mul(width) != Some(cells.len()) {
            return Err("Snapshot block data does not match the region volume".into());
        }
        Ok(match width {
            1 => cells.iter().map(|&b| b as u32).collect(),
            2 => cells
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]) as u32)
                .collect(),
            _ => cells
                .chunks_exact(4)
                .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect(),
        })
    }
}

impl Serialize for PackedBlocks {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for PackedBlocks {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct BytesVisitor;
        impl<'de> serde::de::Visitor<'de> for BytesVisitor {
            type Value = PackedBlocks;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("packed block bytes")
            }
            fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> std::result::Result<Self::Value, E> {
                Ok(PackedBlocks(v.to_vec()))
            }
            fn visit_byte_buf<E: serde::de::Error>(self, v: Vec<u8>) -> std::result::Result<Self::Value, E> {
                Ok(PackedBlocks(v))
            }
        }
        deserializer.deserialize_byte_buf(BytesVisitor)
    }
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
    let mut named: Vec<(&str, &Region)> = schematic
        .other_regions
        .iter()
        .map(|(name, region)| (name.as_str(), region))
        .collect();
    named.sort_unstable_by(|a, b| a.0.cmp(b.0));
    let snapshot = SnapshotV4Ref {
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
        default_region_name: &schematic.default_region_name,
        default_region: RegionV4Ref::new(&schematic.default_region),
        other_regions: named
            .into_iter()
            .map(|(name, region)| (name, RegionV4Ref::new(region)))
            .collect(),
        definition_regions: &schematic.definition_regions,
    };
    let payload_len = bincode::serialized_size(&snapshot)? as usize;
    let mut buf = Vec::with_capacity(8 + payload_len);
    buf.extend_from_slice(MAGIC);
    buf.extend_from_slice(&VERSION.to_le_bytes());
    bincode::serialize_into(&mut buf, &snapshot)?;
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
        3 => schematic_from_v3(options().deserialize(&data[8..])?)?,
        VERSION => schematic_from_v4(options().deserialize(&data[8..])?)?,
        _ => return Err(format!("Unsupported snapshot version: {}", version).into()),
    };

    // Rebuild cached fields that are #[serde(skip)] on Region
    rebuild_region(&mut schematic.default_region)?;
    for region in schematic.other_regions.values_mut() {
        rebuild_region(region)?;
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

fn schematic_from_v4(snapshot: SnapshotV4) -> Result<UniversalSchematic> {
    let metadata = metadata_from_v3(snapshot.metadata)?;
    let region = |wire: RegionV4| -> Result<Region> {
        let mut region = Region::new(wire.name, (0, 0, 0), (1, 1, 1));
        let volume = [wire.size.0, wire.size.1, wire.size.2]
            .iter()
            .try_fold(1usize, |acc, &d| usize::try_from(d).ok().and_then(|d| acc.checked_mul(d)))
            .ok_or("Snapshot region size is invalid")?;
        region.set_storage(wire.position, wire.size, wire.blocks.decode(volume)?);
        region.palette = wire.palette;
        region.entities = wire.entities;
        region.block_entities = wire.block_entities;
        Ok(region)
    };
    let default_region = region(snapshot.default_region)?;
    let other_regions = snapshot
        .other_regions
        .into_iter()
        .map(|(name, wire)| Ok((name, region(wire)?)))
        .collect::<Result<HashMap<_, _>>>()?;
    Ok(schematic_from_parts(
        metadata,
        default_region,
        other_regions,
        snapshot.default_region_name,
        snapshot.definition_regions,
    ))
}

fn metadata_from_v3(metadata: SnapshotMetadataV3) -> Result<Metadata> {
    let provenance = metadata
        .provenance_json
        .as_deref()
        .map(serde_json::from_str::<SchematicProvenance>)
        .transpose()?;
    let transformation_history = metadata
        .transformation_history_json
        .as_deref()
        .map(serde_json::from_str::<Vec<TransformationRecord>>)
        .transpose()?
        .unwrap_or_default();
    Ok(Metadata {
        name: metadata.name,
        author: metadata.author,
        description: metadata.description,
        created: metadata.created,
        modified: metadata.modified,
        lm_version: metadata.lm_version,
        mc_version: metadata.mc_version,
        we_version: metadata.we_version,
        provenance,
        transformation_history,
        source_data_version: None,
        embedded_test: None,
        cell_contract: None,
    })
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

/// Rebuild the `#[serde(skip)]` caches, after checking that the decoded block
/// array matches the region volume and only references palette entries.
fn rebuild_region(region: &mut Region) -> Result<()> {
    let volume = [region.size.0, region.size.1, region.size.2]
        .iter()
        .try_fold(1usize, |acc, &d| usize::try_from(d).ok().and_then(|d| acc.checked_mul(d)));
    if volume != Some(region.blocks.len()) {
        return Err("Snapshot block data does not match the region volume".into());
    }
    let max_index = region.blocks.iter().copied().max();
    if max_index.is_some_and(|max| max as usize >= region.palette.len()) {
        return Err("Snapshot block data references a missing palette entry".into());
    }
    region.adopt_storage();
    region.rebuild_palette_index();
    region.rebuild_air_index();
    region.rescan_content();
    Ok(())
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

    fn sample() -> UniversalSchematic {
        let mut schematic = UniversalSchematic::new("sample".into());
        schematic.metadata.author = Some("snapshot".into());
        for i in 0..40 {
            schematic.set_block(i % 7, i % 5, i % 3, &BlockState::new(format!("minecraft:b{i}")));
        }
        schematic
            .set_block_from_string(
                1,
                1,
                1,
                "minecraft:chest[facing=west]{Items:[{Slot:0b,id:\"minecraft:diamond\",Count:2b}]}",
            )
            .unwrap();
        schematic.set_block_in_region("Side", -3, 2, 9, &BlockState::new("minecraft:glass"));
        schematic
    }

    fn assert_same_content(a: &UniversalSchematic, b: &UniversalSchematic) {
        let cells = |s: &UniversalSchematic| {
            let mut v: Vec<_> = s
                .iter_blocks()
                .filter(|(_, b)| b.name != "minecraft:air")
                .map(|(p, b)| ((p.x, p.y, p.z), b.to_string()))
                .collect();
            v.sort();
            v
        };
        assert_eq!(cells(a), cells(b));
        assert_eq!(
            a.get_block_entity(crate::block_position::BlockPosition { x: 1, y: 1, z: 1 }),
            b.get_block_entity(crate::block_position::BlockPosition { x: 1, y: 1, z: 1 })
        );
        assert_eq!(a.metadata.author, b.metadata.author);
    }

    #[test]
    fn version_4_round_trips_and_is_deterministic() {
        let schematic = sample();
        let bytes = to_snapshot(&schematic).unwrap();
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 4);
        assert_eq!(bytes, to_snapshot(&schematic.clone()).unwrap());
        assert_same_content(&schematic, &from_snapshot(&bytes).unwrap());
    }

    #[test]
    fn version_4_uses_wider_cells_for_large_palettes() {
        let mut schematic = UniversalSchematic::new("wide".into());
        for i in 0..300 {
            schematic.set_block(i, 0, 0, &BlockState::new(format!("minecraft:b{i}")));
        }
        let restored = from_snapshot(&to_snapshot(&schematic).unwrap()).unwrap();
        assert_same_content(&schematic, &restored);
    }

    #[test]
    fn reads_version_3_snapshots() {
        let schematic = sample();
        assert_same_content(&schematic, &from_snapshot(&v3_bytes(&schematic)).unwrap());
    }

    fn v3_bytes(schematic: &UniversalSchematic) -> Vec<u8> {
        let snapshot = SnapshotV3Ref {
            metadata: SnapshotMetadataV3 {
                name: None,
                author: schematic.metadata.author.clone(),
                description: None,
                created: None,
                modified: None,
                lm_version: None,
                mc_version: None,
                we_version: None,
                provenance_json: None,
                transformation_history_json: None,
            },
            default_region: &schematic.default_region,
            other_regions: &schematic.other_regions,
            default_region_name: &schematic.default_region_name,
            definition_regions: &schematic.definition_regions,
        };
        let mut bytes = MAGIC.to_vec();
        bytes.extend_from_slice(&3u32.to_le_bytes());
        bytes.extend_from_slice(&bincode::serialize(&snapshot).unwrap());
        bytes
    }

    #[test]
    fn rejects_block_data_that_does_not_fit_the_region() {
        let mut bytes = to_snapshot(&sample()).unwrap();
        // Truncating the payload must fail cleanly, never panic.
        bytes.truncate(bytes.len() - 3);
        assert!(from_snapshot(&bytes).is_err());

        // An index past the palette (writable through v3's u64 cells) must be
        // rejected at load instead of panicking on first access.
        let mut schematic = sample();
        schematic.default_region.blocks[0] = 10_000;
        assert!(from_snapshot(&v3_bytes(&schematic)).is_err());
    }
}
