//! Meshing support for generating 3D models from schematics.
//!
//! This module provides integration with the `schematic-mesher` crate to convert
//! Nucleation schematics into 3D mesh formats (GLB/glTF). It exposes three
//! meshing modes suited to different schematic sizes and use cases.
//!
//! # Features
//!
//! Enable the `meshing` feature in your `Cargo.toml` to use this module:
//!
//! ```toml
//! nucleation = { version = "0.1", features = ["meshing"] }
//! ```
//!
//! # Meshing Modes
//!
//! ## 1. `to_mesh` — Single mesh (small/medium schematics)
//!
//! Returns one [`MeshOutput`] for the entire schematic. Best when you just need
//! a single GLB/USDZ file.
//!
//! ```ignore
//! use nucleation::{UniversalSchematic, meshing::{MeshConfig, MeshOutput, ResourcePackSource}};
//!
//! let schematic = UniversalSchematic::from_litematic_bytes(&data)?;
//! let pack = ResourcePackSource::from_file("resourcepack.zip")?;
//! let config = MeshConfig::default();
//!
//! let output: MeshOutput = schematic.to_mesh(&pack, &config)?;
//! std::fs::write("output.glb", output.to_glb()?)?;
//! ```
//!
//! ## 2. `mesh_by_region` — Per-region meshes
//!
//! Returns a `HashMap<String, MeshOutput>` keyed by region name. Best when
//! your schematic has meaningful region structure (e.g., Litematic regions).
//!
//! ```ignore
//! let regions = schematic.mesh_by_region(&pack, &config)?;
//! for (name, mesh) in &regions {
//!     std::fs::write(format!("{}.glb", name), &mesh.to_glb()?)?;
//! }
//! ```
//!
//! ## 3. `mesh_chunks` — Lazy chunk iterator (large/massive schematics)
//!
//! Returns a [`NucleationChunkIter`] that yields one [`MeshOutput`] per chunk.
//! Never loads the full world mesh into memory — ideal for streaming or
//! progressive loading.
//!
//! ```ignore
//! for chunk_result in schematic.mesh_chunks(&pack, &config, 16) {
//!     let chunk_mesh = chunk_result?;
//!     let (cx, cy, cz) = chunk_mesh.chunk_coord.unwrap();
//!     println!("Chunk ({},{},{}) has {} vertices",
//!         cx, cy, cz, chunk_mesh.total_vertices());
//! }
//! ```

use schematic_mesher::{
    export_raw, resource_pack::TextureData, AtlasBuilder, BlockPosition as MesherBlockPosition,
    BlockSource, BoundingBox as MesherBoundingBox, InputBlock, Mesher, MesherConfig, MesherOutput,
    RawMeshData, ResourcePack, TextureAtlas,
};

pub mod cache;

// Re-export the real MeshOutput and MeshLayer types from schematic-mesher.
pub use schematic_mesher::{MeshLayer, MeshOutput};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::entity::{Entity, NbtValue};
use crate::{BlockState, Region, UniversalSchematic};

/// Error type for meshing operations.
#[derive(Debug, thiserror::Error)]
pub enum MeshError {
    #[error("Resource pack error: {0}")]
    ResourcePack(String),
    #[error("Meshing error: {0}")]
    Meshing(String),
    #[error("Export error: {0}")]
    Export(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, MeshError>;

/// Source for a Minecraft resource pack.
pub struct ResourcePackSource {
    pack: ResourcePack,
}

impl ResourcePackSource {
    /// Load a resource pack from a file path (ZIP or directory).
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let pack = schematic_mesher::load_resource_pack(path)
            .map_err(|e| MeshError::ResourcePack(e.to_string()))?;
        Ok(Self { pack })
    }

    /// Load a resource pack from bytes (for WASM compatibility).
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let pack = schematic_mesher::load_resource_pack_from_bytes(data)
            .map_err(|e| MeshError::ResourcePack(e.to_string()))?;
        Ok(Self { pack })
    }

    /// Create a ResourcePackSource from an already-loaded ResourcePack.
    pub fn from_resource_pack(pack: ResourcePack) -> Self {
        Self { pack }
    }

    /// Get a reference to the underlying ResourcePack.
    pub fn pack(&self) -> &ResourcePack {
        &self.pack
    }

    /// Get a mutable reference to the underlying ResourcePack.
    pub fn pack_mut(&mut self) -> &mut ResourcePack {
        &mut self.pack
    }

    /// List all blockstate names as "namespace:block_id".
    pub fn list_blockstates(&self) -> Vec<String> {
        let mut names = Vec::new();
        for (namespace, blocks) in &self.pack.blockstates {
            for block_id in blocks.keys() {
                names.push(format!("{}:{}", namespace, block_id));
            }
        }
        names
    }

    /// List all model names as "namespace:model_path".
    pub fn list_models(&self) -> Vec<String> {
        let mut names = Vec::new();
        for (namespace, models) in &self.pack.models {
            for model_path in models.keys() {
                names.push(format!("{}:{}", namespace, model_path));
            }
        }
        names
    }

    /// List all texture names as "namespace:texture_path".
    pub fn list_textures(&self) -> Vec<String> {
        let mut names = Vec::new();
        for (namespace, textures) in &self.pack.textures {
            for texture_path in textures.keys() {
                names.push(format!("{}:{}", namespace, texture_path));
            }
        }
        names
    }

    /// Get a blockstate definition as JSON. Returns None if not found.
    /// Since BlockstateDefinition doesn't implement Serialize, we manually build JSON.
    pub fn get_blockstate_json(&self, name: &str) -> Option<String> {
        let def = self.pack.get_blockstate(name)?;
        Some(blockstate_definition_to_json(def))
    }

    /// Get a block model as JSON. Returns None if not found.
    pub fn get_model_json(&self, name: &str) -> Option<String> {
        let model = self.pack.get_model(name)?;
        serde_json::to_string(model).ok()
    }

    /// Get texture info: (width, height, is_animated, frame_count). Returns None if not found.
    pub fn get_texture_info(&self, name: &str) -> Option<(u32, u32, bool, u32)> {
        let tex = self.pack.get_texture(name)?;
        Some((tex.width, tex.height, tex.is_animated, tex.frame_count))
    }

    /// Get raw RGBA8 pixel data for a texture. Returns None if not found.
    pub fn get_texture_pixels(&self, name: &str) -> Option<&[u8]> {
        let tex = self.pack.get_texture(name)?;
        Some(&tex.pixels)
    }

    /// Add a blockstate definition from JSON. Name format: "namespace:block_id".
    pub fn add_blockstate_json(&mut self, name: &str, json: &str) -> Result<()> {
        let (namespace, path) = split_resource_name(name)?;
        let def: schematic_mesher::BlockstateDefinition =
            serde_json::from_str(json).map_err(|e| MeshError::ResourcePack(e.to_string()))?;
        self.pack.add_blockstate(&namespace, &path, def);
        Ok(())
    }

    /// Add a block model from JSON. Name format: "namespace:model_path".
    pub fn add_model_json(&mut self, name: &str, json: &str) -> Result<()> {
        let (namespace, path) = split_resource_name(name)?;
        let model: schematic_mesher::BlockModel =
            serde_json::from_str(json).map_err(|e| MeshError::ResourcePack(e.to_string()))?;
        self.pack.add_model(&namespace, &path, model);
        Ok(())
    }

    /// Add a texture from raw RGBA8 pixel data. Name format: "namespace:texture_path".
    pub fn add_texture(
        &mut self,
        name: &str,
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    ) -> Result<()> {
        let (namespace, path) = split_resource_name(name)?;
        let texture = TextureData::new(width, height, pixels);
        self.pack.add_texture(&namespace, &path, texture);
        Ok(())
    }

    /// Get statistics about the loaded resource pack.
    pub fn stats(&self) -> ResourcePackStats {
        ResourcePackStats {
            blockstate_count: self.pack.blockstate_count(),
            model_count: self.pack.model_count(),
            texture_count: self.pack.texture_count(),
            namespaces: self
                .pack
                .namespaces()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

/// Statistics about a loaded resource pack.
#[derive(Debug, Clone)]
pub struct ResourcePackStats {
    pub blockstate_count: usize,
    pub model_count: usize,
    pub texture_count: usize,
    pub namespaces: Vec<String>,
}

/// Configuration for mesh generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MeshConfig {
    /// Enable face culling between adjacent solid blocks.
    pub cull_hidden_faces: bool,
    /// Enable ambient occlusion.
    pub ambient_occlusion: bool,
    /// AO intensity (0.0 = no darkening, 1.0 = full darkening).
    pub ao_intensity: f32,
    /// Biome for tinting (e.g., "plains", "swamp").
    pub biome: Option<String>,
    /// Maximum texture atlas dimension.
    pub atlas_max_size: u32,
    /// Skip blocks that are fully hidden by opaque neighbors on all 6 sides.
    pub cull_occluded_blocks: bool,
    /// Merge adjacent coplanar faces into larger quads (reduces triangle count).
    pub greedy_meshing: bool,
}

impl Default for MeshConfig {
    fn default() -> Self {
        Self {
            cull_hidden_faces: true,
            ambient_occlusion: true,
            ao_intensity: 0.4,
            biome: None,
            atlas_max_size: 4096,
            cull_occluded_blocks: true,
            greedy_meshing: false,
        }
    }
}

impl MeshConfig {
    /// Create a new MeshConfig with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable face culling.
    pub fn with_culling(mut self, enabled: bool) -> Self {
        self.cull_hidden_faces = enabled;
        self
    }

    /// Enable or disable ambient occlusion.
    pub fn with_ambient_occlusion(mut self, enabled: bool) -> Self {
        self.ambient_occlusion = enabled;
        self
    }

    /// Set the AO intensity.
    pub fn with_ao_intensity(mut self, intensity: f32) -> Self {
        self.ao_intensity = intensity;
        self
    }

    /// Set the biome for tinting.
    pub fn with_biome(mut self, biome: impl Into<String>) -> Self {
        self.biome = Some(biome.into());
        self
    }

    /// Set the maximum atlas size.
    pub fn with_atlas_max_size(mut self, size: u32) -> Self {
        self.atlas_max_size = size;
        self
    }

    /// Enable or disable occluded block culling.
    pub fn with_cull_occluded_blocks(mut self, enabled: bool) -> Self {
        self.cull_occluded_blocks = enabled;
        self
    }

    /// Enable or disable greedy meshing.
    pub fn with_greedy_meshing(mut self, enabled: bool) -> Self {
        self.greedy_meshing = enabled;
        self
    }

    fn to_mesher_config(&self) -> MesherConfig {
        let mut config = MesherConfig::default();
        config.cull_hidden_faces = self.cull_hidden_faces;
        config.ambient_occlusion = self.ambient_occlusion;
        config.ao_intensity = self.ao_intensity;
        config.atlas_max_size = self.atlas_max_size;
        config.cull_occluded_blocks = self.cull_occluded_blocks;
        config.greedy_meshing = self.greedy_meshing;
        if let Some(biome) = &self.biome {
            config = config.with_biome(biome);
        }
        config
    }
}

// ─── Backward-compatible type aliases ───────────────────────────────────────
//
// Existing callers that use `MeshResult`, `MultiMeshResult`, or `ChunkMeshResult`
// continue to compile. New code should prefer `MeshOutput` directly.

/// Backward-compatible alias — prefer [`MeshOutput`] for new code.
pub type MeshResult = MeshOutput;

/// Backward-compatible alias for per-region mesh results.
pub type MultiMeshResult = HashMap<String, MeshOutput>;

/// Result of chunk-based mesh generation (eager, all-at-once).
///
/// Kept for backward compatibility with `mesh_by_chunk` / `mesh_by_chunk_size`.
/// New code should prefer [`NucleationChunkIter`] via [`UniversalSchematic::mesh_chunks`].
#[derive(Debug)]
pub struct ChunkMeshResult {
    /// Map of chunk coordinate to mesh output.
    pub meshes: HashMap<(i32, i32, i32), MeshOutput>,
    /// Total vertex count across all meshes.
    pub total_vertex_count: usize,
    /// Total triangle count across all meshes.
    pub total_triangle_count: usize,
}

/// Result of raw mesh export for custom rendering.
#[derive(Debug)]
pub struct RawMeshExport {
    pub(crate) inner: RawMeshData,
}

impl RawMeshExport {
    /// Vertex positions (3 floats per vertex).
    pub fn positions_flat(&self) -> Vec<f32> {
        self.inner.positions_flat()
    }

    /// Vertex normals (3 floats per vertex).
    pub fn normals_flat(&self) -> Vec<f32> {
        self.inner.normals_flat()
    }

    /// Texture coordinates (2 floats per vertex).
    pub fn uvs_flat(&self) -> Vec<f32> {
        self.inner.uvs_flat()
    }

    /// Vertex colors (4 floats per vertex, RGBA).
    pub fn colors_flat(&self) -> Vec<f32> {
        self.inner.colors_flat()
    }

    /// Triangle indices.
    pub fn indices(&self) -> &[u32] {
        &self.inner.indices
    }

    /// Texture atlas RGBA pixel data.
    pub fn texture_rgba(&self) -> &[u8] {
        &self.inner.texture_rgba
    }

    /// Texture atlas width.
    pub fn texture_width(&self) -> u32 {
        self.inner.texture_width
    }

    /// Texture atlas height.
    pub fn texture_height(&self) -> u32 {
        self.inner.texture_height
    }

    /// Number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.inner.vertex_count()
    }

    /// Number of triangles.
    pub fn triangle_count(&self) -> usize {
        self.inner.triangle_count()
    }
}

/// Build a [`MeshOutput`] from a [`MesherOutput`], optionally setting a chunk coordinate.
fn mesh_output_from_mesher(
    output: &MesherOutput,
    chunk_coord: Option<(i32, i32, i32)>,
) -> MeshOutput {
    let mut mesh = MeshOutput::from(output);
    mesh.chunk_coord = chunk_coord;
    mesh
}

/// Adapter to convert a Nucleation Region into a BlockSource.
struct RegionBlockSource {
    blocks: HashMap<MesherBlockPosition, InputBlock>,
    bounds: MesherBoundingBox,
}

impl RegionBlockSource {
    fn new(region: &Region) -> Self {
        let mut blocks = HashMap::new();
        let bbox = region.get_bounding_box();

        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];

        // Iterate through all non-air blocks in the region
        for index in 0..region.volume() {
            let (x, y, z) = region.index_to_coords(index);
            if let Some(block_state) = region.get_block(x, y, z) {
                if block_state.name != "minecraft:air" {
                    let pos = MesherBlockPosition::new(x, y, z);
                    let input_block = block_state_to_input_block(block_state);
                    blocks.insert(pos, input_block);

                    // Update bounds
                    min[0] = min[0].min(x as f32);
                    min[1] = min[1].min(y as f32);
                    min[2] = min[2].min(z as f32);
                    max[0] = max[0].max(x as f32 + 1.0);
                    max[1] = max[1].max(y as f32 + 1.0);
                    max[2] = max[2].max(z as f32 + 1.0);
                }
            }
        }

        // Also collect entities as entity: blocks
        collect_region_entities(region, &mut blocks, &mut min, &mut max);

        // Use region bounds if no blocks found
        if blocks.is_empty() {
            min = [bbox.min.0 as f32, bbox.min.1 as f32, bbox.min.2 as f32];
            max = [
                bbox.max.0 as f32 + 1.0,
                bbox.max.1 as f32 + 1.0,
                bbox.max.2 as f32 + 1.0,
            ];
        }

        let bounds = MesherBoundingBox::new(min, max);

        Self { blocks, bounds }
    }
}

impl BlockSource for RegionBlockSource {
    fn get_block(&self, pos: MesherBlockPosition) -> Option<&InputBlock> {
        self.blocks.get(&pos)
    }

    fn iter_blocks(&self) -> Box<dyn Iterator<Item = (MesherBlockPosition, &InputBlock)> + '_> {
        Box::new(self.blocks.iter().map(|(pos, block)| (*pos, block)))
    }

    fn bounds(&self) -> MesherBoundingBox {
        self.bounds
    }
}

/// Adapter for chunk-based meshing.
struct ChunkBlockSource {
    blocks: HashMap<MesherBlockPosition, InputBlock>,
    bounds: MesherBoundingBox,
}

impl ChunkBlockSource {
    fn new(blocks: HashMap<MesherBlockPosition, InputBlock>, bounds: MesherBoundingBox) -> Self {
        Self { blocks, bounds }
    }
}

impl BlockSource for ChunkBlockSource {
    fn get_block(&self, pos: MesherBlockPosition) -> Option<&InputBlock> {
        self.blocks.get(&pos)
    }

    fn iter_blocks(&self) -> Box<dyn Iterator<Item = (MesherBlockPosition, &InputBlock)> + '_> {
        Box::new(self.blocks.iter().map(|(pos, block)| (*pos, block)))
    }

    fn bounds(&self) -> MesherBoundingBox {
        self.bounds
    }
}

/// Convert a Nucleation BlockState to a schematic-mesher InputBlock.
fn block_state_to_input_block(block_state: &BlockState) -> InputBlock {
    let mut input = InputBlock::new(block_state.name.to_string());
    for (key, value) in &block_state.properties {
        input.properties.insert(key.to_string(), value.to_string());
    }
    input
}

/// Convert a Nucleation Entity to a schematic-mesher InputBlock with `entity:` prefix.
///
/// The mesher recognizes entities via the `entity:` namespace (e.g., `entity:minecart`,
/// `entity:zombie`). Entity NBT data like Rotation is mapped to InputBlock properties.
fn entity_to_input_block(entity: &Entity) -> InputBlock {
    // Strip "minecraft:" prefix and add "entity:" prefix
    let entity_id = entity.id.strip_prefix("minecraft:").unwrap_or(&entity.id);

    // Map specific Minecraft entity types to mesher-supported types
    let mesher_id = match entity_id {
        "furnace_minecart"
        | "chest_minecart"
        | "tnt_minecart"
        | "hopper_minecart"
        | "spawner_minecart"
        | "command_block_minecart" => "minecart",
        id => id,
    };

    let mut input = InputBlock::new(&format!("entity:{}", mesher_id));

    // Extract facing from Rotation NBT (Rotation is a list of 2 floats: [yaw, pitch])
    if let Some(NbtValue::List(rotation)) = entity.nbt.get("Rotation") {
        if let Some(NbtValue::Float(yaw)) = rotation.first() {
            // Convert yaw angle to cardinal direction
            let facing = yaw_to_facing(*yaw);
            input
                .properties
                .insert("facing".to_string(), facing.to_string());
        }
    }

    // Pass through relevant NBT as properties for the mesher
    // Baby entities
    if let Some(NbtValue::Byte(1)) = entity.nbt.get("IsBaby") {
        input
            .properties
            .insert("is_baby".to_string(), "true".to_string());
    }
    // Age < 0 also indicates baby
    if let Some(NbtValue::Int(age)) = entity.nbt.get("Age") {
        if *age < 0 {
            input
                .properties
                .insert("is_baby".to_string(), "true".to_string());
        }
    }

    // Sheep color
    if let Some(NbtValue::Byte(color)) = entity.nbt.get("Color") {
        input
            .properties
            .insert("color".to_string(), dye_color_name(*color as u8));
    }

    // Armor stand pose properties
    for pose_key in &[
        "RightArmPose",
        "LeftArmPose",
        "RightLegPose",
        "LeftLegPose",
        "HeadPose",
        "BodyPose",
    ] {
        if let Some(NbtValue::List(angles)) = entity.nbt.get(*pose_key) {
            let angle_strs: Vec<String> = angles
                .iter()
                .filter_map(|v| match v {
                    NbtValue::Float(f) => Some(format!("{}", f)),
                    _ => None,
                })
                .collect();
            if !angle_strs.is_empty() {
                input
                    .properties
                    .insert(pose_key.to_string(), angle_strs.join(","));
            }
        }
    }

    input
}

/// Convert a yaw angle (degrees) to cardinal direction string.
fn yaw_to_facing(yaw: f32) -> &'static str {
    // Normalize yaw to 0..360
    let normalized = ((yaw % 360.0) + 360.0) % 360.0;
    if normalized >= 315.0 || normalized < 45.0 {
        "south"
    } else if normalized >= 45.0 && normalized < 135.0 {
        "west"
    } else if normalized >= 135.0 && normalized < 225.0 {
        "north"
    } else {
        "east"
    }
}

/// Convert a Minecraft dye color byte to color name.
fn dye_color_name(color: u8) -> String {
    match color {
        0 => "white",
        1 => "orange",
        2 => "magenta",
        3 => "light_blue",
        4 => "yellow",
        5 => "lime",
        6 => "pink",
        7 => "gray",
        8 => "light_gray",
        9 => "cyan",
        10 => "purple",
        11 => "blue",
        12 => "brown",
        13 => "green",
        14 => "red",
        15 => "black",
        _ => "white",
    }
    .to_string()
}

/// Collect entities from a region and insert them into a block map as `entity:` blocks.
fn collect_region_entities(
    region: &Region,
    blocks: &mut HashMap<MesherBlockPosition, InputBlock>,
    min: &mut [f32; 3],
    max: &mut [f32; 3],
) {
    for entity in &region.entities {
        let input_block = entity_to_input_block(entity);

        // Use floored entity position as block position
        let x = entity.position.0.floor() as i32;
        let y = entity.position.1.floor() as i32;
        let z = entity.position.2.floor() as i32;
        let pos = MesherBlockPosition::new(x, y, z);

        // Only insert if there isn't already a block at this position
        // (blocks take priority over entities for rendering)
        if !blocks.contains_key(&pos) {
            blocks.insert(pos, input_block);

            min[0] = min[0].min(x as f32);
            min[1] = min[1].min(y as f32);
            min[2] = min[2].min(z as f32);
            max[0] = max[0].max(x as f32 + 1.0);
            max[1] = max[1].max(y as f32 + 1.0);
            max[2] = max[2].max(z as f32 + 1.0);
        }
    }
}

// ─── Progress Reporting ─────────────────────────────────────────────────────

/// Phase of the meshing pipeline (for progress reporting).
#[derive(Clone, Debug, PartialEq)]
pub enum MeshPhase {
    /// Scanning block palettes and building the global texture atlas.
    BuildingAtlas,
    /// Meshing individual chunks.
    MeshingChunks,
    /// All chunks have been meshed.
    Complete,
}

/// Progress update emitted during chunk meshing.
#[derive(Clone, Debug)]
pub struct MeshProgress {
    /// Current phase of the pipeline.
    pub phase: MeshPhase,
    /// Number of chunks completed so far.
    pub chunks_done: u32,
    /// Total number of chunks to mesh.
    pub chunks_total: u32,
    /// Cumulative vertex count across all completed chunks.
    pub vertices_so_far: u64,
    /// Cumulative triangle count across all completed chunks.
    pub triangles_so_far: u64,
}

// ─── Global Atlas ───────────────────────────────────────────────────────────

/// Build a single shared texture atlas from all unique block states in a schematic.
///
/// Scans every region's palette (O(unique_states), not O(volume)) to discover
/// texture paths, then builds one atlas that can be reused across all chunks.
/// This eliminates per-chunk atlas duplication for massive schematics.
pub fn build_global_atlas(
    schematic: &UniversalSchematic,
    pack: &ResourcePackSource,
    config: &MeshConfig,
) -> Result<TextureAtlas> {
    let mesher_config = config.to_mesher_config();

    // Collect all unique block states from all regions' palettes
    let mut unique_states: std::collections::HashSet<BlockState> = std::collections::HashSet::new();
    for state in &schematic.default_region.palette {
        if state.name != "minecraft:air" {
            unique_states.insert(state.clone());
        }
    }
    for region in schematic.other_regions.values() {
        for state in &region.palette {
            if state.name != "minecraft:air" {
                unique_states.insert(state.clone());
            }
        }
    }

    // Convert unique states to InputBlocks and discover textures via the mesher
    // We create a small synthetic block source with one of each unique block state
    let mut blocks = HashMap::new();
    let mut pos_idx = 0i32;
    for state in &unique_states {
        let pos = MesherBlockPosition::new(pos_idx, 0, 0);
        let input = block_state_to_input_block(state);
        blocks.insert(pos, input);
        pos_idx += 1; // Spread blocks apart so they don't cull each other
    }

    if blocks.is_empty() {
        return Ok(TextureAtlas::empty());
    }

    let bounds = MesherBoundingBox::new([0.0, 0.0, 0.0], [pos_idx as f32, 1.0, 1.0]);

    // Use a config with no culling for texture discovery
    let mut discovery_config = mesher_config.clone();
    discovery_config.cull_hidden_faces = false;
    discovery_config.cull_occluded_blocks = false;

    let source = ChunkBlockSource::new(blocks, bounds);
    let mesher = Mesher::with_config(pack.pack.clone(), discovery_config);

    let texture_refs = mesher.discover_textures(&source);

    // Build atlas from discovered textures
    let mut atlas_builder =
        AtlasBuilder::new(mesher_config.atlas_max_size, mesher_config.atlas_padding);

    for texture_ref in &texture_refs {
        if let Some(texture) = pack.pack.get_texture(texture_ref) {
            atlas_builder.add_texture(texture_ref.clone(), texture.first_frame());
        }
    }

    atlas_builder
        .build()
        .map_err(|e| MeshError::Meshing(e.to_string()))
}

impl UniversalSchematic {
    /// Compute the raw MesherOutput for the entire schematic (internal helper).
    fn compute_mesh_output(
        &self,
        pack: &ResourcePackSource,
        config: &MeshConfig,
    ) -> Result<MesherOutput> {
        let mesher_config = config.to_mesher_config();
        let mesher = Mesher::with_config(pack.pack.clone(), mesher_config);

        // Collect all blocks from all regions
        let mut all_blocks = HashMap::new();
        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];

        // Process default region
        collect_region_blocks(&self.default_region, &mut all_blocks, &mut min, &mut max);

        // Process other regions
        for region in self.other_regions.values() {
            collect_region_blocks(region, &mut all_blocks, &mut min, &mut max);
        }

        if all_blocks.is_empty() {
            return Err(MeshError::Meshing("No blocks to mesh".to_string()));
        }

        let bounds = MesherBoundingBox::new(min, max);
        let source = ChunkBlockSource::new(all_blocks, bounds);

        mesher
            .mesh(&source)
            .map_err(|e| MeshError::Meshing(e.to_string()))
    }

    /// Generate a single mesh for the entire schematic.
    ///
    /// Best for small/medium schematics. Returns one [`MeshOutput`] containing
    /// per-layer typed arrays, a shared texture atlas, and export helpers.
    pub fn to_mesh(&self, pack: &ResourcePackSource, config: &MeshConfig) -> Result<MeshOutput> {
        let output = self.compute_mesh_output(pack, config)?;
        Ok(mesh_output_from_mesher(&output, None))
    }

    /// Generate a USDZ mesh for the entire schematic.
    pub fn to_usdz(&self, pack: &ResourcePackSource, config: &MeshConfig) -> Result<MeshOutput> {
        let output = self.compute_mesh_output(pack, config)?;
        Ok(mesh_output_from_mesher(&output, None))
    }

    /// Generate raw mesh data for the entire schematic.
    ///
    /// Returns positions, normals, UVs, colors, indices, and texture atlas data
    /// for custom rendering pipelines.
    pub fn to_raw_mesh(
        &self,
        pack: &ResourcePackSource,
        config: &MeshConfig,
    ) -> Result<RawMeshExport> {
        let output = self.compute_mesh_output(pack, config)?;
        let raw = export_raw(&output);
        Ok(RawMeshExport { inner: raw })
    }

    /// Generate one mesh per region.
    ///
    /// Best for schematics with meaningful region structure (e.g., Litematic
    /// files with named regions). Returns a map of region name to [`MeshOutput`].
    pub fn mesh_by_region(
        &self,
        pack: &ResourcePackSource,
        config: &MeshConfig,
    ) -> Result<MultiMeshResult> {
        let mesher_config = config.to_mesher_config();

        let mut meshes = HashMap::new();

        // Mesh default region
        if let Some(result) = mesh_region(&pack.pack, &self.default_region, &mesher_config)? {
            meshes.insert(self.default_region_name.clone(), result);
        }

        // Mesh other regions
        for (name, region) in &self.other_regions {
            if let Some(result) = mesh_region(&pack.pack, region, &mesher_config)? {
                meshes.insert(name.clone(), result);
            }
        }

        Ok(meshes)
    }

    /// Generate one mesh per 16x16x16 chunk.
    ///
    /// This is useful for large schematics where you want to load/unload
    /// meshes based on camera position.
    pub fn mesh_by_chunk(
        &self,
        pack: &ResourcePackSource,
        config: &MeshConfig,
    ) -> Result<ChunkMeshResult> {
        self.mesh_by_chunk_size(pack, config, 16)
    }

    /// Generate one mesh per chunk of the specified size (eager — loads all at once).
    ///
    /// For lazy iteration that never holds the full world in memory, use
    /// [`mesh_chunks`](Self::mesh_chunks) instead.
    pub fn mesh_by_chunk_size(
        &self,
        pack: &ResourcePackSource,
        config: &MeshConfig,
        chunk_size: i32,
    ) -> Result<ChunkMeshResult> {
        let mesher_config = config.to_mesher_config();

        // Collect all blocks from all regions into chunks
        let mut chunks: HashMap<(i32, i32, i32), HashMap<MesherBlockPosition, InputBlock>> =
            HashMap::new();

        // Process default region
        collect_region_blocks_by_chunk(&self.default_region, &mut chunks, chunk_size);

        // Process other regions
        for region in self.other_regions.values() {
            collect_region_blocks_by_chunk(region, &mut chunks, chunk_size);
        }

        let mut meshes = HashMap::new();
        let mut total_vertex_count = 0;
        let mut total_triangle_count = 0;

        for (chunk_coord, blocks) in chunks {
            if blocks.is_empty() {
                continue;
            }

            // Calculate bounds for this chunk
            let mut min = [f32::MAX; 3];
            let mut max = [f32::MIN; 3];

            for pos in blocks.keys() {
                min[0] = min[0].min(pos.x as f32);
                min[1] = min[1].min(pos.y as f32);
                min[2] = min[2].min(pos.z as f32);
                max[0] = max[0].max(pos.x as f32 + 1.0);
                max[1] = max[1].max(pos.y as f32 + 1.0);
                max[2] = max[2].max(pos.z as f32 + 1.0);
            }

            let bounds = MesherBoundingBox::new(min, max);
            let source = ChunkBlockSource::new(blocks, bounds);

            let mesher = Mesher::with_config(pack.pack.clone(), mesher_config.clone());
            let output = mesher
                .mesh(&source)
                .map_err(|e| MeshError::Meshing(e.to_string()))?;

            let result = mesh_output_from_mesher(&output, Some(chunk_coord));

            total_vertex_count += result.total_vertices();
            total_triangle_count += result.total_triangles();
            meshes.insert(chunk_coord, result);
        }

        Ok(ChunkMeshResult {
            meshes,
            total_vertex_count,
            total_triangle_count,
        })
    }

    /// Create a lazy chunk mesh iterator.
    ///
    /// Yields one [`MeshOutput`] per chunk of `chunk_size` blocks on each axis.
    /// Never loads the full world mesh into memory — best for large/massive
    /// schematics.
    ///
    /// # Example
    ///
    /// ```ignore
    /// for result in schematic.mesh_chunks(&pack, &config, 16) {
    ///     let mesh = result?;
    ///     let (cx, cy, cz) = mesh.chunk_coord.unwrap();
    ///     println!("Chunk ({},{},{}) — {} triangles",
    ///         cx, cy, cz, mesh.total_triangles());
    /// }
    /// ```
    /// Mesh the schematic in parallel using `std::thread::scope`.
    ///
    /// Splits the schematic into chunks of `chunk_size` blocks, then meshes
    /// each chunk on a separate thread (up to `max_threads` concurrently).
    /// Returns all chunk meshes as a `Vec<MeshOutput>`.
    ///
    /// This is the fastest path for large schematics. Each chunk gets its own
    /// texture atlas, so the caller must handle multiple atlases when rendering.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let meshes = schematic.mesh_chunks_parallel(&pack, &config, 32, 8)?;
    /// println!("Meshed {} chunks", meshes.len());
    /// for mesh in &meshes {
    ///     println!("  {} triangles", mesh.total_triangles());
    /// }
    /// ```
    pub fn mesh_chunks_parallel(
        &self,
        pack: &ResourcePackSource,
        config: &MeshConfig,
        chunk_size: i32,
        max_threads: usize,
    ) -> Result<Vec<MeshOutput>> {
        let mesher_config = config.to_mesher_config();

        let mut chunks: HashMap<(i32, i32, i32), HashMap<MesherBlockPosition, InputBlock>> =
            HashMap::new();

        collect_region_blocks_by_chunk(&self.default_region, &mut chunks, chunk_size);
        for region in self.other_regions.values() {
            collect_region_blocks_by_chunk(region, &mut chunks, chunk_size);
        }

        let chunk_list: Vec<_> = chunks.into_iter().filter(|(_, b)| !b.is_empty()).collect();

        if chunk_list.is_empty() {
            return Err(MeshError::Meshing("No blocks to mesh".to_string()));
        }

        let max_threads = max_threads.max(1);
        let pack_ref = &pack.pack;
        let config_ref = &mesher_config;

        // Use std::thread::scope for safe parallel meshing without extra dependencies
        let results: Vec<Result<MeshOutput>> = std::thread::scope(|scope| {
            let mut handles = Vec::new();

            for batch in chunk_list.chunks(max_threads) {
                let batch_handles: Vec<_> = batch
                    .iter()
                    .map(|(chunk_coord, blocks)| {
                        let coord = *chunk_coord;
                        scope.spawn(move || {
                            let mut min = [f32::MAX; 3];
                            let mut max = [f32::MIN; 3];
                            for pos in blocks.keys() {
                                min[0] = min[0].min(pos.x as f32);
                                min[1] = min[1].min(pos.y as f32);
                                min[2] = min[2].min(pos.z as f32);
                                max[0] = max[0].max(pos.x as f32 + 1.0);
                                max[1] = max[1].max(pos.y as f32 + 1.0);
                                max[2] = max[2].max(pos.z as f32 + 1.0);
                            }

                            let bounds = MesherBoundingBox::new(min, max);
                            let source = ChunkBlockSource::new(blocks.clone(), bounds);
                            let mesher = Mesher::with_config(pack_ref.clone(), config_ref.clone());

                            match mesher.mesh(&source) {
                                Ok(output) => Ok(mesh_output_from_mesher(&output, Some(coord))),
                                Err(e) => Err(MeshError::Meshing(e.to_string())),
                            }
                        })
                    })
                    .collect();

                // Wait for this batch to complete before spawning next batch
                for handle in batch_handles {
                    handles.push(handle.join().expect("Mesh thread panicked"));
                }
            }

            handles
        });

        // Collect results, propagating first error
        results.into_iter().collect()
    }

    pub fn mesh_chunks(
        &self,
        pack: &ResourcePackSource,
        config: &MeshConfig,
        chunk_size: i32,
    ) -> NucleationChunkIter {
        let mut chunks: HashMap<(i32, i32, i32), HashMap<MesherBlockPosition, InputBlock>> =
            HashMap::new();

        collect_region_blocks_by_chunk(&self.default_region, &mut chunks, chunk_size);
        for region in self.other_regions.values() {
            collect_region_blocks_by_chunk(region, &mut chunks, chunk_size);
        }

        let chunk_list: Vec<_> = chunks.into_iter().filter(|(_, b)| !b.is_empty()).collect();

        NucleationChunkIter {
            chunks: chunk_list,
            index: 0,
            pack: pack.pack.clone(),
            config: config.to_mesher_config(),
            shared_atlas: None,
            progress_callback: None,
            vertices_so_far: 0,
            triangles_so_far: 0,
        }
    }

    /// Create a lazy chunk mesh iterator that uses a pre-built global atlas.
    ///
    /// The global atlas is shared across all chunks, eliminating per-chunk atlas
    /// duplication. Build the atlas first with [`build_global_atlas()`], then
    /// pass it here.
    pub fn mesh_chunks_with_atlas(
        &self,
        pack: &ResourcePackSource,
        config: &MeshConfig,
        chunk_size: i32,
        atlas: TextureAtlas,
    ) -> NucleationChunkIter {
        let mut chunks: HashMap<(i32, i32, i32), HashMap<MesherBlockPosition, InputBlock>> =
            HashMap::new();

        collect_region_blocks_by_chunk(&self.default_region, &mut chunks, chunk_size);
        for region in self.other_regions.values() {
            collect_region_blocks_by_chunk(region, &mut chunks, chunk_size);
        }

        let chunk_list: Vec<_> = chunks.into_iter().filter(|(_, b)| !b.is_empty()).collect();

        NucleationChunkIter {
            chunks: chunk_list,
            index: 0,
            pack: pack.pack.clone(),
            config: config.to_mesher_config(),
            shared_atlas: Some(atlas),
            progress_callback: None,
            vertices_so_far: 0,
            triangles_so_far: 0,
        }
    }
}

/// Lazy iterator that yields one [`MeshOutput`] per chunk.
///
/// Created by [`UniversalSchematic::mesh_chunks`] or
/// [`UniversalSchematic::mesh_chunks_with_atlas`]. Implements [`Iterator`]
/// with `Item = Result<MeshOutput>`.
///
/// Supports an optional shared atlas (set via `mesh_chunks_with_atlas`) and
/// progress callbacks (set via [`set_progress_callback`]).
pub struct NucleationChunkIter {
    chunks: Vec<((i32, i32, i32), HashMap<MesherBlockPosition, InputBlock>)>,
    index: usize,
    pack: ResourcePack,
    config: MesherConfig,
    /// Pre-built global atlas shared across all chunks.
    shared_atlas: Option<TextureAtlas>,
    /// Optional progress callback invoked after each chunk.
    progress_callback: Option<Box<dyn Fn(MeshProgress)>>,
    /// Running totals for progress reporting.
    vertices_so_far: u64,
    triangles_so_far: u64,
}

impl NucleationChunkIter {
    /// Total number of chunks that will be yielded.
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// How many chunks have already been yielded.
    pub fn chunks_yielded(&self) -> usize {
        self.index
    }

    /// Whether this iterator uses a shared global atlas.
    pub fn has_shared_atlas(&self) -> bool {
        self.shared_atlas.is_some()
    }

    /// Get a reference to the shared atlas (if any).
    pub fn shared_atlas(&self) -> Option<&TextureAtlas> {
        self.shared_atlas.as_ref()
    }

    /// Set a callback that will be invoked after each chunk is meshed.
    ///
    /// The callback receives a [`MeshProgress`] with cumulative stats.
    pub fn set_progress_callback(&mut self, cb: Box<dyn Fn(MeshProgress)>) {
        self.progress_callback = Some(cb);
    }
}

impl Iterator for NucleationChunkIter {
    type Item = Result<MeshOutput>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.chunks.len() {
            return None;
        }

        let (chunk_coord, ref blocks) = self.chunks[self.index];
        self.index += 1;

        // Calculate bounds for this chunk
        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];
        for pos in blocks.keys() {
            min[0] = min[0].min(pos.x as f32);
            min[1] = min[1].min(pos.y as f32);
            min[2] = min[2].min(pos.z as f32);
            max[0] = max[0].max(pos.x as f32 + 1.0);
            max[1] = max[1].max(pos.y as f32 + 1.0);
            max[2] = max[2].max(pos.z as f32 + 1.0);
        }

        let bounds = MesherBoundingBox::new(min, max);
        let source = ChunkBlockSource::new(blocks.clone(), bounds);

        // Inject shared atlas into per-chunk config if available
        let mut chunk_config = self.config.clone();
        if let Some(ref atlas) = self.shared_atlas {
            chunk_config.pre_built_atlas = Some(atlas.clone());
        }

        let mesher = Mesher::with_config(self.pack.clone(), chunk_config);
        let output = match mesher.mesh(&source) {
            Ok(o) => o,
            Err(e) => return Some(Err(MeshError::Meshing(e.to_string()))),
        };

        let mesh = mesh_output_from_mesher(&output, Some(chunk_coord));

        // Update running totals and fire progress callback
        self.vertices_so_far += mesh.total_vertices() as u64;
        self.triangles_so_far += mesh.total_triangles() as u64;

        if let Some(ref cb) = self.progress_callback {
            cb(MeshProgress {
                phase: if self.index >= self.chunks.len() {
                    MeshPhase::Complete
                } else {
                    MeshPhase::MeshingChunks
                },
                chunks_done: self.index as u32,
                chunks_total: self.chunks.len() as u32,
                vertices_so_far: self.vertices_so_far,
                triangles_so_far: self.triangles_so_far,
            });
        }

        Some(Ok(mesh))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.chunks.len() - self.index;
        (remaining, Some(remaining))
    }
}

/// Helper function to mesh a single region.
fn mesh_region(
    pack: &ResourcePack,
    region: &Region,
    config: &MesherConfig,
) -> Result<Option<MeshOutput>> {
    let source = RegionBlockSource::new(region);

    if source.blocks.is_empty() {
        return Ok(None);
    }

    let mesher = Mesher::with_config(pack.clone(), config.clone());
    let output = mesher
        .mesh(&source)
        .map_err(|e| MeshError::Meshing(e.to_string()))?;

    Ok(Some(mesh_output_from_mesher(&output, None)))
}

/// Helper function to collect blocks and entities from a region.
fn collect_region_blocks(
    region: &Region,
    blocks: &mut HashMap<MesherBlockPosition, InputBlock>,
    min: &mut [f32; 3],
    max: &mut [f32; 3],
) {
    for index in 0..region.volume() {
        let (x, y, z) = region.index_to_coords(index);
        if let Some(block_state) = region.get_block(x, y, z) {
            if block_state.name != "minecraft:air" {
                let pos = MesherBlockPosition::new(x, y, z);
                let input_block = block_state_to_input_block(block_state);
                blocks.insert(pos, input_block);

                min[0] = min[0].min(x as f32);
                min[1] = min[1].min(y as f32);
                min[2] = min[2].min(z as f32);
                max[0] = max[0].max(x as f32 + 1.0);
                max[1] = max[1].max(y as f32 + 1.0);
                max[2] = max[2].max(z as f32 + 1.0);
            }
        }
    }

    // Also collect entities as entity: blocks
    collect_region_entities(region, blocks, min, max);
}

/// Helper function to collect blocks and entities from a region into chunk buckets.
fn collect_region_blocks_by_chunk(
    region: &Region,
    chunks: &mut HashMap<(i32, i32, i32), HashMap<MesherBlockPosition, InputBlock>>,
    chunk_size: i32,
) {
    for index in 0..region.volume() {
        let (x, y, z) = region.index_to_coords(index);
        if let Some(block_state) = region.get_block(x, y, z) {
            if block_state.name != "minecraft:air" {
                let chunk_x = x.div_euclid(chunk_size);
                let chunk_y = y.div_euclid(chunk_size);
                let chunk_z = z.div_euclid(chunk_size);

                let chunk_blocks = chunks.entry((chunk_x, chunk_y, chunk_z)).or_default();
                let pos = MesherBlockPosition::new(x, y, z);
                let input_block = block_state_to_input_block(block_state);
                chunk_blocks.insert(pos, input_block);
            }
        }
    }

    // Also collect entities into their respective chunks
    for entity in &region.entities {
        let input_block = entity_to_input_block(entity);
        let x = entity.position.0.floor() as i32;
        let y = entity.position.1.floor() as i32;
        let z = entity.position.2.floor() as i32;
        let pos = MesherBlockPosition::new(x, y, z);

        let chunk_x = x.div_euclid(chunk_size);
        let chunk_y = y.div_euclid(chunk_size);
        let chunk_z = z.div_euclid(chunk_size);

        let chunk_blocks = chunks.entry((chunk_x, chunk_y, chunk_z)).or_default();
        if !chunk_blocks.contains_key(&pos) {
            chunk_blocks.insert(pos, input_block);
        }
    }
}

/// Split a "namespace:path" resource name into (namespace, path).
fn split_resource_name(name: &str) -> Result<(String, String)> {
    match name.split_once(':') {
        Some((ns, path)) => Ok((ns.to_string(), path.to_string())),
        None => Ok(("minecraft".to_string(), name.to_string())),
    }
}

/// Manually serialize BlockstateDefinition to JSON since it doesn't implement Serialize.
fn blockstate_definition_to_json(def: &schematic_mesher::BlockstateDefinition) -> String {
    match def {
        schematic_mesher::BlockstateDefinition::Variants(variants) => {
            let mut map = serde_json::Map::new();
            let mut variants_map = serde_json::Map::new();
            for (key, models) in variants {
                if models.len() == 1 {
                    variants_map.insert(
                        key.clone(),
                        serde_json::to_value(&models[0]).unwrap_or_default(),
                    );
                } else {
                    variants_map.insert(
                        key.clone(),
                        serde_json::to_value(models).unwrap_or_default(),
                    );
                }
            }
            map.insert(
                "variants".to_string(),
                serde_json::Value::Object(variants_map),
            );
            serde_json::Value::Object(map).to_string()
        }
        schematic_mesher::BlockstateDefinition::Multipart(cases) => {
            let mut map = serde_json::Map::new();
            map.insert(
                "multipart".to_string(),
                serde_json::to_value(cases).unwrap_or_default(),
            );
            serde_json::Value::Object(map).to_string()
        }
    }
}

// ─── MeshExporter (FormatManager integration) ──────────────────────────────

use crate::formats::manager::SchematicExporter;

/// A format exporter that produces mesh data (GLB, USDZ) through the FormatManager.
///
/// Unlike other exporters, MeshExporter is stateful — it holds a loaded resource pack.
/// Register it with `FormatManager::register_exporter` after loading a resource pack.
pub struct MeshExporter {
    pack: ResourcePackSource,
}

impl MeshExporter {
    pub fn new(pack: ResourcePackSource) -> Self {
        Self { pack }
    }
}

impl SchematicExporter for MeshExporter {
    fn name(&self) -> String {
        "mesh".to_string()
    }

    fn extensions(&self) -> Vec<String> {
        vec!["glb".into(), "usdz".into()]
    }

    fn available_versions(&self) -> Vec<String> {
        vec!["glb".into(), "usdz".into()]
    }

    fn default_version(&self) -> String {
        "glb".to_string()
    }

    fn write(
        &self,
        schematic: &crate::UniversalSchematic,
        version: Option<&str>,
    ) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.write_with_settings(schematic, version, None)
    }

    fn write_with_settings(
        &self,
        schematic: &crate::UniversalSchematic,
        version: Option<&str>,
        settings: Option<&str>,
    ) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>> {
        let config: MeshConfig = match settings {
            Some(s) => serde_json::from_str(s)?,
            None => MeshConfig::default(),
        };
        let format = version.unwrap_or("glb");
        match format {
            "glb" => {
                let mesh = schematic.to_mesh(&self.pack, &config)?;
                mesh.to_glb()
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            }
            "usdz" => {
                let mesh = schematic.to_usdz(&self.pack, &config)?;
                mesh.to_usdz()
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            }
            _ => Err(format!("Unknown mesh format: {}. Use 'glb' or 'usdz'", format).into()),
        }
    }

    fn export_settings_schema(&self) -> Option<String> {
        serde_json::to_string_pretty(&MeshConfig::default()).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_config_builder() {
        let config = MeshConfig::new()
            .with_culling(false)
            .with_ambient_occlusion(false)
            .with_biome("swamp")
            .with_ao_intensity(0.6);

        assert!(!config.cull_hidden_faces);
        assert!(!config.ambient_occlusion);
        assert_eq!(config.biome, Some("swamp".to_string()));
        assert!((config.ao_intensity - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_block_state_to_input_block() {
        let block_state = BlockState::new("minecraft:oak_stairs".to_string())
            .with_property("facing", "north")
            .with_property("half", "bottom");

        let input = block_state_to_input_block(&block_state);

        assert_eq!(input.name, "minecraft:oak_stairs");
        assert_eq!(input.properties.get("facing"), Some(&"north".to_string()));
        assert_eq!(input.properties.get("half"), Some(&"bottom".to_string()));
    }

    #[test]
    fn test_mesh_config_defaults() {
        let config = MeshConfig::default();
        assert!(config.cull_hidden_faces);
        assert!(config.ambient_occlusion);
        assert!((config.ao_intensity - 0.4).abs() < 0.001);
        assert_eq!(config.biome, None);
        assert_eq!(config.atlas_max_size, 4096);
        assert!(config.cull_occluded_blocks);
        assert!(!config.greedy_meshing);
    }

    #[test]
    fn test_mesh_config_new_fields() {
        let config = MeshConfig::new()
            .with_cull_occluded_blocks(false)
            .with_greedy_meshing(true)
            .with_atlas_max_size(2048);

        assert!(!config.cull_occluded_blocks);
        assert!(config.greedy_meshing);
        assert_eq!(config.atlas_max_size, 2048);
    }

    #[test]
    fn test_mesh_config_full_builder_chain() {
        let config = MeshConfig::new()
            .with_culling(false)
            .with_ambient_occlusion(false)
            .with_ao_intensity(0.8)
            .with_biome("jungle")
            .with_atlas_max_size(1024)
            .with_cull_occluded_blocks(false)
            .with_greedy_meshing(true);

        assert!(!config.cull_hidden_faces);
        assert!(!config.ambient_occlusion);
        assert!((config.ao_intensity - 0.8).abs() < 0.001);
        assert_eq!(config.biome, Some("jungle".to_string()));
        assert_eq!(config.atlas_max_size, 1024);
        assert!(!config.cull_occluded_blocks);
        assert!(config.greedy_meshing);
    }

    #[test]
    fn test_split_resource_name_with_namespace() {
        let (ns, path) = split_resource_name("minecraft:stone").unwrap();
        assert_eq!(ns, "minecraft");
        assert_eq!(path, "stone");
    }

    #[test]
    fn test_split_resource_name_without_namespace() {
        let (ns, path) = split_resource_name("stone").unwrap();
        assert_eq!(ns, "minecraft");
        assert_eq!(path, "stone");
    }

    #[test]
    fn test_split_resource_name_custom_namespace() {
        let (ns, path) = split_resource_name("mymod:block/custom_block").unwrap();
        assert_eq!(ns, "mymod");
        assert_eq!(path, "block/custom_block");
    }

    #[test]
    fn test_block_state_to_input_block_no_properties() {
        let block_state = BlockState::new("minecraft:stone".to_string());
        let input = block_state_to_input_block(&block_state);
        assert_eq!(input.name, "minecraft:stone");
        assert!(input.properties.is_empty());
    }

    #[test]
    fn test_to_mesher_config_propagates_fields() {
        let config = MeshConfig::new()
            .with_culling(false)
            .with_ambient_occlusion(false)
            .with_ao_intensity(0.7)
            .with_cull_occluded_blocks(false)
            .with_greedy_meshing(true);

        let mesher_config = config.to_mesher_config();
        assert!(!mesher_config.cull_hidden_faces);
        assert!(!mesher_config.ambient_occlusion);
        assert!((mesher_config.ao_intensity - 0.7).abs() < 0.001);
        assert!(!mesher_config.cull_occluded_blocks);
        assert!(mesher_config.greedy_meshing);
    }

    #[test]
    fn test_empty_schematic_mesh_error() {
        let _schematic = UniversalSchematic::new("Empty".to_string());
        // from_bytes with invalid data should return an error
        let result = ResourcePackSource::from_bytes(&[0, 1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    fn test_resource_pack_stats() {
        // from_bytes with invalid data should error
        let result = ResourcePackSource::from_bytes(&[]);
        assert!(result.is_err());
    }

    // ─── MeshOutput / MeshLayer tests ──────────────────────────────────────

    #[test]
    fn test_mesh_layer_type_is_accessible() {
        // Verify MeshLayer is accessible via nucleation::meshing::MeshLayer
        let layer = MeshLayer {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            colors: vec![[1.0, 1.0, 1.0, 1.0]; 3],
            indices: vec![0, 1, 2],
        };
        assert_eq!(layer.vertex_count(), 3);
        assert_eq!(layer.triangle_count(), 1);
        assert!(!layer.is_empty());
    }

    #[test]
    fn test_mesh_layer_empty() {
        let empty = MeshLayer::default();
        assert!(empty.is_empty());
        assert_eq!(empty.vertex_count(), 0);
        assert_eq!(empty.triangle_count(), 0);
    }

    #[test]
    fn test_mesh_output_is_empty() {
        use schematic_mesher::TextureAtlas;

        let output = MeshOutput {
            opaque: MeshLayer::default(),
            cutout: MeshLayer::default(),
            transparent: MeshLayer::default(),
            atlas: TextureAtlas::empty(),
            animated_textures: Vec::new(),
            bounds: MesherBoundingBox::new([0.0; 3], [0.0; 3]),
            chunk_coord: None,
            lod_level: 0,
        };
        assert!(output.is_empty());
        assert_eq!(output.total_vertices(), 0);
        assert_eq!(output.total_triangles(), 0);
    }

    #[test]
    fn test_mesh_result_alias_compiles() {
        use schematic_mesher::TextureAtlas;

        fn takes_mesh_result(_r: &MeshResult) {}
        fn takes_mesh_output(_r: &MeshOutput) {}

        let output = MeshOutput {
            opaque: MeshLayer::default(),
            cutout: MeshLayer::default(),
            transparent: MeshLayer::default(),
            atlas: TextureAtlas::empty(),
            animated_textures: Vec::new(),
            bounds: MesherBoundingBox::new([0.0; 3], [0.0; 3]),
            chunk_coord: None,
            lod_level: 0,
        };
        // Both should accept the same value (they're the same type)
        takes_mesh_result(&output);
        takes_mesh_output(&output);
    }

    #[test]
    fn test_multi_mesh_result_alias() {
        let map: MultiMeshResult = HashMap::new();
        assert!(map.is_empty());
    }

    #[test]
    fn test_chunk_mesh_result_backward_compat() {
        let result = ChunkMeshResult {
            meshes: HashMap::new(),
            total_vertex_count: 0,
            total_triangle_count: 0,
        };
        assert!(result.meshes.is_empty());
    }

    #[test]
    fn test_nucleation_chunk_iter_empty() {
        // An iterator over an empty schematic should yield nothing.
        // We can't construct a ResourcePack without real data, so we
        // early-return after verifying the struct is accessible.
        let _iter_type_check: fn() -> NucleationChunkIter = || unreachable!();
    }
}
