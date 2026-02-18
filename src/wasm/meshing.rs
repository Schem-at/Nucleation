//! Meshing WASM bindings
//!
//! Generate 3D meshes (GLB/USDZ) from schematics using resource packs.
//!
//! Exposes [`MeshOutputWrapper`] with per-layer typed array accessors
//! (opaque, cutout, transparent), atlas data, and export helpers.

use bytemuck;
use js_sys::{Array, Float32Array, Object, Reflect, Uint32Array, Uint8Array};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use crate::meshing::{
    ChunkMeshResult, MeshConfig, MeshOutput, NucleationChunkIter, RawMeshExport,
    ResourcePackSource,
};

use super::schematic::SchematicWrapper;

/// Wrapper for a Minecraft resource pack.
#[wasm_bindgen]
pub struct ResourcePackWrapper {
    inner: ResourcePackSource,
}

#[wasm_bindgen]
impl ResourcePackWrapper {
    /// Load a resource pack from bytes (ZIP file contents).
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Result<ResourcePackWrapper, JsValue> {
        let inner =
            ResourcePackSource::from_bytes(data).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self { inner })
    }

    /// Get the number of blockstates in the resource pack.
    #[wasm_bindgen(getter, js_name = blockstateCount)]
    pub fn blockstate_count(&self) -> usize {
        self.inner.stats().blockstate_count
    }

    /// Get the number of models in the resource pack.
    #[wasm_bindgen(getter, js_name = modelCount)]
    pub fn model_count(&self) -> usize {
        self.inner.stats().model_count
    }

    /// Get the number of textures in the resource pack.
    #[wasm_bindgen(getter, js_name = textureCount)]
    pub fn texture_count(&self) -> usize {
        self.inner.stats().texture_count
    }

    /// Get the namespaces in the resource pack.
    #[wasm_bindgen(getter)]
    pub fn namespaces(&self) -> Array {
        let stats = self.inner.stats();
        let arr = Array::new();
        for ns in stats.namespaces {
            arr.push(&JsValue::from_str(&ns));
        }
        arr
    }

    /// List all blockstate names.
    #[wasm_bindgen(js_name = listBlockstates)]
    pub fn list_blockstates(&self) -> Array {
        let names = self.inner.list_blockstates();
        let arr = Array::new();
        for name in names {
            arr.push(&JsValue::from_str(&name));
        }
        arr
    }

    /// List all model names.
    #[wasm_bindgen(js_name = listModels)]
    pub fn list_models(&self) -> Array {
        let names = self.inner.list_models();
        let arr = Array::new();
        for name in names {
            arr.push(&JsValue::from_str(&name));
        }
        arr
    }

    /// List all texture names.
    #[wasm_bindgen(js_name = listTextures)]
    pub fn list_textures(&self) -> Array {
        let names = self.inner.list_textures();
        let arr = Array::new();
        for name in names {
            arr.push(&JsValue::from_str(&name));
        }
        arr
    }

    /// Get a blockstate definition as a JSON string. Returns null if not found.
    #[wasm_bindgen(js_name = getBlockstateJson)]
    pub fn get_blockstate_json(&self, name: &str) -> Option<String> {
        self.inner.get_blockstate_json(name)
    }

    /// Get a block model as a JSON string. Returns null if not found.
    #[wasm_bindgen(js_name = getModelJson)]
    pub fn get_model_json(&self, name: &str) -> Option<String> {
        self.inner.get_model_json(name)
    }

    /// Get texture info as a JS object with width, height, isAnimated, frameCount.
    /// Returns null if not found.
    #[wasm_bindgen(js_name = getTextureInfo)]
    pub fn get_texture_info(&self, name: &str) -> JsValue {
        match self.inner.get_texture_info(name) {
            Some((w, h, animated, frames)) => {
                let obj = Object::new();
                Reflect::set(&obj, &"width".into(), &JsValue::from(w)).unwrap();
                Reflect::set(&obj, &"height".into(), &JsValue::from(h)).unwrap();
                Reflect::set(&obj, &"isAnimated".into(), &JsValue::from(animated)).unwrap();
                Reflect::set(&obj, &"frameCount".into(), &JsValue::from(frames)).unwrap();
                obj.into()
            }
            None => JsValue::NULL,
        }
    }

    /// Get raw RGBA8 pixel data for a texture. Returns null if not found.
    #[wasm_bindgen(js_name = getTexturePixels)]
    pub fn get_texture_pixels(&self, name: &str) -> Option<Vec<u8>> {
        self.inner.get_texture_pixels(name).map(|p| p.to_vec())
    }

    /// Add a blockstate definition from a JSON string.
    #[wasm_bindgen(js_name = addBlockstateJson)]
    pub fn add_blockstate_json(&mut self, name: &str, json: &str) -> Result<(), JsValue> {
        self.inner
            .add_blockstate_json(name, json)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Add a block model from a JSON string.
    #[wasm_bindgen(js_name = addModelJson)]
    pub fn add_model_json(&mut self, name: &str, json: &str) -> Result<(), JsValue> {
        self.inner
            .add_model_json(name, json)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Add a texture from raw RGBA8 pixel data.
    #[wasm_bindgen(js_name = addTexture)]
    pub fn add_texture(
        &mut self,
        name: &str,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<(), JsValue> {
        self.inner
            .add_texture(name, width, height, pixels.to_vec())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get statistics about the resource pack as a JS object.
    #[wasm_bindgen(js_name = getStats)]
    pub fn get_stats(&self) -> Object {
        let stats = self.inner.stats();
        let obj = Object::new();
        Reflect::set(
            &obj,
            &"blockstateCount".into(),
            &JsValue::from(stats.blockstate_count),
        )
        .unwrap();
        Reflect::set(
            &obj,
            &"modelCount".into(),
            &JsValue::from(stats.model_count),
        )
        .unwrap();
        Reflect::set(
            &obj,
            &"textureCount".into(),
            &JsValue::from(stats.texture_count),
        )
        .unwrap();

        let namespaces = Array::new();
        for ns in stats.namespaces {
            namespaces.push(&JsValue::from_str(&ns));
        }
        Reflect::set(&obj, &"namespaces".into(), &namespaces).unwrap();

        obj
    }
}

/// Configuration for mesh generation.
#[wasm_bindgen]
pub struct MeshConfigWrapper {
    inner: MeshConfig,
}

#[wasm_bindgen]
impl MeshConfigWrapper {
    /// Create a new MeshConfig with default settings.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: MeshConfig::default(),
        }
    }

    /// Enable or disable face culling between adjacent blocks.
    #[wasm_bindgen(js_name = setCullHiddenFaces)]
    pub fn set_cull_hidden_faces(&mut self, enabled: bool) {
        self.inner.cull_hidden_faces = enabled;
    }

    /// Enable or disable ambient occlusion.
    #[wasm_bindgen(js_name = setAmbientOcclusion)]
    pub fn set_ambient_occlusion(&mut self, enabled: bool) {
        self.inner.ambient_occlusion = enabled;
    }

    /// Set the AO intensity (0.0 = no darkening, 1.0 = full darkening).
    #[wasm_bindgen(js_name = setAoIntensity)]
    pub fn set_ao_intensity(&mut self, intensity: f32) {
        self.inner.ao_intensity = intensity;
    }

    /// Set the biome for tinting (e.g., "plains", "swamp").
    #[wasm_bindgen(js_name = setBiome)]
    pub fn set_biome(&mut self, biome: &str) {
        self.inner.biome = Some(biome.to_string());
    }

    /// Set the maximum atlas texture size.
    #[wasm_bindgen(js_name = setAtlasMaxSize)]
    pub fn set_atlas_max_size(&mut self, size: u32) {
        self.inner.atlas_max_size = size;
    }

    /// Get whether face culling is enabled.
    #[wasm_bindgen(getter, js_name = cullHiddenFaces)]
    pub fn cull_hidden_faces(&self) -> bool {
        self.inner.cull_hidden_faces
    }

    /// Get whether ambient occlusion is enabled.
    #[wasm_bindgen(getter, js_name = ambientOcclusion)]
    pub fn ambient_occlusion(&self) -> bool {
        self.inner.ambient_occlusion
    }

    /// Get the AO intensity.
    #[wasm_bindgen(getter, js_name = aoIntensity)]
    pub fn ao_intensity(&self) -> f32 {
        self.inner.ao_intensity
    }

    /// Get the biome name.
    #[wasm_bindgen(getter)]
    pub fn biome(&self) -> Option<String> {
        self.inner.biome.clone()
    }

    /// Get the maximum atlas size.
    #[wasm_bindgen(getter, js_name = atlasMaxSize)]
    pub fn atlas_max_size(&self) -> u32 {
        self.inner.atlas_max_size
    }

    /// Enable or disable occluded block culling.
    #[wasm_bindgen(js_name = setCullOccludedBlocks)]
    pub fn set_cull_occluded_blocks(&mut self, enabled: bool) {
        self.inner.cull_occluded_blocks = enabled;
    }

    /// Get whether occluded block culling is enabled.
    #[wasm_bindgen(getter, js_name = cullOccludedBlocks)]
    pub fn cull_occluded_blocks(&self) -> bool {
        self.inner.cull_occluded_blocks
    }

    /// Enable or disable greedy meshing.
    #[wasm_bindgen(js_name = setGreedyMeshing)]
    pub fn set_greedy_meshing(&mut self, enabled: bool) {
        self.inner.greedy_meshing = enabled;
    }

    /// Get whether greedy meshing is enabled.
    #[wasm_bindgen(getter, js_name = greedyMeshing)]
    pub fn greedy_meshing(&self) -> bool {
        self.inner.greedy_meshing
    }
}

// ─── MeshOutputWrapper ──────────────────────────────────────────────────────

/// WASM wrapper for [`MeshOutput`].
///
/// Exposes per-layer typed arrays (opaque, cutout, transparent), shared atlas,
/// export helpers, and metadata. This is the primary return type for all
/// meshing methods.
#[wasm_bindgen]
pub struct MeshOutputWrapper {
    inner: MeshOutput,
}

#[wasm_bindgen]
impl MeshOutputWrapper {
    // ── Opaque layer ────────────────────────────────────────────────────

    /// Opaque layer vertex positions as Float32Array (x,y,z per vertex).
    #[wasm_bindgen(js_name = opaquePositions)]
    pub fn opaque_positions(&self) -> Float32Array {
        let flat: &[f32] = bytemuck::cast_slice(&self.inner.opaque.positions);
        Float32Array::from(flat)
    }

    /// Opaque layer vertex normals as Float32Array.
    #[wasm_bindgen(js_name = opaqueNormals)]
    pub fn opaque_normals(&self) -> Float32Array {
        let flat: &[f32] = bytemuck::cast_slice(&self.inner.opaque.normals);
        Float32Array::from(flat)
    }

    /// Opaque layer texture coordinates as Float32Array (u,v per vertex).
    #[wasm_bindgen(js_name = opaqueUvs)]
    pub fn opaque_uvs(&self) -> Float32Array {
        let flat: &[f32] = bytemuck::cast_slice(&self.inner.opaque.uvs);
        Float32Array::from(flat)
    }

    /// Opaque layer vertex colors as Float32Array (r,g,b,a per vertex).
    #[wasm_bindgen(js_name = opaqueColors)]
    pub fn opaque_colors(&self) -> Float32Array {
        let flat: &[f32] = bytemuck::cast_slice(&self.inner.opaque.colors);
        Float32Array::from(flat)
    }

    /// Opaque layer triangle indices as Uint32Array.
    #[wasm_bindgen(js_name = opaqueIndices)]
    pub fn opaque_indices(&self) -> Uint32Array {
        Uint32Array::from(self.inner.opaque.indices.as_slice())
    }

    // ── Cutout layer ────────────────────────────────────────────────────

    /// Cutout layer vertex positions as Float32Array.
    #[wasm_bindgen(js_name = cutoutPositions)]
    pub fn cutout_positions(&self) -> Float32Array {
        let flat: &[f32] = bytemuck::cast_slice(&self.inner.cutout.positions);
        Float32Array::from(flat)
    }

    /// Cutout layer vertex normals as Float32Array.
    #[wasm_bindgen(js_name = cutoutNormals)]
    pub fn cutout_normals(&self) -> Float32Array {
        let flat: &[f32] = bytemuck::cast_slice(&self.inner.cutout.normals);
        Float32Array::from(flat)
    }

    /// Cutout layer texture coordinates as Float32Array.
    #[wasm_bindgen(js_name = cutoutUvs)]
    pub fn cutout_uvs(&self) -> Float32Array {
        let flat: &[f32] = bytemuck::cast_slice(&self.inner.cutout.uvs);
        Float32Array::from(flat)
    }

    /// Cutout layer vertex colors as Float32Array.
    #[wasm_bindgen(js_name = cutoutColors)]
    pub fn cutout_colors(&self) -> Float32Array {
        let flat: &[f32] = bytemuck::cast_slice(&self.inner.cutout.colors);
        Float32Array::from(flat)
    }

    /// Cutout layer triangle indices as Uint32Array.
    #[wasm_bindgen(js_name = cutoutIndices)]
    pub fn cutout_indices(&self) -> Uint32Array {
        Uint32Array::from(self.inner.cutout.indices.as_slice())
    }

    // ── Transparent layer ───────────────────────────────────────────────

    /// Transparent layer vertex positions as Float32Array.
    #[wasm_bindgen(js_name = transparentPositions)]
    pub fn transparent_positions(&self) -> Float32Array {
        let flat: &[f32] = bytemuck::cast_slice(&self.inner.transparent.positions);
        Float32Array::from(flat)
    }

    /// Transparent layer vertex normals as Float32Array.
    #[wasm_bindgen(js_name = transparentNormals)]
    pub fn transparent_normals(&self) -> Float32Array {
        let flat: &[f32] = bytemuck::cast_slice(&self.inner.transparent.normals);
        Float32Array::from(flat)
    }

    /// Transparent layer texture coordinates as Float32Array.
    #[wasm_bindgen(js_name = transparentUvs)]
    pub fn transparent_uvs(&self) -> Float32Array {
        let flat: &[f32] = bytemuck::cast_slice(&self.inner.transparent.uvs);
        Float32Array::from(flat)
    }

    /// Transparent layer vertex colors as Float32Array.
    #[wasm_bindgen(js_name = transparentColors)]
    pub fn transparent_colors(&self) -> Float32Array {
        let flat: &[f32] = bytemuck::cast_slice(&self.inner.transparent.colors);
        Float32Array::from(flat)
    }

    /// Transparent layer triangle indices as Uint32Array.
    #[wasm_bindgen(js_name = transparentIndices)]
    pub fn transparent_indices(&self) -> Uint32Array {
        Uint32Array::from(self.inner.transparent.indices.as_slice())
    }

    // ── Atlas ───────────────────────────────────────────────────────────

    /// Shared texture atlas RGBA pixel data as Uint8Array.
    #[wasm_bindgen(js_name = atlasRgba)]
    pub fn atlas_rgba(&self) -> Uint8Array {
        Uint8Array::from(self.inner.atlas.pixels.as_slice())
    }

    /// Atlas width in pixels.
    #[wasm_bindgen(getter, js_name = atlasWidth)]
    pub fn atlas_width(&self) -> u32 {
        self.inner.atlas.width
    }

    /// Atlas height in pixels.
    #[wasm_bindgen(getter, js_name = atlasHeight)]
    pub fn atlas_height(&self) -> u32 {
        self.inner.atlas.height
    }

    // ── Export helpers ───────────────────────────────────────────────────

    /// Get GLB binary data as Uint8Array.
    #[wasm_bindgen(js_name = toGlb)]
    pub fn to_glb(&self) -> Result<Uint8Array, JsValue> {
        let data = self.inner.to_glb().map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Uint8Array::from(data.as_slice()))
    }

    /// Get USDZ binary data as Uint8Array.
    #[wasm_bindgen(js_name = toUsdz)]
    pub fn to_usdz(&self) -> Result<Uint8Array, JsValue> {
        let data = self.inner.to_usdz().map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Uint8Array::from(data.as_slice()))
    }

    // ── Backward-compat: GLB data as glbData getter (matches old MeshResultWrapper) ──

    /// Get the GLB binary data (backward-compat getter).
    #[wasm_bindgen(getter, js_name = glbData)]
    pub fn glb_data(&self) -> Result<Uint8Array, JsValue> {
        self.to_glb()
    }

    // ── Metadata ────────────────────────────────────────────────────────

    /// LOD level (0 = full detail).
    #[wasm_bindgen(getter, js_name = lodLevel)]
    pub fn lod_level(&self) -> u8 {
        self.inner.lod_level
    }

    /// Chunk coordinate as [cx, cy, cz] or null if not a chunk mesh.
    #[wasm_bindgen(js_name = chunkCoord)]
    pub fn chunk_coord(&self) -> JsValue {
        match self.inner.chunk_coord {
            Some((cx, cy, cz)) => {
                let arr = Array::new();
                arr.push(&JsValue::from(cx));
                arr.push(&JsValue::from(cy));
                arr.push(&JsValue::from(cz));
                arr.into()
            }
            None => JsValue::NULL,
        }
    }

    /// Total vertex count across all layers.
    #[wasm_bindgen(getter, js_name = totalVertices)]
    pub fn total_vertices(&self) -> usize {
        self.inner.total_vertices()
    }

    /// Total triangle count across all layers.
    #[wasm_bindgen(getter, js_name = totalTriangles)]
    pub fn total_triangles(&self) -> usize {
        self.inner.total_triangles()
    }

    /// Whether the mesh has transparent geometry.
    #[wasm_bindgen(getter, js_name = hasTransparency)]
    pub fn has_transparency(&self) -> bool {
        self.inner.has_transparency()
    }

    /// Whether all layers are empty.
    #[wasm_bindgen(getter, js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    // ── Backward-compat getters (match old MeshResultWrapper) ───────────

    /// Get the vertex count (backward-compat).
    #[wasm_bindgen(getter, js_name = vertexCount)]
    pub fn vertex_count(&self) -> usize {
        self.inner.total_vertices()
    }

    /// Get the triangle count (backward-compat).
    #[wasm_bindgen(getter, js_name = triangleCount)]
    pub fn triangle_count(&self) -> usize {
        self.inner.total_triangles()
    }

    /// Get the bounding box as [minX, minY, minZ, maxX, maxY, maxZ].
    #[wasm_bindgen(getter)]
    pub fn bounds(&self) -> Array {
        let arr = Array::new();
        for v in &self.inner.bounds.min {
            arr.push(&JsValue::from(*v));
        }
        for v in &self.inner.bounds.max {
            arr.push(&JsValue::from(*v));
        }
        arr
    }

    /// Get mesh statistics as a JS object.
    #[wasm_bindgen(js_name = getStats)]
    pub fn get_stats(&self) -> Object {
        let obj = Object::new();
        Reflect::set(
            &obj,
            &"vertexCount".into(),
            &JsValue::from(self.inner.total_vertices()),
        )
        .unwrap();
        Reflect::set(
            &obj,
            &"triangleCount".into(),
            &JsValue::from(self.inner.total_triangles()),
        )
        .unwrap();
        Reflect::set(
            &obj,
            &"hasTransparency".into(),
            &JsValue::from(self.inner.has_transparency()),
        )
        .unwrap();
        obj
    }
}

/// Backward-compatible alias — old code using `MeshResultWrapper` still compiles.
pub type MeshResultWrapper = MeshOutputWrapper;

// ─── MultiMeshResultWrapper ─────────────────────────────────────────────────

/// Result of per-region mesh generation.
///
/// Wraps a `HashMap<String, MeshOutput>` for WASM consumption.
#[wasm_bindgen]
pub struct MultiMeshResultWrapper {
    inner: HashMap<String, MeshOutput>,
}

#[wasm_bindgen]
impl MultiMeshResultWrapper {
    /// Get the region names.
    #[wasm_bindgen(js_name = getRegionNames)]
    pub fn get_region_names(&self) -> Array {
        let arr = Array::new();
        for name in self.inner.keys() {
            arr.push(&JsValue::from_str(name));
        }
        arr
    }

    /// Get the mesh for a specific region.
    #[wasm_bindgen(js_name = getMesh)]
    pub fn get_mesh(&self, region_name: &str) -> Option<MeshOutputWrapper> {
        self.inner
            .get(region_name)
            .map(|mesh| MeshOutputWrapper {
                inner: mesh.clone(),
            })
    }

    /// Get the total vertex count across all regions.
    #[wasm_bindgen(getter, js_name = totalVertexCount)]
    pub fn total_vertex_count(&self) -> usize {
        self.inner.values().map(|m| m.total_vertices()).sum()
    }

    /// Get the total triangle count across all regions.
    #[wasm_bindgen(getter, js_name = totalTriangleCount)]
    pub fn total_triangle_count(&self) -> usize {
        self.inner.values().map(|m| m.total_triangles()).sum()
    }

    /// Get the number of meshes.
    #[wasm_bindgen(getter, js_name = meshCount)]
    pub fn mesh_count(&self) -> usize {
        self.inner.len()
    }
}

// ─── ChunkMeshResultWrapper ─────────────────────────────────────────────────

/// Result of chunk-based mesh generation (eager, all at once).
#[wasm_bindgen]
pub struct ChunkMeshResultWrapper {
    inner: ChunkMeshResult,
}

#[wasm_bindgen]
impl ChunkMeshResultWrapper {
    /// Get the chunk coordinates as an array of [x, y, z] arrays.
    #[wasm_bindgen(js_name = getChunkCoordinates)]
    pub fn get_chunk_coordinates(&self) -> Array {
        let arr = Array::new();
        for (cx, cy, cz) in self.inner.meshes.keys() {
            let coord = Array::new();
            coord.push(&JsValue::from(*cx));
            coord.push(&JsValue::from(*cy));
            coord.push(&JsValue::from(*cz));
            arr.push(&coord);
        }
        arr
    }

    /// Get the mesh for a specific chunk.
    #[wasm_bindgen(js_name = getMesh)]
    pub fn get_mesh(&self, cx: i32, cy: i32, cz: i32) -> Option<MeshOutputWrapper> {
        self.inner
            .meshes
            .get(&(cx, cy, cz))
            .map(|mesh| MeshOutputWrapper {
                inner: mesh.clone(),
            })
    }

    /// Get the total vertex count.
    #[wasm_bindgen(getter, js_name = totalVertexCount)]
    pub fn total_vertex_count(&self) -> usize {
        self.inner.total_vertex_count
    }

    /// Get the total triangle count.
    #[wasm_bindgen(getter, js_name = totalTriangleCount)]
    pub fn total_triangle_count(&self) -> usize {
        self.inner.total_triangle_count
    }

    /// Get the number of chunks.
    #[wasm_bindgen(getter, js_name = chunkCount)]
    pub fn chunk_count(&self) -> usize {
        self.inner.meshes.len()
    }
}

// ─── ChunkMeshIteratorWrapper ───────────────────────────────────────────────

/// Lazy chunk mesh iterator for WASM.
///
/// Pull-based: call `advance()` to step to the next chunk, then
/// `current()` to get the [`MeshOutputWrapper`].
#[wasm_bindgen]
pub struct ChunkMeshIteratorWrapper {
    inner: NucleationChunkIter,
    current: Option<MeshOutput>,
}

#[wasm_bindgen]
impl ChunkMeshIteratorWrapper {
    /// Advance to the next chunk. Returns `true` if a chunk is available.
    pub fn advance(&mut self) -> Result<bool, JsValue> {
        match self.inner.next() {
            Some(Ok(mesh)) => {
                self.current = Some(mesh);
                Ok(true)
            }
            Some(Err(e)) => Err(JsValue::from_str(&e.to_string())),
            None => {
                self.current = None;
                Ok(false)
            }
        }
    }

    /// Get the current chunk mesh (after a successful `advance()`).
    /// Returns null if no current mesh.
    pub fn current(&self) -> Option<MeshOutputWrapper> {
        self.current
            .as_ref()
            .map(|m| MeshOutputWrapper { inner: m.clone() })
    }

    /// Get the chunk coordinate of the current mesh as [cx, cy, cz].
    #[wasm_bindgen(js_name = currentCoord)]
    pub fn current_coord(&self) -> JsValue {
        match self.current.as_ref().and_then(|m| m.chunk_coord) {
            Some((cx, cy, cz)) => {
                let arr = Array::new();
                arr.push(&JsValue::from(cx));
                arr.push(&JsValue::from(cy));
                arr.push(&JsValue::from(cz));
                arr.into()
            }
            None => JsValue::NULL,
        }
    }

    /// Total number of chunks in the iterator.
    #[wasm_bindgen(js_name = chunkCount)]
    pub fn chunk_count(&self) -> usize {
        self.inner.chunk_count()
    }
}

// ─── RawMeshExportWrapper ───────────────────────────────────────────────────

/// Result of raw mesh export for custom rendering.
#[wasm_bindgen]
pub struct RawMeshExportWrapper {
    inner: RawMeshExport,
}

#[wasm_bindgen]
impl RawMeshExportWrapper {
    /// Get vertex positions as a flat Float32Array.
    #[wasm_bindgen(js_name = positionsFlat)]
    pub fn positions_flat(&self) -> Vec<f32> {
        self.inner.positions_flat()
    }

    /// Get vertex normals as a flat Float32Array.
    #[wasm_bindgen(js_name = normalsFlat)]
    pub fn normals_flat(&self) -> Vec<f32> {
        self.inner.normals_flat()
    }

    /// Get texture coordinates as a flat Float32Array.
    #[wasm_bindgen(js_name = uvsFlat)]
    pub fn uvs_flat(&self) -> Vec<f32> {
        self.inner.uvs_flat()
    }

    /// Get vertex colors as a flat Float32Array.
    #[wasm_bindgen(js_name = colorsFlat)]
    pub fn colors_flat(&self) -> Vec<f32> {
        self.inner.colors_flat()
    }

    /// Get triangle indices.
    pub fn indices(&self) -> Vec<u32> {
        self.inner.indices().to_vec()
    }

    /// Get texture atlas RGBA pixel data.
    #[wasm_bindgen(js_name = textureRgba)]
    pub fn texture_rgba(&self) -> Vec<u8> {
        self.inner.texture_rgba().to_vec()
    }

    /// Get texture atlas width.
    #[wasm_bindgen(getter, js_name = textureWidth)]
    pub fn texture_width(&self) -> u32 {
        self.inner.texture_width()
    }

    /// Get texture atlas height.
    #[wasm_bindgen(getter, js_name = textureHeight)]
    pub fn texture_height(&self) -> u32 {
        self.inner.texture_height()
    }

    /// Get the vertex count.
    #[wasm_bindgen(getter, js_name = vertexCount)]
    pub fn vertex_count(&self) -> usize {
        self.inner.vertex_count()
    }

    /// Get the triangle count.
    #[wasm_bindgen(getter, js_name = triangleCount)]
    pub fn triangle_count(&self) -> usize {
        self.inner.triangle_count()
    }
}

// ─── SchematicWrapper meshing methods ───────────────────────────────────────

#[wasm_bindgen]
impl SchematicWrapper {
    /// Generate a single mesh for the entire schematic.
    ///
    /// Returns a [`MeshOutputWrapper`] with per-layer typed arrays, atlas, and GLB data.
    #[wasm_bindgen(js_name = toMesh)]
    pub fn to_mesh(
        &self,
        pack: &ResourcePackWrapper,
        config: &MeshConfigWrapper,
    ) -> Result<MeshOutputWrapper, JsValue> {
        self.0
            .to_mesh(&pack.inner, &config.inner)
            .map(|result| MeshOutputWrapper { inner: result })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Generate one mesh per region.
    #[wasm_bindgen(js_name = meshByRegion)]
    pub fn mesh_by_region(
        &self,
        pack: &ResourcePackWrapper,
        config: &MeshConfigWrapper,
    ) -> Result<MultiMeshResultWrapper, JsValue> {
        self.0
            .mesh_by_region(&pack.inner, &config.inner)
            .map(|result| MultiMeshResultWrapper { inner: result })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Generate one mesh per 16x16x16 chunk (eager, all at once).
    #[wasm_bindgen(js_name = meshByChunk)]
    pub fn mesh_by_chunk(
        &self,
        pack: &ResourcePackWrapper,
        config: &MeshConfigWrapper,
    ) -> Result<ChunkMeshResultWrapper, JsValue> {
        self.0
            .mesh_by_chunk(&pack.inner, &config.inner)
            .map(|result| ChunkMeshResultWrapper { inner: result })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Generate one mesh per chunk of the specified size (eager).
    #[wasm_bindgen(js_name = meshByChunkSize)]
    pub fn mesh_by_chunk_size(
        &self,
        pack: &ResourcePackWrapper,
        config: &MeshConfigWrapper,
        chunk_size: i32,
    ) -> Result<ChunkMeshResultWrapper, JsValue> {
        self.0
            .mesh_by_chunk_size(&pack.inner, &config.inner, chunk_size)
            .map(|result| ChunkMeshResultWrapper { inner: result })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Create a lazy chunk mesh iterator.
    ///
    /// Call `advance()` to step to the next chunk, then `current()` to get the
    /// mesh. Never loads the full world mesh into memory.
    #[wasm_bindgen(js_name = chunkMeshIterator)]
    pub fn chunk_mesh_iterator(
        &self,
        pack: &ResourcePackWrapper,
        config: &MeshConfigWrapper,
        chunk_size: i32,
    ) -> ChunkMeshIteratorWrapper {
        let iter = self.0.mesh_chunks(&pack.inner, &config.inner, chunk_size);
        ChunkMeshIteratorWrapper {
            inner: iter,
            current: None,
        }
    }

    /// Generate a USDZ mesh for the entire schematic.
    #[wasm_bindgen(js_name = toUsdz)]
    pub fn to_usdz(
        &self,
        pack: &ResourcePackWrapper,
        config: &MeshConfigWrapper,
    ) -> Result<MeshOutputWrapper, JsValue> {
        self.0
            .to_usdz(&pack.inner, &config.inner)
            .map(|result| MeshOutputWrapper { inner: result })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Generate raw mesh data for custom rendering pipelines.
    #[wasm_bindgen(js_name = toRawMesh)]
    pub fn to_raw_mesh(
        &self,
        pack: &ResourcePackWrapper,
        config: &MeshConfigWrapper,
    ) -> Result<RawMeshExportWrapper, JsValue> {
        self.0
            .to_raw_mesh(&pack.inner, &config.inner)
            .map(|result| RawMeshExportWrapper { inner: result })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Register a mesh exporter with the FormatManager, enabling save_as("mesh", ...).
    #[wasm_bindgen(js_name = registerMeshExporter)]
    pub fn register_mesh_exporter(&self, pack: &ResourcePackWrapper) -> Result<(), JsValue> {
        let mesh_exporter = crate::meshing::MeshExporter::new(
            ResourcePackSource::from_resource_pack(pack.inner.pack().clone()),
        );

        let manager = crate::formats::manager::get_manager();
        let mut manager = manager
            .lock()
            .map_err(|e| JsValue::from_str(&format!("Lock error: {}", e)))?;
        manager.register_exporter(mesh_exporter);
        Ok(())
    }
}
