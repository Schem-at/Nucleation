//! Circuit Builder WASM bindings
//!
//! Fluent API for creating TypedCircuitExecutor instances.

use super::{
    DefinitionRegionWrapper, IoTypeWrapper, LayoutFunctionWrapper, SchematicWrapper,
    SimulationOptionsWrapper, TypedCircuitExecutorWrapper,
};
use crate::simulation::typed_executor::StateMode;
use crate::simulation::CircuitBuilder;
use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;

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

    /// Add an input with automatic layout inference
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

    /// Add an output with full control
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

    /// Add an output with automatic layout inference
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
