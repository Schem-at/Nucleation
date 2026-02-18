//! Meshing Python bindings
//!
//! Generate 3D meshes (GLB/USDZ) from schematics using resource packs.
//!
//! Exposes [`PyMeshOutput`] (aliased as `PyMeshResult` for backward compat)
//! with per-layer data, atlas access, `.save()`, and export helpers.

use bytemuck;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::collections::HashMap;

use crate::meshing::{
    ChunkMeshResult, MeshConfig, MeshOutput, RawMeshExport, ResourcePackSource,
};

/// Wrapper for a Minecraft resource pack.
#[pyclass(name = "ResourcePack")]
pub struct PyResourcePack {
    pub(crate) inner: ResourcePackSource,
}

#[pymethods]
impl PyResourcePack {
    /// Load a resource pack from a file path (ZIP or directory).
    #[staticmethod]
    pub fn from_file(path: &str) -> PyResult<Self> {
        let inner = ResourcePackSource::from_file(path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Load a resource pack from bytes (ZIP file contents).
    #[staticmethod]
    pub fn from_bytes(data: &[u8]) -> PyResult<Self> {
        let inner = ResourcePackSource::from_bytes(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Get the number of blockstates in the resource pack.
    #[getter]
    pub fn blockstate_count(&self) -> usize {
        self.inner.stats().blockstate_count
    }

    /// Get the number of models in the resource pack.
    #[getter]
    pub fn model_count(&self) -> usize {
        self.inner.stats().model_count
    }

    /// Get the number of textures in the resource pack.
    #[getter]
    pub fn texture_count(&self) -> usize {
        self.inner.stats().texture_count
    }

    /// Get the namespaces in the resource pack.
    #[getter]
    pub fn namespaces(&self) -> Vec<String> {
        self.inner.stats().namespaces
    }

    /// Get statistics about the resource pack as a dictionary.
    pub fn stats(&self) -> HashMap<String, PyObject> {
        let stats = self.inner.stats();
        Python::with_gil(|py| {
            let mut result = HashMap::new();
            result.insert(
                "blockstate_count".to_string(),
                stats
                    .blockstate_count
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            result.insert(
                "model_count".to_string(),
                stats
                    .model_count
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            result.insert(
                "texture_count".to_string(),
                stats
                    .texture_count
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            result.insert(
                "namespaces".to_string(),
                stats
                    .namespaces
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            result
        })
    }

    /// List all blockstate names as "namespace:block_id".
    pub fn list_blockstates(&self) -> Vec<String> {
        self.inner.list_blockstates()
    }

    /// List all model names as "namespace:model_path".
    pub fn list_models(&self) -> Vec<String> {
        self.inner.list_models()
    }

    /// List all texture names as "namespace:texture_path".
    pub fn list_textures(&self) -> Vec<String> {
        self.inner.list_textures()
    }

    /// Get a blockstate definition as a JSON string. Returns None if not found.
    pub fn get_blockstate_json(&self, name: &str) -> Option<String> {
        self.inner.get_blockstate_json(name)
    }

    /// Get a block model as a JSON string. Returns None if not found.
    pub fn get_model_json(&self, name: &str) -> Option<String> {
        self.inner.get_model_json(name)
    }

    /// Get texture info as a dict with width, height, is_animated, frame_count.
    /// Returns None if not found.
    pub fn get_texture_info(&self, name: &str) -> Option<HashMap<String, PyObject>> {
        let (w, h, animated, frames) = self.inner.get_texture_info(name)?;
        Python::with_gil(|py| {
            let mut map = HashMap::new();
            map.insert(
                "width".to_string(),
                w.into_pyobject(py).unwrap().into_any().unbind(),
            );
            map.insert(
                "height".to_string(),
                h.into_pyobject(py).unwrap().into_any().unbind(),
            );
            map.insert(
                "is_animated".to_string(),
                (animated as u32)
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            map.insert(
                "frame_count".to_string(),
                frames.into_pyobject(py).unwrap().into_any().unbind(),
            );
            Some(map)
        })
    }

    /// Get raw RGBA8 pixel data for a texture. Returns None if not found.
    pub fn get_texture_pixels<'py>(
        &self,
        py: Python<'py>,
        name: &str,
    ) -> Option<Bound<'py, PyBytes>> {
        let pixels = self.inner.get_texture_pixels(name)?;
        Some(PyBytes::new(py, pixels))
    }

    /// Add a blockstate definition from a JSON string.
    pub fn add_blockstate_json(&mut self, name: &str, json: &str) -> PyResult<()> {
        self.inner
            .add_blockstate_json(name, json)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Add a block model from a JSON string.
    pub fn add_model_json(&mut self, name: &str, json: &str) -> PyResult<()> {
        self.inner
            .add_model_json(name, json)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Add a texture from raw RGBA8 pixel data.
    pub fn add_texture(
        &mut self,
        name: &str,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> PyResult<()> {
        self.inner
            .add_texture(name, width, height, pixels.to_vec())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn __repr__(&self) -> String {
        let stats = self.inner.stats();
        format!(
            "<ResourcePack blockstates={} models={} textures={}>",
            stats.blockstate_count, stats.model_count, stats.texture_count
        )
    }
}

/// Configuration for mesh generation.
#[pyclass(name = "MeshConfig")]
#[derive(Clone)]
pub struct PyMeshConfig {
    pub(crate) inner: MeshConfig,
}

#[pymethods]
impl PyMeshConfig {
    /// Create a new MeshConfig with default settings.
    #[new]
    #[pyo3(signature = (cull_hidden_faces=true, ambient_occlusion=true, ao_intensity=0.4, biome=None, atlas_max_size=4096, cull_occluded_blocks=true, greedy_meshing=false))]
    pub fn new(
        cull_hidden_faces: bool,
        ambient_occlusion: bool,
        ao_intensity: f32,
        biome: Option<String>,
        atlas_max_size: u32,
        cull_occluded_blocks: bool,
        greedy_meshing: bool,
    ) -> Self {
        Self {
            inner: MeshConfig {
                cull_hidden_faces,
                ambient_occlusion,
                ao_intensity,
                biome,
                atlas_max_size,
                cull_occluded_blocks,
                greedy_meshing,
            },
        }
    }

    /// Enable or disable face culling between adjacent blocks.
    #[getter]
    pub fn cull_hidden_faces(&self) -> bool {
        self.inner.cull_hidden_faces
    }

    #[setter]
    pub fn set_cull_hidden_faces(&mut self, value: bool) {
        self.inner.cull_hidden_faces = value;
    }

    /// Enable or disable ambient occlusion.
    #[getter]
    pub fn ambient_occlusion(&self) -> bool {
        self.inner.ambient_occlusion
    }

    #[setter]
    pub fn set_ambient_occlusion(&mut self, value: bool) {
        self.inner.ambient_occlusion = value;
    }

    /// Get the AO intensity.
    #[getter]
    pub fn ao_intensity(&self) -> f32 {
        self.inner.ao_intensity
    }

    #[setter]
    pub fn set_ao_intensity(&mut self, value: f32) {
        self.inner.ao_intensity = value;
    }

    /// Get the biome name.
    #[getter]
    pub fn biome(&self) -> Option<String> {
        self.inner.biome.clone()
    }

    #[setter]
    pub fn set_biome(&mut self, value: Option<String>) {
        self.inner.biome = value;
    }

    /// Get the maximum atlas size.
    #[getter]
    pub fn atlas_max_size(&self) -> u32 {
        self.inner.atlas_max_size
    }

    #[setter]
    pub fn set_atlas_max_size(&mut self, value: u32) {
        self.inner.atlas_max_size = value;
    }

    /// Skip blocks fully hidden by opaque neighbors on all 6 sides.
    #[getter]
    pub fn cull_occluded_blocks(&self) -> bool {
        self.inner.cull_occluded_blocks
    }

    #[setter]
    pub fn set_cull_occluded_blocks(&mut self, value: bool) {
        self.inner.cull_occluded_blocks = value;
    }

    /// Merge adjacent coplanar faces into larger quads (reduces triangle count).
    #[getter]
    pub fn greedy_meshing(&self) -> bool {
        self.inner.greedy_meshing
    }

    #[setter]
    pub fn set_greedy_meshing(&mut self, value: bool) {
        self.inner.greedy_meshing = value;
    }

    fn __repr__(&self) -> String {
        format!(
            "<MeshConfig cull={} ao={} greedy={} biome={:?}>",
            self.inner.cull_hidden_faces,
            self.inner.ambient_occlusion,
            self.inner.greedy_meshing,
            self.inner.biome
        )
    }
}

// ─── PyMeshResult (backed by MeshOutput) ────────────────────────────────────

/// Result of mesh generation.
///
/// Wraps a [`MeshOutput`] with per-layer data, atlas, GLB/USDZ bytes, and
/// a convenience `.save(path)` method.
#[pyclass(name = "MeshResult")]
#[derive(Clone)]
pub struct PyMeshResult {
    pub(crate) inner: MeshOutput,
}

#[pymethods]
impl PyMeshResult {
    /// Get the GLB binary data.
    #[getter]
    pub fn glb_data<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let data = self.inner.to_glb()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(PyBytes::new(py, &data))
    }

    /// Get the vertex count.
    #[getter]
    pub fn vertex_count(&self) -> usize {
        self.inner.total_vertices()
    }

    /// Get the triangle count.
    #[getter]
    pub fn triangle_count(&self) -> usize {
        self.inner.total_triangles()
    }

    /// Check if the mesh has transparency.
    #[getter]
    pub fn has_transparency(&self) -> bool {
        self.inner.has_transparency()
    }

    /// Get the bounding box as [minX, minY, minZ, maxX, maxY, maxZ].
    #[getter]
    pub fn bounds(&self) -> Vec<f32> {
        let mut b = Vec::with_capacity(6);
        b.extend_from_slice(&self.inner.bounds.min);
        b.extend_from_slice(&self.inner.bounds.max);
        b
    }

    /// Chunk coordinate [cx, cy, cz] or None if not a chunk mesh.
    #[getter]
    pub fn chunk_coord(&self) -> Option<Vec<i32>> {
        self.inner.chunk_coord.map(|(cx, cy, cz)| vec![cx, cy, cz])
    }

    /// LOD level (0 = full detail).
    #[getter]
    pub fn lod_level(&self) -> u8 {
        self.inner.lod_level
    }

    /// Total vertices across all layers.
    #[getter]
    pub fn total_vertices(&self) -> usize {
        self.inner.total_vertices()
    }

    /// Total triangles across all layers.
    #[getter]
    pub fn total_triangles(&self) -> usize {
        self.inner.total_triangles()
    }

    /// Whether all layers are empty.
    #[getter]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Atlas width in pixels.
    #[getter]
    pub fn atlas_width(&self) -> u32 {
        self.inner.atlas.width
    }

    /// Atlas height in pixels.
    #[getter]
    pub fn atlas_height(&self) -> u32 {
        self.inner.atlas.height
    }

    /// Atlas RGBA pixel data.
    pub fn atlas_rgba<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.atlas.pixels)
    }

    // ── Per-layer accessors ─────────────────────────────────────────────

    /// Opaque layer positions as flat list of floats.
    pub fn opaque_positions(&self) -> Vec<f32> {
        bytemuck::cast_slice(&self.inner.opaque.positions).to_vec()
    }

    /// Opaque layer normals as flat list of floats.
    pub fn opaque_normals(&self) -> Vec<f32> {
        bytemuck::cast_slice(&self.inner.opaque.normals).to_vec()
    }

    /// Opaque layer UVs as flat list of floats.
    pub fn opaque_uvs(&self) -> Vec<f32> {
        bytemuck::cast_slice(&self.inner.opaque.uvs).to_vec()
    }

    /// Opaque layer colors as flat list of floats.
    pub fn opaque_colors(&self) -> Vec<f32> {
        bytemuck::cast_slice(&self.inner.opaque.colors).to_vec()
    }

    /// Opaque layer indices.
    pub fn opaque_indices(&self) -> Vec<u32> {
        self.inner.opaque.indices.clone()
    }

    /// Cutout layer positions as flat list of floats.
    pub fn cutout_positions(&self) -> Vec<f32> {
        bytemuck::cast_slice(&self.inner.cutout.positions).to_vec()
    }

    /// Cutout layer indices.
    pub fn cutout_indices(&self) -> Vec<u32> {
        self.inner.cutout.indices.clone()
    }

    /// Transparent layer positions as flat list of floats.
    pub fn transparent_positions(&self) -> Vec<f32> {
        bytemuck::cast_slice(&self.inner.transparent.positions).to_vec()
    }

    /// Transparent layer indices.
    pub fn transparent_indices(&self) -> Vec<u32> {
        self.inner.transparent.indices.clone()
    }

    // ── Export helpers ───────────────────────────────────────────────────

    /// Save the GLB data to a file.
    pub fn save(&self, path: &str) -> PyResult<()> {
        let glb = self.inner.to_glb()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        std::fs::write(path, &glb)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))
    }

    /// Get USDZ data.
    pub fn usdz_data<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let data = self.inner.to_usdz()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(PyBytes::new(py, &data))
    }

    fn __repr__(&self) -> String {
        format!(
            "<MeshResult vertices={} triangles={} transparent={} layers=[opaque={}, cutout={}, transparent={}]>",
            self.inner.total_vertices(),
            self.inner.total_triangles(),
            self.inner.has_transparency(),
            self.inner.opaque.vertex_count(),
            self.inner.cutout.vertex_count(),
            self.inner.transparent.vertex_count(),
        )
    }
}

// ─── PyMultiMeshResult ──────────────────────────────────────────────────────

/// Result of mesh generation for multiple regions.
///
/// Wraps a `HashMap<String, MeshOutput>`.
#[pyclass(name = "MultiMeshResult")]
pub struct PyMultiMeshResult {
    pub(crate) inner: HashMap<String, MeshOutput>,
}

#[pymethods]
impl PyMultiMeshResult {
    /// Get the region names.
    #[getter]
    pub fn region_names(&self) -> Vec<String> {
        self.inner.keys().cloned().collect()
    }

    /// Get the mesh for a specific region.
    pub fn get_mesh(&self, region_name: &str) -> Option<PyMeshResult> {
        self.inner.get(region_name).map(|mesh| PyMeshResult {
            inner: mesh.clone(),
        })
    }

    /// Get all meshes as a dictionary.
    pub fn get_all_meshes(&self) -> HashMap<String, PyMeshResult> {
        self.inner
            .iter()
            .map(|(k, v)| (k.clone(), PyMeshResult { inner: v.clone() }))
            .collect()
    }

    /// Get the total vertex count.
    #[getter]
    pub fn total_vertex_count(&self) -> usize {
        self.inner.values().map(|m| m.total_vertices()).sum()
    }

    /// Get the total triangle count.
    #[getter]
    pub fn total_triangle_count(&self) -> usize {
        self.inner.values().map(|m| m.total_triangles()).sum()
    }

    /// Get the number of meshes.
    #[getter]
    pub fn mesh_count(&self) -> usize {
        self.inner.len()
    }

    /// Save all meshes to files with the pattern "{prefix}_{region_name}.glb".
    pub fn save_all(&self, prefix: &str) -> PyResult<Vec<String>> {
        let mut paths = Vec::new();
        for (name, mesh) in &self.inner {
            let path = format!("{}_{}.glb", prefix, name);
            let glb = mesh.to_glb()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            std::fs::write(&path, &glb)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
            paths.push(path);
        }
        Ok(paths)
    }

    fn __repr__(&self) -> String {
        let total_v: usize = self.inner.values().map(|m| m.total_vertices()).sum();
        let total_t: usize = self.inner.values().map(|m| m.total_triangles()).sum();
        format!(
            "<MultiMeshResult regions={} total_vertices={} total_triangles={}>",
            self.inner.len(),
            total_v,
            total_t
        )
    }
}

// ─── PyChunkMeshResult ──────────────────────────────────────────────────────

/// Result of chunk-based mesh generation (eager).
#[pyclass(name = "ChunkMeshResult")]
pub struct PyChunkMeshResult {
    pub(crate) inner: ChunkMeshResult,
}

#[pymethods]
impl PyChunkMeshResult {
    /// Get the chunk coordinates as a list of (x, y, z) tuples.
    #[getter]
    pub fn chunk_coordinates(&self) -> Vec<(i32, i32, i32)> {
        self.inner.meshes.keys().cloned().collect()
    }

    /// Get the mesh for a specific chunk.
    pub fn get_mesh(&self, cx: i32, cy: i32, cz: i32) -> Option<PyMeshResult> {
        self.inner
            .meshes
            .get(&(cx, cy, cz))
            .map(|mesh| PyMeshResult {
                inner: mesh.clone(),
            })
    }

    /// Get all meshes as a dictionary with (x, y, z) tuple keys.
    pub fn get_all_meshes(&self) -> HashMap<(i32, i32, i32), PyMeshResult> {
        self.inner
            .meshes
            .iter()
            .map(|(k, v)| (*k, PyMeshResult { inner: v.clone() }))
            .collect()
    }

    /// Get the total vertex count.
    #[getter]
    pub fn total_vertex_count(&self) -> usize {
        self.inner.total_vertex_count
    }

    /// Get the total triangle count.
    #[getter]
    pub fn total_triangle_count(&self) -> usize {
        self.inner.total_triangle_count
    }

    /// Get the number of chunks.
    #[getter]
    pub fn chunk_count(&self) -> usize {
        self.inner.meshes.len()
    }

    /// Save all meshes to files with the pattern "{prefix}_{x}_{y}_{z}.glb".
    pub fn save_all(&self, prefix: &str) -> PyResult<Vec<String>> {
        let mut paths = Vec::new();
        for ((cx, cy, cz), mesh) in &self.inner.meshes {
            let path = format!("{}_{}_{}_{}_.glb", prefix, cx, cy, cz);
            let glb = mesh.to_glb()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            std::fs::write(&path, &glb)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
            paths.push(path);
        }
        Ok(paths)
    }

    fn __repr__(&self) -> String {
        format!(
            "<ChunkMeshResult chunks={} total_vertices={} total_triangles={}>",
            self.inner.meshes.len(),
            self.inner.total_vertex_count,
            self.inner.total_triangle_count
        )
    }
}

// ─── PyRawMeshExport ────────────────────────────────────────────────────────

/// Result of raw mesh export for custom rendering.
#[pyclass(name = "RawMeshExport")]
pub struct PyRawMeshExport {
    pub(crate) inner: RawMeshExport,
}

#[pymethods]
impl PyRawMeshExport {
    /// Get vertex positions as a flat list of floats (x, y, z, x, y, z, ...).
    pub fn positions_flat(&self) -> Vec<f32> {
        self.inner.positions_flat()
    }

    /// Get vertex normals as a flat list of floats.
    pub fn normals_flat(&self) -> Vec<f32> {
        self.inner.normals_flat()
    }

    /// Get texture coordinates as a flat list of floats (u, v, u, v, ...).
    pub fn uvs_flat(&self) -> Vec<f32> {
        self.inner.uvs_flat()
    }

    /// Get vertex colors as a flat list of floats (r, g, b, a, ...).
    pub fn colors_flat(&self) -> Vec<f32> {
        self.inner.colors_flat()
    }

    /// Get triangle indices.
    pub fn indices(&self) -> Vec<u32> {
        self.inner.indices().to_vec()
    }

    /// Get texture atlas RGBA pixel data.
    pub fn texture_rgba<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, self.inner.texture_rgba())
    }

    /// Get texture atlas width.
    #[getter]
    pub fn texture_width(&self) -> u32 {
        self.inner.texture_width()
    }

    /// Get texture atlas height.
    #[getter]
    pub fn texture_height(&self) -> u32 {
        self.inner.texture_height()
    }

    /// Get the vertex count.
    #[getter]
    pub fn vertex_count(&self) -> usize {
        self.inner.vertex_count()
    }

    /// Get the triangle count.
    #[getter]
    pub fn triangle_count(&self) -> usize {
        self.inner.triangle_count()
    }

    fn __repr__(&self) -> String {
        format!(
            "<RawMeshExport vertices={} triangles={} atlas={}x{}>",
            self.inner.vertex_count(),
            self.inner.triangle_count(),
            self.inner.texture_width(),
            self.inner.texture_height()
        )
    }
}

// Meshing methods for PySchematic are in src/python/schematic.rs
// (behind #[cfg(feature = "meshing")] inside the main #[pymethods] block)
