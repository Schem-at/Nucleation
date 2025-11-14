//! Builder for constructing IO layouts
//!
//! Provides a fluent API for defining circuit inputs and outputs with types and layouts.

use super::{IoMapping, IoType, LayoutFunction};
use std::collections::HashMap;

/// Builder for constructing IO layouts
#[derive(Debug, Clone)]
pub struct IoLayoutBuilder {
    inputs: HashMap<String, IoMapping>,
    outputs: HashMap<String, IoMapping>,
}

impl IoLayoutBuilder {
    /// Create a new empty IO layout builder
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
            outputs: HashMap::new(),
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
        } else if positions.len() == (bit_count + 3) / 4 {
            LayoutFunction::Packed4
        } else {
            return Err(format!(
                "Cannot infer layout: {} bits need {} positions (OneToOne) or {} positions (Packed4), but got {}",
                bit_count,
                bit_count,
                (bit_count + 3) / 4,
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
        } else if positions.len() == (bit_count + 3) / 4 {
            LayoutFunction::Packed4
        } else {
            return Err(format!(
                "Cannot infer layout: {} bits need {} positions (OneToOne) or {} positions (Packed4), but got {}",
                bit_count,
                bit_count,
                (bit_count + 3) / 4,
                positions.len()
            ));
        };

        self.add_output(name, io_type, layout, positions)
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

        Ok(self)
    }

    /// Build the final IO layout
    pub fn build(self) -> IoLayout {
        IoLayout {
            inputs: self.inputs,
            outputs: self.outputs,
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
#[derive(Debug, Clone)]
pub struct IoLayout {
    pub inputs: HashMap<String, IoMapping>,
    pub outputs: HashMap<String, IoMapping>,
}

impl IoLayout {
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
}
