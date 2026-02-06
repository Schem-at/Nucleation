//! Meshing WASM bindings
//!
//! Generate 3D meshes (GLB) from schematics using resource packs.

use js_sys::{Array, Object, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;

use crate::meshing::{
    ChunkMeshResult, MeshConfig, MeshResult, MultiMeshResult, ResourcePackSource,
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
}

/// Result of mesh generation.
#[wasm_bindgen]
pub struct MeshResultWrapper {
    inner: MeshResult,
}

#[wasm_bindgen]
impl MeshResultWrapper {
    /// Get the GLB binary data.
    #[wasm_bindgen(getter, js_name = glbData)]
    pub fn glb_data(&self) -> Uint8Array {
        Uint8Array::from(self.inner.glb_data.as_slice())
    }

    /// Get the vertex count.
    #[wasm_bindgen(getter, js_name = vertexCount)]
    pub fn vertex_count(&self) -> usize {
        self.inner.vertex_count
    }

    /// Get the triangle count.
    #[wasm_bindgen(getter, js_name = triangleCount)]
    pub fn triangle_count(&self) -> usize {
        self.inner.triangle_count
    }

    /// Check if the mesh has transparency.
    #[wasm_bindgen(getter, js_name = hasTransparency)]
    pub fn has_transparency(&self) -> bool {
        self.inner.has_transparency
    }

    /// Get the bounding box as [minX, minY, minZ, maxX, maxY, maxZ].
    #[wasm_bindgen(getter)]
    pub fn bounds(&self) -> Array {
        let arr = Array::new();
        for v in &self.inner.bounds {
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
            &JsValue::from(self.inner.vertex_count),
        )
        .unwrap();
        Reflect::set(
            &obj,
            &"triangleCount".into(),
            &JsValue::from(self.inner.triangle_count),
        )
        .unwrap();
        Reflect::set(
            &obj,
            &"hasTransparency".into(),
            &JsValue::from(self.inner.has_transparency),
        )
        .unwrap();
        obj
    }
}

/// Result of mesh generation for multiple regions.
#[wasm_bindgen]
pub struct MultiMeshResultWrapper {
    inner: MultiMeshResult,
}

#[wasm_bindgen]
impl MultiMeshResultWrapper {
    /// Get the region names.
    #[wasm_bindgen(js_name = getRegionNames)]
    pub fn get_region_names(&self) -> Array {
        let arr = Array::new();
        for name in self.inner.meshes.keys() {
            arr.push(&JsValue::from_str(name));
        }
        arr
    }

    /// Get the mesh for a specific region.
    #[wasm_bindgen(js_name = getMesh)]
    pub fn get_mesh(&self, region_name: &str) -> Option<MeshResultWrapper> {
        self.inner
            .meshes
            .get(region_name)
            .map(|mesh| MeshResultWrapper {
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

    /// Get the number of meshes.
    #[wasm_bindgen(getter, js_name = meshCount)]
    pub fn mesh_count(&self) -> usize {
        self.inner.meshes.len()
    }
}

/// Result of chunk-based mesh generation.
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
    pub fn get_mesh(&self, cx: i32, cy: i32, cz: i32) -> Option<MeshResultWrapper> {
        self.inner
            .meshes
            .get(&(cx, cy, cz))
            .map(|mesh| MeshResultWrapper {
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

// Add meshing methods to SchematicWrapper
#[wasm_bindgen]
impl SchematicWrapper {
    /// Generate a single mesh for the entire schematic.
    #[wasm_bindgen(js_name = toMesh)]
    pub fn to_mesh(
        &self,
        pack: &ResourcePackWrapper,
        config: &MeshConfigWrapper,
    ) -> Result<MeshResultWrapper, JsValue> {
        self.0
            .to_mesh(&pack.inner, &config.inner)
            .map(|result| MeshResultWrapper { inner: result })
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

    /// Generate one mesh per 16x16x16 chunk.
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

    /// Generate one mesh per chunk of the specified size.
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
}
