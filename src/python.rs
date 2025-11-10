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

#[pymodule]
fn nucleation(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySchematic>()?;
    m.add_class::<PyBlockState>()?;
    m.add_function(wrap_pyfunction!(debug_schematic, m)?)?;
    m.add_function(wrap_pyfunction!(debug_json_schematic, m)?)?;
    m.add_function(wrap_pyfunction!(load_schematic, m)?)?;
    m.add_function(wrap_pyfunction!(save_schematic, m)?)?;

    #[cfg(feature = "simulation")]
    m.add_class::<PyMchprsWorld>()?;

    Ok(())
}
