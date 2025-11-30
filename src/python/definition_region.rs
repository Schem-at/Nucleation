//! DefinitionRegion Python bindings
//!
//! Region manipulation for circuit IO definitions.

use crate::definition_region::DefinitionRegion;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;

use super::PySchematic;

/// DefinitionRegion wrapper for Python
/// Represents a logical region defined by one or more bounding boxes.
#[pyclass(name = "DefinitionRegion")]
#[derive(Clone)]
pub struct PyDefinitionRegion {
    pub(crate) inner: DefinitionRegion,
}

#[pymethods]
impl PyDefinitionRegion {
    /// Create a new empty DefinitionRegion
    #[new]
    fn new() -> Self {
        Self {
            inner: DefinitionRegion::new(),
        }
    }

    /// Create a DefinitionRegion from bounding box coordinates
    #[staticmethod]
    fn from_bounds(min: (i32, i32, i32), max: (i32, i32, i32)) -> Self {
        Self {
            inner: DefinitionRegion::from_bounds(min, max),
        }
    }

    /// Add a bounding box to the region
    fn add_bounds(&mut self, min: (i32, i32, i32), max: (i32, i32, i32)) {
        self.inner.add_bounds(min, max);
    }

    /// Add a single point to the region
    fn add_point(&mut self, x: i32, y: i32, z: i32) {
        self.inner.add_point(x, y, z);
    }

    /// Set metadata on the region (returns new instance for chaining)
    fn with_metadata(&self, key: String, value: String) -> Self {
        Self {
            inner: self.inner.clone().with_metadata(key, value),
        }
    }

    /// Merge another region into this one
    fn merge(&mut self, other: &PyDefinitionRegion) {
        self.inner.merge(&other.inner);
    }

    // ========================================================================
    // Boolean Operations (Mutating)
    // ========================================================================

    /// Subtract another region from this one (removes points present in `other`)
    fn subtract(&mut self, other: &PyDefinitionRegion) {
        self.inner.subtract(&other.inner);
    }

    /// Keep only points present in both regions (intersection)
    fn intersect(&mut self, other: &PyDefinitionRegion) {
        self.inner.intersect(&other.inner);
    }

    /// Add all points from another region to this one (mutating union)
    fn union_into(&mut self, other: &PyDefinitionRegion) {
        self.inner.union_into(&other.inner);
    }

    // ========================================================================
    // Boolean Operations (Immutable)
    // ========================================================================

    /// Create a new region that is the union of this region and another
    fn union(&self, other: &PyDefinitionRegion) -> Self {
        Self {
            inner: self.inner.union(&other.inner),
        }
    }

    /// Create a new region with points from `other` removed (immutable)
    fn subtracted(&self, other: &PyDefinitionRegion) -> Self {
        Self {
            inner: self.inner.subtracted(&other.inner),
        }
    }

    /// Create a new region with only points in both (immutable)
    fn intersected(&self, other: &PyDefinitionRegion) -> Self {
        Self {
            inner: self.inner.intersected(&other.inner),
        }
    }

    // ========================================================================
    // Geometric Transformations
    // ========================================================================

    /// Translate all boxes by the given offset
    fn shift(&mut self, x: i32, y: i32, z: i32) {
        self.inner.shift(x, y, z);
    }

    /// Expand all boxes by the given amounts in each direction
    fn expand(&mut self, x: i32, y: i32, z: i32) {
        self.inner.expand(x, y, z);
    }

    /// Contract all boxes by the given amount uniformly
    fn contract(&mut self, amount: i32) {
        self.inner.contract(amount);
    }

    /// Get the overall bounding box encompassing all boxes in this region
    /// Returns a dict with {min: (x,y,z), max: (x,y,z)} or None if empty
    fn get_bounds(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        match self.inner.get_bounds() {
            Some(bbox) => {
                let dict = PyDict::new(py);
                dict.set_item("min", (bbox.min.0, bbox.min.1, bbox.min.2))?;
                dict.set_item("max", (bbox.max.0, bbox.max.1, bbox.max.2))?;
                Ok(Some(dict.into()))
            }
            None => Ok(None),
        }
    }

    // ========================================================================
    // Connectivity Analysis
    // ========================================================================

    /// Check if all points in the region are connected (6-connectivity)
    fn is_contiguous(&self) -> bool {
        self.inner.is_contiguous()
    }

    /// Get the number of connected components in this region
    fn connected_components(&self) -> usize {
        self.inner.connected_components()
    }

    // ========================================================================
    // Filtering
    // ========================================================================

    /// Filter positions by block name (substring match)
    fn filter_by_block(&self, schematic: &PySchematic, block_name: &str) -> Self {
        Self {
            inner: self.inner.filter_by_block(&schematic.inner, block_name),
        }
    }

    /// Filter positions by block state properties
    /// Only keeps positions where the block has ALL specified properties matching
    fn filter_by_properties(
        &self,
        schematic: &PySchematic,
        properties: HashMap<String, String>,
    ) -> Self {
        Self {
            inner: self
                .inner
                .filter_by_properties(&schematic.inner, &properties),
        }
    }

    // ========================================================================
    // Utility Methods
    // ========================================================================

    /// Check if the region is empty
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Check if the region contains a specific point
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        self.inner.contains(x, y, z)
    }

    /// Get total volume (number of blocks) covered by all boxes
    fn volume(&self) -> u32 {
        self.inner.volume() as u32
    }

    /// Get a list of all positions as a list of (x, y, z) tuples
    fn positions(&self) -> Vec<(i32, i32, i32)> {
        self.inner.iter_positions().collect()
    }

    /// Get positions in globally sorted order (Y, then X, then Z)
    ///
    /// This provides **deterministic bit ordering** for circuits regardless of
    /// how the region was constructed. Use this for IO bit assignment.
    fn positions_sorted(&self) -> Vec<(i32, i32, i32)> {
        self.inner.iter_positions_sorted()
    }

    /// Simplify the region by merging adjacent/overlapping boxes
    fn simplify(&mut self) {
        self.inner.simplify();
    }

    // ========================================================================
    // Box Access (for Rendering)
    // ========================================================================

    /// Create a DefinitionRegion from multiple bounding boxes
    ///
    /// Unlike from_positions() which takes individual points and merges them,
    /// this takes pre-defined bounding boxes directly as a list of ((min_x, min_y, min_z), (max_x, max_y, max_z)) tuples.
    #[staticmethod]
    fn from_bounding_boxes(boxes: Vec<((i32, i32, i32), (i32, i32, i32))>) -> Self {
        Self {
            inner: DefinitionRegion::from_bounding_boxes(boxes),
        }
    }

    /// Create a DefinitionRegion from an array of positions
    ///
    /// Takes a list of (x, y, z) tuples. Adjacent points will be merged into boxes.
    #[staticmethod]
    fn from_positions(positions: Vec<(i32, i32, i32)>) -> Self {
        Self {
            inner: DefinitionRegion::from_positions(&positions),
        }
    }

    /// Get the number of bounding boxes in this region
    fn box_count(&self) -> usize {
        self.inner.box_count()
    }

    /// Get a specific bounding box by index
    ///
    /// Returns ((min_x, min_y, min_z), (max_x, max_y, max_z)) or None if index is out of bounds
    fn get_box(&self, index: usize) -> Option<((i32, i32, i32), (i32, i32, i32))> {
        self.inner.get_box(index)
    }

    /// Get all bounding boxes in this region
    ///
    /// Returns a list of ((min_x, min_y, min_z), (max_x, max_y, max_z)) tuples.
    /// Useful for rendering each box separately.
    fn get_boxes(&self) -> Vec<((i32, i32, i32), (i32, i32, i32))> {
        self.inner.get_boxes()
    }

    // ========================================================================
    // Metadata Access
    // ========================================================================

    /// Get a metadata value by key
    ///
    /// Returns the value string or None if not found
    fn get_metadata(&self, key: &str) -> Option<String> {
        self.inner.get_metadata(key).cloned()
    }

    /// Set a metadata value (mutating version)
    fn set_metadata(&mut self, key: String, value: String) {
        self.inner.set_metadata(key, value);
    }

    /// Get all metadata as a dictionary
    fn get_all_metadata(&self) -> HashMap<String, String> {
        self.inner.metadata_ref().clone()
    }

    /// Get all metadata keys
    fn metadata_keys(&self) -> Vec<String> {
        self.inner.metadata_keys().into_iter().cloned().collect()
    }

    // ========================================================================
    // Geometry Helpers (for Rendering)
    // ========================================================================

    /// Get the dimensions (width, height, length) of the overall bounding box
    ///
    /// Returns (0, 0, 0) if empty
    fn dimensions(&self) -> (i32, i32, i32) {
        self.inner.dimensions()
    }

    /// Get the center point of the region (integer coordinates)
    ///
    /// Returns None if empty
    fn center(&self) -> Option<(i32, i32, i32)> {
        self.inner.center()
    }

    /// Get the center point of the region as floats (for rendering)
    ///
    /// Returns None if empty
    fn center_f32(&self) -> Option<(f32, f32, f32)> {
        self.inner.center_f32()
    }

    /// Check if this region intersects with a bounding box
    ///
    /// Useful for frustum culling in renderers.
    fn intersects_bounds(&self, min: (i32, i32, i32), max: (i32, i32, i32)) -> bool {
        self.inner.intersects_bounds(min, max)
    }

    // ========================================================================
    // Immutable Geometric Transformations
    // ========================================================================

    /// Create a new region shifted by the given offset (immutable)
    fn shifted(&self, x: i32, y: i32, z: i32) -> Self {
        Self {
            inner: self.inner.shifted(x, y, z),
        }
    }

    /// Create a new region expanded by the given amounts (immutable)
    fn expanded(&self, x: i32, y: i32, z: i32) -> Self {
        Self {
            inner: self.inner.expanded(x, y, z),
        }
    }

    /// Create a new region contracted by the given amount (immutable)
    fn contracted(&self, amount: i32) -> Self {
        Self {
            inner: self.inner.contracted(amount),
        }
    }

    /// Create a deep copy of this region
    fn copy(&self) -> Self {
        Self {
            inner: self.inner.copy(),
        }
    }

    // ========================================================================
    // Python Special Methods
    // ========================================================================

    fn __repr__(&self) -> String {
        let bounds = self.inner.get_bounds();
        match bounds {
            Some(bbox) => format!(
                "<DefinitionRegion {} points, {} boxes, bounds=({},{},{}) to ({},{},{})>",
                self.inner.volume(),
                self.inner.box_count(),
                bbox.min.0,
                bbox.min.1,
                bbox.min.2,
                bbox.max.0,
                bbox.max.1,
                bbox.max.2
            ),
            None => "<DefinitionRegion empty>".to_string(),
        }
    }

    fn __len__(&self) -> usize {
        self.inner.volume() as usize
    }

    fn __bool__(&self) -> bool {
        !self.inner.is_empty()
    }

    fn __copy__(&self) -> Self {
        self.copy()
    }

    fn __deepcopy__(&self, _memo: &Bound<'_, PyDict>) -> Self {
        self.copy()
    }
}

impl Default for PyDefinitionRegion {
    fn default() -> Self {
        Self::new()
    }
}
