//! Builder for constructing IO layouts
//!
//! Provides a fluent API for defining circuit inputs and outputs with types and layouts.

use super::bus::{BusPort, BusSpec};
use super::physical::{Face, PortDirection};
use super::{IoMapping, IoType, LayoutFunction, SortStrategy};
use crate::definition_region::DefinitionRegion;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Builder for constructing IO layouts
#[derive(Debug, Clone)]
pub struct IoLayoutBuilder {
    inputs: HashMap<String, IoMapping>,
    outputs: HashMap<String, IoMapping>,
    buses: HashMap<String, BusPort>,
}

impl IoLayoutBuilder {
    /// Create a new empty IO layout builder
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            buses: HashMap::new(),
        }
    }

    /// Add an input with full control
    pub fn add_input(
        mut self,
        name: impl Into<String>,
        io_type: IoType,
        layout: LayoutFunction,
        positions: Vec<(i32, i32, i32)>,
    ) -> Result<Self, String> {
        let name = name.into();

        // Create the mapping
        let mapping = IoMapping {
            io_type,
            layout,
            positions,
            face: None,
            direction: Some(PortDirection::Input),
        };

        // Validate the mapping
        mapping.validate()?;

        // Check for duplicates
        if self.inputs.contains_key(&name) {
            return Err(format!("Duplicate input name: {}", name));
        }

        self.inputs.insert(name, mapping);
        Ok(self)
    }

    /// Add an input defined by a DefinitionRegion
    ///
    /// Uses the default sort strategy (YXZ - Y first, then X, then Z).
    /// For custom ordering, use `add_input_from_region_sorted`.
    pub fn add_input_from_region(
        self,
        name: impl Into<String>,
        io_type: IoType,
        layout: LayoutFunction,
        region: DefinitionRegion,
    ) -> Result<Self, String> {
        self.add_input_from_region_sorted(name, io_type, layout, region, SortStrategy::default())
    }

    /// Add an input defined by a DefinitionRegion with a custom sort strategy
    pub fn add_input_from_region_sorted(
        self,
        name: impl Into<String>,
        io_type: IoType,
        layout: LayoutFunction,
        region: DefinitionRegion,
        sort: SortStrategy,
    ) -> Result<Self, String> {
        let positions = region.iter_positions().collect::<Vec<_>>();
        let sorted_positions = sort.sort(&positions);
        self.add_input(name, io_type, layout, sorted_positions)
    }

    /// Add an input defined by a DefinitionRegion with automatic layout inference
    ///
    /// Uses the default sort strategy (YXZ - Y first, then X, then Z).
    /// For custom ordering, use `add_input_from_region_auto_sorted`.
    pub fn add_input_from_region_auto(
        self,
        name: impl Into<String>,
        io_type: IoType,
        region: DefinitionRegion,
    ) -> Result<Self, String> {
        self.add_input_from_region_auto_sorted(name, io_type, region, SortStrategy::default())
    }

    /// Add an input defined by a DefinitionRegion with automatic layout and custom sort strategy
    pub fn add_input_from_region_auto_sorted(
        self,
        name: impl Into<String>,
        io_type: IoType,
        region: DefinitionRegion,
        sort: SortStrategy,
    ) -> Result<Self, String> {
        let positions = region.iter_positions().collect::<Vec<_>>();
        let sorted_positions = sort.sort(&positions);
        self.add_input_auto(name, io_type, sorted_positions)
    }

    /// Add an output with full control
    pub fn add_output(
        mut self,
        name: impl Into<String>,
        io_type: IoType,
        layout: LayoutFunction,
        positions: Vec<(i32, i32, i32)>,
    ) -> Result<Self, String> {
        let name = name.into();

        // Create the mapping
        let mapping = IoMapping {
            io_type,
            layout,
            positions,
            face: None,
            direction: Some(PortDirection::Output),
        };

        // Validate the mapping
        mapping.validate()?;

        // Check for duplicates
        if self.outputs.contains_key(&name) {
            return Err(format!("Duplicate output name: {}", name));
        }

        self.outputs.insert(name, mapping);
        Ok(self)
    }

    /// Add an output defined by a DefinitionRegion
    ///
    /// Uses the default sort strategy (YXZ - Y first, then X, then Z).
    /// For custom ordering, use `add_output_from_region_sorted`.
    pub fn add_output_from_region(
        self,
        name: impl Into<String>,
        io_type: IoType,
        layout: LayoutFunction,
        region: DefinitionRegion,
    ) -> Result<Self, String> {
        self.add_output_from_region_sorted(name, io_type, layout, region, SortStrategy::default())
    }

    /// Add an output defined by a DefinitionRegion with a custom sort strategy
    pub fn add_output_from_region_sorted(
        self,
        name: impl Into<String>,
        io_type: IoType,
        layout: LayoutFunction,
        region: DefinitionRegion,
        sort: SortStrategy,
    ) -> Result<Self, String> {
        let positions = region.iter_positions().collect::<Vec<_>>();
        let sorted_positions = sort.sort(&positions);
        self.add_output(name, io_type, layout, sorted_positions)
    }

    /// Add an output defined by a DefinitionRegion with automatic layout inference
    ///
    /// Uses the default sort strategy (YXZ - Y first, then X, then Z).
    /// For custom ordering, use `add_output_from_region_auto_sorted`.
    pub fn add_output_from_region_auto(
        self,
        name: impl Into<String>,
        io_type: IoType,
        region: DefinitionRegion,
    ) -> Result<Self, String> {
        self.add_output_from_region_auto_sorted(name, io_type, region, SortStrategy::default())
    }

    /// Add an output defined by a DefinitionRegion with automatic layout and custom sort strategy
    pub fn add_output_from_region_auto_sorted(
        self,
        name: impl Into<String>,
        io_type: IoType,
        region: DefinitionRegion,
        sort: SortStrategy,
    ) -> Result<Self, String> {
        let positions = region.iter_positions().collect::<Vec<_>>();
        let sorted_positions = sort.sort(&positions);
        self.add_output_auto(name, io_type, sorted_positions)
    }

    /// Add an input with automatic layout inference
    /// Infers OneToOne or Packed4 based on position count
    pub fn add_input_auto(
        self,
        name: impl Into<String>,
        io_type: IoType,
        positions: Vec<(i32, i32, i32)>,
    ) -> Result<Self, String> {
        let bit_count = io_type.bit_count();

        // Infer layout based on position count
        let layout = if positions.len() == bit_count {
            LayoutFunction::OneToOne
        } else if positions.len() == bit_count.div_ceil(4) {
            LayoutFunction::Packed4
        } else {
            return Err(format!(
                "Cannot infer layout: {} bits need {} positions (OneToOne) or {} positions (Packed4), but got {}",
                bit_count,
                bit_count,
                bit_count.div_ceil(4),
                positions.len()
            ));
        };

        self.add_input(name, io_type, layout, positions)
    }

    /// Add an output with automatic layout inference
    pub fn add_output_auto(
        self,
        name: impl Into<String>,
        io_type: IoType,
        positions: Vec<(i32, i32, i32)>,
    ) -> Result<Self, String> {
        let bit_count = io_type.bit_count();

        // Infer layout based on position count
        let layout = if positions.len() == bit_count {
            LayoutFunction::OneToOne
        } else if positions.len() == bit_count.div_ceil(4) {
            LayoutFunction::Packed4
        } else {
            return Err(format!(
                "Cannot infer layout: {} bits need {} positions (OneToOne) or {} positions (Packed4), but got {}",
                bit_count,
                bit_count,
                bit_count.div_ceil(4),
                positions.len()
            ));
        };

        self.add_output(name, io_type, layout, positions)
    }

    /// Set the cell face of an already-added port (input or output)
    pub fn port_face(mut self, name: &str, face: Face) -> Result<Self, String> {
        if let Some(m) = self.inputs.get_mut(name) {
            m.face = Some(face);
        } else if let Some(m) = self.outputs.get_mut(name) {
            m.face = Some(face);
        } else {
            return Err(format!("No port named '{}' to set face on", name));
        }
        Ok(self)
    }

    /// Add a bus-typed input: wire positions are derived from the spec's
    /// pitch starting at `bit0`, and the port is registered both as a regular
    /// typed input (so the executor's word set/read binds to it by name) and
    /// as a [`BusPort`] carrying the geometry/encoding contract.
    pub fn add_bus_input(
        self,
        name: impl Into<String>,
        spec: BusSpec,
        bit0: (i32, i32, i32),
    ) -> Result<Self, String> {
        self.add_bus_port(name, spec, bit0, PortDirection::Input)
    }

    /// Add a bus-typed output. See [`Self::add_bus_input`].
    pub fn add_bus_output(
        self,
        name: impl Into<String>,
        spec: BusSpec,
        bit0: (i32, i32, i32),
    ) -> Result<Self, String> {
        self.add_bus_port(name, spec, bit0, PortDirection::Output)
    }

    fn add_bus_port(
        mut self,
        name: impl Into<String>,
        spec: BusSpec,
        bit0: (i32, i32, i32),
        direction: PortDirection,
    ) -> Result<Self, String> {
        let name = name.into();
        spec.validate()?;
        let positions = spec.wire_positions(bit0);
        let layout = spec.encoding.layout_function();
        let io_type = spec.ty.clone();
        let face = spec.face;
        self = match direction {
            PortDirection::Input => self.add_input(name.clone(), io_type, layout, positions)?,
            PortDirection::Output => self.add_output(name.clone(), io_type, layout, positions)?,
        };
        // Record face on the underlying mapping too, so port and bus agree.
        self = self.port_face(&name, face)?;
        self.buses.insert(
            name,
            BusPort {
                spec,
                bit0,
                direction,
            },
        );
        Ok(self)
    }

    /// Merge with another builder
    /// Returns error if there are duplicate names
    pub fn merge(mut self, other: IoLayoutBuilder) -> Result<Self, String> {
        // Merge inputs
        for (name, mapping) in other.inputs {
            if self.inputs.contains_key(&name) {
                return Err(format!("Duplicate input name during merge: {}", name));
            }
            self.inputs.insert(name, mapping);
        }

        // Merge outputs
        for (name, mapping) in other.outputs {
            if self.outputs.contains_key(&name) {
                return Err(format!("Duplicate output name during merge: {}", name));
            }
            self.outputs.insert(name, mapping);
        }

        // Merge bus ports
        for (name, bus) in other.buses {
            if self.buses.contains_key(&name) {
                return Err(format!("Duplicate bus name during merge: {}", name));
            }
            self.buses.insert(name, bus);
        }

        Ok(self)
    }

    /// Build the final IO layout
    pub fn build(self) -> IoLayout {
        IoLayout {
            inputs: self.inputs,
            outputs: self.outputs,
            buses: self.buses,
        }
    }

    /// Get the number of inputs defined
    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    /// Get the number of outputs defined
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }
}

impl Default for IoLayoutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete IO layout for a circuit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IoLayout {
    pub inputs: HashMap<String, IoMapping>,
    pub outputs: HashMap<String, IoMapping>,

    /// Bus ports by name: geometry/encoding contracts layered over the
    /// identically-named entries in `inputs`/`outputs`. The executor's word
    /// set/read binds to a bus through that shared name.
    // TODO(executor): skew-aware reads and HexAnalog drive need explicit
    // executor support; today binding falls back to the port mapping.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub buses: HashMap<String, BusPort>,
}

impl IoLayout {
    /// Get a bus port by name
    pub fn get_bus(&self, name: &str) -> Option<&BusPort> {
        self.buses.get(name)
    }

    /// Create a new builder
    pub fn builder() -> IoLayoutBuilder {
        IoLayoutBuilder::new()
    }

    /// Get an input mapping by name
    pub fn get_input(&self, name: &str) -> Option<&IoMapping> {
        self.inputs.get(name)
    }

    /// Get an output mapping by name
    pub fn get_output(&self, name: &str) -> Option<&IoMapping> {
        self.outputs.get(name)
    }

    /// Get all input names
    pub fn input_names(&self) -> Vec<&str> {
        self.inputs.keys().map(|s| s.as_str()).collect()
    }

    /// Get all output names
    pub fn output_names(&self) -> Vec<&str> {
        self.outputs.keys().map(|s| s.as_str()).collect()
    }

    /// Validate the entire layout
    pub fn validate(&self) -> Result<(), String> {
        // Validate all inputs
        for (name, mapping) in &self.inputs {
            mapping
                .validate()
                .map_err(|e| format!("Input '{}': {}", name, e))?;
        }

        // Validate all outputs
        for (name, mapping) in &self.outputs {
            mapping
                .validate()
                .map_err(|e| format!("Output '{}': {}", name, e))?;
        }

        // Validate bus ports: spec consistency + a matching typed port
        for (name, bus) in &self.buses {
            bus.spec
                .validate()
                .map_err(|e| format!("Bus '{}': {}", name, e))?;
            if !self.inputs.contains_key(name) && !self.outputs.contains_key(name) {
                return Err(format!("Bus '{}' has no matching input/output port", name));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let layout = IoLayoutBuilder::new()
            .add_input(
                "a",
                IoType::UnsignedInt { bits: 8 },
                LayoutFunction::OneToOne,
                vec![
                    (0, 0, 0),
                    (1, 0, 0),
                    (2, 0, 0),
                    (3, 0, 0),
                    (4, 0, 0),
                    (5, 0, 0),
                    (6, 0, 0),
                    (7, 0, 0),
                ],
            )
            .unwrap()
            .add_output(
                "result",
                IoType::Boolean,
                LayoutFunction::OneToOne,
                vec![(10, 0, 0)],
            )
            .unwrap()
            .build();

        assert_eq!(layout.inputs.len(), 1);
        assert_eq!(layout.outputs.len(), 1);
        assert!(layout.get_input("a").is_some());
        assert!(layout.get_output("result").is_some());
    }

    #[test]
    fn test_builder_auto_inference() {
        // OneToOne inference (8 bits, 8 positions)
        let layout = IoLayoutBuilder::new()
            .add_input_auto(
                "a",
                IoType::UnsignedInt { bits: 8 },
                vec![
                    (0, 0, 0),
                    (1, 0, 0),
                    (2, 0, 0),
                    (3, 0, 0),
                    (4, 0, 0),
                    (5, 0, 0),
                    (6, 0, 0),
                    (7, 0, 0),
                ],
            )
            .unwrap()
            .build();

        let mapping = layout.get_input("a").unwrap();
        assert!(matches!(mapping.layout, LayoutFunction::OneToOne));

        // Packed4 inference (8 bits, 2 positions)
        let layout = IoLayoutBuilder::new()
            .add_input_auto(
                "b",
                IoType::UnsignedInt { bits: 8 },
                vec![(0, 0, 0), (1, 0, 0)],
            )
            .unwrap()
            .build();

        let mapping = layout.get_input("b").unwrap();
        assert!(matches!(mapping.layout, LayoutFunction::Packed4));
    }

    #[test]
    fn test_builder_merge() {
        let builder1 = IoLayoutBuilder::new()
            .add_input(
                "a",
                IoType::UnsignedInt { bits: 8 },
                LayoutFunction::OneToOne,
                vec![(0, 0, 0); 8],
            )
            .unwrap();

        let builder2 = IoLayoutBuilder::new()
            .add_input(
                "b",
                IoType::UnsignedInt { bits: 8 },
                LayoutFunction::OneToOne,
                vec![(10, 0, 0); 8],
            )
            .unwrap();

        let layout = builder1.merge(builder2).unwrap().build();

        assert_eq!(layout.inputs.len(), 2);
        assert!(layout.get_input("a").is_some());
        assert!(layout.get_input("b").is_some());
    }

    #[test]
    fn test_builder_duplicate_error() {
        let result = IoLayoutBuilder::new()
            .add_input(
                "a",
                IoType::Boolean,
                LayoutFunction::OneToOne,
                vec![(0, 0, 0)],
            )
            .unwrap()
            .add_input(
                "a", // Duplicate!
                IoType::Boolean,
                LayoutFunction::OneToOne,
                vec![(1, 0, 0)],
            );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate input name"));
    }

    #[test]
    fn test_layout_validation() {
        let layout = IoLayoutBuilder::new()
            .add_input(
                "a",
                IoType::UnsignedInt { bits: 8 },
                LayoutFunction::OneToOne,
                vec![(0, 0, 0); 8],
            )
            .unwrap()
            .build();

        assert!(layout.validate().is_ok());
    }

    #[test]
    fn test_layout_names() {
        let layout = IoLayoutBuilder::new()
            .add_input(
                "input_a",
                IoType::Boolean,
                LayoutFunction::OneToOne,
                vec![(0, 0, 0)],
            )
            .unwrap()
            .add_input(
                "input_b",
                IoType::Boolean,
                LayoutFunction::OneToOne,
                vec![(1, 0, 0)],
            )
            .unwrap()
            .add_output(
                "output",
                IoType::Boolean,
                LayoutFunction::OneToOne,
                vec![(10, 0, 0)],
            )
            .unwrap()
            .build();

        let input_names = layout.input_names();
        assert_eq!(input_names.len(), 2);
        assert!(input_names.contains(&"input_a"));
        assert!(input_names.contains(&"input_b"));

        let output_names = layout.output_names();
        assert_eq!(output_names.len(), 1);
        assert!(output_names.contains(&"output"));
    }

    #[test]
    fn test_bus_port_binding_and_serde() {
        use crate::io_contract::bus::{BusEncoding, BusSpec, Pitch};
        use crate::io_contract::physical::{Face, PortDirection};
        use crate::transforms::Axis;

        let spec = BusSpec {
            width: 4,
            ty: IoType::UnsignedInt { bits: 4 },
            pitch: Pitch {
                axis: Axis::Z,
                spacing: 2,
            },
            face: Face::West,
            encoding: BusEncoding::Binary1PerWire,
        };
        let layout = IoLayoutBuilder::new()
            .add_bus_input("a", spec.clone(), (0, 0, 0))
            .unwrap()
            .add_bus_output("sum", spec.clone(), (9, 0, 0))
            .unwrap()
            .build();

        layout.validate().unwrap();

        // Bus registered and typed port materialized at pitch offsets
        let bus = layout.get_bus("a").unwrap();
        assert_eq!(bus.direction, PortDirection::Input);
        let mapping = layout.get_input("a").unwrap();
        assert_eq!(
            mapping.positions,
            vec![(0, 0, 0), (0, 0, 2), (0, 0, 4), (0, 0, 6)]
        );
        assert_eq!(mapping.face, Some(Face::West));
        assert_eq!(mapping.direction, Some(PortDirection::Input));

        // JSON round trip preserves buses + per-port face/direction
        let json = serde_json::to_string(&layout).unwrap();
        let back: IoLayout = serde_json::from_str(&json).unwrap();
        assert_eq!(back, layout);
    }

    #[test]
    fn test_port_face_setter() {
        let layout = IoLayoutBuilder::new()
            .add_input(
                "a",
                IoType::Boolean,
                LayoutFunction::OneToOne,
                vec![(0, 0, 0)],
            )
            .unwrap()
            .port_face("a", crate::io_contract::physical::Face::North)
            .unwrap()
            .build();
        assert_eq!(
            layout.get_input("a").unwrap().face,
            Some(crate::io_contract::physical::Face::North)
        );
        assert!(IoLayoutBuilder::new().port_face("ghost", crate::io_contract::physical::Face::Up).is_err());
    }
}
