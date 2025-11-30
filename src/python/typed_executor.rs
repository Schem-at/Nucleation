//! Typed Circuit Executor Python bindings
//!
//! High-level circuit execution with typed inputs/outputs.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

use crate::simulation::typed_executor::{
    ExecutionMode, IoLayout, IoLayoutBuilder, IoType, LayoutFunction, OutputCondition, StateMode,
    TypedCircuitExecutor, Value,
};
use crate::simulation::MchprsWorld;

use super::{PyDefinitionRegion, PyMchprsWorld, PySchematic};

/// Python-compatible Value wrapper
#[pyclass(name = "Value")]
#[derive(Clone)]
pub struct PyValue {
    pub(crate) inner: Value,
}

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
#[pyclass(name = "IoType")]
#[derive(Clone)]
pub struct PyIoType {
    pub(crate) inner: IoType,
}

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
#[pyclass(name = "LayoutFunction")]
#[derive(Clone)]
pub struct PyLayoutFunction {
    pub(crate) inner: LayoutFunction,
}

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
#[pyclass(name = "OutputCondition")]
#[derive(Clone)]
pub struct PyOutputCondition {
    pub(crate) inner: OutputCondition,
}

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
#[pyclass(name = "ExecutionMode")]
#[derive(Clone)]
pub struct PyExecutionMode {
    pub(crate) inner: ExecutionMode,
}

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
#[pyclass(name = "IoLayoutBuilder")]
pub struct PyIoLayoutBuilder {
    pub(crate) inner: IoLayoutBuilder,
}

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

    /// Add an input from a DefinitionRegion
    fn add_input_from_region<'py>(
        mut slf: PyRefMut<'py, Self>,
        name: String,
        io_type: &PyIoType,
        layout: &PyLayoutFunction,
        region: &PyDefinitionRegion,
    ) -> PyResult<PyRefMut<'py, Self>> {
        slf.inner = slf
            .inner
            .clone()
            .add_input_from_region(
                name,
                io_type.inner.clone(),
                layout.inner.clone(),
                region.inner.clone(),
            )
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        Ok(slf)
    }

    /// Add an input from a DefinitionRegion with automatic layout inference
    fn add_input_from_region_auto<'py>(
        mut slf: PyRefMut<'py, Self>,
        name: String,
        io_type: &PyIoType,
        region: &PyDefinitionRegion,
    ) -> PyResult<PyRefMut<'py, Self>> {
        slf.inner = slf
            .inner
            .clone()
            .add_input_from_region_auto(name, io_type.inner.clone(), region.inner.clone())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        Ok(slf)
    }

    /// Add an output from a DefinitionRegion
    fn add_output_from_region<'py>(
        mut slf: PyRefMut<'py, Self>,
        name: String,
        io_type: &PyIoType,
        layout: &PyLayoutFunction,
        region: &PyDefinitionRegion,
    ) -> PyResult<PyRefMut<'py, Self>> {
        slf.inner = slf
            .inner
            .clone()
            .add_output_from_region(
                name,
                io_type.inner.clone(),
                layout.inner.clone(),
                region.inner.clone(),
            )
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        Ok(slf)
    }

    /// Add an output from a DefinitionRegion with automatic layout inference
    fn add_output_from_region_auto<'py>(
        mut slf: PyRefMut<'py, Self>,
        name: String,
        io_type: &PyIoType,
        region: &PyDefinitionRegion,
    ) -> PyResult<PyRefMut<'py, Self>> {
        slf.inner = slf
            .inner
            .clone()
            .add_output_from_region_auto(name, io_type.inner.clone(), region.inner.clone())
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
#[pyclass(name = "IoLayout")]
#[derive(Clone)]
pub struct PyIoLayout {
    pub(crate) inner: IoLayout,
}

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
#[pyclass(name = "TypedCircuitExecutor")]
pub struct PyTypedCircuitExecutor {
    pub(crate) inner: TypedCircuitExecutor,
}

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

    // ========================================================================
    // Manual Control Methods (NEW - Feature Parity with WASM)
    // ========================================================================

    /// Advance the simulation by the specified number of ticks (manual control)
    fn tick(&mut self, ticks: u32) {
        self.inner.tick(ticks);
    }

    /// Propagate all pending changes (manual control)
    fn flush(&mut self) {
        self.inner.flush();
    }

    /// Set a single input value by name
    fn set_input(&mut self, name: &str, value: &PyValue) -> PyResult<()> {
        self.inner
            .set_input(name, &value.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
    }

    /// Read a single output value by name
    fn read_output(&mut self, py: Python, name: &str) -> PyResult<PyObject> {
        let value = self
            .inner
            .read_output(name)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

        // Convert Value to Python object
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

        Ok(py_value)
    }

    /// Get all input names
    fn input_names(&self) -> Vec<String> {
        self.inner
            .input_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get all output names
    fn output_names(&self) -> Vec<String> {
        self.inner
            .output_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get detailed layout information for debugging
    /// Returns a dict with 'inputs' and 'outputs' keys, each containing
    /// detailed information about the IO layout including bit positions
    fn get_layout_info(&self, py: Python) -> PyResult<PyObject> {
        let layout_info = self.inner.get_layout_info();

        let result = PyDict::new(py);

        // Inputs
        let inputs_dict = PyDict::new(py);
        for (name, info) in layout_info.inputs {
            let info_dict = PyDict::new(py);
            info_dict.set_item("io_type", &info.io_type)?;
            info_dict.set_item("bit_count", info.bit_count)?;
            info_dict.set_item("positions", info.positions)?;
            inputs_dict.set_item(name, info_dict)?;
        }
        result.set_item("inputs", inputs_dict)?;

        // Outputs
        let outputs_dict = PyDict::new(py);
        for (name, info) in layout_info.outputs {
            let info_dict = PyDict::new(py);
            info_dict.set_item("io_type", &info.io_type)?;
            info_dict.set_item("bit_count", info.bit_count)?;
            info_dict.set_item("positions", info.positions)?;
            outputs_dict.set_item(name, info_dict)?;
        }
        result.set_item("outputs", outputs_dict)?;

        Ok(result.into())
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
        format!(
            "TypedCircuitExecutor(inputs={}, outputs={})",
            self.inner.input_names().len(),
            self.inner.output_names().len()
        )
    }
}
