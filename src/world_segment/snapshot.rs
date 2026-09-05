//! Immutable, content-addressed world observations and incremental extraction inputs.
//!
//! An archive is traversed once. Region bytes are retained verbatim in the Store;
//! chunk fingerprints describe the blocks and block-entity NBT consumed by the
//! segmenter, independently of Anvil timestamps, compression, and sector layout.
//! A manifest is published only after every selected region decodes successfully.

use std::collections::BTreeMap;
use std::io::{BufReader, Read};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::formats::world_stream::WorldSource;
use crate::store::Store;

use super::{Access, TileError, TileId, TileSource, VoxelTile, WorldSourceTiles};

pub const SNAPSHOT_SCHEMA: u32 = 1;
const MAX_REGION_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotRegion {
    pub x: i32,
    pub z: i32,
    pub object_key: String,
    pub object_hash: String,
    pub semantic_hash: String,
    pub bytes: u64,
    pub chunks: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotManifest {
    pub schema_version: u32,
    pub source_id: String,
    pub dimension: String,
    pub regions: BTreeMap<String, SnapshotRegion>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapshotProgress {
    pub protocol: u32,
    pub event: String,
    pub regions: usize,
    pub reused_regions: usize,
    #[serde(default)]
    pub unreadable_regions: usize,
    pub chunks: usize,
    pub bytes: u64,
    pub current: Option<String>,
    pub manifest_hash: Option<String>,
    pub manifest_key: Option<String>,
}

impl Default for SnapshotProgress {
    fn default() -> Self {
        Self {
            protocol: 1,
            event: "snapshot_started".into(),
            regions: 0,
            reused_regions: 0,
            unreadable_regions: 0,
            chunks: 0,
            bytes: 0,
            current: None,
            manifest_hash: None,
            manifest_key: None,
        }
    }
}

impl SnapshotManifest {
    pub fn hash(&self) -> Result<String, TileError> {
        let bytes = serde_json::to_vec(self).map_err(malformed)?;
        Ok(blake3::hash(&bytes).to_hex().to_string())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, TileError> {
        let manifest: Self = serde_json::from_slice(bytes).map_err(malformed)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), TileError> {
        if self.schema_version != SNAPSHOT_SCHEMA || self.source_id.is_empty() {
            return Err(malformed(
                "unsupported snapshot schema or empty source identity",
            ));
        }
        for (key, region) in &self.regions {
            let expected = format!("r.{}.{}.mca", region.x, region.z);
            if *key != expected
                || !valid_hash(&region.object_hash)
                || !valid_hash(&region.semantic_hash)
                || region.object_key != format!("objects/{}.mca", region.object_hash)
                || region.bytes > MAX_REGION_BYTES
                || region.x.unsigned_abs() > 120_000
                || region.z.unsigned_abs() > 120_000
            {
                return Err(malformed(format!("invalid snapshot region {key}")));
            }
            for (chunk, hash) in &region.chunks {
                let coords: Vec<_> = chunk.split(',').collect();
                let parsed = coords
                    .first()
                    .and_then(|v| v.parse::<i32>().ok())
                    .zip(coords.get(1).and_then(|v| v.parse::<i32>().ok()));
                if coords.len() != 2
                    || !valid_hash(hash)
                    || !parsed.is_some_and(|(x, z)| {
                        x.div_euclid(32) == region.x && z.div_euclid(32) == region.z
                    })
                {
                    return Err(malformed(format!("invalid chunk {chunk} in {key}")));
                }
            }
        }
        Ok(())
    }

    /// Dependency key for one complete extraction rectangle, including its halo.
    /// Missing regions/chunks affect the key as soon as they appear or disappear.
    pub fn rectangle_hash(&self, rect: (i32, i32, i32, i32)) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"nucleation.snapshot.rectangle.v1\0");
        hasher.update(self.dimension.as_bytes());
        for (key, region) in &self.regions {
            if region.x < rect.0.div_euclid(512)
                || region.x > rect.2.div_euclid(512)
                || region.z < rect.1.div_euclid(512)
                || region.z > rect.3.div_euclid(512)
            {
                continue;
            }
            if region.error.is_some() {
                hasher.update(b"unreadable-region\0");
                hasher.update(key.as_bytes());
                hasher.update(region.object_hash.as_bytes());
            }
            for (chunk, hash) in &region.chunks {
                let coords: Vec<_> = chunk.split(',').collect();
                let (Ok(x), Ok(z)) = (coords[0].parse::<i32>(), coords[1].parse::<i32>()) else {
                    continue;
                };
                if x < rect.0.div_euclid(16)
                    || x > rect.2.div_euclid(16)
                    || z < rect.1.div_euclid(16)
                    || z > rect.3.div_euclid(16)
                {
                    continue;
                }
                for part in [key.as_bytes(), chunk.as_bytes(), hash.as_bytes()] {
                    hasher.update(&(part.len() as u64).to_le_bytes());
                    hasher.update(part);
                }
            }
        }
        hasher.finalize().to_hex().to_string()
    }
}

fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

fn malformed(error: impl std::fmt::Display) -> TileError {
    TileError::Malformed(error.to_string())
}

fn io(error: impl std::fmt::Display) -> TileError {
    TileError::Io(error.to_string())
}

fn region_coords(name: &str, prefix: Option<&str>) -> Option<(String, i32, i32)> {
    let normalized = name.trim_start_matches("./");
    let path = Path::new(normalized);
    if path
        .components()
        .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return None;
    }
    let parent = path.parent()?.to_str()?;
    if let Some(prefix) = prefix {
        if parent != prefix.trim_matches('/') {
            return None;
        }
    } else if path.parent()?.file_name()?.to_str()? != "region" {
        return None;
    }
    let file = path.file_name()?.to_str()?;
    let coords = super::targz_source::parse_region_coords(&format!("region/{file}"))?;
    if coords.0.unsigned_abs() > 120_000 || coords.1.unsigned_abs() > 120_000 {
        return None;
    }
    Some((parent.to_string(), coords.0, coords.1))
}

struct Indexer<'a> {
    output: &'a dyn Store,
    previous: Option<&'a SnapshotManifest>,
    manifest: SnapshotManifest,
    progress: SnapshotProgress,
    region_prefix: Option<String>,
    report: &'a mut dyn FnMut(&SnapshotProgress),
    capture_unreadable: bool,
}

impl Indexer<'_> {
    fn add(&mut self, parent: String, x: i32, z: i32, bytes: Vec<u8>) -> Result<(), TileError> {
        if self.region_prefix.as_ref().is_some_and(|p| p != &parent) {
            return Err(malformed(
                "multiple world/dimension region directories; specify --world-prefix",
            ));
        }
        self.region_prefix = Some(parent);
        if bytes.len() as u64 > MAX_REGION_BYTES {
            return Err(malformed(format!("invalid region size at {x},{z}")));
        }
        let key = format!("r.{x}.{z}.mca");
        if self.manifest.regions.contains_key(&key) {
            return Err(malformed(format!("duplicate region {key}")));
        }
        let object_hash = blake3::hash(&bytes).to_hex().to_string();
        let object_key = format!("objects/{object_hash}.mca");
        let previous = self.previous.and_then(|p| p.regions.get(&key)).filter(|p| {
            p.object_hash == object_hash && (p.error.is_none() || self.capture_unreadable)
        });
        let decoded = if let Some(previous) = previous {
            self.progress.reused_regions += 1;
            Ok(previous.clone())
        } else {
            (|| -> Result<SnapshotRegion, TileError> {
                let source = WorldSource::from_mca_bytes(bytes.clone()).map_err(malformed)?;
                let mut chunks = BTreeMap::new();
                for chunk in source.chunks().map_err(malformed)? {
                    let chunk = chunk.map_err(malformed)?;
                    if chunk.cx().div_euclid(32) != x || chunk.cz().div_euclid(32) != z {
                        return Err(malformed(format!(
                            "chunk coordinates disagree with region {key}"
                        )));
                    }
                    // Preserve positions: a translation-invariant schematic fingerprint
                    // would miss a build moving within the same chunk.
                    let mut hash = blake3::Hasher::new();
                    hash.update(b"nucleation.chunk.blocks-nbt.v1\0");
                    let mut blocks: Vec<_> = chunk.blocks().collect();
                    blocks.sort_by_key(|(x, y, z, _)| (*x, *y, *z));
                    for (bx, by, bz, state) in blocks {
                        for coordinate in [bx, by, bz] {
                            hash.update(&coordinate.to_le_bytes());
                        }
                        let state = super::tile::palette_key(state);
                        hash.update(&(state.len() as u64).to_le_bytes());
                        hash.update(state.as_bytes());
                    }
                    let mut entities: Vec<_> = chunk.block_entities().iter().collect();
                    entities.sort_by_key(|entity| entity.position);
                    for entity in entities {
                        hash.update(b"block-entity\0");
                        hash.update(&(entity.id.len() as u64).to_le_bytes());
                        hash.update(entity.id.as_bytes());
                        for coordinate in [entity.position.0, entity.position.1, entity.position.2]
                        {
                            hash.update(&coordinate.to_le_bytes());
                        }
                        if let Some(token) =
                            crate::fingerprint::stable_nbt_token(&entity.nbt, false)
                        {
                            hash.update(&(token.len() as u64).to_le_bytes());
                            hash.update(token.as_bytes());
                        }
                    }
                    let digest = hash.finalize().to_hex().to_string();
                    chunks.insert(format!("{},{}", chunk.cx(), chunk.cz()), digest);
                }
                let semantic_hash = blake3::hash(&serde_json::to_vec(&chunks).map_err(malformed)?)
                    .to_hex()
                    .to_string();
                Ok(SnapshotRegion {
                    x,
                    z,
                    object_key: object_key.clone(),
                    object_hash: object_hash.clone(),
                    semantic_hash,
                    bytes: bytes.len() as u64,
                    chunks,
                    error: None,
                })
            })()
        };
        let region = match decoded {
            Ok(region) => region,
            Err(error) if self.capture_unreadable => SnapshotRegion {
                x,
                z,
                object_key: object_key.clone(),
                object_hash: object_hash.clone(),
                semantic_hash: object_hash.clone(),
                bytes: bytes.len() as u64,
                chunks: BTreeMap::new(),
                error: Some(error.to_string()),
            },
            Err(error) => return Err(error),
        };
        // Store::put is an atomic replacement on FsStore. Existing content is
        // verified; an interrupted earlier upload is repaired from these bytes.
        let existing = self.output.get(&object_key).map_err(io)?;
        if existing
            .as_ref()
            .map(|b| blake3::hash(b).to_hex().to_string())
            .as_deref()
            != Some(&object_hash)
        {
            self.output.put(&object_key, &bytes).map_err(io)?;
        }
        self.progress.regions += 1;
        self.progress.unreadable_regions += usize::from(region.error.is_some());
        self.progress.chunks += region.chunks.len();
        self.progress.bytes += region.bytes;
        self.progress.current = Some(key.clone());
        self.progress.event = "region_indexed".into();
        self.manifest.regions.insert(key, region);
        (self.report)(&self.progress);
        Ok(())
    }
}

/// Index an immutable backup directory, tar(.gz/.zst), or Store region prefix.
/// A directory must already be a consistent backup, not a running server world.
pub fn index_snapshot(
    input: &str,
    output: &dyn Store,
    source_id: &str,
    dimension: &str,
    prefix: Option<&str>,
    previous: Option<&SnapshotManifest>,
    report: &mut dyn FnMut(&SnapshotProgress),
) -> Result<(SnapshotManifest, SnapshotProgress), TileError> {
    index_snapshot_with_policy(
        input, output, source_id, dimension, prefix, previous, false, report,
    )
}

/// Capture damaged region bytes and their errors explicitly when requested.
/// This preserves archival evidence, NOT a promise of readable world coverage:
/// SnapshotTiles always fails when an extraction intersects a damaged region.
pub fn index_snapshot_with_policy(
    input: &str,
    output: &dyn Store,
    source_id: &str,
    dimension: &str,
    prefix: Option<&str>,
    previous: Option<&SnapshotManifest>,
    capture_unreadable: bool,
    report: &mut dyn FnMut(&SnapshotProgress),
) -> Result<(SnapshotManifest, SnapshotProgress), TileError> {
    if let Some(previous) = previous {
        previous.validate()?;
        if previous.source_id != source_id || previous.dimension != dimension {
            return Err(malformed(
                "previous manifest belongs to another world or dimension",
            ));
        }
    }
    let mut indexer = Indexer {
        output,
        previous,
        manifest: SnapshotManifest {
            schema_version: SNAPSHOT_SCHEMA,
            source_id: source_id.into(),
            dimension: dimension.into(),
            regions: BTreeMap::new(),
        },
        progress: SnapshotProgress::default(),
        region_prefix: None,
        report,
        capture_unreadable,
    };
    (indexer.report)(&indexer.progress);
    if input.contains("://") {
        let store = crate::store::open(input).map_err(io)?;
        let prefix = prefix.ok_or_else(|| malformed("Store snapshots require --world-prefix"))?;
        for key in store
            .list(&format!("{}/", prefix.trim_matches('/')))
            .map_err(io)?
        {
            if let Some((parent, x, z)) = region_coords(&key, Some(prefix)) {
                let bytes = bounded_read(store.reader(&key).map_err(io)?)?;
                indexer.add(parent, x, z, bytes)?;
            }
        }
    } else if Path::new(input).is_dir() {
        fn visit(
            root: &Path,
            dir: &Path,
            prefix: Option<&str>,
            indexer: &mut Indexer<'_>,
        ) -> Result<(), TileError> {
            for entry in std::fs::read_dir(dir).map_err(io)? {
                let entry = entry.map_err(io)?;
                let kind = entry.file_type().map_err(io)?;
                if kind.is_dir() {
                    visit(root, &entry.path(), prefix, indexer)?;
                } else if kind.is_file() {
                    let path = entry.path();
                    let name = path.strip_prefix(root).map_err(io)?.to_string_lossy();
                    if let Some((parent, x, z)) = region_coords(&name, prefix) {
                        indexer.add(
                            parent,
                            x,
                            z,
                            bounded_read(std::fs::File::open(path).map_err(io)?)?,
                        )?;
                    }
                }
            }
            Ok(())
        }
        visit(Path::new(input), Path::new(input), prefix, &mut indexer)?;
    } else {
        let mut archive = tar::Archive::new(archive_reader(input)?);
        for entry in archive.entries().map_err(io)? {
            let mut entry = entry.map_err(io)?;
            if !entry.header().entry_type().is_file() {
                continue;
            }
            let name = entry.path().map_err(io)?.to_string_lossy().to_string();
            if let Some((parent, x, z)) = region_coords(&name, prefix) {
                indexer.add(parent, x, z, bounded_read(&mut entry)?)?;
            }
        }
    }
    if indexer.manifest.regions.is_empty() {
        return Err(malformed(
            "no region files selected; check the world prefix",
        ));
    }
    indexer.manifest.validate()?;
    let hash = indexer.manifest.hash()?;
    let key = format!("manifests/{hash}.json");
    output
        .put(
            &key,
            &serde_json::to_vec(&indexer.manifest).map_err(malformed)?,
        )
        .map_err(io)?;
    indexer.progress.event = "snapshot_completed".into();
    indexer.progress.manifest_hash = Some(hash);
    indexer.progress.manifest_key = Some(key);
    indexer.progress.current = None;
    (indexer.report)(&indexer.progress);
    Ok((indexer.manifest, indexer.progress))
}

fn bounded_read(reader: impl Read) -> Result<Vec<u8>, TileError> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_REGION_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(io)?;
    if bytes.len() as u64 > MAX_REGION_BYTES {
        return Err(malformed("region exceeds 128 MiB limit"));
    }
    Ok(bytes)
}

fn archive_reader(path: &str) -> Result<Box<dyn Read>, TileError> {
    let mut file = std::fs::File::open(path).map_err(io)?;
    let mut magic = [0u8; 4];
    let n = file.read(&mut magic).map_err(io)?;
    let file = std::fs::File::open(path).map_err(io)?;
    if n >= 2 && magic[..2] == [0x1f, 0x8b] {
        Ok(Box::new(flate2::read::GzDecoder::new(BufReader::new(file))))
    } else if n == 4 && magic == [0x28, 0xb5, 0x2f, 0xfd] {
        Ok(Box::new(
            zstd::stream::read::Decoder::new(file).map_err(io)?,
        ))
    } else {
        Ok(Box::new(BufReader::new(file)))
    }
}

/// Random-access source whose object hashes are verified before decoding.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EmptyRegionPolicy {
    #[default]
    Reject,
    /// Explicit acknowledgement of missing source bytes, not proof of air.
    AcknowledgeZeroByte,
}

pub struct SnapshotTiles {
    manifest: SnapshotManifest,
    store: Box<dyn Store>,
    rect: (i32, i32, i32, i32),
    empty_region_policy: EmptyRegionPolicy,
}

impl SnapshotTiles {
    pub fn new(
        manifest: SnapshotManifest,
        store: Box<dyn Store>,
        rect: (i32, i32, i32, i32),
    ) -> Result<Self, TileError> {
        Self::with_empty_region_policy(manifest, store, rect, EmptyRegionPolicy::Reject)
    }

    pub fn with_empty_region_policy(
        manifest: SnapshotManifest,
        store: Box<dyn Store>,
        rect: (i32, i32, i32, i32),
        empty_region_policy: EmptyRegionPolicy,
    ) -> Result<Self, TileError> {
        manifest.validate()?;
        if rect.0 > rect.2 || rect.1 > rect.3 {
            return Err(malformed("snapshot rectangle minimum exceeds maximum"));
        }
        Ok(Self {
            manifest,
            store,
            rect,
            empty_region_policy,
        })
    }
}

impl TileSource for SnapshotTiles {
    fn access(&self) -> Access {
        Access::Random
    }

    fn tile_ids(&self) -> Result<Vec<TileId>, TileError> {
        let mut ids: Vec<_> = self
            .manifest
            .regions
            .values()
            .filter(|r| {
                r.x >= self.rect.0.div_euclid(512)
                    && r.x <= self.rect.2.div_euclid(512)
                    && r.z >= self.rect.1.div_euclid(512)
                    && r.z <= self.rect.3.div_euclid(512)
            })
            .map(|r| TileId { x: r.x, z: r.z })
            .collect();
        ids.sort();
        Ok(ids)
    }

    fn tile(&self, id: TileId) -> Result<Option<VoxelTile>, TileError> {
        if id.x < self.rect.0.div_euclid(512)
            || id.x > self.rect.2.div_euclid(512)
            || id.z < self.rect.1.div_euclid(512)
            || id.z > self.rect.3.div_euclid(512)
        {
            return Ok(None);
        }
        let Some(region) = self
            .manifest
            .regions
            .get(&format!("r.{}.{}.mca", id.x, id.z))
        else {
            return Ok(None);
        };
        if let Some(error) = &region.error {
            if self.empty_region_policy != EmptyRegionPolicy::AcknowledgeZeroByte
                || region.bytes != 0
                || !region.chunks.is_empty()
            {
                return Err(malformed(format!(
                    "snapshot region {},{} is unreadable: {error}",
                    id.x, id.z
                )));
            }
        }
        let bytes = bounded_read(self.store.reader(&region.object_key).map_err(io)?)?;
        if bytes.len() as u64 != region.bytes
            || blake3::hash(&bytes).to_hex().as_str() != region.object_hash
        {
            return Err(malformed(format!(
                "snapshot object failed integrity check: {}",
                region.object_key
            )));
        }
        // Verify the actual object above even for acknowledged placeholders:
        // a missing or tampered object is never an accepted coverage gap.
        if region.error.is_some() && bytes.is_empty() {
            return Ok(None);
        }
        WorldSourceTiles::new(
            WorldSource::from_mca_bytes(bytes).map_err(malformed)?,
            -64,
            320,
        )
        .with_world_rect(self.rect.0, self.rect.1, self.rect.2, self.rect.3)
        .tile(id)
    }
}
