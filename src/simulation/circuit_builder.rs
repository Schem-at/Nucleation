//! Unified CircuitBuilder for streamlined executor creation
//!
//! This module provides a builder pattern for creating `TypedCircuitExecutor` instances
//! with a fluent API that combines IO layout definition and simulation configuration.
//!
//! # Example
//!
//! ```ignore
//! use nucleation::simulation::circuit_builder::CircuitBuilder;
//! use nucleation::simulation::typed_executor::{IoType, LayoutFunction, SortStrategy};
//!
//! let executor = CircuitBuilder::new(schematic)
//!     .with_input("a", IoType::UnsignedInt { bits: 8 }, region_a)?
//!     .with_input("b", IoType::UnsignedInt { bits: 8 }, region_b)?
//!     .with_output("sum", IoType::UnsignedInt { bits: 9 }, region_sum)?
//!     .with_options(SimulationOptions::default())
//!     .validate()?
//!     .build()?;
//!
//! // With custom sort strategy:
//! let executor = CircuitBuilder::new(schematic)
//!     .with_input_sorted("a", IoType::UnsignedInt { bits: 8 }, region_a, SortStrategy::YDescXZ)?
//!     .build()?;
//! ```

use crate::definition_region::DefinitionRegion;
use crate::simulation::typed_executor::{
    IoLayoutBuilder, IoType, LayoutFunction, SortStrategy, StateMode, TypedCircuitExecutor,
};
use crate::simulation::{MchprsWorld, SimulationOptions};
use crate::UniversalSchematic;
use std::collections::HashMap;

/// Validation errors for circuit builder
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Two IO regions overlap
    OverlappingRegions {
        name1: String,
        name2: String,
        overlap_count: usize,
    },
    /// No inputs defined
    NoInputs,
    /// No outputs defined
    NoOutputs,
    /// Input has no positions
    EmptyInput(String),
    /// Output has no positions
    EmptyOutput(String),
    /// Custom validation error
    Custom(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::OverlappingRegions {
                name1,
                name2,
                overlap_count,
            } => write!(
                f,
                "IO regions '{}' and '{}' overlap at {} positions",
                name1, name2, overlap_count
            ),
            ValidationError::NoInputs => write!(f, "No inputs defined"),
            ValidationError::NoOutputs => write!(f, "No outputs defined"),
            ValidationError::EmptyInput(name) => write!(f, "Input '{}' has no positions", name),
            ValidationError::EmptyOutput(name) => write!(f, "Output '{}' has no positions", name),
            ValidationError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Builder for creating TypedCircuitExecutor instances
pub struct CircuitBuilder {
    schematic: UniversalSchematic,
    layout_builder: IoLayoutBuilder,
    simulation_options: SimulationOptions,
    state_mode: StateMode,
    /// Store input regions for validation
    input_regions: HashMap<String, Vec<(i32, i32, i32)>>,
    /// Store output regions for validation
    output_regions: HashMap<String, Vec<(i32, i32, i32)>>,
}

impl CircuitBuilder {
    /// Create a new CircuitBuilder from a schematic
    pub fn new(schematic: UniversalSchematic) -> Self {
        Self {
            schematic,
            layout_builder: IoLayoutBuilder::new(),
            simulation_options: SimulationOptions::default(),
            state_mode: StateMode::Stateless,
            input_regions: HashMap::new(),
            output_regions: HashMap::new(),
        }
    }

    /// Create a CircuitBuilder pre-populated from Insign annotations in the schematic
    ///
    /// This parses Insign DSL annotations from signs in the schematic and
    /// pre-populates the IO layout.
    pub fn from_insign(schematic: UniversalSchematic) -> Result<Self, String> {
        use crate::insign;
        use crate::simulation::typed_executor::insign_io;

        // Extract signs from schematic
        let signs = insign::extract_signs(&schematic);
        let input: Vec<([i32; 3], String)> = signs.into_iter().map(|s| (s.pos, s.text)).collect();

        // Parse IO layout using insign_io
        let layout_builder = insign_io::parse_io_layout_from_insign(&input, &schematic)
            .map_err(|e| e.to_string())?;

        // Extract regions from the layout for validation
        let layout = layout_builder.build();
        let mut input_regions = HashMap::new();
        let mut output_regions = HashMap::new();

        for (name, mapping) in &layout.inputs {
            input_regions.insert(name.clone(), mapping.positions.clone());
        }
        for (name, mapping) in &layout.outputs {
            output_regions.insert(name.clone(), mapping.positions.clone());
        }

        // Rebuild the layout builder with the same data
        let mut new_builder = IoLayoutBuilder::new();
        for (name, mapping) in layout.inputs {
            new_builder = new_builder
                .add_input(name, mapping.io_type, mapping.layout, mapping.positions)
                .map_err(|e| e)?;
        }
        for (name, mapping) in layout.outputs {
            new_builder = new_builder
                .add_output(name, mapping.io_type, mapping.layout, mapping.positions)
                .map_err(|e| e)?;
        }

        Ok(Self {
            schematic,
            layout_builder: new_builder,
            simulation_options: SimulationOptions::default(),
            state_mode: StateMode::Stateless,
            input_regions,
            output_regions,
        })
    }

    /// Add an input with full control over type, layout, and region
    ///
    /// Uses the default sort strategy (YXZ - Y first, then X, then Z).
    /// For custom ordering, use `with_input_sorted`.
    pub fn with_input(
        self,
        name: impl Into<String>,
        io_type: IoType,
        layout: LayoutFunction,
        region: DefinitionRegion,
    ) -> Result<Self, String> {
        self.with_input_sorted(name, io_type, layout, region, SortStrategy::default())
    }

    /// Add an input with full control and custom sort strategy
    pub fn with_input_sorted(
        mut self,
        name: impl Into<String>,
        io_type: IoType,
        layout: LayoutFunction,
        region: DefinitionRegion,
        sort: SortStrategy,
    ) -> Result<Self, String> {
        let name = name.into();
        let raw_positions: Vec<_> = region.iter_positions().collect();
        let positions = sort.sort(&raw_positions);

        if positions.is_empty() {
            return Err(format!("Input '{}' has no positions", name));
        }

        self.input_regions.insert(name.clone(), positions.clone());
        self.layout_builder = self
            .layout_builder
            .add_input(name, io_type, layout, positions)?;
        Ok(self)
    }

    /// Add an input with automatic layout inference
    ///
    /// Uses the default sort strategy (YXZ - Y first, then X, then Z).
    /// For custom ordering, use `with_input_auto_sorted`.
    pub fn with_input_auto(
        self,
        name: impl Into<String>,
        io_type: IoType,
        region: DefinitionRegion,
    ) -> Result<Self, String> {
        self.with_input_auto_sorted(name, io_type, region, SortStrategy::default())
    }

    /// Add an input with automatic layout inference and custom sort strategy
    pub fn with_input_auto_sorted(
        mut self,
        name: impl Into<String>,
        io_type: IoType,
        region: DefinitionRegion,
        sort: SortStrategy,
    ) -> Result<Self, String> {
        let name = name.into();
        let raw_positions: Vec<_> = region.iter_positions().collect();
        let positions = sort.sort(&raw_positions);

        if positions.is_empty() {
            return Err(format!("Input '{}' has no positions", name));
        }

        self.input_regions.insert(name.clone(), positions.clone());
        self.layout_builder = self
            .layout_builder
            .add_input_auto(name, io_type, positions)?;
        Ok(self)
    }

    /// Add an output with full control over type, layout, and region
    ///
    /// Uses the default sort strategy (YXZ - Y first, then X, then Z).
    /// For custom ordering, use `with_output_sorted`.
    pub fn with_output(
        self,
        name: impl Into<String>,
        io_type: IoType,
        layout: LayoutFunction,
        region: DefinitionRegion,
    ) -> Result<Self, String> {
        self.with_output_sorted(name, io_type, layout, region, SortStrategy::default())
    }

    /// Add an output with full control and custom sort strategy
    pub fn with_output_sorted(
        mut self,
        name: impl Into<String>,
        io_type: IoType,
        layout: LayoutFunction,
        region: DefinitionRegion,
        sort: SortStrategy,
    ) -> Result<Self, String> {
        let name = name.into();
        let raw_positions: Vec<_> = region.iter_positions().collect();
        let positions = sort.sort(&raw_positions);

        if positions.is_empty() {
            return Err(format!("Output '{}' has no positions", name));
        }

        self.output_regions.insert(name.clone(), positions.clone());
        self.layout_builder = self
            .layout_builder
            .add_output(name, io_type, layout, positions)?;
        Ok(self)
    }

    /// Add an output with automatic layout inference
    ///
    /// Uses the default sort strategy (YXZ - Y first, then X, then Z).
    /// For custom ordering, use `with_output_auto_sorted`.
    pub fn with_output_auto(
        self,
        name: impl Into<String>,
        io_type: IoType,
        region: DefinitionRegion,
    ) -> Result<Self, String> {
        self.with_output_auto_sorted(name, io_type, region, SortStrategy::default())
    }

    /// Add an output with automatic layout inference and custom sort strategy
    pub fn with_output_auto_sorted(
        mut self,
        name: impl Into<String>,
        io_type: IoType,
        region: DefinitionRegion,
        sort: SortStrategy,
    ) -> Result<Self, String> {
        let name = name.into();
        let raw_positions: Vec<_> = region.iter_positions().collect();
        let positions = sort.sort(&raw_positions);

        if positions.is_empty() {
            return Err(format!("Output '{}' has no positions", name));
        }

        self.output_regions.insert(name.clone(), positions.clone());
        self.layout_builder = self
            .layout_builder
            .add_output_auto(name, io_type, positions)?;
        Ok(self)
    }

    /// Set simulation options
    pub fn with_options(mut self, options: SimulationOptions) -> Self {
        self.simulation_options = options;
        self
    }

    /// Set state mode
    pub fn with_state_mode(mut self, mode: StateMode) -> Self {
        self.state_mode = mode;
        self
    }

    /// Validate the circuit configuration
    ///
    /// Checks for:
    /// - Overlapping input/output regions
    /// - Empty inputs/outputs
    /// - Invalid configurations
    pub fn validate(&self) -> Result<&Self, ValidationError> {
        // Check for empty inputs
        if self.input_regions.is_empty() {
            return Err(ValidationError::NoInputs);
        }

        // Check for empty outputs
        if self.output_regions.is_empty() {
            return Err(ValidationError::NoOutputs);
        }

        // Check for empty regions
        for (name, positions) in &self.input_regions {
            if positions.is_empty() {
                return Err(ValidationError::EmptyInput(name.clone()));
            }
        }

        for (name, positions) in &self.output_regions {
            if positions.is_empty() {
                return Err(ValidationError::EmptyOutput(name.clone()));
            }
        }

        // Check for overlaps between all regions
        let all_regions: Vec<(&str, &Vec<(i32, i32, i32)>)> = self
            .input_regions
            .iter()
            .map(|(n, p)| (n.as_str(), p))
            .chain(self.output_regions.iter().map(|(n, p)| (n.as_str(), p)))
            .collect();

        for i in 0..all_regions.len() {
            for j in (i + 1)..all_regions.len() {
                let (name1, positions1) = all_regions[i];
                let (name2, positions2) = all_regions[j];

                let set1: std::collections::HashSet<_> = positions1.iter().collect();
                let overlap_count = positions2.iter().filter(|p| set1.contains(p)).count();

                if overlap_count > 0 {
                    return Err(ValidationError::OverlappingRegions {
                        name1: name1.to_string(),
                        name2: name2.to_string(),
                        overlap_count,
                    });
                }
            }
        }

        Ok(self)
    }

    /// Build the TypedCircuitExecutor
    pub fn build(self) -> Result<TypedCircuitExecutor, String> {
        let layout = self.layout_builder.build();

        // Create the world
        let world = MchprsWorld::with_options(self.schematic, self.simulation_options)?;

        // Create executor
        let mut executor = TypedCircuitExecutor::from_layout(world, layout);
        executor.set_state_mode(self.state_mode);

        Ok(executor)
    }

    /// Build with validation (convenience method)
    pub fn build_validated(self) -> Result<TypedCircuitExecutor, String> {
        self.validate().map_err(|e| e.to_string())?;

        // Need to move self after validation
        let layout = self.layout_builder.build();
        let world = MchprsWorld::with_options(self.schematic, self.simulation_options)?;
        let mut executor = TypedCircuitExecutor::from_layout(world, layout);
        executor.set_state_mode(self.state_mode);
        Ok(executor)
    }

    /// Get the current number of inputs
    pub fn input_count(&self) -> usize {
        self.input_regions.len()
    }

    /// Get the current number of outputs
    pub fn output_count(&self) -> usize {
        self.output_regions.len()
    }

    /// Get the names of defined inputs
    pub fn input_names(&self) -> Vec<&str> {
        self.input_regions.keys().map(|s| s.as_str()).collect()
    }

    /// Get the names of defined outputs
    pub fn output_names(&self) -> Vec<&str> {
        self.output_regions.keys().map(|s| s.as_str()).collect()
    }
}

/// Create an executor from Insign annotations (convenience function)
pub fn create_circuit_from_insign(
    schematic: &UniversalSchematic,
) -> Result<TypedCircuitExecutor, String> {
    use crate::simulation::typed_executor::insign_io;
    insign_io::create_executor_from_insign(schematic).map_err(|e| e.to_string())
}

/// Create an executor from Insign annotations with options (convenience function)
pub fn create_circuit_from_insign_with_options(
    schematic: &UniversalSchematic,
    options: SimulationOptions,
) -> Result<TypedCircuitExecutor, String> {
    use crate::simulation::typed_executor::insign_io;
    insign_io::create_executor_from_insign_with_options(schematic, options)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockState;

    fn create_test_schematic() -> UniversalSchematic {
        let mut schematic = UniversalSchematic::new("test".to_string());
        // Add some blocks so the world isn't empty
        for x in 0..10 {
            schematic.set_block(x, 0, 0, BlockState::new("minecraft:stone".to_string()));
        }
        schematic
    }

    #[test]
    fn test_circuit_builder_creation() {
        let schematic = create_test_schematic();
        let builder = CircuitBuilder::new(schematic);

        assert_eq!(builder.input_count(), 0);
        assert_eq!(builder.output_count(), 0);
    }

    #[test]
    fn test_circuit_builder_validation_no_inputs() {
        let schematic = create_test_schematic();
        let builder = CircuitBuilder::new(schematic);

        let result = builder.validate();
        assert!(matches!(result, Err(ValidationError::NoInputs)));
    }
}
