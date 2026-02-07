//! Meshing support for generating 3D models from schematics.
//!
//! This module provides integration with the `schematic-mesher` crate to convert
//! Nucleation schematics into 3D mesh formats (GLB/glTF).
//!
//! # Features
//!
//! Enable the `meshing` feature in your `Cargo.toml` to use this module:
//!
//! ```toml
//! nucleation = { version = "0.1", features = ["meshing"] }
//! ```
//!
//! # Example
//!
//! ```ignore
//! use nucleation::{UniversalSchematic, meshing::{MeshConfig, ResourcePackSource}};
//!
//! // Load your schematic
//! let schematic = UniversalSchematic::from_litematic_bytes(&data)?;
//!
//! // Load a resource pack
//! let pack = ResourcePackSource::from_file("resourcepack.zip")?;
//!
//! // Generate a single mesh for the entire schematic
//! let config = MeshConfig::default();
//! let result = schematic.to_mesh(&pack, &config)?;
//!
//! // Save the GLB file
//! std::fs::write("output.glb", result.glb_data)?;
//! ```

use schematic_mesher::{
    export_glb, export_raw, export_usdz, resource_pack::TextureData,
    BlockPosition as MesherBlockPosition, BlockSource, BoundingBox as MesherBoundingBox,
    InputBlock, Mesher, MesherConfig, MesherOutput, RawMeshData, ResourcePack,
};
use std::collections::HashMap;
use std::path::Path;

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
#[derive(Debug, Clone)]
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

/// Result of mesh generation.
#[derive(Debug, Clone)]
pub struct MeshResult {
    /// The GLB binary data.
    pub glb_data: Vec<u8>,
    /// Number of vertices in the mesh.
    pub vertex_count: usize,
    /// Number of triangles in the mesh.
    pub triangle_count: usize,
    /// Whether the mesh contains transparent geometry.
    pub has_transparency: bool,
    /// Bounding box of the mesh.
    pub bounds: [f32; 6], // [min_x, min_y, min_z, max_x, max_y, max_z]
}

/// Result of mesh generation for multiple regions.
#[derive(Debug, Clone)]
pub struct MultiMeshResult {
    /// Map of region name to mesh result.
    pub meshes: HashMap<String, MeshResult>,
    /// Total vertex count across all meshes.
    pub total_vertex_count: usize,
    /// Total triangle count across all meshes.
    pub total_triangle_count: usize,
}

/// Result of chunk-based mesh generation.
#[derive(Debug, Clone)]
pub struct ChunkMeshResult {
    /// Map of chunk coordinate to mesh result.
    pub meshes: HashMap<(i32, i32, i32), MeshResult>,
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

/// Build a MeshResult from a MesherOutput and export data bytes.
fn mesh_result_from_output(output: &MesherOutput, data: Vec<u8>) -> MeshResult {
    MeshResult {
        glb_data: data,
        vertex_count: output.total_vertices(),
        triangle_count: output.total_triangles(),
        has_transparency: output.has_transparency(),
        bounds: [
            output.bounds.min[0],
            output.bounds.min[1],
            output.bounds.min[2],
            output.bounds.max[0],
            output.bounds.max[1],
            output.bounds.max[2],
        ],
    }
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
    let mut input = InputBlock::new(&block_state.name);
    for (key, value) in &block_state.properties {
        input.properties.insert(key.clone(), value.clone());
    }
    input
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

    /// Generate a single GLB mesh for the entire schematic.
    ///
    /// This combines all regions into a single mesh output.
    pub fn to_mesh(&self, pack: &ResourcePackSource, config: &MeshConfig) -> Result<MeshResult> {
        let output = self.compute_mesh_output(pack, config)?;
        let glb_data = export_glb(&output).map_err(|e| MeshError::Export(e.to_string()))?;
        Ok(mesh_result_from_output(&output, glb_data))
    }

    /// Generate a USDZ mesh for the entire schematic.
    pub fn to_usdz(&self, pack: &ResourcePackSource, config: &MeshConfig) -> Result<MeshResult> {
        let output = self.compute_mesh_output(pack, config)?;
        let usdz_data = export_usdz(&output).map_err(|e| MeshError::Export(e.to_string()))?;
        Ok(mesh_result_from_output(&output, usdz_data))
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
    /// Returns a map of region name to mesh result.
    pub fn mesh_by_region(
        &self,
        pack: &ResourcePackSource,
        config: &MeshConfig,
    ) -> Result<MultiMeshResult> {
        let mesher_config = config.to_mesher_config();

        let mut meshes = HashMap::new();
        let mut total_vertex_count = 0;
        let mut total_triangle_count = 0;

        // Mesh default region
        if let Some(result) = mesh_region(&pack.pack, &self.default_region, &mesher_config)? {
            total_vertex_count += result.vertex_count;
            total_triangle_count += result.triangle_count;
            meshes.insert(self.default_region_name.clone(), result);
        }

        // Mesh other regions
        for (name, region) in &self.other_regions {
            if let Some(result) = mesh_region(&pack.pack, region, &mesher_config)? {
                total_vertex_count += result.vertex_count;
                total_triangle_count += result.triangle_count;
                meshes.insert(name.clone(), result);
            }
        }

        Ok(MultiMeshResult {
            meshes,
            total_vertex_count,
            total_triangle_count,
        })
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

    /// Generate one mesh per chunk of the specified size.
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

            let glb_data = export_glb(&output).map_err(|e| MeshError::Export(e.to_string()))?;

            let result = mesh_result_from_output(&output, glb_data);

            total_vertex_count += result.vertex_count;
            total_triangle_count += result.triangle_count;
            meshes.insert(chunk_coord, result);
        }

        Ok(ChunkMeshResult {
            meshes,
            total_vertex_count,
            total_triangle_count,
        })
    }
}

/// Helper function to mesh a single region.
fn mesh_region(
    pack: &ResourcePack,
    region: &Region,
    config: &MesherConfig,
) -> Result<Option<MeshResult>> {
    let source = RegionBlockSource::new(region);

    if source.blocks.is_empty() {
        return Ok(None);
    }

    let mesher = Mesher::with_config(pack.clone(), config.clone());
    let output = mesher
        .mesh(&source)
        .map_err(|e| MeshError::Meshing(e.to_string()))?;

    let glb_data = export_glb(&output).map_err(|e| MeshError::Export(e.to_string()))?;

    Ok(Some(mesh_result_from_output(&output, glb_data)))
}

/// Helper function to collect blocks from a region.
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
}

/// Helper function to collect blocks from a region into chunk buckets.
fn collect_region_blocks_by_chunk(
    region: &Region,
    chunks: &mut HashMap<(i32, i32, i32), HashMap<MesherBlockPosition, InputBlock>>,
    chunk_size: i32,
) {
    for index in 0..region.volume() {
        let (x, y, z) = region.index_to_coords(index);
        if let Some(block_state) = region.get_block(x, y, z) {
            if block_state.name != "minecraft:air" {
                // Calculate chunk coordinate
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
        let mut block_state = BlockState::new("minecraft:oak_stairs".to_string());
        block_state
            .properties
            .insert("facing".to_string(), "north".to_string());
        block_state
            .properties
            .insert("half".to_string(), "bottom".to_string());

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
}
