//! Circuit Builder WASM bindings
//!
//! Fluent API for creating TypedCircuitExecutor instances.

use super::{
    DefinitionRegionWrapper, IoTypeWrapper, LayoutFunctionWrapper, SchematicWrapper,
    SimulationOptionsWrapper, TypedCircuitExecutorWrapper,
};
use crate::simulation::typed_executor::{SortStrategy, StateMode};
use crate::simulation::CircuitBuilder;
use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;

// --- Sort Strategy ---

/// Sort strategy for ordering positions in IO layouts
///
/// Controls how positions are ordered when assigned to bits.
/// Position 0 corresponds to bit 0 (LSB), position 1 to bit 1, etc.
#[wasm_bindgen]
pub struct SortStrategyWrapper {
    pub(crate) inner: SortStrategy,
}

#[wasm_bindgen]
impl SortStrategyWrapper {
    // ========================================================================
    // Axis-first sorting (ascending)
    // ========================================================================

    /// Sort by Y first (ascending), then X, then Z
    /// Standard Minecraft layer-based ordering. This is the default.
    #[wasm_bindgen(js_name = yxz)]
    pub fn yxz() -> Self {
        Self {
            inner: SortStrategy::YXZ,
        }
    }

    /// Sort by X first (ascending), then Y, then Z
    #[wasm_bindgen(js_name = xyz)]
    pub fn xyz() -> Self {
        Self {
            inner: SortStrategy::XYZ,
        }
    }

    /// Sort by Z first (ascending), then Y, then X
    #[wasm_bindgen(js_name = zyx)]
    pub fn zyx() -> Self {
        Self {
            inner: SortStrategy::ZYX,
        }
    }

    // ========================================================================
    // Axis-first sorting (descending)
    // ========================================================================

    /// Sort by Y first (descending), then X ascending, then Z ascending
    #[wasm_bindgen(js_name = yDescXZ)]
    pub fn y_desc_xz() -> Self {
        Self {
            inner: SortStrategy::YDescXZ,
        }
    }

    /// Sort by X first (descending), then Y ascending, then Z ascending
    #[wasm_bindgen(js_name = xDescYZ)]
    pub fn x_desc_yz() -> Self {
        Self {
            inner: SortStrategy::XDescYZ,
        }
    }

    /// Sort by Z first (descending), then Y ascending, then X ascending
    #[wasm_bindgen(js_name = zDescYX)]
    pub fn z_desc_yx() -> Self {
        Self {
            inner: SortStrategy::ZDescYX,
        }
    }

    // ========================================================================
    // Fully descending
    // ========================================================================

    /// Sort by Y descending, then X descending, then Z descending
    #[wasm_bindgen(js_name = descending)]
    pub fn descending() -> Self {
        Self {
            inner: SortStrategy::YXZDesc,
        }
    }

    // ========================================================================
    // Distance-based sorting
    // ========================================================================

    /// Sort by Euclidean distance from a reference point (ascending)
    /// Closest positions first. Useful for radial layouts.
    #[wasm_bindgen(js_name = distanceFrom)]
    pub fn distance_from(x: i32, y: i32, z: i32) -> Self {
        Self {
            inner: SortStrategy::distance_from(x, y, z),
        }
    }

    /// Sort by Euclidean distance from a reference point (descending)
    /// Farthest positions first.
    #[wasm_bindgen(js_name = distanceFromDesc)]
    pub fn distance_from_desc(x: i32, y: i32, z: i32) -> Self {
        Self {
            inner: SortStrategy::distance_from_desc(x, y, z),
        }
    }

    // ========================================================================
    // Special strategies
    // ========================================================================

    /// Preserve the order positions were added (no sorting)
    /// Useful when you've manually ordered positions or are using `fromBoundingBoxes`
    /// where box order matters.
    #[wasm_bindgen(js_name = preserve)]
    pub fn preserve() -> Self {
        Self {
            inner: SortStrategy::Preserve,
        }
    }

    /// Reverse of whatever order positions were added
    #[wasm_bindgen(js_name = reverse)]
    pub fn reverse() -> Self {
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
    #[wasm_bindgen(js_name = fromString)]
    pub fn from_string(s: &str) -> Result<SortStrategyWrapper, JsValue> {
        SortStrategy::from_str(s)
            .map(|inner| Self { inner })
            .ok_or_else(|| JsValue::from_str(&format!("Unknown sort strategy: {}", s)))
    }

    /// Get the name of this strategy
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }
}

impl Default for SortStrategyWrapper {
    fn default() -> Self {
        Self {
            inner: SortStrategy::default(),
        }
    }
}

// --- CircuitBuilder Support ---

/// CircuitBuilder wrapper for JavaScript
/// Provides a fluent API for creating TypedCircuitExecutor instances
#[wasm_bindgen]
pub struct CircuitBuilderWrapper {
    inner: CircuitBuilder,
}

#[wasm_bindgen]
impl CircuitBuilderWrapper {
    /// Create a new CircuitBuilder from a schematic
    #[wasm_bindgen(constructor)]
    pub fn new(schematic: &SchematicWrapper) -> Self {
        Self {
            inner: CircuitBuilder::new(schematic.0.clone()),
        }
    }

    /// Create a CircuitBuilder pre-populated from Insign annotations
    #[wasm_bindgen(js_name = fromInsign)]
    pub fn from_insign(schematic: &SchematicWrapper) -> Result<CircuitBuilderWrapper, JsValue> {
        let builder =
            CircuitBuilder::from_insign(schematic.0.clone()).map_err(|e| JsValue::from_str(&e))?;
        Ok(Self { inner: builder })
    }

    /// Add an input with full control
    ///
    /// Uses the default sort strategy (YXZ - Y first, then X, then Z).
    /// For custom ordering, use `withInputSorted`.
    #[wasm_bindgen(js_name = withInput)]
    pub fn with_input(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        region: &DefinitionRegionWrapper,
    ) -> Result<CircuitBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .with_input(
                name,
                io_type.inner.clone(),
                layout.inner.clone(),
                region.inner.clone(),
            )
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Add an input with full control and custom sort strategy
    #[wasm_bindgen(js_name = withInputSorted)]
    pub fn with_input_sorted(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        region: &DefinitionRegionWrapper,
        sort: &SortStrategyWrapper,
    ) -> Result<CircuitBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .with_input_sorted(
                name,
                io_type.inner.clone(),
                layout.inner.clone(),
                region.inner.clone(),
                sort.inner.clone(),
            )
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Add an input with automatic layout inference
    ///
    /// Uses the default sort strategy (YXZ - Y first, then X, then Z).
    /// For custom ordering, use `withInputAutoSorted`.
    #[wasm_bindgen(js_name = withInputAuto)]
    pub fn with_input_auto(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        region: &DefinitionRegionWrapper,
    ) -> Result<CircuitBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .with_input_auto(name, io_type.inner.clone(), region.inner.clone())
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Add an input with automatic layout inference and custom sort strategy
    #[wasm_bindgen(js_name = withInputAutoSorted)]
    pub fn with_input_auto_sorted(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        region: &DefinitionRegionWrapper,
        sort: &SortStrategyWrapper,
    ) -> Result<CircuitBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .with_input_auto_sorted(
                name,
                io_type.inner.clone(),
                region.inner.clone(),
                sort.inner.clone(),
            )
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Add an output with full control
    ///
    /// Uses the default sort strategy (YXZ - Y first, then X, then Z).
    /// For custom ordering, use `withOutputSorted`.
    #[wasm_bindgen(js_name = withOutput)]
    pub fn with_output(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        region: &DefinitionRegionWrapper,
    ) -> Result<CircuitBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .with_output(
                name,
                io_type.inner.clone(),
                layout.inner.clone(),
                region.inner.clone(),
            )
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Add an output with full control and custom sort strategy
    #[wasm_bindgen(js_name = withOutputSorted)]
    pub fn with_output_sorted(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        region: &DefinitionRegionWrapper,
        sort: &SortStrategyWrapper,
    ) -> Result<CircuitBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .with_output_sorted(
                name,
                io_type.inner.clone(),
                layout.inner.clone(),
                region.inner.clone(),
                sort.inner.clone(),
            )
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Add an output with automatic layout inference
    ///
    /// Uses the default sort strategy (YXZ - Y first, then X, then Z).
    /// For custom ordering, use `withOutputAutoSorted`.
    #[wasm_bindgen(js_name = withOutputAuto)]
    pub fn with_output_auto(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        region: &DefinitionRegionWrapper,
    ) -> Result<CircuitBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .with_output_auto(name, io_type.inner.clone(), region.inner.clone())
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Add an output with automatic layout inference and custom sort strategy
    #[wasm_bindgen(js_name = withOutputAutoSorted)]
    pub fn with_output_auto_sorted(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        region: &DefinitionRegionWrapper,
        sort: &SortStrategyWrapper,
    ) -> Result<CircuitBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .with_output_auto_sorted(
                name,
                io_type.inner.clone(),
                region.inner.clone(),
                sort.inner.clone(),
            )
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Set simulation options
    #[wasm_bindgen(js_name = withOptions)]
    pub fn with_options(mut self, options: &SimulationOptionsWrapper) -> Self {
        self.inner = self.inner.with_options(options.inner.clone());
        self
    }

    /// Set state mode: 'stateless', 'stateful', or 'manual'
    #[wasm_bindgen(js_name = withStateMode)]
    pub fn with_state_mode(mut self, mode: &str) -> Result<CircuitBuilderWrapper, JsValue> {
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
        self.inner = self.inner.with_state_mode(state_mode);
        Ok(self)
    }

    /// Validate the circuit configuration
    #[wasm_bindgen(js_name = validate)]
    pub fn validate(&self) -> Result<(), JsValue> {
        self.inner
            .validate()
            .map(|_| ())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Build the TypedCircuitExecutor
    #[wasm_bindgen(js_name = build)]
    pub fn build(self) -> Result<TypedCircuitExecutorWrapper, JsValue> {
        let executor = self.inner.build().map_err(|e| JsValue::from_str(&e))?;
        Ok(TypedCircuitExecutorWrapper { inner: executor })
    }

    /// Build with validation (convenience method)
    #[wasm_bindgen(js_name = buildValidated)]
    pub fn build_validated(self) -> Result<TypedCircuitExecutorWrapper, JsValue> {
        let executor = self
            .inner
            .build_validated()
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(TypedCircuitExecutorWrapper { inner: executor })
    }

    /// Get the current number of inputs
    #[wasm_bindgen(js_name = inputCount)]
    pub fn input_count(&self) -> usize {
        self.inner.input_count()
    }

    /// Get the current number of outputs
    #[wasm_bindgen(js_name = outputCount)]
    pub fn output_count(&self) -> usize {
        self.inner.output_count()
    }

    /// Get the names of defined inputs
    #[wasm_bindgen(js_name = inputNames)]
    pub fn input_names(&self) -> Vec<String> {
        self.inner
            .input_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get the names of defined outputs
    #[wasm_bindgen(js_name = outputNames)]
    pub fn output_names(&self) -> Vec<String> {
        self.inner
            .output_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }
}

// --- State Mode Constants ---

/// State mode constants for JavaScript
#[wasm_bindgen]
pub struct StateModeConstants;

#[wasm_bindgen]
impl StateModeConstants {
    /// Always reset before execution (default)
    #[wasm_bindgen(getter = STATELESS)]
    pub fn stateless() -> String {
        "stateless".to_string()
    }

    /// Preserve state between executions
    #[wasm_bindgen(getter = STATEFUL)]
    pub fn stateful() -> String {
        "stateful".to_string()
    }

    /// Manual state control
    #[wasm_bindgen(getter = MANUAL)]
    pub fn manual() -> String {
        "manual".to_string()
    }
}
