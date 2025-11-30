//! Circuit Builder Python bindings
//!
//! Fluent API for creating TypedCircuitExecutor instances.

use pyo3::prelude::*;

use crate::simulation::typed_executor::StateMode;
use crate::simulation::CircuitBuilder;
use crate::UniversalSchematic;

use super::{PyDefinitionRegion, PyIoType, PyLayoutFunction, PySchematic, PyTypedCircuitExecutor};

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

    /// Add an input with auto-inferred layout
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

    /// Add an output with explicit layout
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

    /// Add an output with auto-inferred layout
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
