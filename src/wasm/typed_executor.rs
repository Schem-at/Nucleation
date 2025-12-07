//! Typed Circuit Executor WASM bindings
//!
//! High-level circuit execution with typed inputs/outputs.

use super::{
    DefinitionRegionWrapper, MchprsWorldWrapper, SchematicWrapper, SimulationOptionsWrapper,
};
use crate::block_position::BlockPosition;
use crate::simulation::typed_executor::{
    ExecutionMode, IoLayout, IoLayoutBuilder, IoMapping, IoType, LayoutFunction, OutputCondition,
    StateMode, TypedCircuitExecutor, Value,
};
use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;

#[cfg(feature = "simulation")]
#[wasm_bindgen]
#[derive(Clone)]
pub struct ValueWrapper {
    inner: Value,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl ValueWrapper {
    /// Create a U32 value
    #[wasm_bindgen(js_name = fromU32)]
    pub fn from_u32(value: u32) -> Self {
        Self {
            inner: Value::U32(value),
        }
    }

    /// Create an I32 value
    #[wasm_bindgen(js_name = fromI32)]
    pub fn from_i32(value: i32) -> Self {
        Self {
            inner: Value::I32(value),
        }
    }

    /// Create an F32 value
    #[wasm_bindgen(js_name = fromF32)]
    pub fn from_f32(value: f32) -> Self {
        Self {
            inner: Value::F32(value),
        }
    }

    /// Create a Bool value
    #[wasm_bindgen(js_name = fromBool)]
    pub fn from_bool(value: bool) -> Self {
        Self {
            inner: Value::Bool(value),
        }
    }

    /// Create a String value
    #[wasm_bindgen(js_name = fromString)]
    pub fn from_string(value: String) -> Self {
        Self {
            inner: Value::String(value),
        }
    }

    /// Convert to JavaScript value
    #[wasm_bindgen(js_name = toJs)]
    pub fn to_js(&self) -> JsValue {
        match &self.inner {
            Value::U32(v) => JsValue::from_f64(*v as f64),
            Value::I32(v) => JsValue::from_f64(*v as f64),
            Value::U64(v) => JsValue::from_f64(*v as f64),
            Value::I64(v) => JsValue::from_f64(*v as f64),
            Value::F32(v) => JsValue::from_f64(*v as f64),
            Value::Bool(v) => JsValue::from_bool(*v),
            Value::String(v) => JsValue::from_str(v),
            Value::Array(arr) => {
                let js_arr = Array::new();
                for val in arr {
                    let wrapper = ValueWrapper { inner: val.clone() };
                    js_arr.push(&wrapper.to_js());
                }
                js_arr.into()
            }
            Value::Struct(fields) => {
                let obj = Object::new();
                for (key, val) in fields {
                    let wrapper = ValueWrapper { inner: val.clone() };
                    Reflect::set(&obj, &JsValue::from_str(key), &wrapper.to_js()).unwrap();
                }
                obj.into()
            }
            Value::BitArray(bits) => {
                let js_arr = Array::new();
                for bit in bits {
                    js_arr.push(&JsValue::from_bool(*bit));
                }
                js_arr.into()
            }
            Value::Bytes(bytes) => {
                let js_arr = Array::new();
                for byte in bytes {
                    js_arr.push(&JsValue::from_f64(*byte as f64));
                }
                js_arr.into()
            }
        }
    }

    /// Get type name
    #[wasm_bindgen(js_name = typeName)]
    pub fn type_name(&self) -> String {
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
}

/// IoType builder for JavaScript
#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct IoTypeWrapper {
    pub(crate) inner: IoType,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl IoTypeWrapper {
    /// Create an unsigned integer type
    #[wasm_bindgen(js_name = unsignedInt)]
    pub fn unsigned_int(bits: usize) -> Self {
        Self {
            inner: IoType::UnsignedInt { bits },
        }
    }

    /// Create a signed integer type
    #[wasm_bindgen(js_name = signedInt)]
    pub fn signed_int(bits: usize) -> Self {
        Self {
            inner: IoType::SignedInt { bits },
        }
    }

    /// Create a Float32 type
    #[wasm_bindgen(js_name = float32)]
    pub fn float32() -> Self {
        Self {
            inner: IoType::Float32,
        }
    }

    /// Create a Boolean type
    #[wasm_bindgen(js_name = boolean)]
    pub fn boolean() -> Self {
        Self {
            inner: IoType::Boolean,
        }
    }

    /// Create an ASCII string type
    #[wasm_bindgen(js_name = ascii)]
    pub fn ascii(chars: usize) -> Self {
        Self {
            inner: IoType::Ascii { chars },
        }
    }
}

/// LayoutFunction builder for JavaScript
#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct LayoutFunctionWrapper {
    pub(crate) inner: LayoutFunction,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl LayoutFunctionWrapper {
    /// One bit per position (0 or 15)
    #[wasm_bindgen(js_name = oneToOne)]
    pub fn one_to_one() -> Self {
        Self {
            inner: LayoutFunction::OneToOne,
        }
    }

    /// Four bits per position (0-15)
    #[wasm_bindgen(js_name = packed4)]
    pub fn packed4() -> Self {
        Self {
            inner: LayoutFunction::Packed4,
        }
    }

    /// Custom bit-to-position mapping
    #[wasm_bindgen(js_name = custom)]
    pub fn custom(mapping: Vec<usize>) -> Self {
        Self {
            inner: LayoutFunction::Custom(mapping),
        }
    }

    /// Row-major 2D layout
    #[wasm_bindgen(js_name = rowMajor)]
    pub fn row_major(rows: usize, cols: usize, bits_per_element: usize) -> Self {
        Self {
            inner: LayoutFunction::RowMajor {
                rows,
                cols,
                bits_per_element,
            },
        }
    }

    /// Column-major 2D layout
    #[wasm_bindgen(js_name = columnMajor)]
    pub fn column_major(rows: usize, cols: usize, bits_per_element: usize) -> Self {
        Self {
            inner: LayoutFunction::ColumnMajor {
                rows,
                cols,
                bits_per_element,
            },
        }
    }

    /// Scanline layout for screens
    #[wasm_bindgen(js_name = scanline)]
    pub fn scanline(width: usize, height: usize, bits_per_pixel: usize) -> Self {
        Self {
            inner: LayoutFunction::Scanline {
                width,
                height,
                bits_per_pixel,
            },
        }
    }
}

/// OutputCondition for conditional execution
#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct OutputConditionWrapper {
    inner: OutputCondition,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl OutputConditionWrapper {
    /// Output equals a value
    #[wasm_bindgen(js_name = equals)]
    pub fn equals(value: &ValueWrapper) -> Self {
        Self {
            inner: OutputCondition::Equals(value.inner.clone()),
        }
    }

    /// Output not equals a value
    #[wasm_bindgen(js_name = notEquals)]
    pub fn not_equals(value: &ValueWrapper) -> Self {
        Self {
            inner: OutputCondition::NotEquals(value.inner.clone()),
        }
    }

    /// Output greater than a value
    #[wasm_bindgen(js_name = greaterThan)]
    pub fn greater_than(value: &ValueWrapper) -> Self {
        Self {
            inner: OutputCondition::GreaterThan(value.inner.clone()),
        }
    }

    /// Output less than a value
    #[wasm_bindgen(js_name = lessThan)]
    pub fn less_than(value: &ValueWrapper) -> Self {
        Self {
            inner: OutputCondition::LessThan(value.inner.clone()),
        }
    }

    /// Bitwise AND with mask
    #[wasm_bindgen(js_name = bitwiseAnd)]
    pub fn bitwise_and(mask: u32) -> Self {
        Self {
            inner: OutputCondition::BitwiseAnd(mask as u64),
        }
    }
}

/// ExecutionMode for circuit execution
#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct ExecutionModeWrapper {
    inner: ExecutionMode,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl ExecutionModeWrapper {
    /// Run for a fixed number of ticks
    #[wasm_bindgen(js_name = fixedTicks)]
    pub fn fixed_ticks(ticks: u32) -> Self {
        Self {
            inner: ExecutionMode::FixedTicks { ticks },
        }
    }

    /// Run until an output meets a condition
    #[wasm_bindgen(js_name = untilCondition)]
    pub fn until_condition(
        output_name: String,
        condition: &OutputConditionWrapper,
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
    #[wasm_bindgen(js_name = untilChange)]
    pub fn until_change(max_ticks: u32, check_interval: u32) -> Self {
        Self {
            inner: ExecutionMode::UntilChange {
                max_ticks,
                check_interval,
            },
        }
    }

    /// Run until outputs are stable
    #[wasm_bindgen(js_name = untilStable)]
    pub fn until_stable(stable_ticks: u32, max_ticks: u32) -> Self {
        Self {
            inner: ExecutionMode::UntilStable {
                stable_ticks,
                max_ticks,
            },
        }
    }
}

/// IoLayoutBuilder for JavaScript
#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct IoLayoutBuilderWrapper {
    inner: IoLayoutBuilder,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl IoLayoutBuilderWrapper {
    /// Create a new IO layout builder
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: IoLayoutBuilder::new(),
        }
    }

    /// Add an input
    #[wasm_bindgen(js_name = addInput)]
    pub fn add_input(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        positions: Vec<JsValue>,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        // Convert positions from JsValue array to Vec<(i32, i32, i32)>
        let mut pos_vec = Vec::new();
        for pos in positions {
            let array = js_sys::Array::from(&pos);
            if array.length() != 3 {
                return Err(JsValue::from_str("Position must be [x, y, z]"));
            }
            let x = array.get(0).as_f64().ok_or("Invalid x")? as i32;
            let y = array.get(1).as_f64().ok_or("Invalid y")? as i32;
            let z = array.get(2).as_f64().ok_or("Invalid z")? as i32;
            pos_vec.push((x, y, z));
        }

        self.inner = self
            .inner
            .add_input(name, io_type.inner.clone(), layout.inner.clone(), pos_vec)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(self)
    }

    /// Add an output
    #[wasm_bindgen(js_name = addOutput)]
    pub fn add_output(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        positions: Vec<JsValue>,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        // Convert positions
        let mut pos_vec = Vec::new();
        for pos in positions {
            let array = js_sys::Array::from(&pos);
            if array.length() != 3 {
                return Err(JsValue::from_str("Position must be [x, y, z]"));
            }
            let x = array.get(0).as_f64().ok_or("Invalid x")? as i32;
            let y = array.get(1).as_f64().ok_or("Invalid y")? as i32;
            let z = array.get(2).as_f64().ok_or("Invalid z")? as i32;
            pos_vec.push((x, y, z));
        }

        self.inner = self
            .inner
            .add_output(name, io_type.inner.clone(), layout.inner.clone(), pos_vec)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(self)
    }

    /// Add an input with automatic layout inference
    #[wasm_bindgen(js_name = addInputAuto)]
    pub fn add_input_auto(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        positions: Vec<JsValue>,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        // Convert positions
        let mut pos_vec = Vec::new();
        for pos in positions {
            let array = js_sys::Array::from(&pos);
            if array.length() != 3 {
                return Err(JsValue::from_str("Position must be [x, y, z]"));
            }
            let x = array.get(0).as_f64().ok_or("Invalid x")? as i32;
            let y = array.get(1).as_f64().ok_or("Invalid y")? as i32;
            let z = array.get(2).as_f64().ok_or("Invalid z")? as i32;
            pos_vec.push((x, y, z));
        }

        self.inner = self
            .inner
            .add_input_auto(name, io_type.inner.clone(), pos_vec)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(self)
    }

    /// Add an input defined by a region (bounding box)
    /// Iterates Y (layers), then X (rows), then Z (columns)
    #[wasm_bindgen(js_name = addInputRegion)]
    pub fn add_input_region(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        min: BlockPosition,
        max: BlockPosition,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        let mut positions = Vec::new();

        let min_x = std::cmp::min(min.x, max.x);
        let max_x = std::cmp::max(min.x, max.x);
        let min_y = std::cmp::min(min.y, max.y);
        let max_y = std::cmp::max(min.y, max.y);
        let min_z = std::cmp::min(min.z, max.z);
        let max_z = std::cmp::max(min.z, max.z);

        // Standard redstone order: Y-axis first (layers), then X (rows), then Z (columns)
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                for z in min_z..=max_z {
                    positions.push((x, y, z));
                }
            }
        }

        self.inner = self
            .inner
            .add_input(name, io_type.inner.clone(), layout.inner.clone(), positions)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(self)
    }

    /// Add an input defined by a DefinitionRegion
    #[wasm_bindgen(js_name = addInputFromRegion)]
    pub fn add_input_from_region(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        region: &DefinitionRegionWrapper,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .add_input_from_region(
                name,
                io_type.inner.clone(),
                layout.inner.clone(),
                region.inner.clone(),
            )
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Add an input defined by a region with automatic layout inference
    #[wasm_bindgen(js_name = addInputRegionAuto)]
    pub fn add_input_region_auto(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        min: BlockPosition,
        max: BlockPosition,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        let mut positions = Vec::new();

        let min_x = std::cmp::min(min.x, max.x);
        let max_x = std::cmp::max(min.x, max.x);
        let min_y = std::cmp::min(min.y, max.y);
        let max_y = std::cmp::max(min.y, max.y);
        let min_z = std::cmp::min(min.z, max.z);
        let max_z = std::cmp::max(min.z, max.z);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                for z in min_z..=max_z {
                    positions.push((x, y, z));
                }
            }
        }

        self.inner = self
            .inner
            .add_input_auto(name, io_type.inner.clone(), positions)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(self)
    }

    /// Add an input defined by a DefinitionRegion with automatic layout inference
    #[wasm_bindgen(js_name = addInputFromRegionAuto)]
    pub fn add_input_from_region_auto(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        region: &DefinitionRegionWrapper,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .add_input_from_region_auto(name, io_type.inner.clone(), region.inner.clone())
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Add an output with automatic layout inference
    #[wasm_bindgen(js_name = addOutputAuto)]
    pub fn add_output_auto(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        positions: Vec<JsValue>,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        // Convert positions
        let mut pos_vec = Vec::new();
        for pos in positions {
            let array = js_sys::Array::from(&pos);
            if array.length() != 3 {
                return Err(JsValue::from_str("Position must be [x, y, z]"));
            }
            let x = array.get(0).as_f64().ok_or("Invalid x")? as i32;
            let y = array.get(1).as_f64().ok_or("Invalid y")? as i32;
            let z = array.get(2).as_f64().ok_or("Invalid z")? as i32;
            pos_vec.push((x, y, z));
        }

        self.inner = self
            .inner
            .add_output_auto(name, io_type.inner.clone(), pos_vec)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(self)
    }

    /// Add an output defined by a region (bounding box)
    /// Iterates Y (layers), then X (rows), then Z (columns)
    #[wasm_bindgen(js_name = addOutputRegion)]
    pub fn add_output_region(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        min: BlockPosition,
        max: BlockPosition,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        let mut positions = Vec::new();

        let min_x = std::cmp::min(min.x, max.x);
        let max_x = std::cmp::max(min.x, max.x);
        let min_y = std::cmp::min(min.y, max.y);
        let max_y = std::cmp::max(min.y, max.y);
        let min_z = std::cmp::min(min.z, max.z);
        let max_z = std::cmp::max(min.z, max.z);

        // Standard redstone order: Y-axis first (layers), then X (rows), then Z (columns)
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                for z in min_z..=max_z {
                    positions.push((x, y, z));
                }
            }
        }

        self.inner = self
            .inner
            .add_output(name, io_type.inner.clone(), layout.inner.clone(), positions)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(self)
    }

    /// Add an output defined by a DefinitionRegion
    #[wasm_bindgen(js_name = addOutputFromRegion)]
    pub fn add_output_from_region(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        region: &DefinitionRegionWrapper,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .add_output_from_region(
                name,
                io_type.inner.clone(),
                layout.inner.clone(),
                region.inner.clone(),
            )
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Add an output defined by a region with automatic layout inference
    #[wasm_bindgen(js_name = addOutputRegionAuto)]
    pub fn add_output_region_auto(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        min: BlockPosition,
        max: BlockPosition,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        let mut positions = Vec::new();

        let min_x = std::cmp::min(min.x, max.x);
        let max_x = std::cmp::max(min.x, max.x);
        let min_y = std::cmp::min(min.y, max.y);
        let max_y = std::cmp::max(min.y, max.y);
        let min_z = std::cmp::min(min.z, max.z);
        let max_z = std::cmp::max(min.z, max.z);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                for z in min_z..=max_z {
                    positions.push((x, y, z));
                }
            }
        }

        self.inner = self
            .inner
            .add_output_auto(name, io_type.inner.clone(), positions)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(self)
    }

    /// Add an output defined by a DefinitionRegion with automatic layout inference
    #[wasm_bindgen(js_name = addOutputFromRegionAuto)]
    pub fn add_output_from_region_auto(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        region: &DefinitionRegionWrapper,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .add_output_from_region_auto(name, io_type.inner.clone(), region.inner.clone())
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Build the IO layout
    pub fn build(self) -> IoLayoutWrapper {
        IoLayoutWrapper {
            inner: self.inner.build(),
        }
    }
}

/// IoLayout wrapper for JavaScript
#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct IoLayoutWrapper {
    inner: IoLayout,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl IoLayoutWrapper {
    /// Get input names
    #[wasm_bindgen(js_name = inputNames)]
    pub fn input_names(&self) -> Vec<String> {
        self.inner
            .input_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get output names
    #[wasm_bindgen(js_name = outputNames)]
    pub fn output_names(&self) -> Vec<String> {
        self.inner
            .output_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }
}

/// TypedCircuitExecutor wrapper for JavaScript
#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct TypedCircuitExecutorWrapper {
    pub(crate) inner: TypedCircuitExecutor,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl TypedCircuitExecutorWrapper {
    /// Create executor from world and layout
    #[wasm_bindgen(js_name = fromLayout)]
    pub fn from_layout(
        world: MchprsWorldWrapper,
        layout: IoLayoutWrapper,
    ) -> Result<TypedCircuitExecutorWrapper, JsValue> {
        Ok(Self {
            inner: TypedCircuitExecutor::from_layout(world.world, layout.inner),
        })
    }

    /// Create executor from world, layout, and options
    #[wasm_bindgen(js_name = fromLayoutWithOptions)]
    pub fn from_layout_with_options(
        world: MchprsWorldWrapper,
        layout: IoLayoutWrapper,
        options: &SimulationOptionsWrapper,
    ) -> Result<TypedCircuitExecutorWrapper, JsValue> {
        Ok(Self {
            inner: TypedCircuitExecutor::from_layout_with_options(
                world.world,
                layout.inner,
                options.inner.clone(),
            ),
        })
    }

    /// Create executor from Insign annotations in schematic
    #[wasm_bindgen(js_name = fromInsign)]
    pub fn from_insign(
        schematic: &SchematicWrapper,
    ) -> Result<TypedCircuitExecutorWrapper, JsValue> {
        use crate::simulation::typed_executor::create_executor_from_insign;

        let executor = create_executor_from_insign(&schematic.0).map_err(|e| {
            JsValue::from_str(&format!("Failed to create executor from Insign: {}", e))
        })?;

        Ok(Self { inner: executor })
    }

    /// Create executor from Insign annotations with custom simulation options
    #[wasm_bindgen(js_name = fromInsignWithOptions)]
    pub fn from_insign_with_options(
        schematic: &SchematicWrapper,
        options: &SimulationOptionsWrapper,
    ) -> Result<TypedCircuitExecutorWrapper, JsValue> {
        use crate::simulation::typed_executor::create_executor_from_insign_with_options;

        let executor =
            create_executor_from_insign_with_options(&schematic.0, options.inner.clone()).map_err(
                |e| JsValue::from_str(&format!("Failed to create executor from Insign: {}", e)),
            )?;

        Ok(Self { inner: executor })
    }

    /// Set state mode
    #[wasm_bindgen(js_name = setStateMode)]
    pub fn set_state_mode(&mut self, mode: &str) -> Result<(), JsValue> {
        let state_mode = match mode {
            "stateless" => StateMode::Stateless,
            "stateful" => StateMode::Stateful,
            "manual" => StateMode::Manual,
            _ => {
                return Err(JsValue::from_str(
                    "Invalid state mode. Use 'stateless', 'stateful', or 'manual'",
                ))
            }
        };
        self.inner.set_state_mode(state_mode);
        Ok(())
    }

    /// Reset the simulation
    pub fn reset(&mut self) -> Result<(), JsValue> {
        self.inner.reset().map_err(|e| JsValue::from_str(&e))
    }

    /// Run the circuit with simplified arguments
    #[wasm_bindgen(js_name = run)]
    pub fn run(&mut self, inputs: JsValue, limit: u32, mode: &str) -> Result<JsValue, JsValue> {
        let execution_mode = match mode {
            "fixed" => ExecutionMode::FixedTicks { ticks: limit },
            "stable" => ExecutionMode::UntilStable {
                stable_ticks: limit,
                max_ticks: 10000,
            }, // Default max ticks
            _ => return Err(JsValue::from_str("Invalid mode. Use 'fixed' or 'stable'")),
        };

        let mode_wrapper = ExecutionModeWrapper {
            inner: execution_mode,
        };
        self.execute(inputs, &mode_wrapper)
    }

    /// Execute the circuit
    pub fn execute(
        &mut self,
        inputs: JsValue,
        mode: &ExecutionModeWrapper,
    ) -> Result<JsValue, JsValue> {
        // Convert inputs from JS object to HashMap<String, Value>
        let mut input_map = std::collections::HashMap::new();
        let obj = js_sys::Object::from(inputs);
        let entries = js_sys::Object::entries(&obj);

        for i in 0..entries.length() {
            let entry = js_sys::Array::from(&entries.get(i));
            let key = entry.get(0).as_string().ok_or("Invalid input key")?;
            let value_js = entry.get(1);

            // Try to convert JsValue to Value
            let value = js_to_value(value_js)?;

            input_map.insert(key, value);
        }

        // Execute
        let result = self
            .inner
            .execute(input_map, mode.inner.clone())
            .map_err(|e| JsValue::from_str(&e))?;

        // Convert result to JS object
        let result_obj = js_sys::Object::new();

        // Add outputs
        let outputs_obj = js_sys::Object::new();
        for (name, value) in result.outputs {
            let value_wrapper = ValueWrapper { inner: value };
            js_sys::Reflect::set(
                &outputs_obj,
                &JsValue::from_str(&name),
                &value_wrapper.to_js(),
            )
            .unwrap();
        }
        js_sys::Reflect::set(&result_obj, &JsValue::from_str("outputs"), &outputs_obj).unwrap();

        // Add ticks_elapsed
        js_sys::Reflect::set(
            &result_obj,
            &JsValue::from_str("ticksElapsed"),
            &JsValue::from_f64(result.ticks_elapsed as f64),
        )
        .unwrap();

        // Add condition_met
        js_sys::Reflect::set(
            &result_obj,
            &JsValue::from_str("conditionMet"),
            &JsValue::from_bool(result.condition_met),
        )
        .unwrap();

        Ok(result_obj.into())
    }

    /// Sync the simulation state back to the schematic
    ///
    /// Call this after execute() to update the schematic with the current simulation state.
    /// Returns the updated schematic.
    #[wasm_bindgen(js_name = syncToSchematic)]
    pub fn sync_to_schematic(&mut self) -> SchematicWrapper {
        let schematic = self.inner.sync_and_get_schematic();
        SchematicWrapper(schematic.clone())
    }

    /// Manually advance the simulation by a specified number of ticks
    ///
    /// This is useful for manual state control when using 'manual' state mode.
    /// Unlike execute(), this does not set any inputs or read outputs.
    #[wasm_bindgen(js_name = tick)]
    pub fn tick(&mut self, ticks: u32) {
        self.inner.tick(ticks);
    }

    /// Manually flush the simulation state
    ///
    /// Ensures all pending changes are propagated through the redstone network.
    #[wasm_bindgen(js_name = flush)]
    pub fn flush(&mut self) {
        self.inner.flush();
    }

    /// Set a single input value without executing
    #[wasm_bindgen(js_name = setInput)]
    pub fn set_input(&mut self, name: String, value: &ValueWrapper) -> Result<(), JsValue> {
        self.inner
            .set_input(&name, &value.inner)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Read a single output value without executing
    #[wasm_bindgen(js_name = readOutput)]
    pub fn read_output(&mut self, name: String) -> Result<ValueWrapper, JsValue> {
        let value = self
            .inner
            .read_output(&name)
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(ValueWrapper { inner: value })
    }

    /// Get all input names
    #[wasm_bindgen(js_name = inputNames)]
    pub fn input_names(&self) -> Vec<String> {
        self.inner
            .input_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get all output names
    #[wasm_bindgen(js_name = outputNames)]
    pub fn output_names(&self) -> Vec<String> {
        self.inner
            .output_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get detailed layout information for debugging and visualization
    ///
    /// Returns a JS object with the structure:
    /// ```javascript
    /// {
    ///   inputs: {
    ///     "name": {
    ///       ioType: "UnsignedInt { bits: 8 }",
    ///       positions: [[x, y, z], ...],  // In bit order (LSB first)
    ///       bitCount: 8
    ///     }
    ///   },
    ///   outputs: { ... }
    /// }
    /// ```
    #[wasm_bindgen(js_name = getLayoutInfo)]
    pub fn get_layout_info(&self) -> JsValue {
        let layout_info = self.inner.get_layout_info();

        let result = Object::new();
        let inputs_obj = Object::new();
        let outputs_obj = Object::new();

        // Convert inputs
        for (name, info) in &layout_info.inputs {
            let io_obj = Object::new();
            Reflect::set(&io_obj, &"ioType".into(), &JsValue::from_str(&info.io_type)).unwrap();
            Reflect::set(
                &io_obj,
                &"bitCount".into(),
                &JsValue::from(info.bit_count as u32),
            )
            .unwrap();

            let positions_arr = Array::new();
            for (x, y, z) in &info.positions {
                let pos = Array::new();
                pos.push(&JsValue::from(*x));
                pos.push(&JsValue::from(*y));
                pos.push(&JsValue::from(*z));
                positions_arr.push(&pos);
            }
            Reflect::set(&io_obj, &"positions".into(), &positions_arr).unwrap();
            Reflect::set(&inputs_obj, &name.into(), &io_obj).unwrap();
        }

        // Convert outputs
        for (name, info) in &layout_info.outputs {
            let io_obj = Object::new();
            Reflect::set(&io_obj, &"ioType".into(), &JsValue::from_str(&info.io_type)).unwrap();
            Reflect::set(
                &io_obj,
                &"bitCount".into(),
                &JsValue::from(info.bit_count as u32),
            )
            .unwrap();

            let positions_arr = Array::new();
            for (x, y, z) in &info.positions {
                let pos = Array::new();
                pos.push(&JsValue::from(*x));
                pos.push(&JsValue::from(*y));
                pos.push(&JsValue::from(*z));
                positions_arr.push(&pos);
            }
            Reflect::set(&io_obj, &"positions".into(), &positions_arr).unwrap();
            Reflect::set(&outputs_obj, &name.into(), &io_obj).unwrap();
        }

        Reflect::set(&result, &"inputs".into(), &inputs_obj).unwrap();
        Reflect::set(&result, &"outputs".into(), &outputs_obj).unwrap();

        result.into()
    }
}

fn js_to_value(value_js: JsValue) -> Result<Value, JsValue> {
    if let Some(b) = value_js.as_bool() {
        Ok(Value::Bool(b))
    } else if let Some(n) = value_js.as_f64() {
        Ok(Value::U32(n as u32))
    } else if let Some(s) = value_js.as_string() {
        Ok(Value::String(s))
    } else if Array::is_array(&value_js) {
        let arr = Array::from(&value_js);
        let mut values = Vec::new();
        for i in 0..arr.length() {
            values.push(js_to_value(arr.get(i))?);
        }
        Ok(Value::Array(values))
    } else {
        Err(JsValue::from_str("Unsupported input value type"))
    }
}
