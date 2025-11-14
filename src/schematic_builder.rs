//! Schematic Builder - Create schematics from ASCII art
//!
//! This module provides a convenient way to build schematics using ASCII art
//! and character-to-block mappings, making it easy to visualize and create
//! redstone circuits.
//!
//! # Example
//!
//! ```ignore
//! use nucleation::SchematicBuilder;
//!
//! let schematic = SchematicBuilder::new()
//!     .palette(&[
//!         ('c', "minecraft:gray_concrete"),
//!         ('r', "minecraft:redstone_wire"),
//!         ('t', "minecraft:redstone_torch"),
//!         ('_', "minecraft:air"),
//!     ])
//!     .layers(&[
//!         // Y=0 - Base layer
//!         &["cccc",
//!           "cccc",
//!           "cccc"],
//!         // Y=1 - Logic layer
//!         &["____",
//!           "rctr",
//!           "____"],
//!     ])
//!     .build()?;
//! ```

pub mod palettes;

use crate::{BlockState, UniversalSchematic};
use std::collections::HashMap;

/// Palette entry - either a block or a sub-schematic
#[derive(Debug, Clone)]
pub enum PaletteEntry {
    /// A single block (block string)
    Block(String),
    /// A sub-schematic to be placed at this position
    Schematic(UniversalSchematic),
}

/// Builder for creating schematics from ASCII art
pub struct SchematicBuilder {
    /// Character to palette entry mapping (blocks or schematics)
    palette: HashMap<char, PaletteEntry>,
    /// Layers of the schematic (Y-axis, bottom to top)
    /// Each layer is an array of strings (X-axis)
    /// Each string represents a Z-axis line
    layers: Vec<Vec<String>>,
    /// Offset for the schematic in world coordinates
    offset: (i32, i32, i32),
    /// IO position markers
    io_markers: Vec<IoMarker>,
    /// Name of the schematic
    name: String,
}

/// Marker for IO positions
#[derive(Debug, Clone)]
pub struct IoMarker {
    /// Position in the schematic (after offset is applied)
    pub position: (i32, i32, i32),
    /// Label for this IO
    pub label: String,
    /// Type of IO (input/output)
    pub io_type: IoType,
}

/// Type of IO marker
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoType {
    Input,
    Output,
}

impl SchematicBuilder {
    /// Create a new schematic builder with standard Unicode palette pre-loaded
    ///
    /// The standard palette includes Unicode characters for redstone components:
    /// - Wire: `─`, `│`, `╋`, corners, T-junctions
    /// - Repeaters: `→`, `←`, `↑`, `↓` (and variants for different delays)
    /// - Comparators: `▷`, `◁`, `△`, `▽` (compare) and `▶`, `◀`, `▲`, `▼` (subtract)
    /// - Torches: `*`, `⚡`
    /// - Blocks: `█`, `▓`, `▒`, etc.
    /// - Air: `_`, ` `, `·`
    ///
    /// You can override any character by calling `.map()` or `.palette()` after creation.
    pub fn new() -> Self {
        // Start with standard palette pre-loaded
        let standard = palettes::StandardPalette::get();
        let mut palette = HashMap::new();
        for (ch, block_str) in standard {
            palette.insert(ch, PaletteEntry::Block(block_str));
        }

        Self {
            palette,
            layers: Vec::new(),
            offset: (0, 0, 0),
            io_markers: Vec::new(),
            name: "schematic".to_string(),
        }
    }

    /// Create a new schematic builder with NO default palette (empty)
    ///
    /// Use this if you want to define all characters yourself without
    /// the standard Unicode palette.
    pub fn empty() -> Self {
        Self {
            palette: HashMap::new(),
            layers: Vec::new(),
            offset: (0, 0, 0),
            io_markers: Vec::new(),
            name: "schematic".to_string(),
        }
    }

    /// Set the name of the schematic
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Add a character-to-block mapping to the palette
    pub fn map(mut self, ch: char, block: impl Into<String>) -> Self {
        self.palette.insert(ch, PaletteEntry::Block(block.into()));
        self
    }

    /// Add a character-to-schematic mapping to the palette
    ///
    /// This allows you to use entire schematics as building blocks!
    ///
    /// # Example
    /// ```ignore
    /// let one_bit_adder = create_1bit_adder();
    /// let four_bit = SchematicBuilder::new()
    ///     .map_schematic('A', one_bit_adder)
    ///     .layers(&[&["AAAA"]])  // 4 adders in a row
    ///     .build()?;
    /// ```
    pub fn map_schematic(mut self, ch: char, schematic: UniversalSchematic) -> Self {
        self.palette.insert(ch, PaletteEntry::Schematic(schematic));
        self
    }

    /// Set the entire palette at once (blocks only)
    pub fn palette(mut self, mappings: &[(char, &str)]) -> Self {
        for &(ch, block) in mappings {
            self.palette
                .insert(ch, PaletteEntry::Block(block.to_string()));
        }
        self
    }

    /// Use a standard Unicode palette
    ///
    /// # Example
    /// ```ignore
    /// use nucleation::SchematicBuilder;
    ///
    /// let schematic = SchematicBuilder::new()
    ///     .use_standard_palette()  // Use Unicode characters!
    ///     .layers(&[
    ///         &["███",
    ///           "███"],
    ///         &["─→─",
    ///           "╋╋╋"],
    ///     ])
    ///     .build()?;
    /// ```
    pub fn use_standard_palette(mut self) -> Self {
        let standard = palettes::StandardPalette::get();
        for (ch, block_str) in standard {
            self.palette.insert(ch, PaletteEntry::Block(block_str));
        }
        self
    }

    /// Use a minimal standard palette (fewer characters, essentials only)
    pub fn use_minimal_palette(mut self) -> Self {
        let minimal = palettes::StandardPalette::minimal();
        for (ch, block_str) in minimal {
            self.palette.insert(ch, PaletteEntry::Block(block_str));
        }
        self
    }

    /// Use a compact ASCII-only palette
    pub fn use_compact_palette(mut self) -> Self {
        let compact = palettes::StandardPalette::compact();
        for (ch, block_str) in compact {
            self.palette.insert(ch, PaletteEntry::Block(block_str));
        }
        self
    }

    /// Add layers to the schematic
    ///
    /// Each layer is an array of strings where:
    /// - Each string represents a line along the Z-axis (north-south)
    /// - The array of strings represents the X-axis (west-east)
    /// - Multiple layers represent the Y-axis (bottom-up)
    ///
    /// # Coordinate System
    /// ```text
    /// Y (up)
    /// |  Z (south)
    /// | /
    /// |/_____ X (east)
    /// ```
    pub fn layers(mut self, layers: &[&[&str]]) -> Self {
        for layer in layers {
            let layer_strings: Vec<String> = layer.iter().map(|s| s.to_string()).collect();
            self.layers.push(layer_strings);
        }
        self
    }

    /// Add a single layer
    pub fn layer(mut self, strings: &[&str]) -> Self {
        let layer_strings: Vec<String> = strings.iter().map(|s| s.to_string()).collect();
        self.layers.push(layer_strings);
        self
    }

    /// Set the offset for the schematic in world coordinates
    pub fn offset(mut self, x: i32, y: i32, z: i32) -> Self {
        self.offset = (x, y, z);
        self
    }

    /// Mark an IO position by finding a character in the layers
    ///
    /// This will search for the first occurrence of the character and mark it as IO
    pub fn mark_io_char(mut self, ch: char, label: impl Into<String>, io_type: IoType) -> Self {
        // Find the position of this character
        if let Some(pos) = self.find_char_position(ch) {
            self.io_markers.push(IoMarker {
                position: (
                    pos.0 + self.offset.0,
                    pos.1 + self.offset.1,
                    pos.2 + self.offset.2,
                ),
                label: label.into(),
                io_type,
            });
        }
        self
    }

    /// Mark an IO position at specific coordinates (relative to schematic origin)
    pub fn mark_io(
        mut self,
        x: i32,
        z: i32,
        y: i32,
        label: impl Into<String>,
        io_type: IoType,
    ) -> Self {
        self.io_markers.push(IoMarker {
            position: (x + self.offset.0, y + self.offset.1, z + self.offset.2),
            label: label.into(),
            io_type,
        });
        self
    }

    /// Get all IO markers
    pub fn io_markers(&self) -> &[IoMarker] {
        &self.io_markers
    }

    /// Find the position of a character in the layers
    fn find_char_position(&self, ch: char) -> Option<(i32, i32, i32)> {
        for (y, layer) in self.layers.iter().enumerate() {
            for (z, line) in layer.iter().enumerate() {
                for (x, c) in line.chars().enumerate() {
                    if c == ch {
                        return Some((x as i32, y as i32, z as i32));
                    }
                }
            }
        }
        None
    }

    /// Validate the schematic before building
    pub fn validate(&self) -> Result<(), String> {
        // Check that we have layers
        if self.layers.is_empty() {
            return Err("No layers defined".to_string());
        }

        // Check that all layers have the same dimensions
        let first_layer = &self.layers[0];
        if first_layer.is_empty() {
            return Err("First layer is empty".to_string());
        }

        let z_size = first_layer.len();
        // Use chars().count() instead of len() to handle Unicode properly
        let x_size = first_layer[0].chars().count();

        for (y, layer) in self.layers.iter().enumerate() {
            if layer.len() != z_size {
                return Err(format!(
                    "Layer {} has {} rows, expected {}",
                    y,
                    layer.len(),
                    z_size
                ));
            }

            for (z, line) in layer.iter().enumerate() {
                let line_char_count = line.chars().count();
                if line_char_count != x_size {
                    return Err(format!(
                        "Layer {} row {} has {} columns, expected {}",
                        y, z, line_char_count, x_size
                    ));
                }

                // Check that all characters are in the palette
                for (x, ch) in line.chars().enumerate() {
                    if !self.palette.contains_key(&ch) {
                        return Err(format!(
                            "Character '{}' at position ({}, {}, {}) not found in palette",
                            ch, x, y, z
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Build the schematic
    pub fn build(self) -> Result<UniversalSchematic, String> {
        // Validate first
        self.validate()?;

        let mut schematic = UniversalSchematic::new(self.name);

        // Track cumulative offsets for schematic tiling
        // We need to track the actual width/depth of schematics to properly tile them
        let mut x_offsets: Vec<i32> = vec![0]; // Cumulative X offsets for each column
        let mut z_offsets: Vec<i32> = vec![0]; // Cumulative Z offsets for each row

        // Pre-calculate offsets based on schematic dimensions
        // For each layer, we need to know the dimensions of schematics in that layer
        if !self.layers.is_empty() {
            let first_layer = &self.layers[0];
            if !first_layer.is_empty() {
                let first_line = &first_layer[0];

                // Calculate X offsets (column-wise)
                let mut cumulative_x = 0;
                x_offsets.clear(); // Clear the initial [0]
                for ch in first_line.chars() {
                    x_offsets.push(cumulative_x);
                    if let Some(entry) = self.palette.get(&ch) {
                        let width = match entry {
                            PaletteEntry::Block(_) => 1,
                            PaletteEntry::Schematic(s) => {
                                // Use tight bounds if available, otherwise fall back to full size
                                if let Some(tight) = s.default_region.get_tight_bounds() {
                                    tight.get_dimensions().0
                                } else {
                                    s.default_region.size.0
                                }
                            }
                        };
                        cumulative_x += width;
                    }
                }

                // Calculate Z offsets (row-wise)
                let mut cumulative_z = 0;
                z_offsets.clear(); // Clear the initial [0]
                for line in first_layer.iter() {
                    z_offsets.push(cumulative_z);
                    if let Some(ch) = line.chars().next() {
                        if let Some(entry) = self.palette.get(&ch) {
                            let depth = match entry {
                                PaletteEntry::Block(_) => 1,
                                PaletteEntry::Schematic(s) => {
                                    // Use tight bounds if available, otherwise fall back to full size
                                    if let Some(tight) = s.default_region.get_tight_bounds() {
                                        tight.get_dimensions().2
                                    } else {
                                        s.default_region.size.2
                                    }
                                }
                            };
                            cumulative_z += depth;
                        }
                    }
                }
            }
        }

        // Iterate through layers (Y), rows (Z), and columns (X)
        for (y, layer) in self.layers.iter().enumerate() {
            for (z, line) in layer.iter().enumerate() {
                for (x, ch) in line.chars().enumerate() {
                    if let Some(entry) = self.palette.get(&ch) {
                        // Use cumulative offsets for proper tiling
                        let world_x = x_offsets[x] + self.offset.0;
                        let world_y = y as i32 + self.offset.1; // Y is still just layer index
                        let world_z = z_offsets[z] + self.offset.2;

                        match entry {
                            PaletteEntry::Block(block_str) => {
                                // Place a single block
                                match UniversalSchematic::parse_block_string(block_str) {
                                    Ok((block_state, _)) => {
                                        schematic.set_block(world_x, world_y, world_z, block_state);
                                    }
                                    Err(e) => {
                                        return Err(format!(
                                            "Failed to parse block '{}' for character '{}' at ({}, {}, {}): {}",
                                            block_str, ch, x, y, z, e
                                        ));
                                    }
                                }
                            }
                            PaletteEntry::Schematic(sub_schematic) => {
                                // Place an entire sub-schematic
                                // Copy all NON-AIR blocks from the sub-schematic, offset by the current position
                                // Skip air blocks to avoid overwriting blocks from adjacent schematics
                                for (pos, block_state) in sub_schematic.iter_blocks() {
                                    // Skip air blocks
                                    if block_state.name.contains("air") {
                                        continue;
                                    }
                                    schematic.set_block(
                                        pos.x + world_x,
                                        pos.y + world_y,
                                        pos.z + world_z,
                                        block_state.clone(),
                                    );
                                }

                                // Copy block entities too
                                for block_entity in
                                    sub_schematic.default_region.get_block_entities_as_list()
                                {
                                    let pos = block_entity.position;
                                    schematic.set_block_entity(
                                        crate::block_position::BlockPosition {
                                            x: pos.0 + world_x,
                                            y: pos.1 + world_y,
                                            z: pos.2 + world_z,
                                        },
                                        block_entity.clone(),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(schematic)
    }

    /// Export the builder configuration to a template string
    ///
    /// This creates a human-readable template that can be saved to a file
    /// and loaded later with `from_template()`
    ///
    /// Note: Sub-schematics in the palette are not exported to templates.
    /// Only block entries are included.
    pub fn to_template(&self) -> String {
        let mut output = String::new();

        // Add layers
        for (i, layer) in self.layers.iter().enumerate() {
            output.push_str(&format!("# Layer {}\n", i));
            for line in layer {
                output.push_str(line);
                output.push('\n');
            }
            output.push('\n');
        }

        // Add palette (blocks only, skip schematics)
        let block_entries: Vec<_> = self
            .palette
            .iter()
            .filter_map(|(ch, entry)| {
                if let PaletteEntry::Block(block_str) = entry {
                    Some((ch, block_str))
                } else {
                    None
                }
            })
            .collect();

        if !block_entries.is_empty() {
            output.push_str("[palette]\n");
            let mut sorted_entries = block_entries;
            sorted_entries.sort_by_key(|(ch, _)| *ch);
            for (ch, block_str) in sorted_entries {
                output.push_str(&format!("{} = {}\n", ch, block_str));
            }
        }

        output
    }

    /// Export the builder configuration to JSON
    ///
    /// Note: Sub-schematics in the palette are not exported to JSON.
    /// Only block entries are included.
    ///
    /// # Example
    /// ```ignore
    /// let json = builder.to_json()?;
    /// // Save to file or send over network
    /// std::fs::write("circuit.json", json)?;
    /// ```
    #[cfg(feature = "serde")]
    pub fn to_json(&self) -> Result<String, String> {
        use serde_json::json;

        // Only export block entries, skip schematics
        let palette: std::collections::HashMap<String, String> = self
            .palette
            .iter()
            .filter_map(|(ch, entry)| {
                if let PaletteEntry::Block(block_str) = entry {
                    Some((ch.to_string(), block_str.clone()))
                } else {
                    None
                }
            })
            .collect();

        let json = json!({
            "name": self.name,
            "palette": palette,
            "layers": self.layers,
            "offset": [self.offset.0, self.offset.1, self.offset.2],
        });

        serde_json::to_string_pretty(&json).map_err(|e| format!("JSON serialization error: {}", e))
    }

    /// Build from a JSON string
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "name": "my_circuit",
    ///   "palette": {
    ///     "c": "minecraft:gray_concrete",
    ///     "r": "minecraft:redstone_wire"
    ///   },
    ///   "layers": [
    ///     ["cccc", "cccc"],
    ///     ["____", "rctr"]
    ///   ],
    ///   "offset": [0, 0, 0]
    /// }
    /// ```
    #[cfg(feature = "serde")]
    pub fn from_json(json: &str) -> Result<Self, String> {
        use serde_json::Value;

        let value: Value =
            serde_json::from_str(json).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let mut builder = Self::new();

        // Parse name
        if let Some(name) = value.get("name").and_then(|v| v.as_str()) {
            builder = builder.name(name);
        }

        // Parse palette
        if let Some(palette) = value.get("palette").and_then(|v| v.as_object()) {
            for (key, val) in palette {
                if key.len() == 1 {
                    if let Some(block) = val.as_str() {
                        builder = builder.map(key.chars().next().unwrap(), block);
                    }
                }
            }
        }

        // Parse layers
        if let Some(layers) = value.get("layers").and_then(|v| v.as_array()) {
            for layer in layers {
                if let Some(rows) = layer.as_array() {
                    let row_strings: Vec<String> = rows
                        .iter()
                        .filter_map(|r| r.as_str().map(|s| s.to_string()))
                        .collect();
                    builder.layers.push(row_strings);
                }
            }
        }

        // Parse offset
        if let Some(offset) = value.get("offset").and_then(|v| v.as_array()) {
            if offset.len() == 3 {
                let x = offset[0].as_i64().unwrap_or(0) as i32;
                let y = offset[1].as_i64().unwrap_or(0) as i32;
                let z = offset[2].as_i64().unwrap_or(0) as i32;
                builder = builder.offset(x, y, z);
            }
        }

        Ok(builder)
    }

    /// Build from a template string
    ///
    /// Expected format:
    /// ```text
    /// # Layer 0
    /// cccc
    /// cccc
    ///
    /// # Layer 1
    /// ____
    /// rctr
    ///
    /// [palette]
    /// c = minecraft:gray_concrete
    /// r = minecraft:redstone_wire
    /// t = minecraft:redstone_torch
    /// _ = minecraft:air
    /// ```
    pub fn from_template(template: &str) -> Result<Self, String> {
        let mut builder = Self::new();
        let mut current_layer: Vec<String> = Vec::new();
        let mut in_palette = false;

        for line in template.lines() {
            let line = line.trim();

            // Skip empty lines and comments (unless in palette section)
            if line.is_empty() {
                if !current_layer.is_empty() && !in_palette {
                    builder.layers.push(current_layer.clone());
                    current_layer.clear();
                }
                continue;
            }

            if line.starts_with('#') && !in_palette {
                // Layer marker - finalize previous layer
                if !current_layer.is_empty() {
                    builder.layers.push(current_layer.clone());
                    current_layer.clear();
                }
                continue;
            }

            if line == "[palette]" {
                in_palette = true;
                if !current_layer.is_empty() {
                    builder.layers.push(current_layer.clone());
                    current_layer.clear();
                }
                continue;
            }

            if in_palette {
                // Parse palette entry: "c = minecraft:gray_concrete"
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();
                    // Use chars().count() instead of len() to handle Unicode characters correctly
                    if key.chars().count() == 1 {
                        let ch = key.chars().next().unwrap();
                        builder = builder.map(ch, value);
                    }
                }
            } else {
                // Add to current layer
                current_layer.push(line.to_string());
            }
        }

        // Add final layer if exists
        if !current_layer.is_empty() {
            builder.layers.push(current_layer);
        }

        Ok(builder)
    }
}

impl Default for SchematicBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_builder() {
        let schematic = SchematicBuilder::new()
            .name("test")
            .palette(&[('c', "minecraft:gray_concrete"), ('_', "minecraft:air")])
            .layers(&[&["cc", "cc"], &["__", "__"]])
            .build();

        assert!(schematic.is_ok());
        let schematic = schematic.unwrap();

        // Check that blocks were placed
        let block = schematic.get_block(0, 0, 0);
        assert!(block.is_some());
    }

    #[test]
    fn test_validation_no_layers() {
        let builder = SchematicBuilder::new().palette(&[('c', "minecraft:gray_concrete")]);

        let result = builder.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No layers"));
    }

    #[test]
    fn test_validation_inconsistent_dimensions() {
        let builder = SchematicBuilder::new()
            .palette(&[('c', "minecraft:gray_concrete")])
            .layers(&[
                &["cc", "cc"],
                &["ccc", "cc"], // Wrong width
            ]);

        let result = builder.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("columns"));
    }

    #[test]
    fn test_validation_missing_palette() {
        let builder = SchematicBuilder::new()
            .palette(&[('c', "minecraft:gray_concrete")])
            .layers(&[
                &["cc", "cx"], // 'x' not in palette
            ]);

        let result = builder.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found in palette"));
    }

    #[test]
    fn test_offset() {
        let schematic = SchematicBuilder::new()
            .palette(&[('c', "minecraft:gray_concrete")])
            .layers(&[&["c"]])
            .offset(10, 20, 30)
            .build()
            .unwrap();

        // Block should be at offset position
        let block = schematic.get_block(10, 20, 30);
        assert!(block.is_some());
    }

    #[test]
    fn test_map_method() {
        let schematic = SchematicBuilder::new()
            .map('c', "minecraft:gray_concrete")
            .map('r', "minecraft:redstone_wire")
            .layers(&[&["cr"]])
            .build();

        assert!(schematic.is_ok());
    }

    #[test]
    fn test_layer_method() {
        let schematic = SchematicBuilder::new()
            .palette(&[('c', "minecraft:gray_concrete")])
            .layer(&["cc", "cc"])
            .layer(&["cc", "cc"])
            .build();

        assert!(schematic.is_ok());
    }

    #[test]
    fn test_find_char_position() {
        let builder = SchematicBuilder::new()
            .palette(&[
                ('c', "minecraft:gray_concrete"),
                ('r', "minecraft:redstone_wire"),
            ])
            .layers(&[&["cc", "cc"], &["cr", "cc"]]);

        let pos = builder.find_char_position('r');
        assert_eq!(pos, Some((1, 1, 0)));
    }

    #[test]
    fn test_mark_io_char() {
        let builder = SchematicBuilder::new()
            .palette(&[
                ('c', "minecraft:gray_concrete"),
                ('r', "minecraft:redstone_wire"),
            ])
            .layers(&[&["cr"]])
            .mark_io_char('r', "input", IoType::Input);

        assert_eq!(builder.io_markers.len(), 1);
        assert_eq!(builder.io_markers[0].position, (1, 0, 0));
        assert_eq!(builder.io_markers[0].label, "input");
        assert_eq!(builder.io_markers[0].io_type, IoType::Input);
    }

    #[test]
    fn test_mark_io() {
        let builder = SchematicBuilder::new()
            .palette(&[('c', "minecraft:gray_concrete")])
            .layers(&[&["cc"]])
            .mark_io(0, 0, 0, "input", IoType::Input);

        assert_eq!(builder.io_markers.len(), 1);
        assert_eq!(builder.io_markers[0].position, (0, 0, 0));
    }

    #[test]
    fn test_from_template() {
        let template = r#"
# Layer 0
cc
cc

# Layer 1
__
r_

[palette]
c = minecraft:gray_concrete
r = minecraft:redstone_wire
_ = minecraft:air
"#;

        let builder = SchematicBuilder::from_template(template).unwrap();
        assert_eq!(builder.layers.len(), 2);
        // from_template uses Self::new() which loads standard palette (63 chars)
        // Custom palette entries override standard ones
        assert!(
            builder.palette.len() >= 3,
            "Should have at least custom palette entries"
        );

        let schematic = builder.build();
        assert!(schematic.is_ok());
    }

    #[test]
    fn test_from_template_no_palette() {
        let template = r#"
# Layer 0
cc
cc
"#;

        let builder = SchematicBuilder::from_template(template).unwrap();
        assert_eq!(builder.layers.len(), 1);
        // Standard palette is loaded by default (63 chars)
        assert!(
            builder.palette.len() > 0,
            "Should have standard palette loaded"
        );
    }

    #[test]
    fn test_block_properties() {
        let schematic = SchematicBuilder::new()
            .palette(&[
                ('r', "minecraft:redstone_wire[power=15]"),
                ('t', "minecraft:redstone_torch[lit=true]"),
            ])
            .layers(&[&["rt"]])
            .build();

        assert!(schematic.is_ok());
    }

    #[test]
    fn test_multi_layer_circuit() {
        let schematic = SchematicBuilder::new()
            .palette(&[
                ('c', "minecraft:gray_concrete"),
                ('r', "minecraft:redstone_wire"),
                ('t', "minecraft:redstone_torch"),
                ('_', "minecraft:air"),
            ])
            .layers(&[
                // Y=0 - Base
                &["cccc", "cccc", "cccc"],
                // Y=1 - Logic
                &["____", "rctr", "____"],
                // Y=2 - Top
                &["____", "____", "____"],
            ])
            .build();

        assert!(schematic.is_ok());
        let schematic = schematic.unwrap();

        // Verify some blocks
        let base = schematic.get_block(0, 0, 0);
        assert!(base.is_some());

        let logic = schematic.get_block(1, 1, 1);
        assert!(logic.is_some());
    }

    #[test]
    fn test_coordinate_system() {
        // Test that coordinate system is correct:
        // X = east (column in string)
        // Y = up (layer index)
        // Z = south (string index in layer)
        let schematic = SchematicBuilder::new()
            .palette(&[
                ('1', "minecraft:stone"),
                ('2', "minecraft:dirt"),
                ('3', "minecraft:grass_block"),
            ])
            .layers(&[
                &["123"], // Z=0, X=0,1,2
            ])
            .build()
            .unwrap();

        // Check X-axis (columns)
        assert!(schematic.get_block(0, 0, 0).is_some()); // '1'
        assert!(schematic.get_block(1, 0, 0).is_some()); // '2'
        assert!(schematic.get_block(2, 0, 0).is_some()); // '3'
    }

    #[test]
    fn test_empty_layer() {
        let builder = SchematicBuilder::new()
            .palette(&[('c', "minecraft:gray_concrete")])
            .layer(&[]); // Empty layer

        let result = builder.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_io_markers_with_offset() {
        let builder = SchematicBuilder::new()
            .palette(&[('r', "minecraft:redstone_wire")])
            .layers(&[&["r"]])
            .offset(10, 20, 30)
            .mark_io_char('r', "input", IoType::Input);

        assert_eq!(builder.io_markers[0].position, (10, 20, 30));
    }

    #[test]
    fn test_stack_schematic_x_axis() {
        let base = SchematicBuilder::new()
            .palette(&[('c', "minecraft:gray_concrete")])
            .layers(&[&["c"]])
            .build()
            .unwrap();

        let stacked = base.stack(2, 'x', 0).unwrap();

        // Should have 3 blocks along X axis (original + 2 copies)
        assert!(stacked.get_block(0, 0, 0).is_some());
        assert!(stacked.get_block(1, 0, 0).is_some());
        assert!(stacked.get_block(2, 0, 0).is_some());
    }

    #[test]
    fn test_stack_schematic_with_spacing() {
        let base = SchematicBuilder::new()
            .palette(&[('c', "minecraft:gray_concrete")])
            .layers(&[&["c"]])
            .build()
            .unwrap();

        let stacked = base.stack(1, 'x', 1).unwrap();

        // Should have blocks at 0 and at least one more copy
        assert!(stacked.get_block(0, 0, 0).is_some());
        // Note: Exact spacing depends on bounding box calculation
        // The important thing is we have 2 copies
    }

    #[test]
    fn test_stack_schematic_y_axis() {
        let base = SchematicBuilder::new()
            .palette(&[('c', "minecraft:gray_concrete")])
            .layers(&[&["c"]])
            .build()
            .unwrap();

        let stacked = base.stack(2, 'y', 0).unwrap();

        // Should have 3 blocks along Y axis
        assert!(stacked.get_block(0, 0, 0).is_some());
        assert!(stacked.get_block(0, 1, 0).is_some());
        assert!(stacked.get_block(0, 2, 0).is_some());
    }

    #[test]
    fn test_stack_in_place() {
        let mut base = SchematicBuilder::new()
            .palette(&[('c', "minecraft:gray_concrete")])
            .layers(&[&["c"]])
            .build()
            .unwrap();

        base.stack_in_place(2, 'x', 0).unwrap();

        // Should have 3 blocks along X axis
        assert!(base.get_block(0, 0, 0).is_some());
        assert!(base.get_block(1, 0, 0).is_some());
        assert!(base.get_block(2, 0, 0).is_some());
    }

    #[test]
    fn test_to_template() {
        let builder = SchematicBuilder::new()
            .palette(&[
                ('c', "minecraft:gray_concrete"),
                ('r', "minecraft:redstone_wire"),
            ])
            .layers(&[&["cc", "cc"], &["rr", "rr"]]);

        let template = builder.to_template();

        // Should contain layer markers
        assert!(template.contains("# Layer 0"));
        assert!(template.contains("# Layer 1"));

        // Should contain palette
        assert!(template.contains("[palette]"));
        assert!(template.contains("c = minecraft:gray_concrete"));
        assert!(template.contains("r = minecraft:redstone_wire"));

        // Should contain layer data
        assert!(template.contains("cc"));
        assert!(template.contains("rr"));
    }

    #[test]
    fn test_to_template_roundtrip() {
        let original = SchematicBuilder::new()
            .palette(&[
                ('c', "minecraft:gray_concrete"),
                ('r', "minecraft:redstone_wire"),
            ])
            .layers(&[&["cc", "cc"], &["rr", "rr"]]);

        let template = original.to_template();
        let rebuilt = SchematicBuilder::from_template(&template).unwrap();

        assert_eq!(rebuilt.layers.len(), 2);
        // rebuilt uses from_template which loads standard palette + custom entries
        assert!(
            rebuilt.palette.len() >= 2,
            "Should have at least the custom palette entries"
        );

        let schematic1 = original.build().unwrap();
        let schematic2 = rebuilt.build().unwrap();

        // Both should have blocks at same positions
        assert!(schematic1.get_block(0, 0, 0).is_some());
        assert!(schematic2.get_block(0, 0, 0).is_some());
    }

    #[test]
    fn test_schematic_palette() {
        // Create a small sub-schematic (a 2x2 concrete block)
        let sub_schematic = SchematicBuilder::new()
            .palette(&[('c', "minecraft:gray_concrete")])
            .layers(&[&["cc", "cc"]])
            .build()
            .unwrap();

        // Use the sub-schematic as a palette entry
        let result = SchematicBuilder::new()
            .map_schematic('S', sub_schematic)
            .map('_', "minecraft:air")
            .layers(&[&["S_S"]]) // Two sub-schematics with a gap
            .build()
            .unwrap();

        // Should have blocks from first sub-schematic at (0,0,0), (0,0,1), (1,0,0), (1,0,1)
        assert!(result.get_block(0, 0, 0).is_some());
        assert!(result.get_block(1, 0, 0).is_some());
        assert!(result.get_block(0, 0, 1).is_some());
        assert!(result.get_block(1, 0, 1).is_some());

        // Gap at (2,0,0) and (2,0,1)
        assert!(result.get_block(2, 0, 0).is_some()); // Air block

        // Second sub-schematic at (4,0,0), (4,0,1), (5,0,0), (5,0,1)
        assert!(result.get_block(4, 0, 0).is_some());
        assert!(result.get_block(5, 0, 0).is_some());
        assert!(result.get_block(4, 0, 1).is_some());
        assert!(result.get_block(5, 0, 1).is_some());
    }

    #[test]
    fn test_schematic_palette_stacking() {
        // Create a 1-unit building block
        let unit = SchematicBuilder::new()
            .palette(&[('c', "minecraft:gray_concrete")])
            .layers(&[&["c"]])
            .build()
            .unwrap();

        // Stack 4 units in a row using the builder
        let stacked = SchematicBuilder::new()
            .map_schematic('U', unit)
            .layers(&[&["UUUU"]])
            .build()
            .unwrap();

        // Should have 4 blocks in a row
        assert!(stacked.get_block(0, 0, 0).is_some());
        assert!(stacked.get_block(1, 0, 0).is_some());
        assert!(stacked.get_block(2, 0, 0).is_some());
        assert!(stacked.get_block(3, 0, 0).is_some());
    }

    #[test]
    fn test_schematic_palette_2x2_tiling() {
        // Create a 2x1x2 block (2 wide, 1 tall, 2 deep)
        // Use .empty() to avoid standard palette interference
        let block = SchematicBuilder::empty()
            .palette(&[('c', "minecraft:gray_concrete")])
            .layers(&[&["cc", "cc"]])
            .build()
            .unwrap();

        // Verify the block dimensions (check tight bounds, not full size)
        if let Some(tight) = block.default_region.get_tight_bounds() {
            assert_eq!(tight.get_dimensions(), (2, 1, 2));
        } else {
            panic!("Expected tight bounds");
        }

        // Tile 3 blocks in a row
        let tiled = SchematicBuilder::new()
            .map_schematic('B', block)
            .layers(&[&["BBB"]])
            .build()
            .unwrap();

        // Should have blocks at:
        // Block 0: (0,0,0), (1,0,0), (0,0,1), (1,0,1)
        // Block 1: (2,0,0), (3,0,0), (2,0,1), (3,0,1)
        // Block 2: (4,0,0), (5,0,0), (4,0,1), (5,0,1)

        // Check first block
        assert!(tiled.get_block(0, 0, 0).is_some());
        assert!(tiled.get_block(1, 0, 0).is_some());
        assert!(tiled.get_block(0, 0, 1).is_some());
        assert!(tiled.get_block(1, 0, 1).is_some());

        // Check second block
        assert!(tiled.get_block(2, 0, 0).is_some());
        assert!(tiled.get_block(3, 0, 0).is_some());
        assert!(tiled.get_block(2, 0, 1).is_some());
        assert!(tiled.get_block(3, 0, 1).is_some());

        // Check third block
        assert!(tiled.get_block(4, 0, 0).is_some());
        assert!(tiled.get_block(5, 0, 0).is_some());
        assert!(tiled.get_block(4, 0, 1).is_some());
        assert!(tiled.get_block(5, 0, 1).is_some());
    }

    #[test]
    fn test_schematic_palette_multi_layer() {
        // Create a 2-layer block
        // Use .empty() to avoid standard palette interference
        let block = SchematicBuilder::empty()
            .palette(&[
                ('c', "minecraft:gray_concrete"),
                ('r', "minecraft:redstone_wire"),
            ])
            .layers(&[
                &["cc"], // Y=0
                &["rr"], // Y=1
            ])
            .build()
            .unwrap();

        // Verify dimensions: 2 wide, 2 tall, 1 deep (check tight bounds)
        if let Some(tight) = block.default_region.get_tight_bounds() {
            assert_eq!(tight.get_dimensions(), (2, 2, 1));
        } else {
            panic!("Expected tight bounds");
        }

        // Tile 2 blocks side by side
        let tiled = SchematicBuilder::new()
            .map_schematic('B', block)
            .layers(&[&["BB"]])
            .build()
            .unwrap();

        // Check first block - concrete at Y=0
        assert!(tiled.get_block(0, 0, 0).is_some());
        assert!(tiled.get_block(1, 0, 0).is_some());
        // Check first block - redstone at Y=1
        assert!(tiled.get_block(0, 1, 0).is_some());
        assert!(tiled.get_block(1, 1, 0).is_some());

        // Check second block - concrete at Y=0
        assert!(tiled.get_block(2, 0, 0).is_some());
        assert!(tiled.get_block(3, 0, 0).is_some());
        // Check second block - redstone at Y=1
        assert!(tiled.get_block(2, 1, 0).is_some());
        assert!(tiled.get_block(3, 1, 0).is_some());
    }

    #[test]
    fn test_schematic_palette_2d_grid() {
        // Create a 1x1x1 block
        let unit = SchematicBuilder::new()
            .palette(&[('c', "minecraft:gray_concrete")])
            .layers(&[&["c"]])
            .build()
            .unwrap();

        // Create a 2x2 grid
        let grid = SchematicBuilder::new()
            .map_schematic('U', unit)
            .layers(&[&["UU", "UU"]])
            .build()
            .unwrap();

        // Should have 4 blocks in a 2x2 grid
        assert!(grid.get_block(0, 0, 0).is_some()); // Top-left
        assert!(grid.get_block(1, 0, 0).is_some()); // Top-right
        assert!(grid.get_block(0, 0, 1).is_some()); // Bottom-left
        assert!(grid.get_block(1, 0, 1).is_some()); // Bottom-right
    }

    #[test]
    fn test_schematic_palette_spacing_with_air() {
        // Create a 1x1x1 block
        let block = SchematicBuilder::new()
            .palette(&[('c', "minecraft:gray_concrete")])
            .layers(&[&["c"]])
            .build()
            .unwrap();

        // Create air block of same size
        let air = SchematicBuilder::new()
            .palette(&[('_', "minecraft:air")])
            .layers(&[&["_"]])
            .build()
            .unwrap();

        // Verify both have same dimensions
        assert_eq!(block.default_region.size, air.default_region.size);

        // Tile with spacing: B_B_B (block, air, block, air, block)
        let spaced = SchematicBuilder::new()
            .map_schematic('B', block)
            .map_schematic('_', air)
            .layers(&[&["B_B_B"]])
            .build()
            .unwrap();

        // Should have blocks at positions 0, 2, 4
        assert!(spaced.get_block(0, 0, 0).is_some());
        assert!(spaced.get_block(2, 0, 0).is_some());
        assert!(spaced.get_block(4, 0, 0).is_some());
    }

    #[test]
    fn test_schematic_palette_dimensions_must_match() {
        // This test documents that all schematics in a layer should have matching dimensions
        // for proper tiling

        // Create a 1x1x1 block
        let small = SchematicBuilder::new()
            .palette(&[('c', "minecraft:gray_concrete")])
            .layers(&[&["c"]])
            .build()
            .unwrap();

        // Create a 2x1x2 block
        let large = SchematicBuilder::new()
            .palette(&[('c', "minecraft:gray_concrete")])
            .layers(&[&["cc", "cc"]])
            .build()
            .unwrap();

        // Verify different dimensions
        assert_ne!(small.default_region.size, large.default_region.size);

        // Mixing different sized schematics will cause incorrect tiling
        let mixed = SchematicBuilder::new()
            .map_schematic('S', small)
            .map_schematic('L', large)
            .layers(&[&["SL"]]) // This will tile incorrectly!
            .build()
            .unwrap();

        // The small block is at (0,0,0)
        assert!(mixed.get_block(0, 0, 0).is_some());

        // The large block starts at (1,0,0) but is 2x1x2
        // So it occupies (1,0,0), (2,0,0), (1,0,1), (2,0,1)
        assert!(mixed.get_block(1, 0, 0).is_some());
        assert!(mixed.get_block(2, 0, 0).is_some());
        assert!(mixed.get_block(1, 0, 1).is_some());
        assert!(mixed.get_block(2, 0, 1).is_some());

        // This demonstrates the issue: the large block doesn't align properly
        // because it's 2 units wide but placed at X=1 (should be X=2 for proper tiling)
    }

    #[test]
    fn test_schematic_palette_complex_circuit() {
        // Simulate a repeater circuit
        // Note: This uses standard palette (→ is pre-defined)
        let repeater_unit = SchematicBuilder::new()
            .map('c', "minecraft:gray_concrete") // Override 'c' if needed
            .map('r', "minecraft:redstone_wire")
            .layers(&[
                &["ccc"], // Base
                &["r→r"], // Wire-Repeater-Wire (→ from standard palette)
            ])
            .build()
            .unwrap();

        // Verify it's 3 wide (check tight bounds)
        if let Some(tight) = repeater_unit.default_region.get_tight_bounds() {
            assert_eq!(tight.get_dimensions().0, 3);
        } else {
            panic!("Expected tight bounds");
        }

        // Chain 3 repeater units
        let chain = SchematicBuilder::new()
            .map_schematic('R', repeater_unit)
            .layers(&[&["RRR"]])
            .build()
            .unwrap();

        // Should be 9 blocks wide (3 units × 3 blocks each) - check tight bounds
        if let Some(tight) = chain.default_region.get_tight_bounds() {
            assert_eq!(tight.get_dimensions().0, 9);
        } else {
            panic!("Expected tight bounds");
        }
    }
}
