//! Simulation WASM bindings
//!
//! MCHPRS simulation wrapper: world creation, ticking, signal manipulation.

use super::SchematicWrapper;
use crate::simulation::{generate_truth_table, BlockPos, MchprsWorld, SimulationOptions};
use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SimulationOptionsWrapper {
    pub(crate) inner: SimulationOptions,
}

#[wasm_bindgen]
impl SimulationOptionsWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: SimulationOptions::default(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn optimize(&self) -> bool {
        self.inner.optimize
    }

    #[wasm_bindgen(setter)]
    pub fn set_optimize(&mut self, value: bool) {
        self.inner.optimize = value;
    }

    #[wasm_bindgen(getter)]
    pub fn io_only(&self) -> bool {
        self.inner.io_only
    }

    #[wasm_bindgen(setter)]
    pub fn set_io_only(&mut self, value: bool) {
        self.inner.io_only = value;
    }

    /// Adds a position to the custom IO list
    #[wasm_bindgen(js_name = addCustomIo)]
    pub fn add_custom_io(&mut self, x: i32, y: i32, z: i32) {
        self.inner.custom_io.push(BlockPos::new(x, y, z));
    }

    /// Clears the custom IO list
    #[wasm_bindgen(js_name = clearCustomIo)]
    pub fn clear_custom_io(&mut self) {
        self.inner.custom_io.clear();
    }
}

#[wasm_bindgen]
pub struct MchprsWorldWrapper {
    pub(crate) world: MchprsWorld,
}

#[wasm_bindgen]
impl SchematicWrapper {
    /// Creates a simulation world for this schematic with default options
    ///
    /// This allows you to simulate redstone circuits and interact with them.
    pub fn create_simulation_world(&self) -> Result<MchprsWorldWrapper, JsValue> {
        MchprsWorldWrapper::new(self)
    }

    /// Creates a simulation world for this schematic with custom options
    ///
    /// This allows you to configure simulation behavior like wire state tracking.
    pub fn create_simulation_world_with_options(
        &self,
        options: &SimulationOptionsWrapper,
    ) -> Result<MchprsWorldWrapper, JsValue> {
        MchprsWorldWrapper::with_options(self, options)
    }
}

#[wasm_bindgen]
impl MchprsWorldWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(schematic: &SchematicWrapper) -> Result<MchprsWorldWrapper, JsValue> {
        let world = MchprsWorld::new(schematic.0.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to create MchprsWorld: {}", e)))?;

        Ok(MchprsWorldWrapper { world })
    }

    /// Creates a simulation world with custom options
    pub fn with_options(
        schematic: &SchematicWrapper,
        options: &SimulationOptionsWrapper,
    ) -> Result<MchprsWorldWrapper, JsValue> {
        let world = MchprsWorld::with_options(schematic.0.clone(), options.inner.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to create MchprsWorld: {}", e)))?;

        Ok(MchprsWorldWrapper { world })
    }

    /// Simulates a right-click on a block (typically a lever)
    pub fn on_use_block(&mut self, x: i32, y: i32, z: i32) {
        self.world.on_use_block(BlockPos::new(x, y, z));
    }

    /// Advances the simulation by the specified number of ticks
    pub fn tick(&mut self, number_of_ticks: u32) {
        self.world.tick(number_of_ticks);
    }

    /// Flushes pending changes from the compiler to the world
    pub fn flush(&mut self) {
        self.world.flush();
    }

    /// Checks if a redstone lamp is lit at the given position
    pub fn is_lit(&self, x: i32, y: i32, z: i32) -> bool {
        self.world.is_lit(BlockPos::new(x, y, z))
    }

    /// Gets the power state of a lever
    pub fn get_lever_power(&self, x: i32, y: i32, z: i32) -> bool {
        self.world.get_lever_power(BlockPos::new(x, y, z))
    }

    /// Gets the redstone power level at a position
    pub fn get_redstone_power(&self, x: i32, y: i32, z: i32) -> u8 {
        self.world.get_redstone_power(BlockPos::new(x, y, z))
    }

    /// Sets the signal strength at a specific block position (for custom IO nodes)
    #[wasm_bindgen(js_name = setSignalStrength)]
    pub fn set_signal_strength(&mut self, x: i32, y: i32, z: i32, strength: u8) {
        self.world
            .set_signal_strength(BlockPos::new(x, y, z), strength);
    }

    /// Gets the signal strength at a specific block position (for custom IO nodes)
    #[wasm_bindgen(js_name = getSignalStrength)]
    pub fn get_signal_strength(&self, x: i32, y: i32, z: i32) -> u8 {
        self.world.get_signal_strength(BlockPos::new(x, y, z))
    }

    /// Check for custom IO state changes and queue them
    /// Call this after tick() or setSignalStrength() to detect changes
    #[wasm_bindgen(js_name = checkCustomIoChanges)]
    pub fn check_custom_io_changes(&mut self) {
        self.world.check_custom_io_changes();
    }

    /// Get and clear all custom IO changes since last poll
    /// Returns an array of change objects with {x, y, z, oldPower, newPower}
    #[wasm_bindgen(js_name = pollCustomIoChanges)]
    pub fn poll_custom_io_changes(&mut self) -> JsValue {
        let changes = self.world.poll_custom_io_changes();
        let array = js_sys::Array::new();

        for change in changes {
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("x"),
                &JsValue::from_f64(change.x as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("y"),
                &JsValue::from_f64(change.y as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("z"),
                &JsValue::from_f64(change.z as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("oldPower"),
                &JsValue::from_f64(change.old_power as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("newPower"),
                &JsValue::from_f64(change.new_power as f64),
            )
            .unwrap();
            array.push(&obj);
        }

        array.into()
    }

    /// Get custom IO changes without clearing the queue
    #[wasm_bindgen(js_name = peekCustomIoChanges)]
    pub fn peek_custom_io_changes(&self) -> JsValue {
        let changes = self.world.peek_custom_io_changes();
        let array = js_sys::Array::new();

        for change in changes {
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("x"),
                &JsValue::from_f64(change.x as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("y"),
                &JsValue::from_f64(change.y as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("z"),
                &JsValue::from_f64(change.z as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("oldPower"),
                &JsValue::from_f64(change.old_power as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("newPower"),
                &JsValue::from_f64(change.new_power as f64),
            )
            .unwrap();
            array.push(&obj);
        }

        array.into()
    }

    /// Clear all queued custom IO changes
    #[wasm_bindgen(js_name = clearCustomIoChanges)]
    pub fn clear_custom_io_changes(&mut self) {
        self.world.clear_custom_io_changes();
    }

    /// Generates a truth table for the circuit
    ///
    /// Returns an array of objects with keys like "Input 0", "Output 0", etc.
    pub fn get_truth_table(&self) -> JsValue {
        let truth_table = generate_truth_table(&self.world.schematic);

        // Create a JavaScript array to hold the results
        let result = js_sys::Array::new();

        // Convert each row in the truth table to a JavaScript object
        for row in truth_table {
            let row_obj = js_sys::Object::new();

            // Add each entry in the row to the object
            for (key, value) in row {
                js_sys::Reflect::set(
                    &row_obj,
                    &JsValue::from_str(&key),
                    &JsValue::from_bool(value),
                )
                .unwrap();
            }

            result.push(&row_obj);
        }

        result.into()
    }

    /// Syncs the current simulation state back to the underlying schematic
    ///
    /// Call this after running simulation to update block states (redstone power, lever states, etc.)
    pub fn sync_to_schematic(&mut self) {
        self.world.sync_to_schematic();
    }

    /// Gets a copy of the underlying schematic
    ///
    /// Note: Call sync_to_schematic() first if you want the latest simulation state
    pub fn get_schematic(&self) -> SchematicWrapper {
        SchematicWrapper(self.world.get_schematic().clone())
    }

    /// Consumes the simulation world and returns the schematic with simulation state
    ///
    /// This automatically syncs before returning
    pub fn into_schematic(mut self) -> SchematicWrapper {
        SchematicWrapper(self.world.into_schematic())
    }
}

// =============================================================================
// TYPED CIRCUIT EXECUTOR BINDINGS
// =============================================================================

use crate::simulation::typed_executor::{
    ExecutionMode, ExecutionResult, IoLayout, IoLayoutBuilder, IoMapping, IoType, LayoutFunction,
    OutputCondition, StateMode, TypedCircuitExecutor, Value,
};
