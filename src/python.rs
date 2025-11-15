// src/python.rs
#![cfg(feature = "python")]
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::{
    block_position::BlockPosition,
    bounding_box::BoundingBox,
    formats::{litematic, schematic},
    print_utils::{format_json_schematic, format_schematic},
    universal_schematic::ChunkLoadingStrategy,
    utils::{NbtMap, NbtValue},
    BlockState, UniversalSchematic,
};

#[cfg(feature = "simulation")]
use crate::simulation::{BlockPos, MchprsWorld};

use bytemuck;
#[allow(unused_imports)]
use quartz_nbt::NbtTag;

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
}

#[pymethods]
impl PySchematic {
    #[new]
    fn new(name: Option<String>) -> Self {
        Self {
            inner: UniversalSchematic::new(name.unwrap_or_else(|| "Default".to_string())),
        }
    }

    // test method to check if the Python class is working
    pub fn test(&self) -> String {
        "Schematic class is working!".to_string()
    }

    pub fn from_data(&mut self, data: &[u8]) -> PyResult<()> {
        if litematic::is_litematic(data) {
            self.inner = litematic::from_litematic(data)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        } else if schematic::is_schematic(data) {
            self.inner = schematic::from_schematic(data)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Unknown or unsupported schematic format",
            ));
        }
        Ok(())
    }

    pub fn from_litematic(&mut self, data: &[u8]) -> PyResult<()> {
        self.inner = litematic::from_litematic(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
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
        Ok(())
    }

    pub fn to_schematic(&self, py: Python<'_>) -> PyResult<PyObject> {
        let bytes = schematic::to_schematic(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(PyBytes::new(py, &bytes).into())
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block_name: &str) -> bool {
        self.inner.set_block_str(x, y, z, block_name)
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
        self.inner.set_block(x, y, z, block_state);
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
        self.inner
            .get_block(x, y, z)
            .cloned()
            .map(|bs| PyBlockState { inner: bs })
    }

    /// Get block as formatted string with properties (e.g., "minecraft:lever[powered=true,facing=north]")
    pub fn get_block_string(&self, x: i32, y: i32, z: i32) -> Option<String> {
        self.inner.get_block(x, y, z).map(|bs| bs.to_string())
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

#[pyfunction]
fn debug_schematic(schematic: &PySchematic) -> String {
    format!(
        "{}\n{}",
        schematic.debug_info(),
        format_schematic(&schematic.inner)
    )
}

#[pyfunction]
fn debug_json_schematic(schematic: &PySchematic) -> String {
    format!(
        "{}\n{}",
        schematic.debug_info(),
        format_json_schematic(&schematic.inner)
    )
}

#[pyfunction]
fn load_schematic(path: &str) -> PyResult<PySchematic> {
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
#[pyo3(signature = (schematic, path, format = "auto"))]
fn save_schematic(schematic: &PySchematic, path: &str, format: &str) -> PyResult<()> {
    Python::with_gil(|py| {
        let py_bytes = match format {
            "litematic" => schematic.to_litematic(py)?,
            "schematic" => schematic.to_schematic(py)?,
            "auto" => {
                if path.ends_with(".litematic") {
                    schematic.to_litematic(py)?
                } else {
                    schematic.to_schematic(py)?
                }
            }
            other => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Unknown format '{}', choose 'litematic', 'schematic', or 'auto'",
                    other
                )))
            }
        };

        // Extract actual bytes from PyObject
        let bytes_obj = py_bytes.bind(py).downcast::<PyBytes>()?;
        let bytes = bytes_obj.as_bytes();

        fs::write(path, bytes)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

        Ok(())
    })
}

// --- Simulation Support (Optional) ---

#[cfg(feature = "simulation")]
#[pyclass(name = "MchprsWorld")]
pub struct PyMchprsWorld {
    inner: MchprsWorld,
}

#[cfg(feature = "simulation")]
impl PyMchprsWorld {
    /// Extract the inner MchprsWorld, consuming self
    /// This is used internally by from_layout to take ownership
    pub fn into_inner(self) -> MchprsWorld {
        self.inner
    }
}

#[cfg(feature = "simulation")]
#[pymethods]
impl PyMchprsWorld {
    pub fn on_use_block(&mut self, x: i32, y: i32, z: i32) {
        let pos = BlockPos::new(x, y, z);
        self.inner.on_use_block(pos);
    }

    pub fn tick(&mut self, ticks: u32) {
        self.inner.tick(ticks);
    }

    pub fn flush(&mut self) {
        self.inner.flush();
    }

    pub fn is_lit(&self, x: i32, y: i32, z: i32) -> bool {
        let pos = BlockPos::new(x, y, z);
        self.inner.is_lit(pos)
    }

    pub fn get_lever_power(&self, x: i32, y: i32, z: i32) -> bool {
        let pos = BlockPos::new(x, y, z);
        self.inner.get_lever_power(pos)
    }

    pub fn get_redstone_power(&self, x: i32, y: i32, z: i32) -> u8 {
        let pos = BlockPos::new(x, y, z);
        self.inner.get_redstone_power(pos)
    }

    /// Sets the signal strength at a specific block position (for custom IO nodes)
    pub fn set_signal_strength(&mut self, x: i32, y: i32, z: i32, strength: u8) {
        self.inner
            .set_signal_strength(BlockPos::new(x, y, z), strength);
    }

    /// Gets the signal strength at a specific block position (for custom IO nodes)
    pub fn get_signal_strength(&self, x: i32, y: i32, z: i32) -> u8 {
        self.inner.get_signal_strength(BlockPos::new(x, y, z))
    }

    /// Check for custom IO state changes and queue them
    /// Call this after tick() or set_signal_strength() to detect changes
    pub fn check_custom_io_changes(&mut self) {
        self.inner.check_custom_io_changes();
    }

    /// Get and clear all custom IO changes since last poll
    /// Returns a list of dictionaries with keys: x, y, z, old_power, new_power
    pub fn poll_custom_io_changes(&mut self, py: Python) -> PyResult<PyObject> {
        let changes = self.inner.poll_custom_io_changes();
        let mut list_items: Vec<PyObject> = Vec::new();

        for change in changes {
            let dict = PyDict::new(py);
            dict.set_item("x", change.x)?;
            dict.set_item("y", change.y)?;
            dict.set_item("z", change.z)?;
            dict.set_item("old_power", change.old_power)?;
            dict.set_item("new_power", change.new_power)?;
            list_items.push(dict.into());
        }

        let list = PyList::new(py, list_items)?;
        Ok(list.into())
    }

    /// Get custom IO changes without clearing the queue
    /// Returns a list of dictionaries with keys: x, y, z, old_power, new_power
    pub fn peek_custom_io_changes(&self, py: Python) -> PyResult<PyObject> {
        let changes = self.inner.peek_custom_io_changes();
        let mut list_items: Vec<PyObject> = Vec::new();

        for change in changes {
            let dict = PyDict::new(py);
            dict.set_item("x", change.x)?;
            dict.set_item("y", change.y)?;
            dict.set_item("z", change.z)?;
            dict.set_item("old_power", change.old_power)?;
            dict.set_item("new_power", change.new_power)?;
            list_items.push(dict.into());
        }

        let list = PyList::new(py, list_items)?;
        Ok(list.into())
    }

    /// Clear all queued custom IO changes
    pub fn clear_custom_io_changes(&mut self) {
        self.inner.clear_custom_io_changes();
    }

    pub fn sync_to_schematic(&mut self) {
        self.inner.sync_to_schematic();
    }

    pub fn get_schematic(&self) -> PySchematic {
        PySchematic {
            inner: self.inner.get_schematic().clone(),
        }
    }

    pub fn into_schematic(&mut self) -> PySchematic {
        // Clone and consume the inner world since Python objects can't be moved
        let schematic = self.inner.get_schematic().clone();
        self.inner.sync_to_schematic();
        PySchematic { inner: schematic }
    }

    fn __repr__(&self) -> String {
        "<MchprsWorld (redstone simulation)>".to_string()
    }
}

// =============================================================================
// TYPED CIRCUIT EXECUTOR PYTHON BINDINGS
// =============================================================================

#[cfg(feature = "simulation")]
use crate::simulation::typed_executor::{
    ExecutionMode, IoLayout, IoLayoutBuilder, IoType, LayoutFunction, OutputCondition, StateMode,
    TypedCircuitExecutor, Value,
};

/// Python-compatible Value wrapper
#[cfg(feature = "simulation")]
#[pyclass(name = "Value")]
pub struct PyValue {
    inner: Value,
}

#[cfg(feature = "simulation")]
#[pymethods]
impl PyValue {
    /// Create a U32 value
    #[staticmethod]
    fn u32(value: u32) -> Self {
        Self {
            inner: Value::U32(value),
        }
    }

    /// Create an I32 value
    #[staticmethod]
    fn i32(value: i32) -> Self {
        Self {
            inner: Value::I32(value),
        }
    }

    /// Create an F32 value
    #[staticmethod]
    fn f32(value: f32) -> Self {
        Self {
            inner: Value::F32(value),
        }
    }

    /// Create a Bool value
    #[staticmethod]
    fn bool(value: bool) -> Self {
        Self {
            inner: Value::Bool(value),
        }
    }

    /// Create a String value
    #[staticmethod]
    fn string(value: String) -> Self {
        Self {
            inner: Value::String(value),
        }
    }

    /// Convert to Python object
    fn to_py(&self, py: Python) -> PyObject {
        match &self.inner {
            Value::U32(v) => v.into_pyobject(py).unwrap().into(),
            Value::I32(v) => v.into_pyobject(py).unwrap().into(),
            Value::U64(v) => v.into_pyobject(py).unwrap().into(),
            Value::I64(v) => v.into_pyobject(py).unwrap().into(),
            Value::F32(v) => v.into_pyobject(py).unwrap().into(),
            Value::Bool(v) => v.into_pyobject(py).unwrap().as_any().clone().unbind(),
            Value::String(v) => v.into_pyobject(py).unwrap().into(),
            Value::Array(_) => "[Array]".into_pyobject(py).unwrap().into(),
            Value::Struct(_) => "[Struct]".into_pyobject(py).unwrap().into(),
            Value::BitArray(_) => "[BitArray]".into_pyobject(py).unwrap().into(),
            Value::Bytes(_) => "[Bytes]".into_pyobject(py).unwrap().into(),
        }
    }

    /// Get type name
    fn type_name(&self) -> String {
        match &self.inner {
            Value::U32(_) => "U32".to_string(),
            Value::I32(_) => "I32".to_string(),
            Value::U64(_) => "U64".to_string(),
            Value::I64(_) => "I64".to_string(),
            Value::F32(_) => "F32".to_string(),
            Value::Bool(_) => "Bool".to_string(),
            Value::String(_) => "String".to_string(),
            Value::Array(_) => "Array".to_string(),
            Value::Struct(_) => "Struct".to_string(),
            Value::BitArray(_) => "BitArray".to_string(),
            Value::Bytes(_) => "Bytes".to_string(),
        }
    }

    fn __repr__(&self) -> String {
        format!("Value({})", self.type_name())
    }
}

/// IoType builder for Python
#[cfg(feature = "simulation")]
#[pyclass(name = "IoType")]
pub struct PyIoType {
    inner: IoType,
}

#[cfg(feature = "simulation")]
#[pymethods]
impl PyIoType {
    /// Create an unsigned integer type
    #[staticmethod]
    fn unsigned_int(bits: usize) -> Self {
        Self {
            inner: IoType::UnsignedInt { bits },
        }
    }

    /// Create a signed integer type
    #[staticmethod]
    fn signed_int(bits: usize) -> Self {
        Self {
            inner: IoType::SignedInt { bits },
        }
    }

    /// Create a Float32 type
    #[staticmethod]
    fn float32() -> Self {
        Self {
            inner: IoType::Float32,
        }
    }

    /// Create a Boolean type
    #[staticmethod]
    fn boolean() -> Self {
        Self {
            inner: IoType::Boolean,
        }
    }

    /// Create an ASCII string type
    #[staticmethod]
    fn ascii(chars: usize) -> Self {
        Self {
            inner: IoType::Ascii { chars },
        }
    }

    fn __repr__(&self) -> String {
        match &self.inner {
            IoType::UnsignedInt { bits } => format!("IoType.unsigned_int({})", bits),
            IoType::SignedInt { bits } => format!("IoType.signed_int({})", bits),
            IoType::Float32 => "IoType.float32()".to_string(),
            IoType::Boolean => "IoType.boolean()".to_string(),
            IoType::Ascii { chars } => format!("IoType.ascii({})", chars),
            _ => "IoType(...)".to_string(),
        }
    }
}

/// LayoutFunction builder for Python
#[cfg(feature = "simulation")]
#[pyclass(name = "LayoutFunction")]
pub struct PyLayoutFunction {
    inner: LayoutFunction,
}

#[cfg(feature = "simulation")]
#[pymethods]
impl PyLayoutFunction {
    /// One bit per position (0 or 15)
    #[staticmethod]
    fn one_to_one() -> Self {
        Self {
            inner: LayoutFunction::OneToOne,
        }
    }

    /// Four bits per position (0-15)
    #[staticmethod]
    fn packed4() -> Self {
        Self {
            inner: LayoutFunction::Packed4,
        }
    }

    /// Custom bit-to-position mapping
    #[staticmethod]
    fn custom(mapping: Vec<usize>) -> Self {
        Self {
            inner: LayoutFunction::Custom(mapping),
        }
    }

    /// Row-major 2D layout
    #[staticmethod]
    fn row_major(rows: usize, cols: usize, bits_per_element: usize) -> Self {
        Self {
            inner: LayoutFunction::RowMajor {
                rows,
                cols,
                bits_per_element,
            },
        }
    }

    /// Column-major 2D layout
    #[staticmethod]
    fn column_major(rows: usize, cols: usize, bits_per_element: usize) -> Self {
        Self {
            inner: LayoutFunction::ColumnMajor {
                rows,
                cols,
                bits_per_element,
            },
        }
    }

    /// Scanline layout for screens
    #[staticmethod]
    fn scanline(width: usize, height: usize, bits_per_pixel: usize) -> Self {
        Self {
            inner: LayoutFunction::Scanline {
                width,
                height,
                bits_per_pixel,
            },
        }
    }

    fn __repr__(&self) -> String {
        "LayoutFunction(...)".to_string()
    }
}

/// OutputCondition for conditional execution
#[cfg(feature = "simulation")]
#[pyclass(name = "OutputCondition")]
pub struct PyOutputCondition {
    inner: OutputCondition,
}

#[cfg(feature = "simulation")]
#[pymethods]
impl PyOutputCondition {
    /// Output equals a value
    #[staticmethod]
    fn equals(value: &PyValue) -> Self {
        Self {
            inner: OutputCondition::Equals(value.inner.clone()),
        }
    }

    /// Output not equals a value
    #[staticmethod]
    fn not_equals(value: &PyValue) -> Self {
        Self {
            inner: OutputCondition::NotEquals(value.inner.clone()),
        }
    }

    /// Output greater than a value
    #[staticmethod]
    fn greater_than(value: &PyValue) -> Self {
        Self {
            inner: OutputCondition::GreaterThan(value.inner.clone()),
        }
    }

    /// Output less than a value
    #[staticmethod]
    fn less_than(value: &PyValue) -> Self {
        Self {
            inner: OutputCondition::LessThan(value.inner.clone()),
        }
    }

    /// Bitwise AND with mask
    #[staticmethod]
    fn bitwise_and(mask: u64) -> Self {
        Self {
            inner: OutputCondition::BitwiseAnd(mask),
        }
    }

    fn __repr__(&self) -> String {
        "OutputCondition(...)".to_string()
    }
}

/// ExecutionMode for circuit execution
#[cfg(feature = "simulation")]
#[pyclass(name = "ExecutionMode")]
pub struct PyExecutionMode {
    inner: ExecutionMode,
}

#[cfg(feature = "simulation")]
#[pymethods]
impl PyExecutionMode {
    /// Run for a fixed number of ticks
    #[staticmethod]
    fn fixed_ticks(ticks: u32) -> Self {
        Self {
            inner: ExecutionMode::FixedTicks { ticks },
        }
    }

    /// Run until an output meets a condition
    #[staticmethod]
    fn until_condition(
        output_name: String,
        condition: &PyOutputCondition,
        max_ticks: u32,
        check_interval: u32,
    ) -> Self {
        Self {
            inner: ExecutionMode::UntilCondition {
                output_name,
                condition: condition.inner.clone(),
                max_ticks,
                check_interval,
            },
        }
    }

    /// Run until any output changes
    #[staticmethod]
    fn until_change(max_ticks: u32, check_interval: u32) -> Self {
        Self {
            inner: ExecutionMode::UntilChange {
                max_ticks,
                check_interval,
            },
        }
    }

    /// Run until outputs are stable
    #[staticmethod]
    fn until_stable(stable_ticks: u32, max_ticks: u32) -> Self {
        Self {
            inner: ExecutionMode::UntilStable {
                stable_ticks,
                max_ticks,
            },
        }
    }

    fn __repr__(&self) -> String {
        "ExecutionMode(...)".to_string()
    }
}

/// IoLayoutBuilder for Python
#[cfg(feature = "simulation")]
#[pyclass(name = "IoLayoutBuilder")]
pub struct PyIoLayoutBuilder {
    inner: IoLayoutBuilder,
}

#[cfg(feature = "simulation")]
#[pymethods]
impl PyIoLayoutBuilder {
    /// Create a new IO layout builder
    #[new]
    fn new() -> Self {
        Self {
            inner: IoLayoutBuilder::new(),
        }
    }

    /// Add an input
    fn add_input<'py>(
        mut slf: PyRefMut<'py, Self>,
        name: String,
        io_type: &PyIoType,
        layout: &PyLayoutFunction,
        positions: Vec<(i32, i32, i32)>,
    ) -> PyResult<PyRefMut<'py, Self>> {
        slf.inner = slf
            .inner
            .clone()
            .add_input(name, io_type.inner.clone(), layout.inner.clone(), positions)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        Ok(slf)
    }

    /// Add an output
    fn add_output<'py>(
        mut slf: PyRefMut<'py, Self>,
        name: String,
        io_type: &PyIoType,
        layout: &PyLayoutFunction,
        positions: Vec<(i32, i32, i32)>,
    ) -> PyResult<PyRefMut<'py, Self>> {
        slf.inner = slf
            .inner
            .clone()
            .add_output(name, io_type.inner.clone(), layout.inner.clone(), positions)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        Ok(slf)
    }

    /// Add an input with automatic layout inference
    fn add_input_auto<'py>(
        mut slf: PyRefMut<'py, Self>,
        name: String,
        io_type: &PyIoType,
        positions: Vec<(i32, i32, i32)>,
    ) -> PyResult<PyRefMut<'py, Self>> {
        slf.inner = slf
            .inner
            .clone()
            .add_input_auto(name, io_type.inner.clone(), positions)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        Ok(slf)
    }

    /// Add an output with automatic layout inference
    fn add_output_auto<'py>(
        mut slf: PyRefMut<'py, Self>,
        name: String,
        io_type: &PyIoType,
        positions: Vec<(i32, i32, i32)>,
    ) -> PyResult<PyRefMut<'py, Self>> {
        slf.inner = slf
            .inner
            .clone()
            .add_output_auto(name, io_type.inner.clone(), positions)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        Ok(slf)
    }

    /// Build the IO layout
    fn build(slf: PyRef<'_, Self>) -> PyIoLayout {
        PyIoLayout {
            inner: slf.inner.clone().build(),
        }
    }

    fn __repr__(&self) -> String {
        "IoLayoutBuilder(...)".to_string()
    }
}

/// IoLayout wrapper for Python
#[cfg(feature = "simulation")]
#[pyclass(name = "IoLayout")]
pub struct PyIoLayout {
    inner: IoLayout,
}

#[cfg(feature = "simulation")]
#[pymethods]
impl PyIoLayout {
    /// Get input names
    fn input_names(&self) -> Vec<String> {
        self.inner
            .input_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get output names
    fn output_names(&self) -> Vec<String> {
        self.inner
            .output_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "IoLayout(inputs={}, outputs={})",
            self.inner.inputs.len(),
            self.inner.outputs.len()
        )
    }
}

/// TypedCircuitExecutor wrapper for Python
#[cfg(feature = "simulation")]
#[pyclass(name = "TypedCircuitExecutor")]
pub struct PyTypedCircuitExecutor {
    inner: TypedCircuitExecutor,
}

#[cfg(feature = "simulation")]
#[pymethods]
impl PyTypedCircuitExecutor {
    /// Create executor from world and layout
    /// Note: In Python, this extracts inputs/outputs from the layout and calls new()
    #[staticmethod]
    fn from_layout(world: &PyMchprsWorld, layout: &PyIoLayout) -> PyResult<Self> {
        // In Python, we can't consume the world, so we create a new world from the schematic
        // and use the layout's inputs/outputs
        let schematic = world.inner.get_schematic().clone();
        let new_world = MchprsWorld::new(schematic)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

        Ok(Self {
            inner: TypedCircuitExecutor::from_layout(new_world, layout.inner.clone()),
        })
    }

    /// Create executor from Insign annotations in schematic
    #[staticmethod]
    fn from_insign(schematic: &PySchematic) -> PyResult<Self> {
        use crate::simulation::typed_executor::create_executor_from_insign;

        let executor = create_executor_from_insign(&schematic.inner).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to create executor from Insign: {}",
                e
            ))
        })?;

        Ok(Self { inner: executor })
    }

    /// Set state mode
    fn set_state_mode(&mut self, mode: &str) -> PyResult<()> {
        let state_mode = match mode {
            "stateless" => StateMode::Stateless,
            "stateful" => StateMode::Stateful,
            "manual" => StateMode::Manual,
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Invalid state mode. Use 'stateless', 'stateful', or 'manual'",
                ))
            }
        };
        self.inner.set_state_mode(state_mode);
        Ok(())
    }

    /// Reset the simulation
    fn reset(&mut self) -> PyResult<()> {
        self.inner
            .reset()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
    }

    /// Execute the circuit
    fn execute(
        &mut self,
        py: Python,
        inputs: std::collections::HashMap<String, PyObject>,
        mode: &PyExecutionMode,
    ) -> PyResult<PyObject> {
        // Convert inputs from Python dict to HashMap<String, Value>
        let mut input_map = std::collections::HashMap::new();
        for (key, value_py) in inputs {
            // Try to extract Value from PyObject
            let value = if let Ok(b) = value_py.extract::<bool>(py) {
                Value::Bool(b)
            } else if let Ok(i) = value_py.extract::<i32>(py) {
                Value::I32(i)
            } else if let Ok(u) = value_py.extract::<u32>(py) {
                Value::U32(u)
            } else if let Ok(f) = value_py.extract::<f32>(py) {
                Value::F32(f)
            } else if let Ok(s) = value_py.extract::<String>(py) {
                Value::String(s)
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "Unsupported input value type for key '{}'",
                    key
                )));
            };

            input_map.insert(key, value);
        }

        // Execute
        let result = self
            .inner
            .execute(input_map, mode.inner.clone())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

        // Convert result to Python dict
        let result_dict = pyo3::types::PyDict::new(py);

        // Add outputs
        let outputs_dict = pyo3::types::PyDict::new(py);
        for (name, value) in result.outputs {
            let py_value: PyObject = match value {
                Value::U32(v) => v.into_pyobject(py).unwrap().into(),
                Value::I32(v) => v.into_pyobject(py).unwrap().into(),
                Value::U64(v) => v.into_pyobject(py).unwrap().into(),
                Value::I64(v) => v.into_pyobject(py).unwrap().into(),
                Value::F32(v) => v.into_pyobject(py).unwrap().into(),
                Value::Bool(v) => v.into_pyobject(py).unwrap().as_any().clone().unbind(),
                Value::String(v) => v.into_pyobject(py).unwrap().into(),
                _ => "[Complex]".into_pyobject(py).unwrap().into(),
            };
            outputs_dict.set_item(name, py_value)?;
        }
        result_dict.set_item("outputs", outputs_dict)?;

        // Add ticks_elapsed
        result_dict.set_item("ticks_elapsed", result.ticks_elapsed)?;

        // Add condition_met
        result_dict.set_item("condition_met", result.condition_met)?;

        Ok(result_dict.into())
    }

    fn __repr__(&self) -> String {
        "TypedCircuitExecutor(...)".to_string()
    }
}

// --- SchematicBuilder Support ---

#[pyclass(name = "SchematicBuilder")]
pub struct PySchematicBuilder {
    inner: crate::SchematicBuilder,
}

#[pymethods]
impl PySchematicBuilder {
    #[new]
    fn new() -> Self {
        Self {
            inner: crate::SchematicBuilder::new(),
        }
    }

    /// Set the name of the schematic
    fn name<'py>(mut slf: PyRefMut<'py, Self>, name: String) -> PyRefMut<'py, Self> {
        let old_builder = std::mem::replace(&mut slf.inner, crate::SchematicBuilder::new());
        slf.inner = old_builder.name(name);
        slf
    }

    /// Map a character to a block string
    fn map<'py>(mut slf: PyRefMut<'py, Self>, ch: char, block: String) -> PyRefMut<'py, Self> {
        let old_builder = std::mem::replace(&mut slf.inner, crate::SchematicBuilder::new());
        slf.inner = old_builder.map(ch, &block);
        slf
    }

    /// Add multiple layers (list of list of strings)
    fn layers<'py>(mut slf: PyRefMut<'py, Self>, layers: Vec<Vec<String>>) -> PyRefMut<'py, Self> {
        // Convert Vec<Vec<String>> to Vec<&[&str]>
        let layer_refs: Vec<Vec<&str>> = layers
            .iter()
            .map(|layer| layer.iter().map(|s| s.as_str()).collect())
            .collect();
        let layer_slice_refs: Vec<&[&str]> = layer_refs.iter().map(|v| v.as_slice()).collect();
        let old_builder = std::mem::replace(&mut slf.inner, crate::SchematicBuilder::new());
        slf.inner = old_builder.layers(&layer_slice_refs);
        slf
    }

    /// Build the schematic
    fn build(mut slf: PyRefMut<'_, Self>) -> PyResult<PySchematic> {
        let builder = std::mem::replace(&mut slf.inner, crate::SchematicBuilder::new());
        let schematic = builder
            .build()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        Ok(PySchematic { inner: schematic })
    }

    /// Create from template string
    #[staticmethod]
    fn from_template(template: String) -> PyResult<PySchematicBuilder> {
        let builder = crate::SchematicBuilder::from_template(&template)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        Ok(Self { inner: builder })
    }
}

#[pymodule]
fn nucleation(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySchematic>()?;
    m.add_class::<PyBlockState>()?;
    m.add_class::<PySchematicBuilder>()?;
    m.add_function(wrap_pyfunction!(debug_schematic, m)?)?;
    m.add_function(wrap_pyfunction!(debug_json_schematic, m)?)?;
    m.add_function(wrap_pyfunction!(load_schematic, m)?)?;
    m.add_function(wrap_pyfunction!(save_schematic, m)?)?;

    #[cfg(feature = "simulation")]
    {
        m.add_class::<PyMchprsWorld>()?;
        m.add_class::<PyValue>()?;
        m.add_class::<PyIoType>()?;
        m.add_class::<PyLayoutFunction>()?;
        m.add_class::<PyOutputCondition>()?;
        m.add_class::<PyExecutionMode>()?;
        m.add_class::<PyIoLayoutBuilder>()?;
        m.add_class::<PyIoLayout>()?;
        m.add_class::<PyTypedCircuitExecutor>()?;
    }

    Ok(())
}
