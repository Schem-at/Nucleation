//! Typed circuit executor
//!
//! Main executor that manages the simulation and handles typed IO.

use super::{IoLayout, IoMapping, Value};
use crate::simulation::MchprsWorld;
use std::collections::HashMap;

/// Information about a single IO port's layout
#[derive(Debug, Clone)]
pub struct IoLayoutInfo {
    /// Type description (e.g., "UnsignedInt { bits: 8 }")
    pub io_type: String,
    /// Positions in bit order: index 0 = LSB, index N-1 = MSB
    pub positions: Vec<(i32, i32, i32)>,
    /// Total number of bits
    pub bit_count: usize,
}

/// Complete layout information for debugging and visualization
#[derive(Debug, Clone)]
pub struct LayoutInfo {
    /// Input layouts by name
    pub inputs: HashMap<String, IoLayoutInfo>,
    /// Output layouts by name
    pub outputs: HashMap<String, IoLayoutInfo>,
}

/// Condition to check on outputs
#[derive(Debug, Clone)]
pub enum OutputCondition {
    /// Output equals a specific value
    Equals(Value),

    /// Output does not equal a specific value
    NotEquals(Value),

    /// Output is greater than a value (for numeric types)
    GreaterThan(Value),

    /// Output is less than a value (for numeric types)
    LessThan(Value),

    /// Bitwise AND with mask is non-zero (for flag checking)
    BitwiseAnd(u64),
}

impl OutputCondition {
    /// Check if the condition is met for a given value
    pub fn check(&self, value: &Value) -> bool {
        match self {
            OutputCondition::Equals(expected) => value == expected,
            OutputCondition::NotEquals(expected) => value != expected,
            OutputCondition::GreaterThan(threshold) => match (value, threshold) {
                (Value::U32(v), Value::U32(t)) => v > t,
                (Value::I32(v), Value::I32(t)) => v > t,
                (Value::F32(v), Value::F32(t)) => v > t,
                _ => false,
            },
            OutputCondition::LessThan(threshold) => match (value, threshold) {
                (Value::U32(v), Value::U32(t)) => v < t,
                (Value::I32(v), Value::I32(t)) => v < t,
                (Value::F32(v), Value::F32(t)) => v < t,
                _ => false,
            },
            OutputCondition::BitwiseAnd(mask) => match value {
                Value::U32(v) => (*v as u64) & mask != 0,
                Value::I32(v) => (*v as u64) & mask != 0,
                _ => false,
            },
        }
    }
}

/// Execution mode for the circuit
#[derive(Debug, Clone)]
pub enum ExecutionMode {
    /// Run for a fixed number of ticks
    FixedTicks { ticks: u32 },

    /// Run until an output meets a condition (with timeout)
    UntilCondition {
        output_name: String,
        condition: OutputCondition,
        max_ticks: u32,
        check_interval: u32,
    },

    /// Run until any output changes (with timeout)
    UntilChange { max_ticks: u32, check_interval: u32 },

    /// Run until all outputs are stable for N ticks
    UntilStable { stable_ticks: u32, max_ticks: u32 },
}

/// State management mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateMode {
    /// Always reset before execution (default)
    Stateless,

    /// Preserve state between executions
    Stateful,

    /// Manual state control
    Manual,
}

/// Result of circuit execution
#[derive(Debug)]
pub struct ExecutionResult {
    /// Output values
    pub outputs: HashMap<String, Value>,

    /// Number of ticks elapsed
    pub ticks_elapsed: u32,

    /// Whether the execution condition was met (for conditional modes)
    pub condition_met: bool,
}

/// Typed circuit executor
pub struct TypedCircuitExecutor {
    world: MchprsWorld,
    inputs: HashMap<String, IoMapping>,
    outputs: HashMap<String, IoMapping>,
    state_mode: StateMode,
    /// Store the original schematic for resetting
    original_schematic: crate::UniversalSchematic,
    /// Store simulation options for resetting
    simulation_options: crate::simulation::SimulationOptions,
}

impl TypedCircuitExecutor {
    /// Create a new executor
    pub fn new(
        world: MchprsWorld,
        inputs: HashMap<String, IoMapping>,
        outputs: HashMap<String, IoMapping>,
    ) -> Self {
        // Clone the schematic for resetting
        let original_schematic = world.get_schematic().clone();

        // Collect all IO positions for custom_io
        let mut custom_io_positions = Vec::new();
        for mapping in inputs.values() {
            for &(x, y, z) in &mapping.positions {
                custom_io_positions.push(crate::simulation::BlockPos::new(x, y, z));
            }
        }
        for mapping in outputs.values() {
            for &(x, y, z) in &mapping.positions {
                custom_io_positions.push(crate::simulation::BlockPos::new(x, y, z));
            }
        }

        let simulation_options = crate::simulation::SimulationOptions {
            custom_io: custom_io_positions,
            ..Default::default()
        };

        Self {
            world,
            inputs,
            outputs,
            state_mode: StateMode::Stateless,
            original_schematic,
            simulation_options,
        }
    }

    /// Create a new executor with custom simulation options
    pub fn with_options(
        world: MchprsWorld,
        inputs: HashMap<String, IoMapping>,
        outputs: HashMap<String, IoMapping>,
        mut simulation_options: crate::simulation::SimulationOptions,
    ) -> Self {
        let original_schematic = world.get_schematic().clone();

        // Add all IO positions to custom_io if not already present
        for mapping in inputs.values() {
            for &(x, y, z) in &mapping.positions {
                let pos = crate::simulation::BlockPos::new(x, y, z);
                if !simulation_options.custom_io.contains(&pos) {
                    simulation_options.custom_io.push(pos);
                }
            }
        }
        for mapping in outputs.values() {
            for &(x, y, z) in &mapping.positions {
                let pos = crate::simulation::BlockPos::new(x, y, z);
                if !simulation_options.custom_io.contains(&pos) {
                    simulation_options.custom_io.push(pos);
                }
            }
        }

        Self {
            world,
            inputs,
            outputs,
            state_mode: StateMode::Stateless,
            original_schematic,
            simulation_options,
        }
    }

    /// Create a new executor from an IoLayout
    pub fn from_layout(world: MchprsWorld, layout: IoLayout) -> Self {
        let original_schematic = world.get_schematic().clone();

        // Collect all IO positions for custom_io
        let mut custom_io_positions = Vec::new();
        for mapping in layout.inputs.values() {
            for &(x, y, z) in &mapping.positions {
                custom_io_positions.push(crate::simulation::BlockPos::new(x, y, z));
            }
        }
        for mapping in layout.outputs.values() {
            for &(x, y, z) in &mapping.positions {
                custom_io_positions.push(crate::simulation::BlockPos::new(x, y, z));
            }
        }

        let simulation_options = crate::simulation::SimulationOptions {
            custom_io: custom_io_positions,
            ..Default::default()
        };

        Self {
            world,
            inputs: layout.inputs,
            outputs: layout.outputs,
            state_mode: StateMode::Stateless,
            original_schematic,
            simulation_options,
        }
    }

    /// Create a new executor from an IoLayout with custom simulation options
    pub fn from_layout_with_options(
        world: MchprsWorld,
        layout: IoLayout,
        mut simulation_options: crate::simulation::SimulationOptions,
    ) -> Self {
        let original_schematic = world.get_schematic().clone();

        // Add all IO positions to custom_io if not already present
        for mapping in layout.inputs.values() {
            for &(x, y, z) in &mapping.positions {
                let pos = crate::simulation::BlockPos::new(x, y, z);
                if !simulation_options.custom_io.contains(&pos) {
                    simulation_options.custom_io.push(pos);
                }
            }
        }
        for mapping in layout.outputs.values() {
            for &(x, y, z) in &mapping.positions {
                let pos = crate::simulation::BlockPos::new(x, y, z);
                if !simulation_options.custom_io.contains(&pos) {
                    simulation_options.custom_io.push(pos);
                }
            }
        }

        Self {
            world,
            inputs: layout.inputs,
            outputs: layout.outputs,
            state_mode: StateMode::Stateless,
            original_schematic,
            simulation_options,
        }
    }

    /// Set state mode
    pub fn set_state_mode(&mut self, mode: StateMode) {
        self.state_mode = mode;
    }

    /// Execute the circuit
    pub fn execute(
        &mut self,
        inputs: HashMap<String, Value>,
        mode: ExecutionMode,
    ) -> Result<ExecutionResult, String> {
        // Handle state management
        match self.state_mode {
            StateMode::Stateless => {
                // Always reset for stateless mode
                self.reset()?;
            }
            StateMode::Stateful => {
                // Preserve state - do nothing
            }
            StateMode::Manual => {
                // User controls reset - do nothing
            }
        }

        // Encode and set all inputs
        for (name, value) in inputs {
            let mapping = self
                .inputs
                .get(&name)
                .ok_or_else(|| format!("Unknown input: {}", name))?;

            // Pipeline: Value → Binary → Spread → Nibbles
            let nibbles = mapping.encode(&value)?;

            // Set signals on the world (batch operation)
            self.world.set_signals_batch(&mapping.positions, &nibbles)?;
        }

        // Flush the compiler state to ensure input signals are propagated
        self.world.flush();

        // Execute based on mode
        let (ticks_elapsed, condition_met) = match mode {
            ExecutionMode::FixedTicks { ticks } => {
                self.world.tick(ticks);
                (ticks, true)
            }

            ExecutionMode::UntilCondition {
                output_name,
                condition,
                max_ticks,
                check_interval,
            } => {
                self.execute_until_condition(&output_name, &condition, max_ticks, check_interval)?
            }

            ExecutionMode::UntilChange {
                max_ticks,
                check_interval,
            } => self.execute_until_change(max_ticks, check_interval)?,

            ExecutionMode::UntilStable {
                stable_ticks,
                max_ticks,
            } => self.execute_until_stable(stable_ticks, max_ticks)?,
        };

        // Read all outputs
        let outputs = self.read_outputs()?;

        Ok(ExecutionResult {
            outputs,
            ticks_elapsed,
            condition_met,
        })
    }

    /// Reset the simulation by recreating the world
    pub fn reset(&mut self) -> Result<(), String> {
        // Recreate the world from the original schematic
        self.world = MchprsWorld::with_options(
            self.original_schematic.clone(),
            self.simulation_options.clone(),
        )?;
        Ok(())
    }

    /// Get a reference to the world (for advanced use)
    pub fn world(&self) -> &MchprsWorld {
        &self.world
    }

    /// Get a mutable reference to the world (for advanced use)
    pub fn world_mut(&mut self) -> &mut MchprsWorld {
        &mut self.world
    }

    /// Manually advance the simulation by a specified number of ticks
    ///
    /// This is useful for manual state control when using `StateMode::Manual`.
    /// Unlike `execute()`, this does not set any inputs or read outputs -
    /// it only advances the simulation.
    ///
    /// # Example
    /// ```ignore
    /// executor.set_state_mode(StateMode::Manual);
    /// // Set inputs manually via world
    /// executor.tick(10);
    /// // Read outputs manually
    /// ```
    pub fn tick(&mut self, ticks: u32) {
        self.world.tick(ticks);
    }

    /// Manually flush the simulation state
    ///
    /// This ensures all pending changes are propagated through the redstone network.
    /// Useful after manually setting inputs or before reading outputs.
    pub fn flush(&mut self) {
        self.world.flush();
    }

    /// Set a single input value without executing
    ///
    /// Useful for manual control flow where you want to set inputs
    /// incrementally before ticking.
    pub fn set_input(&mut self, name: &str, value: &Value) -> Result<(), String> {
        let mapping = self
            .inputs
            .get(name)
            .ok_or_else(|| format!("Unknown input: {}", name))?;

        let nibbles = mapping.encode(value)?;
        self.world.set_signals_batch(&mapping.positions, &nibbles)?;
        Ok(())
    }

    /// Read a single output value without executing
    ///
    /// Useful for manual control flow where you want to read outputs
    /// after ticking.
    pub fn read_output(&mut self, name: &str) -> Result<Value, String> {
        // Flush first to ensure state is current
        self.world.flush();

        let mapping = self
            .outputs
            .get(name)
            .ok_or_else(|| format!("Unknown output: {}", name))?;

        let nibbles = self.world.get_signals_batch(&mapping.positions);
        mapping.decode(&nibbles)
    }

    /// Get all input names
    pub fn input_names(&self) -> Vec<&str> {
        self.inputs.keys().map(|s| s.as_str()).collect()
    }

    /// Get all output names
    pub fn output_names(&self) -> Vec<&str> {
        self.outputs.keys().map(|s| s.as_str()).collect()
    }

    /// Get detailed layout information for debugging and visualization
    ///
    /// Returns a map of IO name -> layout details including:
    /// - `io_type`: String description of the type (e.g., "UnsignedInt(8)")
    /// - `positions`: List of (x, y, z) positions in bit order
    /// - `bit_count`: Number of bits
    ///
    /// This is useful for visualizing exactly which blocks map to which bits.
    pub fn get_layout_info(&self) -> LayoutInfo {
        let mut inputs = HashMap::new();
        let mut outputs = HashMap::new();

        for (name, mapping) in &self.inputs {
            inputs.insert(
                name.clone(),
                IoLayoutInfo {
                    io_type: format!("{:?}", mapping.io_type),
                    positions: mapping.positions.clone(),
                    bit_count: mapping.positions.len(),
                },
            );
        }

        for (name, mapping) in &self.outputs {
            outputs.insert(
                name.clone(),
                IoLayoutInfo {
                    io_type: format!("{:?}", mapping.io_type),
                    positions: mapping.positions.clone(),
                    bit_count: mapping.positions.len(),
                },
            );
        }

        LayoutInfo { inputs, outputs }
    }

    /// Sync the simulation state back to the schematic and return a reference
    ///
    /// This updates the schematic with the current block states from the simulation,
    /// useful for visualizing the circuit state after execution.
    pub fn sync_and_get_schematic(&mut self) -> &crate::UniversalSchematic {
        self.world.sync_to_schematic();
        self.world.get_schematic()
    }

    /// Read all outputs from the world
    fn read_outputs(&mut self) -> Result<HashMap<String, Value>, String> {
        // Flush the compiler state to ensure block states (like lamps) are up to date
        // This is critical for reading outputs that rely on block properties (e.g. lit lamps)
        // rather than just signal strength from the compiler graph.
        self.world.flush();

        let mut outputs = HashMap::new();
        for (name, mapping) in &self.outputs {
            // Read signals from the world (batch operation)
            let nibbles = self.world.get_signals_batch(&mapping.positions);

            // Pipeline: Nibbles → Collect → Binary → Value
            let value = mapping.decode(&nibbles)?;

            outputs.insert(name.clone(), value);
        }
        Ok(outputs)
    }

    /// Execute until a specific output condition is met
    fn execute_until_condition(
        &mut self,
        output_name: &str,
        condition: &OutputCondition,
        max_ticks: u32,
        check_interval: u32,
    ) -> Result<(u32, bool), String> {
        let mut ticks_elapsed = 0;

        while ticks_elapsed < max_ticks {
            // Tick the simulation
            self.world.tick(check_interval);
            ticks_elapsed += check_interval;

            // Check the condition
            let outputs = self.read_outputs()?;
            let output_value = outputs
                .get(output_name)
                .ok_or_else(|| format!("Unknown output: {}", output_name))?;

            if condition.check(output_value) {
                return Ok((ticks_elapsed, true));
            }
        }

        // Timeout - condition not met
        Ok((ticks_elapsed, false))
    }

    /// Execute until any output changes
    fn execute_until_change(
        &mut self,
        max_ticks: u32,
        check_interval: u32,
    ) -> Result<(u32, bool), String> {
        let mut ticks_elapsed = 0;
        let initial_outputs = self.read_outputs()?;

        while ticks_elapsed < max_ticks {
            // Tick the simulation
            self.world.tick(check_interval);
            ticks_elapsed += check_interval;

            // Check if any output changed
            let current_outputs = self.read_outputs()?;

            for (name, initial_value) in &initial_outputs {
                if let Some(current_value) = current_outputs.get(name) {
                    if initial_value != current_value {
                        return Ok((ticks_elapsed, true));
                    }
                }
            }
        }

        // Timeout - no change detected
        Ok((ticks_elapsed, false))
    }

    /// Execute until all outputs are stable for a certain number of ticks
    fn execute_until_stable(
        &mut self,
        stable_ticks: u32,
        max_ticks: u32,
    ) -> Result<(u32, bool), String> {
        let mut ticks_elapsed = 0;
        let mut stable_count = 0;
        let mut last_outputs = self.read_outputs()?;

        while ticks_elapsed < max_ticks {
            // Tick the simulation
            self.world.tick(1);
            ticks_elapsed += 1;

            // Check if outputs are the same as last tick
            let current_outputs = self.read_outputs()?;

            if current_outputs == last_outputs {
                stable_count += 1;
                if stable_count >= stable_ticks {
                    return Ok((ticks_elapsed, true));
                }
            } else {
                // Outputs changed, reset stable counter
                stable_count = 0;
            }

            last_outputs = current_outputs;
        }

        // Timeout - not stable
        Ok((ticks_elapsed, false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockState;
    use crate::UniversalSchematic;

    #[test]
    fn test_executor_creation() {
        // Create a minimal schematic for testing
        let mut schematic = UniversalSchematic::new("test".to_string());

        // Add a single block so the world isn't empty
        schematic.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));

        let world = MchprsWorld::new(schematic).unwrap();

        let executor = TypedCircuitExecutor::new(world, HashMap::new(), HashMap::new());

        // Just test that it compiles
        drop(executor);
    }
}
