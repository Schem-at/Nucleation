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

fn load_universal(bytes: &[u8]) -> PhpResult<UniversalSchematic> {
    if litematic::is_litematic(bytes) {
        litematic::from_litematic(bytes)
            .map_err(|e| PhpException::default(format!("Failed to load litematic: {}", e)))
    } else if schematic::is_schematic(bytes) {
        schematic::from_schematic(bytes)
            .map_err(|e| PhpException::default(format!("Failed to load schematic: {}", e)))
    } else {
        Err(PhpException::default(
            "Unknown or unsupported format".to_string(),
        ))
    }
}

fn export_universal(schematic: &UniversalSchematic, format: &str) -> PhpResult<Vec<u8>> {
    match format.to_lowercase().as_str() {
        "litematic" => litematic::to_litematic(schematic)
            .map_err(|e| PhpException::default(format!("Failed to export to litematic: {}", e))),
        "schematic" => schematic::to_schematic(schematic)
            .map_err(|e| PhpException::default(format!("Failed to export to schematic: {}", e))),
        _ => Err(PhpException::default(
            "Unsupported output format".to_string(),
        )),
    }
}

/// Simple test function to verify the extension works
#[php_function]
pub fn nucleation_hello() -> String {
    "Hello from Nucleation PHP Extension!".to_string()
}

/// Get version information
#[php_function]
pub fn nucleation_version() -> HashMap<String, String> {
    let mut info = HashMap::with_capacity(4);
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
    let schematic = load_universal(input_data.as_bytes())?;
    let output_bytes = export_universal(&schematic, &output_format)?;
    Ok(String::from_utf8_lossy(&output_bytes).to_string())
}

/// PHP class representing a Universal Schematic
#[php_class]
#[php(name = "Nucleation\\Schematic")]
pub struct NucleationSchematic {
    inner: UniversalSchematic,
}

#[php_impl]
#[php(change_method_case = "none")]
impl NucleationSchematic {
    /// Constructor
    pub fn __construct(name: Option<String>) -> NucleationSchematic {
        let schematic_name = name.unwrap_or_else(|| "Default".to_string());
        let inner = UniversalSchematic::new(schematic_name);
        NucleationSchematic { inner }
    }

    /// Open a schematic from a URI (local path, `file://`, or `s3://bucket/key.schem`).
    ///
    /// The format is auto-detected. Remote backends such as `s3://` require the
    /// corresponding store feature (e.g. `store-s3`) to be enabled at build time;
    /// local paths and `file://` work with the default build.
    pub fn open(uri: String) -> PhpResult<NucleationSchematic> {
        let inner = UniversalSchematic::open(&uri)
            .map_err(|e| PhpException::default(format!("Failed to open '{}': {}", uri, e)))?;
        Ok(NucleationSchematic { inner })
    }

    /// Save this schematic to a URI (local path, `file://`, or `s3://bucket/key.schem`).
    ///
    /// The output format is inferred from the URI extension. Remote backends such
    /// as `s3://` require the corresponding store feature (e.g. `store-s3`) to be
    /// enabled at build time; local paths and `file://` work with the default build.
    pub fn save(&self, uri: String) -> PhpResult<()> {
        self.inner
            .save(&uri, None)
            .map_err(|e| PhpException::default(format!("Failed to save '{}': {}", uri, e)))?;
        Ok(())
    }

    /// Load from binary data (auto-detect format)
    pub fn load_from_data(&mut self, data: String) -> PhpResult<bool> {
        self.inner = load_universal(data.as_bytes())?;
        Ok(true)
    }

    /// Load from litematic data
    pub fn from_litematic(&mut self, data: String) -> PhpResult<bool> {
        self.inner = litematic::from_litematic(data.as_bytes())
            .map_err(|e| PhpException::default(format!("Failed to load litematic: {}", e)))?;
        Ok(true)
    }

    /// Load from schematic data
    pub fn from_schematic(&mut self, data: String) -> PhpResult<bool> {
        self.inner = schematic::from_schematic(data.as_bytes())
            .map_err(|e| PhpException::default(format!("Failed to load schematic: {}", e)))?;
        Ok(true)
    }

    /// Export to litematic format
    pub fn to_litematic(&self) -> PhpResult<String> {
        let data = litematic::to_litematic(&self.inner)
            .map_err(|e| PhpException::default(format!("Failed to export to litematic: {}", e)))?;
        Ok(String::from_utf8_lossy(&data).to_string())
    }

    /// Export to schematic format
    pub fn to_schematic(&self) -> PhpResult<String> {
        let data = schematic::to_schematic(&self.inner)
            .map_err(|e| PhpException::default(format!("Failed to export to schematic: {}", e)))?;
        Ok(String::from_utf8_lossy(&data).to_string())
    }

    // ----- Data-version conversion (datafixers) -----

    /// Convert data between Minecraft data versions. Forward is lossless;
    /// reverse returns a JSON loss report (`[]` when lossless).
    pub fn convert_to_data_version(
        &mut self,
        target_data_version: i32,
        source_data_version: i32,
    ) -> String {
        if target_data_version == source_data_version {
            return "[]".to_string();
        }
        if target_data_version > source_data_version {
            crate::dataconverter::convert_schematic(
                &mut self.inner,
                source_data_version,
                target_data_version,
            );
            "[]".to_string()
        } else {
            crate::dataconverter::convert_schematic_reverse(
                &mut self.inner,
                source_data_version,
                target_data_version,
            )
            .to_json()
        }
    }

    /// Convert to `target_data_version` using the captured source version as
    /// origin. Returns the JSON loss report (`[]` when lossless).
    pub fn convert_to_version(&mut self, target_data_version: i32) -> String {
        self.inner
            .convert_to_data_version(target_data_version)
            .to_json()
    }

    /// The data version this schematic was loaded from, or `null` if none.
    pub fn get_source_data_version(&self) -> Option<i32> {
        self.inner.metadata.source_data_version
    }

    /// Override the source data version for versionless formats.
    pub fn set_source_data_version(&mut self, version: i32) {
        self.inner.metadata.source_data_version = Some(version);
    }

    /// The canonical in-memory data version (the forward-conversion target).
    pub fn canonical_data_version() -> i32 {
        crate::dataconverter::CANONICAL_DATA_VERSION
    }

    /// Serialize a `.litematic` for a specific data version. Returns a map with
    /// `bytes` (the file content) and `loss` (the JSON data-loss report). The
    /// schematic itself is left unchanged.
    pub fn to_litematic_for_version(
        &self,
        target_data_version: i32,
    ) -> PhpResult<HashMap<String, String>> {
        let (bytes, report) =
            litematic::to_litematic_for_data_version(&self.inner, target_data_version).map_err(
                |e| PhpException::default(format!("Failed to export to litematic: {}", e)),
            )?;
        let mut map = HashMap::new();
        map.insert("bytes".to_string(), String::from_utf8_lossy(&bytes).to_string());
        map.insert("loss".to_string(), report.to_json());
        Ok(map)
    }

    // ----- Faithful (SNBT) block-entity / entity access -----

    /// The block entity's NBT as a typed SNBT string, or `null` if none.
    pub fn get_block_entity_snbt(&self, x: i32, y: i32, z: i32) -> Option<String> {
        self.inner
            .get_block_entity(crate::block_position::BlockPosition { x, y, z })
            .map(|be| quartz_nbt::NbtTag::Compound(be.nbt.to_quartz_nbt()).to_snbt())
    }

    /// Set (or replace) a block entity at a position from a typed SNBT string.
    pub fn set_block_entity(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        id: String,
        snbt: String,
    ) -> PhpResult<()> {
        let compound = quartz_nbt::snbt::parse(&snbt)
            .map_err(|e| PhpException::default(format!("Invalid SNBT: {}", e)))?;
        let nbt = crate::nbt::NbtMap::from_quartz_nbt(&compound);
        let mut be = crate::block_entity::BlockEntity::new(id, (x, y, z));
        be.set_nbt(nbt);
        self.inner
            .set_block_entity(crate::block_position::BlockPosition { x, y, z }, be);
        Ok(())
    }

    /// Remove the block entity at a position. Returns true if one was removed.
    pub fn remove_block_entity(&mut self, x: i32, y: i32, z: i32) -> bool {
        self.inner.remove_block_entity((x, y, z)).is_some()
    }

    /// Every block entity as a JSON array of `{id, position, snbt}`.
    pub fn get_all_block_entities_snbt(&self) -> String {
        let items: Vec<serde_json::Value> = self
            .inner
            .get_block_entities_as_list()
            .into_iter()
            .map(|be| {
                let snbt = quartz_nbt::NbtTag::Compound(be.nbt.to_quartz_nbt()).to_snbt();
                serde_json::json!({
                    "id": be.id,
                    "position": [be.position.0, be.position.1, be.position.2],
                    "snbt": snbt,
                })
            })
            .collect();
        serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string())
    }

    /// Every mobile entity as a typed SNBT string (full compound incl. id/Pos).
    pub fn get_entities_snbt(&self) -> Vec<String> {
        self.inner
            .get_entities_as_list()
            .iter()
            .map(|entity| entity.to_nbt().to_snbt())
            .collect()
    }

    /// Add a mobile entity from a full SNBT entity compound (must contain `id`
    /// and `Pos`).
    pub fn add_entity_from_snbt(&mut self, snbt: String) -> PhpResult<bool> {
        let compound = quartz_nbt::snbt::parse(&snbt)
            .map_err(|e| PhpException::default(format!("Invalid SNBT: {}", e)))?;
        let entity = crate::entity::Entity::from_nbt(&compound)
            .map_err(|e| PhpException::default(format!("Invalid entity NBT: {}", e)))?;
        self.inner.add_entity(entity);
        Ok(true)
    }

    /// Set a block at coordinates
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block_name: String) -> PhpResult<()> {
        let block_state = BlockState::new(block_name);
        self.inner.set_block(x, y, z, &block_state);
        Ok(())
    }

    /// Set a block from a block string
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
        self.inner.set_block(x, y, z, &block_state);
        Ok(())
    }

    /// Get block at coordinates
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<String> {
        self.inner
            .get_block(x, y, z)
            .map(|block_state| block_state.name.to_string())
    }

    /// Get block with properties
    pub fn get_block_with_properties(
        &self,
        x: i32,
        y: i32,
        z: i32,
    ) -> Option<HashMap<String, String>> {
        if let Some(block_state) = self.inner.get_block(x, y, z) {
            let mut result = HashMap::with_capacity(1 + block_state.properties.len());
            result.insert("name".to_string(), block_state.name.to_string());
            for (key, value) in &block_state.properties {
                result.insert(key.to_string(), value.to_string());
            }
            Some(result)
        } else {
            None
        }
    }

    /// Get schematic dimensions
    pub fn get_dimensions(&self) -> Vec<i32> {
        let (x, y, z) = self.inner.get_dimensions();
        vec![x, y, z]
    }

    /// Get total block count
    pub fn get_block_count(&self) -> i32 {
        self.inner.total_blocks()
    }

    /// Get total volume
    pub fn get_volume(&self) -> i32 {
        self.inner.total_volume()
    }

    /// Get region names
    pub fn get_region_names(&self) -> Vec<String> {
        self.inner.get_region_names()
    }

    /// Get basic info
    pub fn get_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::with_capacity(6);

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
    pub fn set_metadata_name(&mut self, name: String) -> PhpResult<()> {
        self.inner.metadata.name = Some(name);
        Ok(())
    }

    /// Get metadata name
    pub fn get_metadata_name(&self) -> Option<String> {
        self.inner.metadata.name.clone()
    }

    /// Set metadata author
    pub fn set_metadata_author(&mut self, author: String) -> PhpResult<()> {
        self.inner.metadata.author = Some(author);
        Ok(())
    }

    /// Get metadata author
    pub fn get_metadata_author(&self) -> Option<String> {
        self.inner.metadata.author.clone()
    }

    /// Set metadata description
    pub fn set_metadata_description(&mut self, description: String) -> PhpResult<()> {
        self.inner.metadata.description = Some(description);
        Ok(())
    }

    /// Get metadata description
    pub fn get_metadata_description(&self) -> Option<String> {
        self.inner.metadata.description.clone()
    }

    /// Format the schematic as a human-readable string
    pub fn format(&self) -> String {
        format_schematic(&self.inner)
    }

    /// Format the schematic as JSON
    pub fn format_json(&self) -> String {
        format_json_schematic(&self.inner)
    }

    /// Get debug information
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
    pub fn get_all_blocks(&self) -> Vec<HashMap<String, String>> {
        self.inner
            .iter_blocks()
            .map(|(pos, block)| {
                let mut block_info = HashMap::with_capacity(5);
                block_info.insert("x".to_string(), pos.x.to_string());
                block_info.insert("y".to_string(), pos.y.to_string());
                block_info.insert("z".to_string(), pos.z.to_string());
                block_info.insert("name".to_string(), block.name.to_string());

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

    // ---- Auto-stack: periodicity detection + resize ----

    /// Detect repeating structures (region coverage). Returns a JSON array
    /// string; decode with `json_decode`.
    pub fn detect_structures(&self) -> String {
        crate::autostack::detect_structures_json(&self.inner)
    }

    /// Resize a 1D / diagonal structure along its period vector.
    pub fn autostack_resize_1d(
        &self,
        vx: i64,
        vy: i64,
        vz: i64,
        units: i64,
    ) -> PhpResult<NucleationSchematic> {
        let inner = crate::autostack::resize_1d(
            &self.inner,
            [vx as i32, vy as i32, vz as i32],
            units.max(1) as usize,
        )
        .map_err(|e| PhpException::default(e.to_string()))?;
        Ok(NucleationSchematic { inner })
    }

    /// Resize a 2D structure to `n1`×`n2` cells along the two period vectors.
    pub fn autostack_resize_2d(
        &self,
        v1x: i64,
        v1y: i64,
        v1z: i64,
        v2x: i64,
        v2y: i64,
        v2z: i64,
        n1: i64,
        n2: i64,
    ) -> PhpResult<NucleationSchematic> {
        let inner = crate::autostack::resize_2d(
            &self.inner,
            [v1x as i32, v1y as i32, v1z as i32],
            [v2x as i32, v2y as i32, v2z as i32],
            n1.max(1) as usize,
            n2.max(1) as usize,
        )
        .map_err(|e| PhpException::default(e.to_string()))?;
        Ok(NucleationSchematic { inner })
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
    let inner = load_universal(&data)?;
    Ok(NucleationSchematic { inner })
}

/// Save schematic to file
#[php_function]
pub fn nucleation_save_to_file(
    schematic: &NucleationSchematic,
    file_path: String,
    format: String,
) -> PhpResult<bool> {
    let data = export_universal(&schematic.inner, &format)?;
    std::fs::write(&file_path, data)
        .map_err(|e| PhpException::default(format!("Failed to write file: {}", e)))?;
    Ok(true)
}
#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<NucleationSchematic>()
        .function(wrap_function!(nucleation_hello))
        .function(wrap_function!(nucleation_version))
        .function(wrap_function!(nucleation_detect_format))
        .function(wrap_function!(nucleation_convert_format))
        .function(wrap_function!(nucleation_create_schematic))
        .function(wrap_function!(nucleation_load_from_file))
        .function(wrap_function!(nucleation_save_to_file))
}
