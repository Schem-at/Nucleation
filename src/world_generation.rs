//! Lazy, random-access chunk generation from immutable world sources.
//!
//! A source describes an unbounded or partially covered world. A request always
//! names one bounded chunk column, keeping scheduling, caching, and persistence
//! separate from generation.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use crate::building::{Brush, BrushEnum, PolygonPrism, Shape};
use crate::formats::anvil::ChunkSection;
use crate::formats::world_stream::WorldChunkView;
use crate::sdf::SdfNode;
use crate::BlockState;
use rustc_hash::FxHashMap;
use thiserror::Error;

const MAX_PROVENANCE_PART_BYTES: usize = 256;
const MAX_COMPOSITE_LAYERS: usize = 64;
const MAX_PROJECTED_FEATURES: usize = 1_000_000;
const MAX_PROJECTED_VERTICES: usize = 1_000_000;
const MAX_PROJECTED_VERTICES_PER_FEATURE: usize = 100_000;
const PROJECTED_INDEX_REGION_BLOCKS: i32 = 512;
const MAX_INDEX_REGIONS_PER_FEATURE: u64 = 4_096;
const MAX_TOTAL_INDEX_REFERENCES: u64 = 4_000_000;
const MAX_CELLULAR_CANDIDATES_PER_CHUNK: u64 = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkCoverage {
    Complete,
    Partial,
    Outside,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProvenance {
    source_id: String,
    version: String,
}

impl SourceProvenance {
    pub fn new(
        source_id: impl Into<String>,
        version: impl Into<String>,
    ) -> Result<Self, WorldGenerationError> {
        let source_id = source_id.into();
        let version = version.into();
        if source_id.is_empty()
            || version.is_empty()
            || source_id.len() > MAX_PROVENANCE_PART_BYTES
            || version.len() > MAX_PROVENANCE_PART_BYTES
        {
            return Err(WorldGenerationError::InvalidProvenance);
        }
        Ok(Self { source_id, version })
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkRequest {
    cx: i32,
    cz: i32,
}

impl ChunkRequest {
    pub fn new(cx: i32, cz: i32) -> Self {
        Self { cx, cz }
    }

    pub fn cx(&self) -> i32 {
        self.cx
    }

    pub fn cz(&self) -> i32 {
        self.cz
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkBounds {
    min_cx: i32,
    min_cz: i32,
    max_cx: i32,
    max_cz: i32,
    count: u64,
}

impl ChunkBounds {
    pub fn new(
        min_cx: i32,
        min_cz: i32,
        max_cx: i32,
        max_cz: i32,
    ) -> Result<Self, WorldGenerationError> {
        if min_cx > max_cx || min_cz > max_cz {
            return Err(WorldGenerationError::InvalidChunkBounds);
        }
        let width = (i64::from(max_cx) - i64::from(min_cx) + 1) as u64;
        let depth = (i64::from(max_cz) - i64::from(min_cz) + 1) as u64;
        let count = width
            .checked_mul(depth)
            .ok_or(WorldGenerationError::TooManyChunks)?;
        Ok(Self {
            min_cx,
            min_cz,
            max_cx,
            max_cz,
            count,
        })
    }

    pub fn min_cx(&self) -> i32 {
        self.min_cx
    }

    pub fn min_cz(&self) -> i32 {
        self.min_cz
    }

    pub fn max_cx(&self) -> i32 {
        self.max_cx
    }

    pub fn max_cz(&self) -> i32 {
        self.max_cz
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    fn first(&self) -> (i32, i32) {
        (self.min_cx, self.min_cz)
    }

    fn next_after(&self, cx: i32, cz: i32) -> Option<(i32, i32)> {
        let rx = cx.div_euclid(32);
        let rz = cz.div_euclid(32);
        let rx_max = self.max_cx.div_euclid(32);
        let rz_min = self.min_cz.div_euclid(32);
        let rz_max = self.max_cz.div_euclid(32);
        let region_x_min = self.min_cx.max(rx * 32);
        let region_x_max = self.max_cx.min(rx * 32 + 31);
        let region_z_max = self.max_cz.min(rz * 32 + 31);

        if cx < region_x_max {
            return Some((cx + 1, cz));
        }
        if cz < region_z_max {
            return Some((region_x_min, cz + 1));
        }
        if rz < rz_max {
            let next_rz = rz + 1;
            return Some((region_x_min, self.min_cz.max(next_rz * 32)));
        }
        if rx < rx_max {
            let next_rx = rx + 1;
            return Some((self.min_cx.max(next_rx * 32), self.min_cz.max(rz_min * 32)));
        }
        None
    }
}

pub struct ChunkResult {
    chunk: WorldChunkView,
    coverage: ChunkCoverage,
    provenance: SourceProvenance,
}

impl ChunkResult {
    pub fn new(
        chunk: WorldChunkView,
        coverage: ChunkCoverage,
        provenance: SourceProvenance,
    ) -> Self {
        Self {
            chunk,
            coverage,
            provenance,
        }
    }

    pub fn chunk(&self) -> &WorldChunkView {
        &self.chunk
    }

    pub fn into_chunk(self) -> WorldChunkView {
        self.chunk
    }

    pub fn coverage(&self) -> ChunkCoverage {
        self.coverage
    }

    pub fn provenance(&self) -> &SourceProvenance {
        &self.provenance
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum WorldGenerationError {
    #[error("minimum Y must not exceed maximum Y")]
    InvalidYBounds,
    #[error("Y bounds are outside the chunk section format")]
    YBoundsOutOfRange,
    #[error("source provenance must contain non-empty ID and version values of at most 256 bytes")]
    InvalidProvenance,
    #[error("invalid SDF: {0}")]
    InvalidSdf(String),
    #[error("chunk coordinates overflow absolute block coordinates")]
    CoordinateOverflow,
    #[error("chunk coordinates cannot be sampled at exact f32 voxel centers")]
    CoordinatePrecision,
    #[error("a chunk source returned coordinates different from its request")]
    MismatchedChunkCoordinates,
    #[error("minimum chunk coordinates must not exceed maximum chunk coordinates")]
    InvalidChunkBounds,
    #[error("chunk bounds contain more chunks than can be counted")]
    TooManyChunks,
    #[error("a composite source may contain at most 64 layers")]
    TooManySourceLayers,
    #[error("invalid projected footprint data: {0}")]
    InvalidProjectedFootprints(String),
    #[error("invalid cellular SDF source: {0}")]
    InvalidCellularSource(String),
}

pub trait ChunkSource: Send + Sync {
    fn generate(&self, request: ChunkRequest) -> Result<ChunkResult, WorldGenerationError>;
}

pub struct GeneratedChunkStream {
    source: Arc<dyn ChunkSource>,
    bounds: ChunkBounds,
    next: Option<(i32, i32)>,
    remaining: u64,
}

impl GeneratedChunkStream {
    pub fn new(source: Arc<dyn ChunkSource>, bounds: ChunkBounds) -> Self {
        Self {
            source,
            bounds,
            next: Some(bounds.first()),
            remaining: bounds.count(),
        }
    }

    pub fn remaining(&self) -> u64 {
        self.remaining
    }
}

impl Iterator for GeneratedChunkStream {
    type Item = Result<ChunkResult, WorldGenerationError>;

    fn next(&mut self) -> Option<Self::Item> {
        let (cx, cz) = self.next?;
        self.next = self.bounds.next_after(cx, cz);
        self.remaining -= 1;
        Some(
            self.source
                .generate(ChunkRequest::new(cx, cz))
                .and_then(|result| {
                    if result.chunk().cx() != cx || result.chunk().cz() != cz {
                        Err(WorldGenerationError::MismatchedChunkCoordinates)
                    } else {
                        Ok(result)
                    }
                }),
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = usize::try_from(self.remaining).unwrap_or(usize::MAX);
        (remaining, Some(remaining))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkOverlayMode {
    Replace,
    KeepExisting,
}

#[derive(Clone)]
struct ChunkSourceLayer {
    source: Arc<dyn ChunkSource>,
    mode: ChunkOverlayMode,
}

#[derive(Clone)]
pub struct CompositeChunkSource {
    layers: Vec<ChunkSourceLayer>,
    provenance: SourceProvenance,
}

impl CompositeChunkSource {
    pub fn new(provenance: SourceProvenance) -> Self {
        Self {
            layers: Vec::new(),
            provenance,
        }
    }

    pub fn add_layer(
        &mut self,
        source: Arc<dyn ChunkSource>,
        mode: ChunkOverlayMode,
    ) -> Result<(), WorldGenerationError> {
        if self.layers.len() >= MAX_COMPOSITE_LAYERS {
            return Err(WorldGenerationError::TooManySourceLayers);
        }
        self.layers.push(ChunkSourceLayer { source, mode });
        Ok(())
    }
}

impl ChunkSource for CompositeChunkSource {
    fn generate(&self, request: ChunkRequest) -> Result<ChunkResult, WorldGenerationError> {
        let mut chunk = WorldChunkView::new(request.cx, request.cz);
        let mut coverage = ChunkCoverage::Outside;

        for layer in &self.layers {
            let result = layer.source.generate(request)?;
            if result.chunk().cx() != request.cx || result.chunk().cz() != request.cz {
                return Err(WorldGenerationError::MismatchedChunkCoordinates);
            }
            coverage = match (coverage, result.coverage()) {
                (ChunkCoverage::Complete, _) | (_, ChunkCoverage::Complete) => {
                    ChunkCoverage::Complete
                }
                (ChunkCoverage::Partial, _) | (_, ChunkCoverage::Partial) => ChunkCoverage::Partial,
                _ => ChunkCoverage::Outside,
            };
            if result.coverage() == ChunkCoverage::Outside {
                continue;
            }

            for (x, y, z, block) in result.chunk().blocks() {
                let should_write = match layer.mode {
                    ChunkOverlayMode::Replace => true,
                    ChunkOverlayMode::KeepExisting => chunk
                        .get_block(x, y, z)
                        .map(|existing| {
                            matches!(
                                existing.get_name(),
                                "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                            )
                        })
                        .unwrap_or(true),
                };
                if should_write {
                    chunk.set_block(x, y, z, block);
                }
            }
        }

        Ok(ChunkResult::new(chunk, coverage, self.provenance.clone()))
    }
}

struct SectionAccumulator {
    y: i8,
    palette: Vec<BlockState>,
    palette_indices: FxHashMap<BlockState, u16>,
    block_states: Vec<u16>,
}

impl SectionAccumulator {
    fn new(y: i8) -> Self {
        let air = BlockState::new("minecraft:air");
        let mut palette_indices = FxHashMap::default();
        palette_indices.insert(air.clone(), 0);
        Self {
            y,
            palette: vec![air],
            palette_indices,
            block_states: vec![0; 4096],
        }
    }

    fn set(&mut self, local_x: usize, local_y: usize, local_z: usize, block: BlockState) {
        if matches!(
            block.get_name(),
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
        ) {
            return;
        }
        let palette_index = if let Some(index) = self.palette_indices.get(&block) {
            *index
        } else {
            let index = self.palette.len() as u16;
            self.palette.push(block.clone());
            self.palette_indices.insert(block, index);
            index
        };
        self.block_states[local_y * 256 + local_z * 16 + local_x] = palette_index;
    }

    fn finish(self) -> ChunkSection {
        ChunkSection {
            y: self.y,
            palette: self.palette,
            block_states: self.block_states,
            biomes: None,
        }
    }
}

#[derive(Clone)]
struct IndexedProjectedFootprint {
    prism: PolygonPrism,
    bounds: (i32, i32, i32, i32, i32, i32),
    block: BlockState,
}

/// A sparse, random-access source for already-projected OSM-style footprints.
///
/// Fetching and geographic projection intentionally remain caller concerns;
/// this source indexes world-coordinate polygons and rasterizes only the
/// requested 16×16 chunk column.
#[derive(Clone)]
pub struct ProjectedFootprintChunkSource {
    footprints: Vec<IndexedProjectedFootprint>,
    region_index: HashMap<(i32, i32), Vec<usize>>,
    global_indices: Vec<usize>,
    data_bounds: Option<(i32, i32, i32, i32)>,
    base_block: Option<BlockState>,
    provenance: SourceProvenance,
}

impl ProjectedFootprintChunkSource {
    pub fn new(
        footprints: Vec<crate::geo::Footprint>,
        base_block: Option<String>,
        provenance: SourceProvenance,
    ) -> Result<Self, WorldGenerationError> {
        if footprints.len() > MAX_PROJECTED_FEATURES {
            return Err(WorldGenerationError::InvalidProjectedFootprints(
                "too many features".to_string(),
            ));
        }
        let mut indexed = Vec::with_capacity(footprints.len());
        let mut total_vertices = 0usize;
        let mut data_bounds: Option<(i32, i32, i32, i32)> = None;

        for footprint in footprints {
            if footprint.polygon.len() < 3
                || footprint.polygon.len() > MAX_PROJECTED_VERTICES_PER_FEATURE
            {
                return Err(WorldGenerationError::InvalidProjectedFootprints(
                    "each polygon must contain 3..=100000 vertices".to_string(),
                ));
            }
            total_vertices = total_vertices
                .checked_add(footprint.polygon.len())
                .ok_or_else(|| {
                    WorldGenerationError::InvalidProjectedFootprints(
                        "vertex count overflow".to_string(),
                    )
                })?;
            if total_vertices > MAX_PROJECTED_VERTICES {
                return Err(WorldGenerationError::InvalidProjectedFootprints(
                    "too many vertices".to_string(),
                ));
            }
            if footprint.y_min > footprint.y_max
                || footprint.y_min.div_euclid(16) < i8::MIN as i32
                || footprint.y_max.div_euclid(16) > i8::MAX as i32
            {
                return Err(WorldGenerationError::InvalidProjectedFootprints(
                    "invalid Y bounds".to_string(),
                ));
            }
            if footprint.block.is_empty()
                || matches!(
                    footprint.block.as_str(),
                    "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                )
            {
                return Err(WorldGenerationError::InvalidProjectedFootprints(
                    "footprint block must be non-air".to_string(),
                ));
            }
            if footprint.polygon.iter().any(|&(x, z)| {
                !x.is_finite()
                    || !z.is_finite()
                    || x < i32::MIN as f64
                    || x > i32::MAX as f64
                    || z < i32::MIN as f64
                    || z > i32::MAX as f64
            }) {
                return Err(WorldGenerationError::InvalidProjectedFootprints(
                    "polygon coordinates must be finite i32-world coordinates".to_string(),
                ));
            }

            let prism = PolygonPrism::new(footprint.polygon, footprint.y_min, footprint.y_max);
            let bounds = prism.bounds();
            data_bounds = Some(match data_bounds {
                Some((min_x, min_z, max_x, max_z)) => (
                    min_x.min(bounds.0),
                    min_z.min(bounds.2),
                    max_x.max(bounds.3),
                    max_z.max(bounds.5),
                ),
                None => (bounds.0, bounds.2, bounds.3, bounds.5),
            });
            indexed.push(IndexedProjectedFootprint {
                prism,
                bounds,
                block: BlockState::new(footprint.block),
            });
        }
        indexed.sort_by_key(|feature| feature.bounds.4);

        let mut region_index = HashMap::<(i32, i32), Vec<usize>>::new();
        let mut global_indices = Vec::new();
        let mut index_references = 0u64;
        for (index, feature) in indexed.iter().enumerate() {
            let min_rx = feature.bounds.0.div_euclid(PROJECTED_INDEX_REGION_BLOCKS);
            let max_rx = feature.bounds.3.div_euclid(PROJECTED_INDEX_REGION_BLOCKS);
            let min_rz = feature.bounds.2.div_euclid(PROJECTED_INDEX_REGION_BLOCKS);
            let max_rz = feature.bounds.5.div_euclid(PROJECTED_INDEX_REGION_BLOCKS);
            let region_count = (i64::from(max_rx) - i64::from(min_rx) + 1) as u64
                * (i64::from(max_rz) - i64::from(min_rz) + 1) as u64;
            if region_count <= MAX_INDEX_REGIONS_PER_FEATURE
                && index_references
                    .checked_add(region_count)
                    .is_some_and(|total| total <= MAX_TOTAL_INDEX_REFERENCES)
            {
                for rx in min_rx..=max_rx {
                    for rz in min_rz..=max_rz {
                        region_index.entry((rx, rz)).or_default().push(index);
                    }
                }
                index_references += region_count;
            } else {
                // Very large polygons stay queryable without expanding into an
                // unbounded number of index buckets.
                global_indices.push(index);
            }
        }

        let base_block = match base_block {
            Some(block)
                if block.is_empty()
                    || matches!(
                        block.as_str(),
                        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                    ) =>
            {
                return Err(WorldGenerationError::InvalidProjectedFootprints(
                    "base block must be non-air".to_string(),
                ));
            }
            Some(block) => Some(BlockState::new(block)),
            None => None,
        };

        Ok(Self {
            footprints: indexed,
            region_index,
            global_indices,
            data_bounds,
            base_block,
            provenance,
        })
    }
}

fn horizontal_bounds_intersect(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> bool {
    a.0 <= b.2 && a.2 >= b.0 && a.1 <= b.3 && a.3 >= b.1
}

fn set_generated_block(
    sections: &mut BTreeMap<i8, SectionAccumulator>,
    x0: i32,
    z0: i32,
    x: i32,
    y: i32,
    z: i32,
    block: BlockState,
) {
    let section_y = y.div_euclid(16) as i8;
    sections
        .entry(section_y)
        .or_insert_with(|| SectionAccumulator::new(section_y))
        .set(
            (x - x0) as usize,
            y.rem_euclid(16) as usize,
            (z - z0) as usize,
            block,
        );
}

impl ChunkSource for ProjectedFootprintChunkSource {
    fn generate(&self, request: ChunkRequest) -> Result<ChunkResult, WorldGenerationError> {
        let x0 = request
            .cx
            .checked_mul(16)
            .ok_or(WorldGenerationError::CoordinateOverflow)?;
        let z0 = request
            .cz
            .checked_mul(16)
            .ok_or(WorldGenerationError::CoordinateOverflow)?;
        let x1 = x0
            .checked_add(15)
            .ok_or(WorldGenerationError::CoordinateOverflow)?;
        let z1 = z0
            .checked_add(15)
            .ok_or(WorldGenerationError::CoordinateOverflow)?;
        let chunk_bounds = (x0, z0, x1, z1);
        let mut sections = BTreeMap::<i8, SectionAccumulator>::new();
        let mut covered = false;

        if let (Some(base), Some(data_bounds)) = (&self.base_block, self.data_bounds) {
            if horizontal_bounds_intersect(chunk_bounds, data_bounds) {
                covered = true;
                for x in x0.max(data_bounds.0)..=x1.min(data_bounds.2) {
                    for z in z0.max(data_bounds.1)..=z1.min(data_bounds.3) {
                        set_generated_block(&mut sections, x0, z0, x, 0, z, base.clone());
                    }
                }
            }
        }

        let region_key = (
            x0.div_euclid(PROJECTED_INDEX_REGION_BLOCKS),
            z0.div_euclid(PROJECTED_INDEX_REGION_BLOCKS),
        );
        let mut candidates = self.global_indices.clone();
        if let Some(local) = self.region_index.get(&region_key) {
            candidates.extend_from_slice(local);
        }
        // Indices follow the stable y_max ordering established at construction,
        // preserving "tallest feature wins" where footprints overlap.
        candidates.sort_unstable();

        for index in candidates {
            let feature = &self.footprints[index];
            let feature_bounds = (
                feature.bounds.0,
                feature.bounds.2,
                feature.bounds.3,
                feature.bounds.5,
            );
            if !horizontal_bounds_intersect(chunk_bounds, feature_bounds) {
                continue;
            }
            covered = true;
            for x in x0.max(feature.bounds.0)..=x1.min(feature.bounds.3) {
                for z in z0.max(feature.bounds.2)..=z1.min(feature.bounds.5) {
                    if !feature.prism.contains(x, feature.bounds.1, z) {
                        continue;
                    }
                    for y in feature.bounds.1..=feature.bounds.4 {
                        set_generated_block(&mut sections, x0, z0, x, y, z, feature.block.clone());
                    }
                }
            }
        }

        let chunk = WorldChunkView::from_generated_sections(
            request.cx,
            request.cz,
            sections
                .into_values()
                .map(SectionAccumulator::finish)
                .collect(),
        );
        Ok(ChunkResult::new(
            chunk,
            if covered {
                ChunkCoverage::Partial
            } else {
                ChunkCoverage::Outside
            },
            self.provenance.clone(),
        ))
    }
}

/// Deterministic per-cell variation applied to a bounded SDF motif.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellularSdfConfig {
    pub cell_size_x: i32,
    pub cell_size_z: i32,
    pub seed: u64,
    pub max_jitter_x: f32,
    pub max_jitter_z: f32,
    pub max_yaw_degrees: f32,
    pub min_scale: f32,
    pub max_scale: f32,
    pub min_y_offset: i32,
    pub max_y_offset: i32,
    pub presence_numerator: u32,
    pub presence_denominator: u32,
    pub feature_salt: u64,
}

impl CellularSdfConfig {
    /// Check every invariant the cellular placement math depends on.
    ///
    /// This is the single source of truth: `CellularSdfChunkSource::new` and the
    /// language bindings' config constructor both call it, so a config that is
    /// accepted at construction can never be rejected later for its own fields.
    pub fn validate(&self) -> Result<(), WorldGenerationError> {
        if self.cell_size_x <= 0
            || self.cell_size_z <= 0
            || self.cell_size_x > 1_000_000
            || self.cell_size_z > 1_000_000
            || !self.max_jitter_x.is_finite()
            || !self.max_jitter_z.is_finite()
            || self.max_jitter_x < 0.0
            || self.max_jitter_z < 0.0
            || !self.max_yaw_degrees.is_finite()
            || !(0.0..=180.0).contains(&self.max_yaw_degrees)
            || !self.min_scale.is_finite()
            || !self.max_scale.is_finite()
            || self.min_scale <= 0.0
            || self.min_scale > self.max_scale
            || self.max_scale > 4.0
            || self.min_y_offset > self.max_y_offset
            || self.presence_denominator == 0
            || self.presence_numerator > self.presence_denominator
        {
            return Err(WorldGenerationError::InvalidCellularSource(
                "invalid cell dimensions, transform range, or presence ratio".to_string(),
            ));
        }
        Ok(())
    }
}

impl Default for CellularSdfConfig {
    fn default() -> Self {
        Self {
            cell_size_x: 128,
            cell_size_z: 128,
            seed: 0,
            max_jitter_x: 0.0,
            max_jitter_z: 0.0,
            max_yaw_degrees: 0.0,
            min_scale: 1.0,
            max_scale: 1.0,
            min_y_offset: 0,
            max_y_offset: 0,
            presence_numerator: 1,
            presence_denominator: 1,
            feature_salt: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct CellularTransform {
    anchor_x: i64,
    anchor_z: i64,
    jitter_x: f32,
    jitter_z: f32,
    y_offset: f32,
    sin_yaw: f32,
    cos_yaw: f32,
    scale: f32,
    radius: f32,
}

/// Places a bounded motif once per deterministically transformed world cell.
#[derive(Clone)]
pub struct CellularSdfChunkSource {
    motif: SdfNode,
    material: BrushEnum,
    min_y: i32,
    max_y: i32,
    config: CellularSdfConfig,
    motif_radius: f32,
    candidate_radius: i32,
    provenance: SourceProvenance,
}

fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn cellular_hash(cell_x: i64, cell_z: i64, seed: u64) -> u64 {
    mix64(
        seed ^ (cell_x as u64).wrapping_mul(0x632b_e59b_d9b4_e019)
            ^ (cell_z as u64).wrapping_mul(0x8cb9_2baa_3f3d_8dd7),
    )
}

fn hash_unit(value: u64) -> f32 {
    ((value >> 40) as u32) as f32 * (1.0 / (1u32 << 24) as f32)
}

fn hash_choice(value: u64, count: u64) -> u64 {
    (u128::from(value).wrapping_mul(u128::from(count)) >> 64) as u64
}

impl CellularSdfChunkSource {
    pub fn new(
        motif: SdfNode,
        material: BrushEnum,
        min_y: i32,
        max_y: i32,
        config: CellularSdfConfig,
        provenance: SourceProvenance,
    ) -> Result<Self, WorldGenerationError> {
        if min_y > max_y {
            return Err(WorldGenerationError::InvalidYBounds);
        }
        if min_y.div_euclid(16) < i8::MIN as i32 || max_y.div_euclid(16) > i8::MAX as i32 {
            return Err(WorldGenerationError::YBoundsOutOfRange);
        }
        motif.validate().map_err(WorldGenerationError::InvalidSdf)?;
        let bounds = motif.bounds().ok_or_else(|| {
            WorldGenerationError::InvalidCellularSource(
                "motif must have conservative finite bounds".to_string(),
            )
        })?;
        if bounds
            .min
            .iter()
            .chain(bounds.max.iter())
            .any(|value| !value.is_finite())
            || (0..3).any(|axis| bounds.min[axis] > bounds.max[axis])
        {
            return Err(WorldGenerationError::InvalidCellularSource(
                "motif bounds must be finite and non-empty".to_string(),
            ));
        }
        config.validate()?;
        if brush_needs_normal(&material) {
            return Err(WorldGenerationError::InvalidCellularSource(
                "normal-dependent brushes are not supported".to_string(),
            ));
        }
        let motif_radius = bounds.min[0]
            .abs()
            .max(bounds.max[0].abs())
            .max(bounds.min[2].abs())
            .max(bounds.max[2].abs());
        let radius = motif_radius * config.max_scale * std::f32::consts::SQRT_2
            + config.max_jitter_x.max(config.max_jitter_z);
        if !radius.is_finite() || radius > config.cell_size_x.max(config.cell_size_z) as f32 * 4.0 {
            return Err(WorldGenerationError::InvalidCellularSource(
                "transformed motif spans too many neighboring cells".to_string(),
            ));
        }
        let halo_x = (radius.ceil() as u64).div_ceil(config.cell_size_x as u64) + 2;
        let halo_z = (radius.ceil() as u64).div_ceil(config.cell_size_z as u64) + 2;
        let candidate_budget = (halo_x * 2 + 1)
            .checked_mul(halo_z * 2 + 1)
            .ok_or_else(|| {
                WorldGenerationError::InvalidCellularSource("candidate budget overflow".to_string())
            })?;
        if candidate_budget > MAX_CELLULAR_CANDIDATES_PER_CHUNK {
            return Err(WorldGenerationError::InvalidCellularSource(
                "configuration exceeds the per-chunk candidate budget".to_string(),
            ));
        }
        Ok(Self {
            motif,
            material,
            min_y,
            max_y,
            config,
            motif_radius,
            candidate_radius: radius.ceil() as i32,
            provenance,
        })
    }

    fn transform(&self, cell_x: i64, cell_z: i64) -> Option<CellularTransform> {
        let base = cellular_hash(cell_x, cell_z, self.config.seed);
        if hash_choice(
            mix64(base ^ self.config.feature_salt),
            u64::from(self.config.presence_denominator),
        ) >= u64::from(self.config.presence_numerator)
        {
            return None;
        }
        let signed = |salt| hash_unit(mix64(base ^ salt)) * 2.0 - 1.0;
        let yaw =
            signed(0xa409_3822_299f_31d0) * self.config.max_yaw_degrees * std::f32::consts::PI
                / 180.0;
        let scale = self.config.min_scale
            + hash_unit(mix64(base ^ 0x082e_fa98_ec4e_6c89))
                * (self.config.max_scale - self.config.min_scale);
        let y_span = i64::from(self.config.max_y_offset) - i64::from(self.config.min_y_offset) + 1;
        let y_offset = i64::from(self.config.min_y_offset)
            + hash_choice(mix64(base ^ 0x4528_21e6_38d0_1377), y_span as u64) as i64;
        Some(CellularTransform {
            anchor_x: cell_x.checked_mul(i64::from(self.config.cell_size_x))?,
            anchor_z: cell_z.checked_mul(i64::from(self.config.cell_size_z))?,
            jitter_x: signed(0x243f_6a88_85a3_08d3) * self.config.max_jitter_x,
            jitter_z: signed(0x1319_8a2e_0370_7344) * self.config.max_jitter_z,
            y_offset: y_offset as f32,
            sin_yaw: yaw.sin(),
            cos_yaw: yaw.cos(),
            scale,
            radius: self.motif_radius * scale * std::f32::consts::SQRT_2,
        })
    }
}

impl ChunkSource for CellularSdfChunkSource {
    fn generate(&self, request: ChunkRequest) -> Result<ChunkResult, WorldGenerationError> {
        let x0 = request
            .cx
            .checked_mul(16)
            .ok_or(WorldGenerationError::CoordinateOverflow)?;
        let z0 = request
            .cz
            .checked_mul(16)
            .ok_or(WorldGenerationError::CoordinateOverflow)?;
        let x1 = x0
            .checked_add(15)
            .ok_or(WorldGenerationError::CoordinateOverflow)?;
        let z1 = z0
            .checked_add(15)
            .ok_or(WorldGenerationError::CoordinateOverflow)?;
        for coordinate in [self.min_y, self.max_y] {
            let center = coordinate as f64 + 0.5;
            if f64::from(center as f32) != center {
                return Err(WorldGenerationError::CoordinatePrecision);
            }
        }
        let radius = i64::from(self.candidate_radius);
        let cell_size_x = i64::from(self.config.cell_size_x);
        let cell_size_z = i64::from(self.config.cell_size_z);
        let min_cell_x = (i64::from(x0) - radius).div_euclid(cell_size_x) - 1;
        let max_cell_x = (i64::from(x1) + radius).div_euclid(cell_size_x) + 1;
        let min_cell_z = (i64::from(z0) - radius).div_euclid(cell_size_z) - 1;
        let max_cell_z = (i64::from(z1) + radius).div_euclid(cell_size_z) + 1;
        let candidate_width = (max_cell_x - min_cell_x + 1) as u64;
        let candidate_depth = (max_cell_z - min_cell_z + 1) as u64;
        let candidate_count = candidate_width
            .checked_mul(candidate_depth)
            .ok_or(WorldGenerationError::CoordinateOverflow)?;
        if candidate_count > MAX_CELLULAR_CANDIDATES_PER_CHUNK {
            return Err(WorldGenerationError::InvalidCellularSource(
                "request exceeds the per-chunk candidate budget".to_string(),
            ));
        }
        let transforms: Vec<_> = (min_cell_x..=max_cell_x)
            .flat_map(|cell_x| {
                (min_cell_z..=max_cell_z).filter_map(move |cell_z| self.transform(cell_x, cell_z))
            })
            .filter(|transform| {
                let center_x = transform.anchor_x as f64 + f64::from(transform.jitter_x);
                let center_z = transform.anchor_z as f64 + f64::from(transform.jitter_z);
                center_x + f64::from(transform.radius) >= f64::from(x0)
                    && center_x - f64::from(transform.radius) <= f64::from(x1) + 1.0
                    && center_z + f64::from(transform.radius) >= f64::from(z0)
                    && center_z - f64::from(transform.radius) <= f64::from(z1) + 1.0
            })
            .collect();

        let mut sections = BTreeMap::<i8, SectionAccumulator>::new();
        for y in self.min_y..=self.max_y {
            for z in z0..=z1 {
                for x in x0..=x1 {
                    let world_y = y as f32 + 0.5;
                    if transforms.iter().any(|transform| {
                        let dx = ((i64::from(x) - transform.anchor_x) as f32 + 0.5
                            - transform.jitter_x)
                            / transform.scale;
                        let dz = ((i64::from(z) - transform.anchor_z) as f32 + 0.5
                            - transform.jitter_z)
                            / transform.scale;
                        let local_x = transform.cos_yaw * dx + transform.sin_yaw * dz;
                        let local_z = -transform.sin_yaw * dx + transform.cos_yaw * dz;
                        self.motif
                            .eval(local_x, world_y - transform.y_offset, local_z)
                            <= 0.0
                    }) {
                        if let Some(block) = self.material.get_block(x, y, z, (0.0, 1.0, 0.0)) {
                            set_generated_block(&mut sections, x0, z0, x, y, z, block);
                        }
                    }
                }
            }
        }
        let chunk = WorldChunkView::from_generated_sections(
            request.cx,
            request.cz,
            sections
                .into_values()
                .map(SectionAccumulator::finish)
                .collect(),
        );
        Ok(ChunkResult::new(
            chunk,
            ChunkCoverage::Partial,
            self.provenance.clone(),
        ))
    }
}

fn brush_needs_normal(brush: &BrushEnum) -> bool {
    matches!(brush, BrushEnum::Shaded(_) | BrushEnum::Spotlight(_))
}

#[derive(Clone)]
pub struct SdfChunkSource {
    volume: SdfNode,
    material: BrushEnum,
    min_y: i32,
    max_y: i32,
    provenance: SourceProvenance,
}

impl SdfChunkSource {
    pub fn new(
        volume: SdfNode,
        material: BrushEnum,
        min_y: i32,
        max_y: i32,
        provenance: SourceProvenance,
    ) -> Result<Self, WorldGenerationError> {
        if min_y > max_y {
            return Err(WorldGenerationError::InvalidYBounds);
        }
        if min_y.div_euclid(16) < i8::MIN as i32 || max_y.div_euclid(16) > i8::MAX as i32 {
            return Err(WorldGenerationError::YBoundsOutOfRange);
        }
        volume
            .validate()
            .map_err(WorldGenerationError::InvalidSdf)?;
        Ok(Self {
            volume,
            material,
            min_y,
            max_y,
            provenance,
        })
    }
}

impl ChunkSource for SdfChunkSource {
    fn generate(&self, request: ChunkRequest) -> Result<ChunkResult, WorldGenerationError> {
        let x0 = request
            .cx
            .checked_mul(16)
            .ok_or(WorldGenerationError::CoordinateOverflow)?;
        let z0 = request
            .cz
            .checked_mul(16)
            .ok_or(WorldGenerationError::CoordinateOverflow)?;
        let x1 = x0
            .checked_add(15)
            .ok_or(WorldGenerationError::CoordinateOverflow)?;
        let z1 = z0
            .checked_add(15)
            .ok_or(WorldGenerationError::CoordinateOverflow)?;
        for coordinate in [x0, x1, z0, z1, self.min_y, self.max_y] {
            let center = coordinate as f64 + 0.5;
            if f64::from(center as f32) != center {
                return Err(WorldGenerationError::CoordinatePrecision);
            }
        }

        let needs_normal = brush_needs_normal(&self.material);
        let mut sections = BTreeMap::<i8, SectionAccumulator>::new();
        for y in self.min_y..=self.max_y {
            for z in z0..=z1 {
                for x in x0..=x1 {
                    let point = (x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
                    if self.volume.eval(point.0, point.1, point.2) > 0.0 {
                        continue;
                    }
                    let normal = if needs_normal {
                        crate::sdf::numerical_normal(&self.volume, [point.0, point.1, point.2], 0.5)
                            .map(|normal| (normal[0], normal[1], normal[2]))
                            .unwrap_or((0.0, 1.0, 0.0))
                    } else {
                        (0.0, 1.0, 0.0)
                    };
                    if let Some(block) = self.material.get_block(x, y, z, normal) {
                        let section_y = y.div_euclid(16) as i8;
                        sections
                            .entry(section_y)
                            .or_insert_with(|| SectionAccumulator::new(section_y))
                            .set(
                                (x - x0) as usize,
                                y.rem_euclid(16) as usize,
                                (z - z0) as usize,
                                block,
                            );
                    }
                }
            }
        }
        let chunk = WorldChunkView::from_generated_sections(
            request.cx,
            request.cz,
            sections
                .into_values()
                .map(SectionAccumulator::finish)
                .collect(),
        );
        Ok(ChunkResult {
            chunk,
            coverage: ChunkCoverage::Complete,
            provenance: self.provenance.clone(),
        })
    }
}
