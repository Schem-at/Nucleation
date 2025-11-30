//! Simulation Python bindings
//!
//! MCHPRS simulation wrapper: world creation, ticking, signal manipulation.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use super::PySchematic;
use crate::simulation::{BlockPos, MchprsWorld};

#[pyclass(name = "MchprsWorld")]
pub struct PyMchprsWorld {
    pub(crate) inner: MchprsWorld,
}

impl PyMchprsWorld {
    /// Extract the inner MchprsWorld, consuming self
    /// This is used internally by from_layout to take ownership
    pub fn into_inner(self) -> MchprsWorld {
        self.inner
    }
}

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
