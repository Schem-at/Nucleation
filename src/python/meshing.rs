//! Meshing Python bindings
//!
//! Generate 3D meshes (GLB) from schematics using resource packs.

use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::collections::HashMap;

use crate::meshing::{
    ChunkMeshResult, MeshConfig, MeshResult, MultiMeshResult, ResourcePackSource,
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
    #[pyo3(signature = (cull_hidden_faces=true, ambient_occlusion=true, ao_intensity=0.4, biome=None, atlas_max_size=4096))]
    pub fn new(
        cull_hidden_faces: bool,
        ambient_occlusion: bool,
        ao_intensity: f32,
        biome: Option<String>,
        atlas_max_size: u32,
    ) -> Self {
        Self {
            inner: MeshConfig {
                cull_hidden_faces,
                ambient_occlusion,
                ao_intensity,
                biome,
                atlas_max_size,
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

    fn __repr__(&self) -> String {
        format!(
            "<MeshConfig cull={} ao={} biome={:?}>",
            self.inner.cull_hidden_faces, self.inner.ambient_occlusion, self.inner.biome
        )
    }
}

/// Result of mesh generation.
#[pyclass(name = "MeshResult")]
#[derive(Clone)]
pub struct PyMeshResult {
    pub(crate) inner: MeshResult,
}

#[pymethods]
impl PyMeshResult {
    /// Get the GLB binary data.
    #[getter]
    pub fn glb_data<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.glb_data)
    }

    /// Get the vertex count.
    #[getter]
    pub fn vertex_count(&self) -> usize {
        self.inner.vertex_count
    }

    /// Get the triangle count.
    #[getter]
    pub fn triangle_count(&self) -> usize {
        self.inner.triangle_count
    }

    /// Check if the mesh has transparency.
    #[getter]
    pub fn has_transparency(&self) -> bool {
        self.inner.has_transparency
    }

    /// Get the bounding box as [minX, minY, minZ, maxX, maxY, maxZ].
    #[getter]
    pub fn bounds(&self) -> Vec<f32> {
        self.inner.bounds.to_vec()
    }

    /// Save the GLB data to a file.
    pub fn save(&self, path: &str) -> PyResult<()> {
        std::fs::write(path, &self.inner.glb_data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))
    }

    fn __repr__(&self) -> String {
        format!(
            "<MeshResult vertices={} triangles={} transparent={}>",
            self.inner.vertex_count, self.inner.triangle_count, self.inner.has_transparency
        )
    }
}

/// Result of mesh generation for multiple regions.
#[pyclass(name = "MultiMeshResult")]
pub struct PyMultiMeshResult {
    pub(crate) inner: MultiMeshResult,
}

#[pymethods]
impl PyMultiMeshResult {
    /// Get the region names.
    #[getter]
    pub fn region_names(&self) -> Vec<String> {
        self.inner.meshes.keys().cloned().collect()
    }

    /// Get the mesh for a specific region.
    pub fn get_mesh(&self, region_name: &str) -> Option<PyMeshResult> {
        self.inner.meshes.get(region_name).map(|mesh| PyMeshResult {
            inner: mesh.clone(),
        })
    }

    /// Get all meshes as a dictionary.
    pub fn get_all_meshes(&self) -> HashMap<String, PyMeshResult> {
        self.inner
            .meshes
            .iter()
            .map(|(k, v)| (k.clone(), PyMeshResult { inner: v.clone() }))
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

    /// Get the number of meshes.
    #[getter]
    pub fn mesh_count(&self) -> usize {
        self.inner.meshes.len()
    }

    /// Save all meshes to files with the pattern "{prefix}_{region_name}.glb".
    pub fn save_all(&self, prefix: &str) -> PyResult<Vec<String>> {
        let mut paths = Vec::new();
        for (name, mesh) in &self.inner.meshes {
            let path = format!("{}_{}.glb", prefix, name);
            std::fs::write(&path, &mesh.glb_data)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
            paths.push(path);
        }
        Ok(paths)
    }

    fn __repr__(&self) -> String {
        format!(
            "<MultiMeshResult regions={} total_vertices={} total_triangles={}>",
            self.inner.meshes.len(),
            self.inner.total_vertex_count,
            self.inner.total_triangle_count
        )
    }
}

/// Result of chunk-based mesh generation.
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
            std::fs::write(&path, &mesh.glb_data)
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

// Meshing methods for PySchematic are in src/python/schematic.rs
// (behind #[cfg(feature = "meshing")] inside the main #[pymethods] block)
