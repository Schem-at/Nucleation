use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;

/// Unified block data structure that all data sources convert to
#[derive(Debug, Clone)]
pub struct UnifiedBlockData {
    pub id: String,
    pub properties: HashMap<String, Vec<String>>,
    pub default_state: HashMap<String, String>,
    pub transparent: bool,
    pub extra_properties: HashMap<String, Value>, // For source-specific data
    pub bedrock_id: Option<String>,
    pub bedrock_properties: Option<HashMap<String, Vec<String>>>,
    pub bedrock_default_state: Option<HashMap<String, String>>,
}

/// Trait for different data source adapters
pub trait DataSourceAdapter {
    fn name(&self) -> &'static str;
    fn fetch_url(&self) -> &'static str;
    fn parse_data(&self, json_data: &str) -> Result<Vec<UnifiedBlockData>>;
    fn validate_structure(&self, json: &Value) -> Result<()>;
}

/// PrismarineJS data source adapter
pub struct PrismarineAdapter;

impl DataSourceAdapter for PrismarineAdapter {
    fn name(&self) -> &'static str {
        "PrismarineJS"
    }

    fn fetch_url(&self) -> &'static str {
        "https://raw.githubusercontent.com/PrismarineJS/minecraft-data/master/data/pc/1.20.4/blocks.json"
    }

    fn parse_data(&self, json_data: &str) -> Result<Vec<UnifiedBlockData>> {
        let parsed: Value =
            serde_json::from_str(json_data).context("Failed to parse PrismarineJS JSON")?;

        let blocks_array = parsed
            .as_array()
            .context("PrismarineJS JSON is not an array")?;

        let mut unified_blocks = Vec::new();

        for block in blocks_array {
            let block_obj = block.as_object().context("Block is not an object")?;

            let name = block_obj
                .get("name")
                .and_then(|n| n.as_str())
                .context("Block missing name field")?;

            let id = format!("minecraft:{}", name);

            // Convert states to properties
            let mut properties = HashMap::new();
            if let Some(states) = block_obj.get("states").and_then(|s| s.as_array()) {
                for state in states {
                    if let Some(state_obj) = state.as_object() {
                        if let (Some(prop_name), Some(prop_type), Some(num_values)) = (
                            state_obj.get("name").and_then(|n| n.as_str()),
                            state_obj.get("type").and_then(|t| t.as_str()),
                            state_obj.get("num_values").and_then(|n| n.as_u64()),
                        ) {
                            let values = match prop_type {
                                "bool" => vec!["false".to_string(), "true".to_string()],
                                "int" => (0..num_values).map(|i| i.to_string()).collect(),
                                "enum" => {
                                    // Extract actual enum values if available
                                    if let Some(values_array) =
                                        state_obj.get("values").and_then(|v| v.as_array())
                                    {
                                        values_array
                                            .iter()
                                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                            .collect()
                                    } else {
                                        (0..num_values).map(|i| format!("value_{}", i)).collect()
                                    }
                                }
                                _ => vec!["unknown".to_string()],
                            };
                            properties.insert(prop_name.to_string(), values);
                        }
                    }
                }
            }

            // Extract extra properties from original data
            let mut extra_properties = HashMap::new();
            if let Some(hardness) = block_obj.get("hardness") {
                extra_properties.insert("hardness".to_string(), hardness.clone());
            }
            if let Some(resistance) = block_obj.get("resistance") {
                extra_properties.insert("resistance".to_string(), resistance.clone());
            }

            unified_blocks.push(UnifiedBlockData {
                id,
                properties,
                default_state: HashMap::new(), // PrismarineJS doesn't provide default states
                transparent: false, // Default to false, can be updated from other sources
                extra_properties,
                bedrock_id: None,
                bedrock_properties: None,
                bedrock_default_state: None,
            });
        }

        Ok(unified_blocks)
    }

    fn validate_structure(&self, json: &Value) -> Result<()> {
        let blocks_array = json
            .as_array()
            .context("PrismarineJS JSON is not a valid array")?;

        if blocks_array.is_empty() {
            anyhow::bail!("No blocks found in PrismarineJS data");
        }

        // Validate a few sample blocks
        for (i, block_data) in blocks_array.iter().take(3).enumerate() {
            let block_obj = block_data
                .as_object()
                .with_context(|| format!("Block at index {} is not an object", i))?;

            if !block_obj.contains_key("name") {
                anyhow::bail!("Block at index {} missing 'name' field", i);
            }
        }

        Ok(())
    }
}

/// MCPropertyEncyclopedia data source adapter
pub struct MCPropertyEncyclopediaAdapter;

impl DataSourceAdapter for MCPropertyEncyclopediaAdapter {
    fn name(&self) -> &'static str {
        "MCPropertyEncyclopedia"
    }

    fn fetch_url(&self) -> &'static str {
        "https://raw.githubusercontent.com/JoakimThorsen/MCPropertyEncyclopedia/main/data/block_data.json"
    }

    fn parse_data(&self, json_data: &str) -> Result<Vec<UnifiedBlockData>> {
        let parsed: Value = serde_json::from_str(json_data)
            .context("Failed to parse MCPropertyEncyclopedia JSON")?;

        let key_list = parsed
            .get("key_list")
            .and_then(|k| k.as_array())
            .context("Missing or invalid key_list")?;

        let properties_obj = parsed
            .get("properties")
            .and_then(|p| p.as_object())
            .context("Missing or invalid properties")?;

        let mut unified_blocks = Vec::new();

        for block_name in key_list {
            let block_name_str = block_name.as_str().context("Block name is not a string")?;

            // Convert display name to minecraft ID format
            let id = format!(
                "minecraft:{}",
                block_name_str
                    .to_lowercase()
                    .replace(" ", "_")
                    .replace("(", "")
                    .replace(")", "")
                    .replace("-", "_")
            );

            let mut extra_properties = HashMap::new();

            // Extract all properties for this block
            for (prop_name, prop_data) in properties_obj {
                if let Some(entries) = prop_data.get("entries").and_then(|e| e.as_object()) {
                    if let Some(value) = entries.get(block_name_str) {
                        extra_properties.insert(prop_name.clone(), value.clone());
                    }
                }
            }

            unified_blocks.push(UnifiedBlockData {
                id,
                properties: HashMap::new(), // MCPropertyEncyclopedia doesn't have block states
                default_state: HashMap::new(),
                transparent: false,
                extra_properties,
                bedrock_id: None,
                bedrock_properties: None,
                bedrock_default_state: None,
            });
        }

        Ok(unified_blocks)
    }

    fn validate_structure(&self, json: &Value) -> Result<()> {
        let _key_list = json
            .get("key_list")
            .and_then(|k| k.as_array())
            .context("Missing or invalid key_list")?;

        let _properties = json
            .get("properties")
            .and_then(|p| p.as_object())
            .context("Missing or invalid properties")?;

        Ok(())
    }
}

/// Bedrock Edition data source adapter (from PrismarineJS)
pub struct BedrockDataAdapter;

impl DataSourceAdapter for BedrockDataAdapter {
    fn name(&self) -> &'static str {
        "BedrockData"
    }

    fn fetch_url(&self) -> &'static str {
        "https://raw.githubusercontent.com/PrismarineJS/minecraft-data/master/data/bedrock/1.21.0/blocks.json"
    }

    fn parse_data(&self, json_data: &str) -> Result<Vec<UnifiedBlockData>> {
        let parsed: Value =
            serde_json::from_str(json_data).context("Failed to parse Bedrock data JSON")?;

        let blocks_array = parsed
            .as_array()
            .context("Bedrock data JSON is not an array")?;

        let mut unified_blocks = Vec::new();

        for block in blocks_array {
            let block_obj = block.as_object().context("Block is not an object")?;

            let name = block_obj
                .get("name")
                .and_then(|n| n.as_str())
                .context("Block missing name field")?;

            let id = format!("minecraft:{}", name);

            // Convert states to properties
            let mut properties = HashMap::new();
            if let Some(states) = block_obj.get("states").and_then(|s| s.as_array()) {
                for state in states {
                    if let Some(state_obj) = state.as_object() {
                        if let (Some(prop_name), Some(prop_type), Some(num_values)) = (
                            state_obj.get("name").and_then(|n| n.as_str()),
                            state_obj.get("type").and_then(|t| t.as_str()),
                            state_obj.get("num_values").and_then(|n| n.as_u64()),
                        ) {
                            let values = match prop_type {
                                "bool" => vec!["false".to_string(), "true".to_string()],
                                "int" => (0..num_values).map(|i| i.to_string()).collect(),
                                "enum" => {
                                    if let Some(values_array) =
                                        state_obj.get("values").and_then(|v| v.as_array())
                                    {
                                        values_array
                                            .iter()
                                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                            .collect()
                                    } else {
                                        (0..num_values).map(|i| format!("value_{}", i)).collect()
                                    }
                                }
                                _ => vec!["unknown".to_string()],
                            };
                            properties.insert(prop_name.to_string(), values);
                        }
                    }
                }
            }

            let transparent = block_obj
                .get("transparent")
                .and_then(|t| t.as_bool())
                .unwrap_or(false);

            let mut extra_properties = HashMap::new();
            if let Some(hardness) = block_obj.get("hardness") {
                extra_properties.insert("hardness".to_string(), hardness.clone());
            }

            unified_blocks.push(UnifiedBlockData {
                id: id.clone(),
                properties: properties.clone(),
                default_state: HashMap::new(), // We'll need to figure this out or map it
                transparent,
                extra_properties,
                bedrock_id: Some(id),
                bedrock_properties: Some(properties),
                bedrock_default_state: Some(HashMap::new()),
            });
        }

        Ok(unified_blocks)
    }

    fn validate_structure(&self, json: &Value) -> Result<()> {
        let blocks_array = json
            .as_array()
            .context("Bedrock data JSON is not a valid array")?;

        if blocks_array.is_empty() {
            anyhow::bail!("No blocks found in Bedrock data");
        }

        Ok(())
    }
}

/// Registry for managing multiple data sources
pub struct DataSourceRegistry {
    sources: Vec<Box<dyn DataSourceAdapter>>,
    primary_source: Option<usize>,
}

impl DataSourceRegistry {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            primary_source: None,
        }
    }

    pub fn register_source(&mut self, source: Box<dyn DataSourceAdapter>) {
        self.sources.push(source);

        // Set first source as primary if none set
        if self.primary_source.is_none() {
            self.primary_source = Some(0);
        }
    }

    pub fn set_primary_source(&mut self, name: &str) -> Result<()> {
        for (i, source) in self.sources.iter().enumerate() {
            if source.name() == name {
                self.primary_source = Some(i);
                return Ok(());
            }
        }
        anyhow::bail!("Data source '{}' not found", name);
    }

    pub fn get_primary_source(&self) -> Result<&dyn DataSourceAdapter> {
        let index = self.primary_source.context("No primary data source set")?;
        Ok(self.sources[index].as_ref())
    }

    pub fn list_sources(&self) -> Vec<&str> {
        self.sources.iter().map(|s| s.name()).collect()
    }

    #[allow(clippy::ptr_arg)] // Vec is intentional for this API
    pub fn merge_data_sources(&self, _unified_blocks: &mut Vec<UnifiedBlockData>) -> Result<()> {
        // Logic to merge data from multiple sources
        // For now, just use primary source data
        // In the future, we can implement intelligent merging
        Ok(())
    }
}

impl Default for DataSourceRegistry {
    fn default() -> Self {
        let mut registry = Self::new();

        // Register default sources
        registry.register_source(Box::new(PrismarineAdapter));
        registry.register_source(Box::new(MCPropertyEncyclopediaAdapter));
        registry.register_source(Box::new(BedrockDataAdapter));

        registry
    }
}
