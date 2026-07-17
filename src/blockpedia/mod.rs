//! Vendored blockpedia (formerly the standalone `blockpedia` crate, v0.2.0).
//!
//! Minecraft block data (Java 26.2 + Bedrock via Geyser mappings) with
//! texture-derived colors, palette/gradient generation, and block-state
//! transforms. The block tables are generated at build time by `build.rs`
//! from the gzipped JSON snapshots in `data/blockpedia/`; refresh them with
//! the `mc-data-refresh` tools (see README).
//!
//! Dropped relative to the standalone crate: the ratatui TUI, the
//! wasm-bindgen glue (nucleation's raw-wasm build has no wasm-bindgen), and
//! the network/build-data code paths (now `tools/mc-data/`).

use std::collections::HashMap;

// Core data structures
#[derive(Debug, Clone)]
pub struct BlockFacts {
    pub id: &'static str,
    pub properties: &'static [(&'static str, &'static [&'static str])],
    pub default_state: &'static [(&'static str, &'static str)],
    pub transparent: bool,
    pub extras: Extras,
}

#[derive(Debug, Clone, Default)]
pub struct Extras {
    // Future extension point for fetcher data
    pub mock_data: Option<i32>,
    pub color: Option<ColorData>,
    pub bedrock: Option<BedrockData>,
}

#[derive(Debug, Clone, Copy)]
pub struct BedrockData {
    pub id: &'static str,
    pub properties: &'static [(&'static str, &'static [&'static str])],
    pub default_state: &'static [(&'static str, &'static str)],
}

#[derive(Debug, Clone, Copy)]
pub struct ColorData {
    pub rgb: [u8; 3],
    pub oklab: [f32; 3],
}

impl ColorData {
    /// Convert to ExtendedColorData for palette operations
    pub fn to_extended(&self) -> color::ExtendedColorData {
        color::ExtendedColorData::from_rgb(self.rgb[0], self.rgb[1], self.rgb[2])
    }
}

impl From<color::ExtendedColorData> for ColorData {
    fn from(extended: color::ExtendedColorData) -> Self {
        ColorData {
            rgb: extended.rgb,
            oklab: extended.oklab,
        }
    }
}

impl Extras {
    pub const fn new() -> Self {
        Extras {
            mock_data: None,
            color: None,
            bedrock: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockState {
    block_id: String,
    properties: HashMap<String, String>,
}

impl BlockFacts {
    pub fn id(&self) -> &str {
        self.id
    }

    pub fn properties(&self) -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();
        for (key, values) in self.properties {
            map.insert(
                key.to_string(),
                values.iter().map(|s| s.to_string()).collect(),
            );
        }
        map
    }

    pub fn has_property(&self, property: &str) -> bool {
        self.properties.iter().any(|(key, _)| *key == property)
    }

    pub fn get_property_values(&self, property: &str) -> Option<Vec<String>> {
        self.properties
            .iter()
            .find(|(key, _)| *key == property)
            .map(|(_, values)| values.iter().map(|s| s.to_string()).collect())
    }

    pub fn get_property(&self, property: &str) -> Option<&str> {
        self.default_state
            .iter()
            .find(|(key, _)| *key == property)
            .map(|(_, value)| *value)
    }
}

impl BlockState {
    pub fn id(&self) -> &str {
        &self.block_id
    }

    pub fn get_property(&self, property: &str) -> Option<&str> {
        self.properties.get(property).map(|s| s.as_str())
    }

    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    pub fn new(block_id: &str) -> Result<Self> {
        // Validate block ID format first
        errors::validation::validate_block_id(block_id)?;

        // Validate block ID exists in our data
        if !BLOCKS.contains_key(block_id) {
            return Err(BlockpediaError::block_not_found(block_id));
        }

        Ok(BlockState {
            block_id: block_id.to_string(),
            properties: HashMap::new(),
        })
    }

    pub fn with(mut self, property: &str, value: &str) -> Result<Self> {
        // Validate property name format
        errors::validation::validate_property_name(property)?;

        // Validate property value format
        errors::validation::validate_property_value(value)?;

        // Get the block data to validate property and value
        let block_facts = BLOCKS
            .get(&self.block_id)
            .ok_or_else(|| BlockpediaError::block_not_found(&self.block_id))?;

        // Check if the property exists for this block
        if !block_facts.has_property(property) {
            return Err(BlockpediaError::property_not_found(
                &self.block_id,
                property,
            ));
        }

        // Check if the value is valid for this property
        let valid_values = block_facts.get_property_values(property).ok_or_else(|| {
            BlockpediaError::Property(errors::PropertyError::NoValues(property.to_string()))
        })?;

        if !valid_values.contains(&value.to_string()) {
            return Err(BlockpediaError::invalid_property_value(
                &self.block_id,
                property,
                value,
                valid_values,
            ));
        }

        self.properties
            .insert(property.to_string(), value.to_string());
        Ok(self)
    }

    /// Create a BlockState from the default state of a block
    pub fn from_default(block_facts: &BlockFacts) -> Result<Self> {
        let mut state = BlockState {
            block_id: block_facts.id().to_string(),
            properties: HashMap::new(),
        };

        // Set all default properties
        for (property, value) in block_facts.default_state {
            state
                .properties
                .insert(property.to_string(), value.to_string());
        }

        Ok(state)
    }

    /// Parse a blockstate string without validation (for Bedrock blockstates)
    fn parse_unvalidated(blockstate_str: &str) -> Result<Self> {
        if let Some(bracket_pos) = blockstate_str.find('[') {
            let block_id = &blockstate_str[..bracket_pos];
            let properties_str = &blockstate_str[bracket_pos + 1..];

            if !properties_str.ends_with(']') {
                return Err(BlockpediaError::parse_failed(
                    blockstate_str,
                    "missing closing bracket",
                ));
            }

            let properties_str = &properties_str[..properties_str.len() - 1];
            let mut properties = HashMap::new();

            if !properties_str.is_empty() {
                for prop_pair in properties_str.split(',') {
                    let parts: Vec<&str> = prop_pair.split('=').collect();
                    if parts.len() != 2 {
                        return Err(BlockpediaError::parse_failed(
                            blockstate_str,
                            &format!("invalid property format: {}", prop_pair),
                        ));
                    }
                    properties.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
                }
            }

            Ok(BlockState {
                block_id: block_id.to_string(),
                properties,
            })
        } else {
            Ok(BlockState {
                block_id: blockstate_str.to_string(),
                properties: HashMap::new(),
            })
        }
    }

    /// Parse a blockstate string like "minecraft:repeater[delay=3,facing=north]"
    pub fn parse(blockstate_str: &str) -> Result<Self> {
        if let Some(bracket_pos) = blockstate_str.find('[') {
            // Block with properties
            let block_id = &blockstate_str[..bracket_pos];
            let properties_str = &blockstate_str[bracket_pos + 1..];

            if !properties_str.ends_with(']') {
                return Err(BlockpediaError::parse_failed(
                    blockstate_str,
                    "missing closing bracket",
                ));
            }

            let properties_str = &properties_str[..properties_str.len() - 1];
            let mut state = BlockState::new(block_id)?;

            if !properties_str.is_empty() {
                for prop_pair in properties_str.split(',') {
                    let parts: Vec<&str> = prop_pair.split('=').collect();
                    if parts.len() != 2 {
                        return Err(BlockpediaError::parse_failed(
                            blockstate_str,
                            &format!("invalid property format: {}", prop_pair),
                        ));
                    }
                    state = state.with(parts[0].trim(), parts[1].trim())?;
                }
            }

            Ok(state)
        } else {
            // Simple block without properties
            BlockState::new(blockstate_str)
        }
    }

    /// Convert this Java BlockState to a Bedrock BlockState using dynamic mappings
    pub fn to_bedrock(&self) -> Result<BlockState> {
        // Get the block facts to fill in default properties
        let facts = BLOCKS
            .get(&self.block_id)
            .ok_or_else(|| BlockpediaError::block_not_found(&self.block_id))?;

        // Build a complete blockstate with all properties (including defaults)
        let mut complete_properties = HashMap::new();

        // First, add all default properties from default_state
        for (name, value) in facts.default_state {
            complete_properties.insert(name.to_string(), value.to_string());
        }

        // For any properties that don't have defaults, use the first allowed value
        for (name, values) in facts.properties {
            if !complete_properties.contains_key(*name) && !values.is_empty() {
                complete_properties.insert(name.to_string(), values[0].to_string());
            }
        }

        // Then, override with any explicitly set properties
        for (name, value) in &self.properties {
            complete_properties.insert(name.clone(), value.clone());
        }

        // Build the Java blockstate string with all properties
        let mut props = Vec::new();
        for (key, value) in &complete_properties {
            props.push(format!("{}={}", key, value));
        }
        props.sort(); // Normalize property order
        let java_blockstate = if props.is_empty() {
            format!("{}[]", self.block_id)
        } else {
            format!("{}[{}]", self.block_id, props.join(","))
        };

        // Look up the Bedrock blockstate string in the mapping
        if let Some(bedrock_blockstate) =
            bedrock_mapping::BedrockBlockStateMapper::java_to_bedrock(&java_blockstate)
        {
            // Parse the Bedrock blockstate string (without validation since it's Bedrock format)
            return Self::parse_unvalidated(bedrock_blockstate);
        }

        // If no direct mapping found, try fallback procedural mapping
        // This is where Section 1.A "Procedural Property Mapping" logic goes
        // For now, try to find a mapping for the default state if the exact state fails
        let default_props = facts.default_state;
        let mut def_props_vec = Vec::new();
        for (k, v) in default_props {
            def_props_vec.push(format!("{}={}", k, v));
        }
        def_props_vec.sort();
        let default_state_str = format!("{}[{}]", self.block_id, def_props_vec.join(","));
        
        if let Some(bedrock_default) = 
            bedrock_mapping::BedrockBlockStateMapper::java_to_bedrock(&default_state_str)
        {
             // We found the default state mapping. Now try to apply the differences.
             // This is a naive heuristic but better than nothing.
             // Parse the bedrock default state
             let mut bedrock_state = Self::parse_unvalidated(bedrock_default)?;
             
             // Apply common property remappings
             for (key, value) in &self.properties {
                 match key.as_str() {
                     "facing" => {
                         // Map facing to minecraft:cardinal_direction or direction
                         // This requires knowing the specific bedrock property name, which varies.
                         // But we can guess or use a look-up if we had one.
                         // For many blocks it is minecraft:cardinal_direction
                         bedrock_state.properties.insert("minecraft:cardinal_direction".to_string(), value.clone());
                         // Sometimes it's just "direction"
                         bedrock_state.properties.insert("direction".to_string(), match value.as_str() {
                             "down" => "0", "up" => "1", "north" => "2", "south" => "3", "west" => "4", "east" => "5",
                             _ => "0"
                         }.to_string());
                     },
                     "powered" => {
                         // Map powered=true/false to some bit property if we knew it.
                     }
                     _ => {}
                 }
             }
             return Ok(bedrock_state);
        }

        Err(BlockpediaError::custom(format!(
            "No Bedrock mapping found for Java blockstate: {}",
            java_blockstate
        )))
    }

    /// Create a Java BlockState from a Bedrock BlockState using dynamic mappings
    pub fn from_bedrock(bedrock_id: &str, properties: HashMap<String, String>) -> Result<Self> {
        // Build the Bedrock blockstate string
        let mut props = Vec::new();
        for (key, value) in &properties {
            props.push(format!("{}={}", key, value));
        }
        props.sort(); // Normalize property order
        let bedrock_blockstate = if props.is_empty() {
            format!("{}[]", bedrock_id)
        } else {
            format!("{}[{}]", bedrock_id, props.join(","))
        };

        // Look up the Java blockstate string in the mapping
        if let Some(java_blockstate) =
            bedrock_mapping::BedrockBlockStateMapper::bedrock_to_java(&bedrock_blockstate)
        {
            // Parse the Java blockstate string (with validation since it's Java format)
            return BlockState::parse(java_blockstate);
        }

        // Fallback: Try to map based on rules if exact match fails
        // Section 1.A: Procedural Property Mapping logic
        
        // 1. Identify the likely Java block ID
        // Bedrock "minecraft:stone" -> Java "minecraft:stone"
        // But Bedrock "minecraft:wool" [color=14] -> Java "minecraft:red_wool"
        // We can try to use a "base" mapping if available, or just guess the ID.
        
        // Try stripping the namespace and seeing if it matches a Java block
        let java_id = if bedrock_id.starts_with("minecraft:") {
            bedrock_id.to_string()
        } else {
            format!("minecraft:{}", bedrock_id)
        };
        
        // Check if this simple ID exists in Java blocks
        if BLOCKS.contains_key(java_id.as_str()) {
            let mut java_state = BlockState::new(&java_id)?;
            
            // Try to map properties
            for (key, value) in properties {
                match key.as_str() {
                    "minecraft:cardinal_direction" | "direction" => {
                        // Map to "facing"
                        // Value mapping: 0,1,2,3... -> down, up, north, south, west, east?
                        // This depends heavily on the block type.
                        // Standard 6-way: 0=down, 1=up, 2=north, 3=south, 4=west, 5=east
                        // Standard 4-way (horizontal): 2=north, 3=south, 4=west, 5=east
                        let facing = match value.as_str() {
                            "0" => "down", "1" => "up", "2" => "north", "3" => "south", "4" => "west", "5" => "east",
                            _ => "north" // default
                        };
                        // Only set if the Java block has this property
                        if BLOCKS.get(&java_id).map(|b| b.has_property("facing")).unwrap_or(false) {
                             // Check valid values
                             if let Some(valid) = BLOCKS.get(&java_id).and_then(|b| b.get_property_values("facing")) {
                                 if valid.contains(&facing.to_string()) {
                                     java_state = java_state.with("facing", facing)?;
                                 }
                             }
                        }
                    },
                    "output_lit_bit" => {
                        if value == "1" {
                            if BLOCKS.get(&java_id).map(|b| b.has_property("powered")).unwrap_or(false) {
                                java_state = java_state.with("powered", "true")?;
                            }
                        }
                    },
                    // Add more procedural rules here
                    _ => {}
                }
            }
            return Ok(java_state);
        }

        Err(BlockpediaError::custom(format!(
            "No Java mapping found for Bedrock blockstate: {}",
            bedrock_blockstate
        )))
    }
}

impl std::fmt::Display for BlockState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.properties.is_empty() {
            write!(f, "{}", self.block_id)
        } else {
            let mut props = Vec::new();
            for (key, value) in &self.properties {
                props.push(format!("{}={}", key, value));
            }
            props.sort();
            write!(f, "{}[{}]", self.block_id, props.join(","))
        }
    }
}

// Bedrock mapping module
pub mod bedrock_mapping;

// Include the generated block table
include!(concat!(env!("OUT_DIR"), "/block_table.rs"));

// Query utilities module
pub mod queries;
pub use queries::*;

// Fetcher framework module
pub mod fetchers;
pub use fetchers::*;

// Error handling module
pub mod errors;
pub use errors::{BlockpediaError, Result};

// Data sources module for multi-source support
pub mod data_sources;
pub use data_sources::*;

// Color processing module
pub mod color;
pub use color::ExtendedColorData;

// Query builder module for chained filtering
pub mod query_builder;
pub use query_builder::{
    AllBlocks, BlockQuery, ColorSamplingMethod, ColorSpace, EasingFunction, GradientConfig,
};

// Block transformation module for rotation and variants
pub mod transforms;
pub use transforms::{BlockShape, BlockTransforms, Direction, Rotation};

/// Get a block by its string ID
pub fn get_block(id: &str) -> Option<&'static BlockFacts> {
    BLOCKS.get(id).copied()
}

/// Get all blocks as an iterator
pub fn all_blocks() -> impl Iterator<Item = &'static BlockFacts> {
    BLOCKS.values().copied()
}

// Block Entity translation
pub mod block_entity;
