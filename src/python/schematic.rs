//! Schematic Python bindings
//!
//! Core schematic operations: loading, saving, block manipulation, iteration.

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::{
    block_position::BlockPosition,
    bounding_box::BoundingBox,
    definition_region::DefinitionRegion,
    formats::{litematic, manager::get_manager, mcstructure, schematic, world},
    print_utils::{format_json_schematic, format_schematic},
    universal_schematic::ChunkLoadingStrategy,
    utils::{NbtMap, NbtValue},
    BlockState, UniversalSchematic,
};

use super::definition_region::PyDefinitionRegion;

use bytemuck;

#[cfg(feature = "simulation")]
use super::typed_executor::PyTypedCircuitExecutor;
#[cfg(feature = "simulation")]
use super::PyMchprsWorld;
#[cfg(feature = "simulation")]
use crate::simulation::typed_executor::IoType;
#[cfg(feature = "simulation")]
use crate::simulation::CircuitBuilder;
#[cfg(feature = "simulation")]
use crate::simulation::MchprsWorld;

use crate::building::{BuildingTool, Cuboid, SolidBrush, Sphere};

#[pyclass(name = "BlockState")]
#[derive(Clone)]
pub struct PyBlockState {
    pub(crate) inner: BlockState,
}

#[pymethods]
impl PyBlockState {
    #[new]
    fn new(name: String) -> Self {
        Self {
            inner: BlockState::new(name),
        }
    }

    pub fn with_property(&self, key: String, value: String) -> Self {
        let new_inner = self.inner.clone().with_property(key, value);
        Self { inner: new_inner }
    }

    #[getter]
    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    pub fn properties(&self) -> HashMap<String, String> {
        self.inner.properties.clone()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("<BlockState '{}'>", self.inner.to_string())
    }
}

#[pyclass(name = "Schematic")]
pub struct PySchematic {
    pub(crate) inner: UniversalSchematic,
    // ── Cached fields for fast-path Python bindings ──
    last_block_name: String,
    last_palette_index: usize,
    default_region_initialized: bool,
}

impl PySchematic {
    /// Create a PySchematic from an existing UniversalSchematic.
    /// Used by builder, simulation, and other internal constructors.
    pub(crate) fn from_inner(inner: UniversalSchematic) -> Self {
        Self {
            inner,
            last_block_name: String::new(),
            last_palette_index: 0,
            default_region_initialized: false,
        }
    }

    /// Ensure the default region is properly initialized for the given coordinate.
    /// Only does real work on the very first call.
    #[inline(always)]
    fn ensure_default_region(&mut self, x: i32, y: i32, z: i32) {
        if !self.default_region_initialized {
            if self.inner.default_region.size == (1, 1, 1) && self.inner.default_region.is_empty() {
                self.inner.default_region = crate::region::Region::new(
                    self.inner.default_region_name.clone(),
                    (x, y, z),
                    (1, 1, 1),
                );
            }
            self.default_region_initialized = true;
        }
    }
}

#[pymethods]
impl PySchematic {
    #[new]
    fn new(name: Option<String>) -> Self {
        Self::from_inner(UniversalSchematic::new(
            name.unwrap_or_else(|| "Default".to_string()),
        ))
    }

    // test method to check if the Python class is working
    pub fn test(&self) -> String {
        "Schematic class is working!".to_string()
    }

    pub fn from_data(&mut self, data: &[u8]) -> PyResult<()> {
        let manager = get_manager();
        let manager = manager
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        match manager.read(data) {
            Ok(schematic) => {
                self.inner = schematic;
                self.last_block_name.clear();
                self.last_palette_index = 0;
                self.default_region_initialized = true; // loaded schematics are already initialized
                Ok(())
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                e.to_string(),
            )),
        }
    }

    pub fn fill_cuboid(
        &mut self,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
        block_state: &str,
    ) {
        // For complex block strings (with properties/NBT), fall back to BuildingTool
        if block_state.contains('[') || block_state.ends_with('}') {
            let block = BlockState::new(block_state.to_string());
            let shape = Cuboid::new((min_x, min_y, min_z), (max_x, max_y, max_z));
            let brush = SolidBrush::new(block);
            let mut tool = BuildingTool::new(&mut self.inner);
            tool.fill(&shape, &brush);
            return;
        }

        // Fast path: direct array fill for simple block names
        self.ensure_default_region(min_x, min_y, min_z);

        let region = &mut self.inner.default_region;

        // Pre-expand to fit both corners at once
        region.ensure_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));

        let palette_index = region.get_or_insert_palette_by_name(block_state);
        region.fill_uniform((min_x, min_y, min_z), (max_x, max_y, max_z), palette_index);
    }

    pub fn fill_sphere(&mut self, cx: i32, cy: i32, cz: i32, radius: f64, block_state: &str) {
        let block = BlockState::new(block_state.to_string());
        let shape = Sphere::new((cx, cy, cz), radius);
        let brush = SolidBrush::new(block);

        let mut tool = BuildingTool::new(&mut self.inner);
        tool.fill(&shape, &brush);
    }

    #[staticmethod]
    pub fn get_supported_import_formats() -> PyResult<Vec<String>> {
        let manager = get_manager();
        let manager = manager.lock().map_err(|_| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Failed to acquire format manager lock",
            )
        })?;
        Ok(manager.list_importers())
    }

    #[staticmethod]
    pub fn get_supported_export_formats() -> PyResult<Vec<String>> {
        let manager = get_manager();
        let manager = manager.lock().map_err(|_| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Failed to acquire format manager lock",
            )
        })?;
        Ok(manager.list_exporters())
    }

    #[staticmethod]
    pub fn get_format_versions(format: &str) -> PyResult<Vec<String>> {
        let manager = get_manager();
        let manager = manager.lock().map_err(|_| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Failed to acquire format manager lock",
            )
        })?;
        Ok(manager.get_exporter_versions(format).unwrap_or_default())
    }

    #[staticmethod]
    pub fn get_default_format_version(format: &str) -> PyResult<Option<String>> {
        let manager = get_manager();
        let manager = manager.lock().map_err(|_| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Failed to acquire format manager lock",
            )
        })?;
        Ok(manager.get_exporter_default_version(format))
    }

    pub fn from_litematic(&mut self, data: &[u8]) -> PyResult<()> {
        self.inner = litematic::from_litematic(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        self.last_block_name.clear();
        self.default_region_initialized = true;
        Ok(())
    }

    pub fn to_litematic(&self, py: Python<'_>) -> PyResult<PyObject> {
        let bytes = litematic::to_litematic(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(PyBytes::new(py, &bytes).into())
    }

    pub fn from_schematic(&mut self, data: &[u8]) -> PyResult<()> {
        self.inner = schematic::from_schematic(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        self.last_block_name.clear();
        self.default_region_initialized = true;
        Ok(())
    }

    pub fn to_schematic(&self, py: Python<'_>) -> PyResult<PyObject> {
        let bytes = schematic::to_schematic(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(PyBytes::new(py, &bytes).into())
    }

    /// Load schematic from McStructure (Bedrock) format bytes.
    pub fn from_mcstructure(&mut self, data: &[u8]) -> PyResult<()> {
        self.inner = mcstructure::from_mcstructure(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        self.last_block_name.clear();
        self.default_region_initialized = true;
        Ok(())
    }

    /// Export schematic to McStructure (Bedrock) format.
    pub fn to_mcstructure(&self, py: Python<'_>) -> PyResult<PyObject> {
        let bytes = mcstructure::to_mcstructure(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(PyBytes::new(py, &bytes).into())
    }

    /// Import from a single MCA region file.
    pub fn from_mca(&mut self, data: &[u8]) -> PyResult<()> {
        self.inner = world::from_mca(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        self.last_block_name.clear();
        self.default_region_initialized = true;
        Ok(())
    }

    /// Import from a single MCA region file with coordinate bounds.
    pub fn from_mca_bounded(
        &mut self,
        data: &[u8],
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> PyResult<()> {
        self.inner = world::from_mca_bounded(data, min_x, min_y, min_z, max_x, max_y, max_z)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        self.last_block_name.clear();
        self.default_region_initialized = true;
        Ok(())
    }

    /// Import from a zipped Minecraft world folder.
    pub fn from_world_zip(&mut self, data: &[u8]) -> PyResult<()> {
        self.inner = world::from_world_zip(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        self.last_block_name.clear();
        self.default_region_initialized = true;
        Ok(())
    }

    /// Import from a zipped world folder with coordinate bounds.
    pub fn from_world_zip_bounded(
        &mut self,
        data: &[u8],
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> PyResult<()> {
        self.inner = world::from_world_zip_bounded(data, min_x, min_y, min_z, max_x, max_y, max_z)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        self.last_block_name.clear();
        self.default_region_initialized = true;
        Ok(())
    }

    /// Import from a Minecraft world directory path.
    pub fn from_world_directory(&mut self, path: &str) -> PyResult<()> {
        self.inner = world::from_world_directory(std::path::Path::new(path))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        self.last_block_name.clear();
        self.default_region_initialized = true;
        Ok(())
    }

    /// Import from a world directory with coordinate bounds.
    pub fn from_world_directory_bounded(
        &mut self,
        path: &str,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> PyResult<()> {
        self.inner = world::from_world_directory_bounded(
            std::path::Path::new(path),
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        )
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        self.last_block_name.clear();
        self.default_region_initialized = true;
        Ok(())
    }

    /// Export schematic as a Minecraft world. Returns dict[str, bytes].
    pub fn to_world(&self, py: Python<'_>, options_json: Option<&str>) -> PyResult<PyObject> {
        let options = match options_json {
            Some(json) => Some(
                serde_json::from_str::<world::WorldExportOptions>(json)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?,
            ),
            None => None,
        };
        let files = world::to_world(&self.inner, options)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

        let dict = PyDict::new(py);
        for (path, data) in &files {
            dict.set_item(path, PyBytes::new(py, data))?;
        }
        Ok(dict.into())
    }

    /// Export schematic as a zipped Minecraft world. Returns bytes.
    pub fn to_world_zip(&self, py: Python<'_>, options_json: Option<&str>) -> PyResult<PyObject> {
        let options = match options_json {
            Some(json) => Some(
                serde_json::from_str::<world::WorldExportOptions>(json)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?,
            ),
            None => None,
        };
        let zip_bytes = world::to_world_zip(&self.inner, options)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(PyBytes::new(py, &zip_bytes).into())
    }

    /// Export schematic as a Minecraft world and write files to disk.
    pub fn save_world(&self, directory: &str, options_json: Option<&str>) -> PyResult<()> {
        let options = match options_json {
            Some(json) => Some(
                serde_json::from_str::<world::WorldExportOptions>(json)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?,
            ),
            None => None,
        };
        world::save_world(&self.inner, std::path::Path::new(directory), options)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block_name: &str) -> bool {
        // Fast path: simple block names (no properties/NBT) go directly to Region
        if block_name.contains('[') || block_name.ends_with('}') {
            // Complex block string — fall back to full parsing path
            return self.inner.set_block_str(x, y, z, block_name);
        }

        // One-time default region initialization
        self.ensure_default_region(x, y, z);

        let region = &mut self.inner.default_region;

        // Expand region if coordinate is outside bounds
        if !region.is_in_region(x, y, z) {
            region.expand_to_fit(x, y, z);
        }

        // Check block name cache — avoid palette lookup on repeated block names
        let palette_index = if block_name == self.last_block_name {
            self.last_palette_index
        } else {
            let idx = region.get_or_insert_palette_by_name(block_name);
            self.last_block_name.clear();
            self.last_block_name.push_str(block_name);
            self.last_palette_index = idx;
            idx
        };

        // Direct array write with bookkeeping
        region.set_block_at_index_unchecked(palette_index, x, y, z);
        true
    }

    pub fn set_block_in_region(
        &mut self,
        region_name: &str,
        x: i32,
        y: i32,
        z: i32,
        block_name: &str,
    ) -> bool {
        self.inner
            .set_block_in_region_str(region_name, x, y, z, block_name)
    }

    /// Expose cache clearing to Python
    pub fn clear_cache(&mut self) {
        self.inner.clear_block_state_cache();
    }

    /// Expose cache stats to Python for debugging
    pub fn cache_info(&self) -> (usize, usize) {
        self.inner.cache_stats()
    }

    pub fn set_block_from_string(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        block_string: &str,
    ) -> PyResult<()> {
        self.inner
            .set_block_from_string(x, y, z, block_string)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        Ok(())
    }

    pub fn set_block_with_properties(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        block_name: &str,
        properties: HashMap<String, String>,
    ) {
        let block_state = BlockState {
            name: block_name.to_string(),
            properties,
        };
        self.inner.set_block(x, y, z, &block_state);
    }

    pub fn set_block_with_nbt(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        block_name: &str,
        nbt_data: HashMap<String, String>,
    ) -> PyResult<()> {
        self.inner
            .set_block_with_nbt(x, y, z, block_name, nbt_data)
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Error setting block with NBT: {}",
                    e
                ))
            })?;
        Ok(())
    }

    pub fn copy_region(
        &mut self,
        from_schematic: &PySchematic,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
        target_x: i32,
        target_y: i32,
        target_z: i32,
        excluded_blocks: Option<Vec<String>>,
    ) -> PyResult<()> {
        let bounds = BoundingBox::new((min_x, min_y, min_z), (max_x, max_y, max_z));
        let excluded: Vec<BlockState> = excluded_blocks
            .unwrap_or_default()
            .iter()
            .map(|s| UniversalSchematic::parse_block_string(s).map(|(bs, _)| bs))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

        self.inner
            .copy_region(
                &from_schematic.inner,
                &bounds,
                (target_x, target_y, target_z),
                &excluded,
            )
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<PyBlockState> {
        // Fast path: try default region directly first
        if self.inner.default_region.is_in_region(x, y, z) {
            return self
                .inner
                .default_region
                .get_block(x, y, z)
                .cloned()
                .map(|bs| PyBlockState { inner: bs });
        }
        // Fall back to multi-region scan
        self.inner
            .get_block(x, y, z)
            .cloned()
            .map(|bs| PyBlockState { inner: bs })
    }

    /// Get block as formatted string with properties (e.g., "minecraft:lever[powered=true,facing=north]")
    pub fn get_block_string(&self, x: i32, y: i32, z: i32) -> Option<String> {
        // Fast path: try default region directly, returning name only if no properties
        if self.inner.default_region.is_in_region(x, y, z) {
            return self
                .inner
                .default_region
                .get_block(x, y, z)
                .map(|bs| bs.to_string());
        }
        // Fall back to multi-region scan
        self.inner.get_block(x, y, z).map(|bs| bs.to_string())
    }

    /// Batch set blocks at multiple positions with the same block name.
    /// Crosses the FFI boundary once. Returns the number of blocks set.
    pub fn set_blocks(&mut self, positions: Vec<(i32, i32, i32)>, block_name: &str) -> usize {
        if positions.is_empty() {
            return 0;
        }

        // Complex block strings fall back to per-call path
        if block_name.contains('[') || block_name.ends_with('}') {
            let mut count = 0;
            for &(x, y, z) in &positions {
                if self.inner.set_block_str(x, y, z, block_name) {
                    count += 1;
                }
            }
            return count;
        }

        // Initialize default region with first position
        let (fx, fy, fz) = positions[0];
        self.ensure_default_region(fx, fy, fz);

        // Pre-expand to fit all positions
        let mut min = (fx, fy, fz);
        let mut max = (fx, fy, fz);
        for &(x, y, z) in &positions[1..] {
            if x < min.0 {
                min.0 = x;
            }
            if y < min.1 {
                min.1 = y;
            }
            if z < min.2 {
                min.2 = z;
            }
            if x > max.0 {
                max.0 = x;
            }
            if y > max.1 {
                max.1 = y;
            }
            if z > max.2 {
                max.2 = z;
            }
        }

        let region = &mut self.inner.default_region;
        region.ensure_bounds(min, max);

        let palette_index = region.get_or_insert_palette_by_name(block_name);

        for &(x, y, z) in &positions {
            region.set_block_at_index_unchecked(palette_index, x, y, z);
        }

        positions.len()
    }

    /// Batch get block names at multiple positions.
    /// Crosses the FFI boundary once. Returns a list of block names (None for out-of-bounds).
    pub fn get_blocks(&self, positions: Vec<(i32, i32, i32)>) -> Vec<Option<String>> {
        let region = &self.inner.default_region;
        positions
            .iter()
            .map(|&(x, y, z)| {
                if region.is_in_region(x, y, z) {
                    region.get_block_name(x, y, z).map(|s| s.to_string())
                } else {
                    // Fall back to multi-region scan
                    self.inner.get_block(x, y, z).map(|bs| bs.name.clone())
                }
            })
            .collect()
    }

    /// Get the palette for the default region
    pub fn get_palette<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        let palette = self.inner.default_region.palette.clone();
        let list = PyList::new(
            py,
            palette.iter().map(|bs| PyBlockState { inner: bs.clone() }),
        )?;
        Ok(list.into())
    }

    pub fn get_block_entity<'py>(
        &self,
        py: Python<'py>,
        x: i32,
        y: i32,
        z: i32,
    ) -> PyResult<Option<PyObject>> {
        let pos = BlockPosition { x, y, z };
        if let Some(be) = self.inner.get_block_entity(pos) {
            let dict = PyDict::new(py);
            dict.set_item("id", &be.id)?;
            dict.set_item("position", (be.position.0, be.position.1, be.position.2))?;

            dict.set_item("nbt", nbt_map_to_python(py, &be.nbt)?)?;
            Ok(Some(dict.into()))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_block_entities<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        let entities = self.inner.get_block_entities_as_list();
        let mut list_items: Vec<PyObject> = Vec::new();

        for be in entities.iter() {
            let dict = PyDict::new(py);
            dict.set_item("id", &be.id)?;
            dict.set_item("position", (be.position.0, be.position.1, be.position.2))?;
            dict.set_item("nbt", nbt_map_to_python(py, &be.nbt)?)?;
            list_items.push(dict.into());
        }

        let list = PyList::new(py, list_items)?;
        Ok(list.into())
    }

    /// Get the number of entities (mobile entities, not block entities).
    pub fn entity_count(&self) -> usize {
        self.inner.default_region.entities.len()
    }

    /// Get all entities as a list of dicts.
    pub fn get_entities<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        let entities = &self.inner.default_region.entities;
        let mut list_items: Vec<PyObject> = Vec::new();

        for entity in entities {
            let dict = PyDict::new(py);
            dict.set_item("id", &entity.id)?;
            dict.set_item(
                "position",
                (entity.position.0, entity.position.1, entity.position.2),
            )?;
            dict.set_item("nbt", entity_nbt_to_python(py, &entity.nbt)?)?;
            list_items.push(dict.into());
        }

        let list = PyList::new(py, list_items)?;
        Ok(list.into())
    }

    /// Add an entity to the schematic.
    pub fn add_entity(
        &mut self,
        id: &str,
        x: f64,
        y: f64,
        z: f64,
        nbt_json: Option<&str>,
    ) -> PyResult<()> {
        let mut entity = crate::entity::Entity::new(id.to_string(), (x, y, z));
        if let Some(json) = nbt_json {
            let nbt_map: HashMap<String, crate::entity::NbtValue> = serde_json::from_str(json)
                .map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Invalid NBT JSON: {}",
                        e
                    ))
                })?;
            entity.nbt = nbt_map;
        }
        self.inner.add_entity(entity);
        Ok(())
    }

    /// Remove an entity by index. Returns true if removed.
    pub fn remove_entity(&mut self, index: usize) -> bool {
        self.inner.remove_entity(index).is_some()
    }

    pub fn get_all_blocks<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        let mut list_items: Vec<PyObject> = Vec::new();

        for (pos, block) in self.inner.iter_blocks() {
            let dict = PyDict::new(py);
            dict.set_item("x", pos.x)?;
            dict.set_item("y", pos.y)?;
            dict.set_item("z", pos.z)?;
            dict.set_item("name", &block.name)?;
            dict.set_item("properties", block.properties.clone())?;
            list_items.push(dict.into());
        }

        let list = PyList::new(py, list_items)?;
        Ok(list.into())
    }

    #[pyo3(signature = (
        chunk_width, chunk_height, chunk_length,
        strategy=None, camera_x=0.0, camera_y=0.0, camera_z=0.0
    ))]
    pub fn get_chunks<'py>(
        &self,
        py: Python<'py>,
        chunk_width: i32,
        chunk_height: i32,
        chunk_length: i32,
        strategy: Option<String>,
        camera_x: f32,
        camera_y: f32,
        camera_z: f32,
    ) -> PyResult<PyObject> {
        let strategy_enum = match strategy.as_deref() {
            Some("distance_to_camera") => Some(ChunkLoadingStrategy::DistanceToCamera(
                camera_x, camera_y, camera_z,
            )),
            Some("top_down") => Some(ChunkLoadingStrategy::TopDown),
            Some("bottom_up") => Some(ChunkLoadingStrategy::BottomUp),
            Some("center_outward") => Some(ChunkLoadingStrategy::CenterOutward),
            Some("random") => Some(ChunkLoadingStrategy::Random),
            _ => None,
        };

        let chunks = self
            .inner
            .iter_chunks(chunk_width, chunk_height, chunk_length, strategy_enum);
        let mut chunk_items: Vec<PyObject> = Vec::new();

        for chunk in chunks {
            let chunk_dict = PyDict::new(py);
            chunk_dict.set_item("chunk_x", chunk.chunk_x)?;
            chunk_dict.set_item("chunk_y", chunk.chunk_y)?;
            chunk_dict.set_item("chunk_z", chunk.chunk_z)?;

            let mut block_items: Vec<PyObject> = Vec::new();
            for pos in chunk.positions.iter() {
                if let Some(block) = self.inner.get_block(pos.x, pos.y, pos.z) {
                    let block_dict = PyDict::new(py);
                    block_dict.set_item("x", pos.x)?;
                    block_dict.set_item("y", pos.y)?;
                    block_dict.set_item("z", pos.z)?;
                    block_dict.set_item("name", &block.name)?;
                    block_dict.set_item("properties", block.properties.clone())?;
                    block_items.push(block_dict.into());
                }
            }

            let blocks_list = PyList::new(py, block_items)?;
            chunk_dict.set_item("blocks", &blocks_list)?;
            chunk_items.push(chunk_dict.into());
        }

        let list = PyList::new(py, chunk_items)?;
        Ok(list.into())
    }

    #[getter]
    pub fn dimensions(&self) -> (i32, i32, i32) {
        // Return tight dimensions if available (actual content size), otherwise allocated
        let tight = self.inner.get_tight_dimensions();
        if tight != (0, 0, 0) {
            tight
        } else {
            self.inner.get_dimensions()
        }
    }

    #[getter]
    pub fn allocated_dimensions(&self) -> (i32, i32, i32) {
        // Return the full allocated buffer size (internal use)
        self.inner.get_dimensions()
    }

    #[getter]
    pub fn block_count(&self) -> i32 {
        self.inner.total_blocks()
    }

    #[getter]
    pub fn volume(&self) -> i32 {
        self.inner.total_volume()
    }

    #[getter]
    pub fn region_names(&self) -> Vec<String> {
        self.inner.get_region_names()
    }

    // --- Metadata Accessors ---

    /// Get the schematic name.
    #[getter]
    pub fn get_name(&self) -> Option<String> {
        self.inner.metadata.name.clone()
    }

    /// Set the schematic name.
    #[setter]
    pub fn set_name(&mut self, name: String) {
        self.inner.metadata.name = Some(name);
    }

    /// Get the schematic author.
    #[getter]
    pub fn author(&self) -> Option<String> {
        self.inner.metadata.author.clone()
    }

    /// Set the schematic author.
    #[setter]
    pub fn set_author(&mut self, author: String) {
        self.inner.metadata.author = Some(author);
    }

    /// Get the schematic description.
    #[getter]
    pub fn description(&self) -> Option<String> {
        self.inner.metadata.description.clone()
    }

    /// Set the schematic description.
    #[setter]
    pub fn set_description(&mut self, description: String) {
        self.inner.metadata.description = Some(description);
    }

    /// Get the creation timestamp (milliseconds since epoch).
    #[getter]
    pub fn created(&self) -> Option<u64> {
        self.inner.metadata.created
    }

    /// Set the creation timestamp.
    #[setter]
    pub fn set_created(&mut self, created: u64) {
        self.inner.metadata.created = Some(created);
    }

    /// Get the modification timestamp (milliseconds since epoch).
    #[getter]
    pub fn modified(&self) -> Option<u64> {
        self.inner.metadata.modified
    }

    /// Set the modification timestamp.
    #[setter]
    pub fn set_modified(&mut self, modified: u64) {
        self.inner.metadata.modified = Some(modified);
    }

    /// Get the Litematic format version.
    #[getter]
    pub fn lm_version(&self) -> Option<i32> {
        self.inner.metadata.lm_version
    }

    /// Set the Litematic format version.
    #[setter]
    pub fn set_lm_version(&mut self, version: i32) {
        self.inner.metadata.lm_version = Some(version);
    }

    /// Get the Minecraft data version.
    #[getter]
    pub fn mc_version(&self) -> Option<i32> {
        self.inner.metadata.mc_version
    }

    /// Set the Minecraft data version.
    #[setter]
    pub fn set_mc_version(&mut self, version: i32) {
        self.inner.metadata.mc_version = Some(version);
    }

    /// Get the WorldEdit version.
    #[getter]
    pub fn we_version(&self) -> Option<i32> {
        self.inner.metadata.we_version
    }

    /// Set the WorldEdit version.
    #[setter]
    pub fn set_we_version(&mut self, version: i32) {
        self.inner.metadata.we_version = Some(version);
    }

    pub fn debug_info(&self) -> String {
        format!(
            "Schematic name: {}, Regions: {}",
            self.inner
                .metadata
                .name
                .as_ref()
                .unwrap_or(&"Unnamed".to_string()),
            self.inner.other_regions.len() + 1 // +1 for the main region
        )
    }

    fn __str__(&self) -> String {
        format_schematic(&self.inner)
    }

    fn __repr__(&self) -> String {
        format!(
            "<Schematic '{}', {} blocks>",
            self.inner
                .metadata
                .name
                .as_ref()
                .unwrap_or(&"Unnamed".to_string()),
            self.inner.total_blocks()
        )
    }

    #[cfg(feature = "simulation")]
    pub fn create_simulation_world(&self) -> PyResult<PyMchprsWorld> {
        let world = MchprsWorld::new(self.inner.clone())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
        Ok(PyMchprsWorld { inner: world })
    }

    #[cfg(feature = "simulation")]
    pub fn build_executor(
        &self,
        inputs: Vec<HashMap<String, String>>,
        outputs: Vec<HashMap<String, String>>,
    ) -> PyResult<PyTypedCircuitExecutor> {
        let mut builder = CircuitBuilder::new(self.inner.clone());

        for input in inputs {
            let name = input.get("name").ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>("Input name missing")
            })?;
            let region_name = input.get("region").ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>("Input region missing")
            })?;
            let bits_str = input.get("bits").map(|s| s.as_str()).unwrap_or("1");
            let bits = bits_str.parse::<u32>().map_err(|_| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid bits value")
            })?;

            let region = self
                .inner
                .definition_regions
                .get(region_name)
                .ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                        "Region '{}' not found",
                        region_name
                    ))
                })?
                .clone();

            builder = builder
                .with_input_auto(
                    name,
                    IoType::UnsignedInt {
                        bits: bits as usize,
                    },
                    region,
                )
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        }

        for output in outputs {
            let name = output.get("name").ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>("Output name missing")
            })?;
            let region_name = output.get("region").ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>("Output region missing")
            })?;
            let bits_str = output.get("bits").map(|s| s.as_str()).unwrap_or("1");
            let bits = bits_str.parse::<u32>().map_err(|_| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid bits value")
            })?;

            let region = self
                .inner
                .definition_regions
                .get(region_name)
                .ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                        "Region '{}' not found",
                        region_name
                    ))
                })?
                .clone();

            builder = builder
                .with_output_auto(
                    name,
                    IoType::UnsignedInt {
                        bits: bits as usize,
                    },
                    region,
                )
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        }

        let executor = builder
            .build()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
        Ok(PyTypedCircuitExecutor { inner: executor })
    }

    // Transformation methods

    /// Flip the schematic along the X axis
    pub fn flip_x(&mut self) {
        self.inner.flip_x();
    }

    /// Flip the schematic along the Y axis
    pub fn flip_y(&mut self) {
        self.inner.flip_y();
    }

    /// Flip the schematic along the Z axis
    pub fn flip_z(&mut self) {
        self.inner.flip_z();
    }

    /// Rotate the schematic around the Y axis (horizontal plane)
    /// Degrees must be 90, 180, or 270
    pub fn rotate_y(&mut self, degrees: i32) {
        self.inner.rotate_y(degrees);
    }

    /// Rotate the schematic around the X axis
    /// Degrees must be 90, 180, or 270
    pub fn rotate_x(&mut self, degrees: i32) {
        self.inner.rotate_x(degrees);
    }

    /// Rotate the schematic around the Z axis
    /// Degrees must be 90, 180, or 270
    pub fn rotate_z(&mut self, degrees: i32) {
        self.inner.rotate_z(degrees);
    }

    /// Flip a specific region along the X axis
    pub fn flip_region_x(&mut self, region_name: &str) -> PyResult<()> {
        self.inner
            .flip_region_x(region_name)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
    }

    /// Flip a specific region along the Y axis
    pub fn flip_region_y(&mut self, region_name: &str) -> PyResult<()> {
        self.inner
            .flip_region_y(region_name)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
    }

    /// Flip a specific region along the Z axis
    pub fn flip_region_z(&mut self, region_name: &str) -> PyResult<()> {
        self.inner
            .flip_region_z(region_name)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
    }

    /// Rotate a specific region around the Y axis
    pub fn rotate_region_y(&mut self, region_name: &str, degrees: i32) -> PyResult<()> {
        self.inner
            .rotate_region_y(region_name, degrees)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
    }

    /// Rotate a specific region around the X axis
    pub fn rotate_region_x(&mut self, region_name: &str, degrees: i32) -> PyResult<()> {
        self.inner
            .rotate_region_x(region_name, degrees)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
    }

    /// Rotate a specific region around the Z axis
    pub fn rotate_region_z(&mut self, region_name: &str, degrees: i32) -> PyResult<()> {
        self.inner
            .rotate_region_z(region_name, degrees)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
    }

    // ============================================================================
    // INSIGN METHODS
    // ============================================================================

    /// Extract all sign text from the schematic
    /// Returns a list of dicts: [{"pos": [x,y,z], "text": "..."}]
    pub fn extract_signs(&self, py: Python<'_>) -> PyResult<PyObject> {
        let signs = crate::insign::extract_signs(&self.inner);

        let list = PyList::new(py, &[] as &[PyObject])?;
        for sign in signs {
            let dict = PyDict::new(py);
            let pos_list = PyList::new(py, &[sign.pos[0], sign.pos[1], sign.pos[2]])?;
            dict.set_item("pos", pos_list)?;
            dict.set_item("text", sign.text)?;
            list.append(dict)?;
        }

        Ok(list.into())
    }

    /// Compile Insign annotations from the schematic's signs
    /// Returns a Python dict with compiled region metadata
    /// This returns raw Insign data - interpretation is up to the consumer
    pub fn compile_insign(&self, py: Python<'_>) -> PyResult<PyObject> {
        let insign_data = crate::insign::compile_schematic_insign(&self.inner).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Insign compilation error: {}",
                e
            ))
        })?;

        // Convert serde_json::Value to Python object
        let json_str = serde_json::to_string(&insign_data).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "JSON serialization error: {}",
                e
            ))
        })?;

        let json_module = py.import("json")?;
        let loads = json_module.getattr("loads")?;
        Ok(loads.call1((json_str,))?.extract()?)
    }

    // Definition Region Methods

    pub fn add_definition_region(&mut self, name: String, region: &PyDefinitionRegion) {
        self.inner
            .definition_regions
            .insert(name, region.inner.clone());
    }

    pub fn get_definition_region(&mut self, name: String) -> PyResult<PyDefinitionRegion> {
        match self.inner.definition_regions.get(&name) {
            Some(region) => Ok(PyDefinitionRegion {
                inner: region.clone(),
                schematic_ptr: &mut self.inner as *mut UniversalSchematic as usize,
                name: Some(name),
            }),
            None => Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                "Definition region '{}' not found",
                name
            ))),
        }
    }

    pub fn create_region(
        &mut self,
        name: String,
        min: (i32, i32, i32),
        max: (i32, i32, i32),
    ) -> PyResult<PyDefinitionRegion> {
        let mut region = DefinitionRegion::new();
        region.add_bounds(min, max);
        self.inner
            .definition_regions
            .insert(name.clone(), region.clone());

        Ok(PyDefinitionRegion {
            inner: region,
            schematic_ptr: &mut self.inner as *mut UniversalSchematic as usize,
            name: Some(name),
        })
    }

    pub fn remove_definition_region(&mut self, name: String) -> bool {
        self.inner.definition_regions.remove(&name).is_some()
    }

    pub fn get_definition_region_names(&self) -> Vec<String> {
        self.inner.definition_regions.keys().cloned().collect()
    }

    pub fn create_definition_region(&mut self, name: String) {
        self.inner
            .definition_regions
            .insert(name, DefinitionRegion::new());
    }

    pub fn create_definition_region_from_point(&mut self, name: String, x: i32, y: i32, z: i32) {
        let mut region = DefinitionRegion::new();
        region.add_point(x, y, z);
        self.inner.definition_regions.insert(name, region);
    }

    pub fn create_definition_region_from_bounds(
        &mut self,
        name: String,
        min: (i32, i32, i32),
        max: (i32, i32, i32),
    ) {
        let mut region = DefinitionRegion::new();
        region.add_bounds(min, max);
        self.inner.definition_regions.insert(name, region);
    }

    pub fn definition_region_add_bounds(
        &mut self,
        name: String,
        min: (i32, i32, i32),
        max: (i32, i32, i32),
    ) -> PyResult<()> {
        match self.inner.definition_regions.get_mut(&name) {
            Some(region) => {
                region.add_bounds(min, max);
                Ok(())
            }
            None => Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                "Definition region '{}' not found",
                name
            ))),
        }
    }

    pub fn definition_region_add_point(
        &mut self,
        name: String,
        x: i32,
        y: i32,
        z: i32,
    ) -> PyResult<()> {
        match self.inner.definition_regions.get_mut(&name) {
            Some(region) => {
                region.add_point(x, y, z);
                Ok(())
            }
            None => Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                "Definition region '{}' not found",
                name
            ))),
        }
    }

    pub fn definition_region_set_metadata(
        &mut self,
        name: String,
        key: String,
        value: String,
    ) -> PyResult<()> {
        match self.inner.definition_regions.get_mut(&name) {
            Some(region) => {
                region.metadata.insert(key, value);
                Ok(())
            }
            None => Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                "Definition region '{}' not found",
                name
            ))),
        }
    }

    pub fn definition_region_shift(
        &mut self,
        name: String,
        x: i32,
        y: i32,
        z: i32,
    ) -> PyResult<()> {
        match self.inner.definition_regions.get_mut(&name) {
            Some(region) => {
                region.shift(x, y, z);
                Ok(())
            }
            None => Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                "Definition region '{}' not found",
                name
            ))),
        }
    }

    pub fn update_region(&mut self, name: String, region: &PyDefinitionRegion) {
        self.inner
            .definition_regions
            .insert(name, region.inner.clone());
    }

    #[pyo3(signature = (format, version=None, settings=None))]
    pub fn save_as(
        &self,
        py: Python<'_>,
        format: &str,
        version: Option<String>,
        settings: Option<&str>,
    ) -> PyResult<PyObject> {
        let manager = get_manager();
        let manager = manager
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        let bytes = manager
            .write_with_settings(format, &self.inner, version.as_deref(), settings)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(PyBytes::new(py, &bytes).into())
    }

    /// Register a mesh exporter with the FormatManager, enabling save_as("mesh", ...).
    #[cfg(feature = "meshing")]
    pub fn register_mesh_exporter(&self, pack: &super::meshing::PyResourcePack) -> PyResult<()> {
        let mesh_exporter = crate::meshing::MeshExporter::new(
            crate::meshing::ResourcePackSource::from_resource_pack(pack.inner.pack().clone()),
        );

        let manager = get_manager();
        let mut manager = manager
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        manager.register_exporter(mesh_exporter);
        Ok(())
    }

    /// Get the settings schema for an export format (returns JSON or None).
    #[staticmethod]
    pub fn get_export_settings_schema(format: &str) -> PyResult<Option<String>> {
        let manager = get_manager();
        let manager = manager.lock().map_err(|_| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Failed to acquire format manager lock",
            )
        })?;
        Ok(manager.get_export_settings_schema(format))
    }

    /// Get the settings schema for an import format (returns JSON or None).
    #[staticmethod]
    pub fn get_import_settings_schema(format: &str) -> PyResult<Option<String>> {
        let manager = get_manager();
        let manager = manager.lock().map_err(|_| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Failed to acquire format manager lock",
            )
        })?;
        Ok(manager.get_import_settings_schema(format))
    }

    pub fn to_schematic_version(&self, py: Python<'_>, version: &str) -> PyResult<PyObject> {
        use crate::schematic::SchematicVersion;
        let sv = SchematicVersion::from_str(version).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Unknown schematic version: {}",
                version
            ))
        })?;
        let bytes = schematic::to_schematic_version(&self.inner, sv)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(PyBytes::new(py, &bytes).into())
    }

    #[staticmethod]
    pub fn get_available_schematic_versions() -> Vec<String> {
        use crate::schematic::SchematicVersion;
        SchematicVersion::get_all()
            .iter()
            .map(|v| v.to_string())
            .collect()
    }

    pub fn get_all_palettes<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        let all_palettes = self.inner.get_all_palettes();

        let result = PyDict::new(py);

        // Default palette
        let default_list = PyList::new(
            py,
            all_palettes
                .default_palette
                .iter()
                .map(|bs| PyBlockState { inner: bs.clone() }),
        )?;
        result.set_item("default", default_list)?;

        // Region palettes
        let regions_dict = PyDict::new(py);
        for (name, palette) in &all_palettes.region_palettes {
            let region_list = PyList::new(
                py,
                palette.iter().map(|bs| PyBlockState { inner: bs.clone() }),
            )?;
            regions_dict.set_item(name, region_list)?;
        }
        result.set_item("regions", regions_dict)?;

        Ok(result.into())
    }

    pub fn get_default_region_palette<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        let palette = self.inner.get_default_region_palette();
        let list = PyList::new(py, palette.into_iter().map(|bs| PyBlockState { inner: bs }))?;
        Ok(list.into())
    }

    pub fn get_palette_from_region<'py>(
        &self,
        py: Python<'py>,
        region_name: &str,
    ) -> PyResult<PyObject> {
        let palette = if region_name == "default" || region_name == "Default" {
            &self.inner.default_region.palette
        } else {
            match self.inner.other_regions.get(region_name) {
                Some(region) => &region.palette,
                None => {
                    return Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                        "Region '{}' not found",
                        region_name
                    )))
                }
            }
        };
        let list = PyList::new(
            py,
            palette.iter().map(|bs| PyBlockState { inner: bs.clone() }),
        )?;
        Ok(list.into())
    }

    pub fn get_bounding_box(&self) -> ((i32, i32, i32), (i32, i32, i32)) {
        let bbox = self.inner.get_bounding_box();
        (bbox.min, bbox.max)
    }

    pub fn get_region_bounding_box(
        &self,
        region_name: &str,
    ) -> PyResult<((i32, i32, i32), (i32, i32, i32))> {
        let bbox = if region_name == "default" || region_name == "Default" {
            self.inner.default_region.get_bounding_box()
        } else {
            match self.inner.other_regions.get(region_name) {
                Some(region) => region.get_bounding_box(),
                None => {
                    return Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                        "Region '{}' not found",
                        region_name
                    )))
                }
            }
        };
        Ok((bbox.min, bbox.max))
    }

    pub fn get_tight_dimensions(&self) -> (i32, i32, i32) {
        self.inner.get_tight_dimensions()
    }

    pub fn get_tight_bounds_min(&self) -> Option<(i32, i32, i32)> {
        self.inner.get_tight_bounds().map(|b| b.min)
    }

    pub fn get_tight_bounds_max(&self) -> Option<(i32, i32, i32)> {
        self.inner.get_tight_bounds().map(|b| b.max)
    }

    pub fn get_block_with_properties(&self, x: i32, y: i32, z: i32) -> Option<PyBlockState> {
        self.inner
            .get_block(x, y, z)
            .cloned()
            .map(|bs| PyBlockState { inner: bs })
    }

    pub fn print_schematic(&self) -> String {
        format_schematic(&self.inner)
    }

    // --- Meshing methods (feature-gated) ---

    #[cfg(feature = "meshing")]
    #[pyo3(signature = (pack, config=None))]
    pub fn to_mesh(
        &self,
        pack: &super::meshing::PyResourcePack,
        config: Option<&super::meshing::PyMeshConfig>,
    ) -> PyResult<super::meshing::PyMeshResult> {
        let default_config = crate::meshing::MeshConfig::default();
        let config = config.map(|c| &c.inner).unwrap_or(&default_config);

        self.inner
            .to_mesh(&pack.inner, config)
            .map(|result| super::meshing::PyMeshResult { inner: result })
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    #[cfg(feature = "meshing")]
    #[pyo3(signature = (pack, config=None))]
    pub fn mesh_by_region(
        &self,
        pack: &super::meshing::PyResourcePack,
        config: Option<&super::meshing::PyMeshConfig>,
    ) -> PyResult<super::meshing::PyMultiMeshResult> {
        let default_config = crate::meshing::MeshConfig::default();
        let config = config.map(|c| &c.inner).unwrap_or(&default_config);

        self.inner
            .mesh_by_region(&pack.inner, config)
            .map(|result| super::meshing::PyMultiMeshResult { inner: result })
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    #[cfg(feature = "meshing")]
    #[pyo3(signature = (pack, config=None))]
    pub fn mesh_by_chunk(
        &self,
        pack: &super::meshing::PyResourcePack,
        config: Option<&super::meshing::PyMeshConfig>,
    ) -> PyResult<super::meshing::PyChunkMeshResult> {
        let default_config = crate::meshing::MeshConfig::default();
        let config = config.map(|c| &c.inner).unwrap_or(&default_config);

        self.inner
            .mesh_by_chunk(&pack.inner, config)
            .map(|result| super::meshing::PyChunkMeshResult { inner: result })
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    #[cfg(feature = "meshing")]
    #[pyo3(signature = (pack, chunk_size, config=None))]
    pub fn mesh_by_chunk_size(
        &self,
        pack: &super::meshing::PyResourcePack,
        chunk_size: i32,
        config: Option<&super::meshing::PyMeshConfig>,
    ) -> PyResult<super::meshing::PyChunkMeshResult> {
        let default_config = crate::meshing::MeshConfig::default();
        let config = config.map(|c| &c.inner).unwrap_or(&default_config);

        self.inner
            .mesh_by_chunk_size(&pack.inner, config, chunk_size)
            .map(|result| super::meshing::PyChunkMeshResult { inner: result })
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Generate a USDZ mesh for the entire schematic.
    #[cfg(feature = "meshing")]
    #[pyo3(signature = (pack, config=None))]
    pub fn to_usdz(
        &self,
        pack: &super::meshing::PyResourcePack,
        config: Option<&super::meshing::PyMeshConfig>,
    ) -> PyResult<super::meshing::PyMeshResult> {
        let default_config = crate::meshing::MeshConfig::default();
        let config = config.map(|c| &c.inner).unwrap_or(&default_config);

        self.inner
            .to_usdz(&pack.inner, config)
            .map(|result| super::meshing::PyMeshResult { inner: result })
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Generate raw mesh data for custom rendering pipelines.
    #[cfg(feature = "meshing")]
    #[pyo3(signature = (pack, config=None))]
    pub fn to_raw_mesh(
        &self,
        pack: &super::meshing::PyResourcePack,
        config: Option<&super::meshing::PyMeshConfig>,
    ) -> PyResult<super::meshing::PyRawMeshExport> {
        let default_config = crate::meshing::MeshConfig::default();
        let config = config.map(|c| &c.inner).unwrap_or(&default_config);

        self.inner
            .to_raw_mesh(&pack.inner, config)
            .map(|result| super::meshing::PyRawMeshExport { inner: result })
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Lazy chunk mesh iterator. Returns a list of MeshResult, one per chunk.
    ///
    /// Yields one MeshResult per chunk of `chunk_size` blocks on each axis.
    /// For truly lazy streaming in Python, iterate over the returned list.
    #[cfg(feature = "meshing")]
    #[pyo3(signature = (pack, config=None, chunk_size=16))]
    pub fn mesh_chunks(
        &self,
        pack: &super::meshing::PyResourcePack,
        config: Option<&super::meshing::PyMeshConfig>,
        chunk_size: i32,
    ) -> PyResult<Vec<super::meshing::PyMeshResult>> {
        let default_config = crate::meshing::MeshConfig::default();
        let config = config.map(|c| &c.inner).unwrap_or(&default_config);

        let iter = self.inner.mesh_chunks(&pack.inner, config, chunk_size);
        let mut results = Vec::new();
        for chunk_result in iter {
            let mesh = chunk_result
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            results.push(super::meshing::PyMeshResult { inner: mesh });
        }
        Ok(results)
    }

    /// Build a global texture atlas for the schematic without meshing.
    ///
    /// The atlas can be reused across multiple `mesh_chunks_with_atlas` calls,
    /// avoiding redundant atlas rebuilds for chunked workflows.
    #[cfg(feature = "meshing")]
    #[pyo3(signature = (pack, config=None))]
    pub fn build_global_atlas(
        &self,
        pack: &super::meshing::PyResourcePack,
        config: Option<&super::meshing::PyMeshConfig>,
    ) -> PyResult<super::meshing::PyTextureAtlas> {
        let default_config = crate::meshing::MeshConfig::default();
        let config = config.map(|c| &c.inner).unwrap_or(&default_config);

        crate::meshing::build_global_atlas(&self.inner, &pack.inner, config)
            .map(|atlas| super::meshing::PyTextureAtlas { inner: atlas })
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Chunk mesh iterator using a pre-built shared atlas.
    ///
    /// Like `mesh_chunks`, but every chunk reuses the provided `atlas`
    /// instead of building its own. Returns a list of MeshResult.
    #[cfg(feature = "meshing")]
    #[pyo3(signature = (pack, atlas, config=None, chunk_size=16))]
    pub fn mesh_chunks_with_atlas(
        &self,
        pack: &super::meshing::PyResourcePack,
        atlas: &super::meshing::PyTextureAtlas,
        config: Option<&super::meshing::PyMeshConfig>,
        chunk_size: i32,
    ) -> PyResult<Vec<super::meshing::PyMeshResult>> {
        let default_config = crate::meshing::MeshConfig::default();
        let config = config.map(|c| &c.inner).unwrap_or(&default_config);

        let iter =
            self.inner
                .mesh_chunks_with_atlas(&pack.inner, config, chunk_size, atlas.inner.clone());
        let mut results = Vec::new();
        for chunk_result in iter {
            let mesh = chunk_result
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            results.push(super::meshing::PyMeshResult { inner: mesh });
        }
        Ok(results)
    }
}

// --- NBT Conversion Helpers ---

fn nbt_map_to_python(py: Python<'_>, map: &NbtMap) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    for (key, value) in map.iter() {
        dict.set_item(key, nbt_value_to_python(py, value)?)?;
    }
    Ok(dict.into())
}

// Helper for your project-specific NbtValue
fn nbt_value_to_python(py: Python<'_>, value: &NbtValue) -> PyResult<PyObject> {
    match value {
        NbtValue::Byte(b) => Ok((*b).into_pyobject(py)?.into()),
        NbtValue::Short(s) => Ok((*s).into_pyobject(py)?.into()),
        NbtValue::Int(i) => Ok((*i).into_pyobject(py)?.into()),
        NbtValue::Long(l) => Ok((*l).into_pyobject(py)?.into()),
        NbtValue::Float(f) => Ok((*f).into_pyobject(py)?.into()),
        NbtValue::Double(d) => Ok((*d).into_pyobject(py)?.into()),
        NbtValue::ByteArray(ba) => Ok(PyBytes::new(py, bytemuck::cast_slice(ba)).into()),
        NbtValue::String(s) => Ok(s.into_pyobject(py)?.into()),
        NbtValue::List(list) => {
            let mut items = Vec::new();
            for item in list.iter() {
                items.push(nbt_value_to_python(py, item)?);
            }
            let pylist = PyList::new(py, items)?;
            Ok(pylist.into())
        }
        NbtValue::Compound(map) => nbt_map_to_python(py, map),
        NbtValue::IntArray(ia) => {
            let pylist = PyList::new(py, ia.clone())?;
            Ok(pylist.into())
        }
        NbtValue::LongArray(la) => {
            let pylist = PyList::new(py, la.clone())?;
            Ok(pylist.into())
        }
    }
}

fn entity_nbt_to_python(
    py: Python<'_>,
    map: &HashMap<String, crate::entity::NbtValue>,
) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    for (key, value) in map.iter() {
        dict.set_item(key, entity_nbt_value_to_python(py, value)?)?;
    }
    Ok(dict.into())
}

fn entity_nbt_value_to_python(
    py: Python<'_>,
    value: &crate::entity::NbtValue,
) -> PyResult<PyObject> {
    use crate::entity::NbtValue as ENV;
    match value {
        ENV::Byte(b) => Ok((*b).into_pyobject(py)?.into()),
        ENV::Short(s) => Ok((*s).into_pyobject(py)?.into()),
        ENV::Int(i) => Ok((*i).into_pyobject(py)?.into()),
        ENV::Long(l) => Ok((*l).into_pyobject(py)?.into()),
        ENV::Float(f) => Ok((*f).into_pyobject(py)?.into()),
        ENV::Double(d) => Ok((*d).into_pyobject(py)?.into()),
        ENV::ByteArray(ba) => Ok(PyBytes::new(py, bytemuck::cast_slice(ba)).into()),
        ENV::String(s) => Ok(s.into_pyobject(py)?.into()),
        ENV::Boolean(b) => Ok((*b).into_pyobject(py)?.to_owned().into()),
        ENV::List(list) => {
            let items: Vec<PyObject> = list
                .iter()
                .map(|item| entity_nbt_value_to_python(py, item))
                .collect::<PyResult<_>>()?;
            let pylist = PyList::new(py, items)?;
            Ok(pylist.into())
        }
        ENV::Compound(map) => entity_nbt_to_python(py, map),
        ENV::IntArray(ia) => {
            let pylist = PyList::new(py, ia.clone())?;
            Ok(pylist.into())
        }
        ENV::LongArray(la) => {
            let pylist = PyList::new(py, la.clone())?;
            Ok(pylist.into())
        }
    }
}

// --- Module Functions ---

#[pyfunction]
pub fn debug_schematic(schematic: &PySchematic) -> String {
    format!(
        "{}\n{}",
        schematic.debug_info(),
        format_schematic(&schematic.inner)
    )
}

#[pyfunction]
pub fn debug_json_schematic(schematic: &PySchematic) -> String {
    format!(
        "{}\n{}",
        schematic.debug_info(),
        format_json_schematic(&schematic.inner)
    )
}

#[pyfunction]
pub fn load_schematic(path: &str) -> PyResult<PySchematic> {
    let data =
        fs::read(path).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    let mut sch = PySchematic::new(Some(
        Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unnamed")
            .to_owned(),
    ));
    sch.from_data(&data)?;
    Ok(sch)
}

#[pyfunction]
#[pyo3(signature = (schematic, path, format = "auto", version = None))]
pub fn save_schematic(
    schematic: &PySchematic,
    path: &str,
    format: &str,
    version: Option<String>,
) -> PyResult<()> {
    Python::with_gil(|_py| {
        let manager = get_manager();
        let manager = manager
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        let bytes = if format == "auto" {
            manager.write_auto(path, &schematic.inner, version.as_deref())
        } else {
            manager.write(format, &schematic.inner, version.as_deref())
        };

        let bytes =
            bytes.map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

        fs::write(path, bytes)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

        Ok(())
    })
}
