//! PHP bindings for Nucleation
//! Based on the actual UniversalSchematic API from wasm.rs

#![cfg(feature = "php")]
#![cfg_attr(windows, feature(abi_vectorcall))]

use crate::{
    formats::{litematic, schematic},
    print_utils::{format_json_schematic, format_schematic},
    BlockState, UniversalSchematic,
};
use ext_php_rs::prelude::*;
use std::collections::HashMap;

/// Simple test function to verify the extension works
#[php_function]
pub fn nucleation_hello() -> String {
    "Hello from Nucleation PHP Extension!".to_string()
}

/// Get version information
#[php_function]
pub fn nucleation_version() -> HashMap<String, String> {
    let mut info = HashMap::new();
    info.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    info.insert(
        "description".to_string(),
        env!("CARGO_PKG_DESCRIPTION").to_string(),
    );
    info.insert("authors".to_string(), env!("CARGO_PKG_AUTHORS").to_string());
    info.insert("features".to_string(), "php".to_string());
    info
}

/// Detect schematic format from binary data
#[php_function]
pub fn nucleation_detect_format(data: String) -> String {
    let bytes = data.as_bytes();

    if litematic::is_litematic(bytes) {
        "litematic".to_string()
    } else if schematic::is_schematic(bytes) {
        "schematic".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Convert between schematic formats
#[php_function]
pub fn nucleation_convert_format(input_data: String, output_format: String) -> PhpResult<String> {
    let bytes = input_data.as_bytes();

    // Load the schematic based on input format
    let schematic = if litematic::is_litematic(bytes) {
        litematic::from_litematic(bytes)
            .map_err(|e| PhpException::default(format!("Failed to load litematic: {}", e)))?
    } else if schematic::is_schematic(bytes) {
        schematic::from_schematic(bytes)
            .map_err(|e| PhpException::default(format!("Failed to load schematic: {}", e)))?
    } else {
        return Err(PhpException::default("Unknown input format".to_string()));
    };

    // Convert to target format
    let output_bytes = match output_format.to_lowercase().as_str() {
        "litematic" => litematic::to_litematic(&schematic)
            .map_err(|e| PhpException::default(format!("Failed to convert to litematic: {}", e)))?,
        "schematic" => schematic::to_schematic(&schematic)
            .map_err(|e| PhpException::default(format!("Failed to convert to schematic: {}", e)))?,
        _ => {
            return Err(PhpException::default(
                "Unsupported output format".to_string(),
            ))
        }
    };

    // Convert bytes to string (this is not ideal but works for now)
    Ok(String::from_utf8_lossy(&output_bytes).to_string())
}

/// PHP class representing a Universal Schematic
#[php_class(name = "Nucleation\\Schematic")]
pub struct NucleationSchematic {
    inner: UniversalSchematic,
}

#[php_impl]
impl NucleationSchematic {
    /// Constructor
    #[php_method]
    pub fn __construct(name: Option<String>) -> NucleationSchematic {
        let schematic_name = name.unwrap_or_else(|| "Default".to_string());
        let inner = UniversalSchematic::new(schematic_name);
        NucleationSchematic { inner }
    }

    /// Load from binary data (auto-detect format)
    #[php_method]
    pub fn load_from_data(&mut self, data: String) -> PhpResult<bool> {
        let bytes = data.as_bytes();

        if litematic::is_litematic(bytes) {
            match litematic::from_litematic(bytes) {
                Ok(schematic) => {
                    self.inner = schematic;
                    Ok(true)
                }
                Err(e) => Err(PhpException::default(format!(
                    "Failed to load litematic: {}",
                    e
                ))),
            }
        } else if schematic::is_schematic(bytes) {
            match schematic::from_schematic(bytes) {
                Ok(schematic) => {
                    self.inner = schematic;
                    Ok(true)
                }
                Err(e) => Err(PhpException::default(format!(
                    "Failed to load schematic: {}",
                    e
                ))),
            }
        } else {
            Err(PhpException::default(
                "Unknown or unsupported format".to_string(),
            ))
        }
    }

    /// Load from litematic data
    #[php_method]
    pub fn from_litematic(&mut self, data: String) -> PhpResult<bool> {
        let bytes = data.as_bytes();
        match litematic::from_litematic(bytes) {
            Ok(schematic) => {
                self.inner = schematic;
                Ok(true)
            }
            Err(e) => Err(PhpException::default(format!(
                "Failed to load litematic: {}",
                e
            ))),
        }
    }

    /// Load from schematic data
    #[php_method]
    pub fn from_schematic(&mut self, data: String) -> PhpResult<bool> {
        let bytes = data.as_bytes();
        match schematic::from_schematic(bytes) {
            Ok(schematic) => {
                self.inner = schematic;
                Ok(true)
            }
            Err(e) => Err(PhpException::default(format!(
                "Failed to load schematic: {}",
                e
            ))),
        }
    }

    /// Export to litematic format
    #[php_method]
    pub fn to_litematic(&self) -> PhpResult<String> {
        match litematic::to_litematic(&self.inner) {
            Ok(data) => Ok(String::from_utf8_lossy(&data).to_string()),
            Err(e) => Err(PhpException::default(format!(
                "Failed to export to litematic: {}",
                e
            ))),
        }
    }

    /// Export to schematic format
    #[php_method]
    pub fn to_schematic(&self) -> PhpResult<String> {
        match schematic::to_schematic(&self.inner) {
            Ok(data) => Ok(String::from_utf8_lossy(&data).to_string()),
            Err(e) => Err(PhpException::default(format!(
                "Failed to export to schematic: {}",
                e
            ))),
        }
    }

    /// Set a block at coordinates
    #[php_method]
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block_name: String) -> PhpResult<()> {
        let block_state = BlockState::new(block_name);
        self.inner.set_block(x, y, z, block_state);
        Ok(())
    }

    /// Set a block from a block string
    #[php_method]
    pub fn set_block_from_string(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        block_string: String,
    ) -> PhpResult<()> {
        self.inner
            .set_block_from_string(x, y, z, &block_string)
            .map_err(|e| {
                PhpException::default(format!("Failed to set block from string: {}", e))
            })?;
        Ok(())
    }

    /// Set a block with properties
    #[php_method]
    pub fn set_block_with_properties(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        block_name: String,
        properties: HashMap<String, String>,
    ) -> PhpResult<()> {
        let mut block_state = BlockState::new(block_name);
        for (key, value) in properties {
            block_state = block_state.with_property(key, value);
        }
        self.inner.set_block(x, y, z, block_state);
        Ok(())
    }

    /// Get block at coordinates
    #[php_method]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<String> {
        self.inner
            .get_block(x, y, z)
            .map(|block_state| block_state.name.clone())
    }

    /// Get block with properties
    #[php_method]
    pub fn get_block_with_properties(
        &self,
        x: i32,
        y: i32,
        z: i32,
    ) -> Option<HashMap<String, String>> {
        if let Some(block_state) = self.inner.get_block(x, y, z) {
            let mut result = HashMap::new();
            result.insert("name".to_string(), block_state.name.clone());
            for (key, value) in &block_state.properties {
                result.insert(key.clone(), value.clone());
            }
            Some(result)
        } else {
            None
        }
    }

    /// Get schematic dimensions
    #[php_method]
    pub fn get_dimensions(&self) -> Vec<i32> {
        let (x, y, z) = self.inner.get_dimensions();
        vec![x, y, z]
    }

    /// Get total block count
    #[php_method]
    pub fn get_block_count(&self) -> i32 {
        self.inner.total_blocks()
    }

    /// Get total volume
    #[php_method]
    pub fn get_volume(&self) -> i32 {
        self.inner.total_volume()
    }

    /// Get region names
    #[php_method]
    pub fn get_region_names(&self) -> Vec<String> {
        self.inner.get_region_names()
    }

    /// Get basic info
    #[php_method]
    pub fn get_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();

        if let Some(name) = &self.inner.metadata.name {
            info.insert("name".to_string(), name.clone());
        }

        if let Some(author) = &self.inner.metadata.author {
            info.insert("author".to_string(), author.clone());
        }

        if let Some(description) = &self.inner.metadata.description {
            info.insert("description".to_string(), description.clone());
        }

        info.insert(
            "regions".to_string(),
            self.inner.other_regions.len().to_string(),
        );
        info.insert(
            "total_blocks".to_string(),
            self.inner.total_blocks().to_string(),
        );
        info.insert(
            "total_volume".to_string(),
            self.inner.total_volume().to_string(),
        );

        info
    }

    /// Set metadata name
    #[php_method]
    pub fn set_metadata_name(&mut self, name: String) -> PhpResult<()> {
        self.inner.metadata.name = Some(name);
        Ok(())
    }

    /// Get metadata name
    #[php_method]
    pub fn get_metadata_name(&self) -> Option<String> {
        self.inner.metadata.name.clone()
    }

    /// Set metadata author
    #[php_method]
    pub fn set_metadata_author(&mut self, author: String) -> PhpResult<()> {
        self.inner.metadata.author = Some(author);
        Ok(())
    }

    /// Get metadata author
    #[php_method]
    pub fn get_metadata_author(&self) -> Option<String> {
        self.inner.metadata.author.clone()
    }

    /// Set metadata description
    #[php_method]
    pub fn set_metadata_description(&mut self, description: String) -> PhpResult<()> {
        self.inner.metadata.description = Some(description);
        Ok(())
    }

    /// Get metadata description
    #[php_method]
    pub fn get_metadata_description(&self) -> Option<String> {
        self.inner.metadata.description.clone()
    }

    /// Format the schematic as a human-readable string
    #[php_method]
    pub fn format(&self) -> String {
        format_schematic(&self.inner)
    }

    /// Format the schematic as JSON
    #[php_method]
    pub fn format_json(&self) -> String {
        format_json_schematic(&self.inner)
    }

    /// Get debug information
    #[php_method]
    pub fn debug_info(&self) -> String {
        format!(
            "Schematic name: {}, Regions: {}",
            self.inner
                .metadata
                .name
                .as_ref()
                .unwrap_or(&"Unnamed".to_string()),
            self.inner.other_regions.len()
        )
    }

    /// Convert to string representation
    #[php_method]
    #[allow(non_snake_case)]
    pub fn __toString(&self) -> String {
        format!(
            "Nucleation Schematic: {} ({} regions)",
            self.inner
                .metadata
                .name
                .as_ref()
                .unwrap_or(&"Unnamed".to_string()),
            self.inner.other_regions.len()
        )
    }

    /// Get all blocks as array
    #[php_method]
    pub fn get_all_blocks(&self) -> Vec<HashMap<String, String>> {
        self.inner
            .iter_blocks()
            .map(|(pos, block)| {
                let mut block_info = HashMap::new();
                block_info.insert("x".to_string(), pos.x.to_string());
                block_info.insert("y".to_string(), pos.y.to_string());
                block_info.insert("z".to_string(), pos.z.to_string());
                block_info.insert("name".to_string(), block.name.clone());

                // Add properties as a JSON string for simplicity
                if !block.properties.is_empty() {
                    let props_json = serde_json::to_string(&block.properties).unwrap_or_default();
                    block_info.insert("properties".to_string(), props_json);
                }

                block_info
            })
            .collect()
    }

    /// Copy a region from another schematic
    #[php_method]
    pub fn copy_region(
        &mut self,
        from_schematic: &NucleationSchematic,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
        target_x: i32,
        target_y: i32,
        target_z: i32,
        excluded_blocks: Option<Vec<String>>,
    ) -> PhpResult<()> {
        let bounds =
            crate::bounding_box::BoundingBox::new((min_x, min_y, min_z), (max_x, max_y, max_z));

        let excluded = if let Some(excluded) = excluded_blocks {
            excluded
                .into_iter()
                .filter_map(|block_string| {
                    UniversalSchematic::parse_block_string(&block_string)
                        .ok()
                        .map(|(block_state, _)| block_state)
                })
                .collect()
        } else {
            Vec::new()
        };

        self.inner
            .copy_region(
                &from_schematic.inner,
                &bounds,
                (target_x, target_y, target_z),
                &excluded,
            )
            .map_err(|e| PhpException::default(format!("Failed to copy region: {}", e)))?;

        Ok(())
    }
}

/// Create a new schematic
#[php_function]
pub fn nucleation_create_schematic(name: String) -> NucleationSchematic {
    let inner = UniversalSchematic::new(name);
    NucleationSchematic { inner }
}

/// Load schematic from file path
#[php_function]
pub fn nucleation_load_from_file(file_path: String) -> PhpResult<NucleationSchematic> {
    let data = std::fs::read(&file_path)
        .map_err(|e| PhpException::default(format!("Failed to read file: {}", e)))?;

    let inner = if litematic::is_litematic(&data) {
        litematic::from_litematic(&data)
            .map_err(|e| PhpException::default(format!("Failed to load litematic: {}", e)))?
    } else if schematic::is_schematic(&data) {
        schematic::from_schematic(&data)
            .map_err(|e| PhpException::default(format!("Failed to load schematic: {}", e)))?
    } else {
        return Err(PhpException::default(
            "Unknown or unsupported format".to_string(),
        ));
    };

    Ok(NucleationSchematic { inner })
}

/// Save schematic to file
#[php_function]
pub fn nucleation_save_to_file(
    schematic: &NucleationSchematic,
    file_path: String,
    format: String,
) -> PhpResult<bool> {
    let data = match format.to_lowercase().as_str() {
        "litematic" => litematic::to_litematic(&schematic.inner)
            .map_err(|e| PhpException::default(format!("Failed to export to litematic: {}", e)))?,
        "schematic" => schematic::to_schematic(&schematic.inner)
            .map_err(|e| PhpException::default(format!("Failed to export to schematic: {}", e)))?,
        _ => return Err(PhpException::default("Unsupported format".to_string())),
    };

    std::fs::write(&file_path, data)
        .map_err(|e| PhpException::default(format!("Failed to write file: {}", e)))?;

    Ok(true)
}
#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
