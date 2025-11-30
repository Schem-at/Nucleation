//! Circuit Builder Python bindings
//!
//! Fluent API for creating TypedCircuitExecutor instances.

use pyo3::prelude::*;

use crate::simulation::typed_executor::{SortStrategy, StateMode};
use crate::simulation::CircuitBuilder;

use super::{PyDefinitionRegion, PyIoType, PyLayoutFunction, PySchematic, PyTypedCircuitExecutor};

// --- Sort Strategy ---

/// Sort strategy for ordering positions in IO layouts
///
/// Controls how positions are ordered when assigned to bits.
/// Position 0 corresponds to bit 0 (LSB), position 1 to bit 1, etc.
#[pyclass(name = "SortStrategy")]
#[derive(Clone)]
pub struct PySortStrategy {
    pub(crate) inner: SortStrategy,
}

#[pymethods]
impl PySortStrategy {
    // ========================================================================
    // Axis-first sorting (ascending)
    // ========================================================================

    /// Sort by Y first (ascending), then X, then Z
    /// Standard Minecraft layer-based ordering. This is the default.
    #[staticmethod]
    fn yxz() -> Self {
        Self {
            inner: SortStrategy::YXZ,
        }
    }

    /// Sort by X first (ascending), then Y, then Z
    #[staticmethod]
    fn xyz() -> Self {
        Self {
            inner: SortStrategy::XYZ,
        }
    }

    /// Sort by Z first (ascending), then Y, then X
    #[staticmethod]
    fn zyx() -> Self {
        Self {
            inner: SortStrategy::ZYX,
        }
    }

    // ========================================================================
    // Axis-first sorting (descending)
    // ========================================================================

    /// Sort by Y first (descending), then X ascending, then Z ascending
    #[staticmethod]
    fn y_desc_xz() -> Self {
        Self {
            inner: SortStrategy::YDescXZ,
        }
    }

    /// Sort by X first (descending), then Y ascending, then Z ascending
    #[staticmethod]
    fn x_desc_yz() -> Self {
        Self {
            inner: SortStrategy::XDescYZ,
        }
    }

    /// Sort by Z first (descending), then Y ascending, then X ascending
    #[staticmethod]
    fn z_desc_yx() -> Self {
        Self {
            inner: SortStrategy::ZDescYX,
        }
    }

    // ========================================================================
    // Fully descending
    // ========================================================================

    /// Sort by Y descending, then X descending, then Z descending
    #[staticmethod]
    fn descending() -> Self {
        Self {
            inner: SortStrategy::YXZDesc,
        }
    }

    // ========================================================================
    // Distance-based sorting
    // ========================================================================

    /// Sort by Euclidean distance from a reference point (ascending)
    /// Closest positions first. Useful for radial layouts.
    #[staticmethod]
    fn distance_from(x: i32, y: i32, z: i32) -> Self {
        Self {
            inner: SortStrategy::distance_from(x, y, z),
        }
    }

    /// Sort by Euclidean distance from a reference point (descending)
    /// Farthest positions first.
    #[staticmethod]
    fn distance_from_desc(x: i32, y: i32, z: i32) -> Self {
        Self {
            inner: SortStrategy::distance_from_desc(x, y, z),
        }
    }

    // ========================================================================
    // Special strategies
    // ========================================================================

    /// Preserve the order positions were added (no sorting)
    /// Useful when you've manually ordered positions or are using `from_bounding_boxes`
    /// where box order matters.
    #[staticmethod]
    fn preserve() -> Self {
        Self {
            inner: SortStrategy::Preserve,
        }
    }

    /// Reverse of whatever order positions were added
    #[staticmethod]
    fn reverse() -> Self {
        Self {
            inner: SortStrategy::Reverse,
        }
    }

    // ========================================================================
    // Parsing
    // ========================================================================

    /// Parse sort strategy from string
    ///
    /// Accepts: "yxz", "xyz", "zyx", "y_desc", "x_desc", "z_desc",
    ///          "descending", "preserve", "reverse"
    #[staticmethod]
    fn from_string(s: &str) -> PyResult<Self> {
        SortStrategy::from_str(s)
            .map(|inner| Self { inner })
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Unknown sort strategy: {}",
                    s
                ))
            })
    }

    /// Get the name of this strategy
    #[getter]
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    fn __repr__(&self) -> String {
        format!("SortStrategy.{}", self.inner.name())
    }
}

impl Default for PySortStrategy {
    fn default() -> Self {
        Self {
            inner: SortStrategy::default(),
        }
    }
}

/// CircuitBuilder wrapper for Python
/// Provides a fluent API for creating TypedCircuitExecutor instances
#[pyclass(name = "CircuitBuilder")]
pub struct PyCircuitBuilder {
    inner: Option<CircuitBuilder>,
}

impl PyCircuitBuilder {
    fn take_builder(&mut self) -> PyResult<CircuitBuilder> {
        self.inner.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "CircuitBuilder has already been consumed (did you call build() twice?)",
            )
        })
    }
}

#[pymethods]
impl PyCircuitBuilder {
    /// Create a new CircuitBuilder from a schematic
    #[new]
    fn new(schematic: &PySchematic) -> Self {
        Self {
            inner: Some(CircuitBuilder::new(schematic.inner.clone())),
        }
    }

    /// Create a CircuitBuilder from Insign annotations in the schematic
    #[staticmethod]
    fn from_insign(schematic: &PySchematic) -> PyResult<Self> {
        let builder = CircuitBuilder::from_insign(schematic.inner.clone())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        Ok(Self {
            inner: Some(builder),
        })
    }

    /// Add an input with explicit layout
    ///
    /// Uses the default sort strategy (YXZ - Y first, then X, then Z).
    /// For custom ordering, use `with_input_sorted`.
    fn with_input(
        &mut self,
        name: String,
        io_type: &PyIoType,
        layout: &PyLayoutFunction,
        region: &PyDefinitionRegion,
    ) -> PyResult<()> {
        let builder = self.take_builder()?;
        self.inner = Some(
            builder
                .with_input(
                    name,
                    io_type.inner.clone(),
                    layout.inner.clone(),
                    region.inner.clone(),
                )
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?,
        );
        Ok(())
    }

    /// Add an input with explicit layout and custom sort strategy
    fn with_input_sorted(
        &mut self,
        name: String,
        io_type: &PyIoType,
        layout: &PyLayoutFunction,
        region: &PyDefinitionRegion,
        sort: &PySortStrategy,
    ) -> PyResult<()> {
        let builder = self.take_builder()?;
        self.inner = Some(
            builder
                .with_input_sorted(
                    name,
                    io_type.inner.clone(),
                    layout.inner.clone(),
                    region.inner.clone(),
                    sort.inner.clone(),
                )
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?,
        );
        Ok(())
    }

    /// Add an input with auto-inferred layout
    ///
    /// Uses the default sort strategy (YXZ - Y first, then X, then Z).
    /// For custom ordering, use `with_input_auto_sorted`.
    fn with_input_auto(
        &mut self,
        name: String,
        io_type: &PyIoType,
        region: &PyDefinitionRegion,
    ) -> PyResult<()> {
        let builder = self.take_builder()?;
        self.inner = Some(
            builder
                .with_input_auto(name, io_type.inner.clone(), region.inner.clone())
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?,
        );
        Ok(())
    }

    /// Add an input with auto-inferred layout and custom sort strategy
    fn with_input_auto_sorted(
        &mut self,
        name: String,
        io_type: &PyIoType,
        region: &PyDefinitionRegion,
        sort: &PySortStrategy,
    ) -> PyResult<()> {
        let builder = self.take_builder()?;
        self.inner = Some(
            builder
                .with_input_auto_sorted(
                    name,
                    io_type.inner.clone(),
                    region.inner.clone(),
                    sort.inner.clone(),
                )
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?,
        );
        Ok(())
    }

    /// Add an output with explicit layout
    ///
    /// Uses the default sort strategy (YXZ - Y first, then X, then Z).
    /// For custom ordering, use `with_output_sorted`.
    fn with_output(
        &mut self,
        name: String,
        io_type: &PyIoType,
        layout: &PyLayoutFunction,
        region: &PyDefinitionRegion,
    ) -> PyResult<()> {
        let builder = self.take_builder()?;
        self.inner = Some(
            builder
                .with_output(
                    name,
                    io_type.inner.clone(),
                    layout.inner.clone(),
                    region.inner.clone(),
                )
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?,
        );
        Ok(())
    }

    /// Add an output with explicit layout and custom sort strategy
    fn with_output_sorted(
        &mut self,
        name: String,
        io_type: &PyIoType,
        layout: &PyLayoutFunction,
        region: &PyDefinitionRegion,
        sort: &PySortStrategy,
    ) -> PyResult<()> {
        let builder = self.take_builder()?;
        self.inner = Some(
            builder
                .with_output_sorted(
                    name,
                    io_type.inner.clone(),
                    layout.inner.clone(),
                    region.inner.clone(),
                    sort.inner.clone(),
                )
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?,
        );
        Ok(())
    }

    /// Add an output with auto-inferred layout
    ///
    /// Uses the default sort strategy (YXZ - Y first, then X, then Z).
    /// For custom ordering, use `with_output_auto_sorted`.
    fn with_output_auto(
        &mut self,
        name: String,
        io_type: &PyIoType,
        region: &PyDefinitionRegion,
    ) -> PyResult<()> {
        let builder = self.take_builder()?;
        self.inner = Some(
            builder
                .with_output_auto(name, io_type.inner.clone(), region.inner.clone())
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?,
        );
        Ok(())
    }

    /// Add an output with auto-inferred layout and custom sort strategy
    fn with_output_auto_sorted(
        &mut self,
        name: String,
        io_type: &PyIoType,
        region: &PyDefinitionRegion,
        sort: &PySortStrategy,
    ) -> PyResult<()> {
        let builder = self.take_builder()?;
        self.inner = Some(
            builder
                .with_output_auto_sorted(
                    name,
                    io_type.inner.clone(),
                    region.inner.clone(),
                    sort.inner.clone(),
                )
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?,
        );
        Ok(())
    }

    /// Set state mode
    fn with_state_mode(&mut self, mode: &str) -> PyResult<()> {
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
        let builder = self.take_builder()?;
        self.inner = Some(builder.with_state_mode(state_mode));
        Ok(())
    }

    /// Validate the builder configuration
    fn validate(&self) -> PyResult<()> {
        if let Some(ref builder) = self.inner {
            builder
                .validate()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            Ok(())
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "CircuitBuilder has already been consumed",
            ))
        }
    }

    /// Build the TypedCircuitExecutor
    fn build(&mut self) -> PyResult<PyTypedCircuitExecutor> {
        let builder = self.take_builder()?;
        let executor = builder
            .build()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
        Ok(PyTypedCircuitExecutor { inner: executor })
    }

    /// Validate and build the TypedCircuitExecutor
    fn build_validated(&mut self) -> PyResult<PyTypedCircuitExecutor> {
        let builder = self.take_builder()?;
        let executor = builder
            .build_validated()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
        Ok(PyTypedCircuitExecutor { inner: executor })
    }

    /// Get the number of inputs defined
    fn input_count(&self) -> usize {
        self.inner.as_ref().map_or(0, |b| b.input_count())
    }

    /// Get the number of outputs defined
    fn output_count(&self) -> usize {
        self.inner.as_ref().map_or(0, |b| b.output_count())
    }

    /// Get input names
    fn input_names(&self) -> Vec<String> {
        self.inner.as_ref().map_or(vec![], |b| {
            b.input_names().into_iter().map(|s| s.to_string()).collect()
        })
    }

    /// Get output names
    fn output_names(&self) -> Vec<String> {
        self.inner.as_ref().map_or(vec![], |b| {
            b.output_names()
                .into_iter()
                .map(|s| s.to_string())
                .collect()
        })
    }

    fn __repr__(&self) -> String {
        if let Some(ref builder) = self.inner {
            format!(
                "CircuitBuilder(inputs={}, outputs={})",
                builder.input_count(),
                builder.output_count()
            )
        } else {
            "CircuitBuilder(consumed)".to_string()
        }
    }
}
