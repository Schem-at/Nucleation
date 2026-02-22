//! Schematic WASM bindings
//!
//! Core schematic operations: loading, saving, block manipulation, iteration.

use smol_str::SmolStr;

use crate::bounding_box::BoundingBox;
use crate::definition_region::DefinitionRegion;
use crate::schematic::SchematicVersion;
use crate::universal_schematic::ChunkLoadingStrategy;
use crate::{
    block_position::BlockPosition,
    formats::{litematic, manager::get_manager, mcstructure, schematic, world},
    print_utils::{
        format_json_schematic as print_json_schematic, format_schematic as print_schematic,
    },
    BlockState, UniversalSchematic,
};
use js_sys::{self, Array, Object, Reflect};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::console;

use super::definition_region::DefinitionRegionWrapper;

#[cfg(feature = "simulation")]
use super::circuit_builder::{CircuitBuilderWrapper, SortStrategyWrapper};
#[cfg(feature = "simulation")]
use super::typed_executor::TypedCircuitExecutorWrapper;
#[cfg(feature = "simulation")]
use crate::simulation::typed_executor::{IoType, SortStrategy};

#[wasm_bindgen]
pub struct LazyChunkIterator {
    // Iterator state - doesn't store all chunks, just iteration parameters
    schematic_wrapper: SchematicWrapper,
    chunk_width: i32,
    chunk_height: i32,
    chunk_length: i32,

    // Current iteration state
    current_chunk_coords: Vec<(i32, i32, i32)>, // Just the coordinates, not the data
    current_index: usize,
}

/// Initialize WASM module with panic hook for better error messages
#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    console::log_1(&"Initializing schematic utilities".into());
}

use crate::building::{BuildingTool, Cuboid, Shape, ShapeEnum, SolidBrush, Sphere};

// Wrapper structs
#[wasm_bindgen]
pub struct SchematicWrapper(pub(crate) UniversalSchematic);

#[wasm_bindgen]
pub struct BlockStateWrapper(pub(crate) BlockState);

// All your existing WASM implementations go here...
#[wasm_bindgen]
impl SchematicWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        SchematicWrapper(UniversalSchematic::new("Default".to_string()))
    }

    #[wasm_bindgen(js_name = fillCuboid)]
    pub fn fill_cuboid(
        &mut self,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
        block_state: &str,
    ) {
        let block = BlockState::new(block_state.to_string());
        let shape = ShapeEnum::Cuboid(Cuboid::new((min_x, min_y, min_z), (max_x, max_y, max_z)));
        let brush = SolidBrush::new(block);

        let mut tool = BuildingTool::new(&mut self.0);
        tool.fill(&shape, &brush);
    }

    #[wasm_bindgen(js_name = fillSphere)]
    pub fn fill_sphere(&mut self, cx: i32, cy: i32, cz: i32, radius: f64, block_state: &str) {
        let block = BlockState::new(block_state.to_string());
        let shape = ShapeEnum::Sphere(Sphere::new((cx, cy, cz), radius));
        let brush = SolidBrush::new(block);

        let mut tool = BuildingTool::new(&mut self.0);
        tool.fill(&shape, &brush);
    }

    pub fn from_data(&mut self, data: &[u8]) -> Result<(), JsValue> {
        let manager = get_manager();
        let manager = manager.lock().unwrap();

        match manager.read(data) {
            Ok(schematic) => {
                self.0 = schematic;
                Ok(())
            }
            Err(e) => Err(JsValue::from_str(&format!(
                "Schematic parsing error: {}",
                e
            ))),
        }
    }

    pub fn get_supported_import_formats() -> Array {
        let manager = get_manager();
        let manager = manager.lock().unwrap();
        let formats = manager.list_importers();
        let js_formats = Array::new();
        for format in formats {
            js_formats.push(&JsValue::from_str(&format));
        }
        js_formats
    }

    pub fn get_supported_export_formats() -> Array {
        let manager = get_manager();
        let manager = manager.lock().unwrap();
        let formats = manager.list_exporters();
        let js_formats = Array::new();
        for format in formats {
            js_formats.push(&JsValue::from_str(&format));
        }
        js_formats
    }

    pub fn get_format_versions(format: &str) -> Array {
        let manager = get_manager();
        let manager = manager.lock().unwrap();
        let versions = manager.get_exporter_versions(format).unwrap_or_default();
        let js_versions = Array::new();
        for version in versions {
            js_versions.push(&JsValue::from_str(&version));
        }
        js_versions
    }

    pub fn get_default_format_version(format: &str) -> Option<String> {
        let manager = get_manager();
        let manager = manager.lock().unwrap();
        manager.get_exporter_default_version(format)
    }

    pub fn save_as(
        &self,
        format: &str,
        version: Option<String>,
        settings: Option<String>,
    ) -> Result<Vec<u8>, JsValue> {
        let manager = get_manager();
        let manager = manager.lock().unwrap();

        manager
            .write_with_settings(format, &self.0, version.as_deref(), settings.as_deref())
            .map_err(|e| JsValue::from_str(&format!("Export error: {}", e)))
    }

    /// Get the settings schema for an export format (returns JSON string or undefined).
    pub fn get_export_settings_schema(format: &str) -> Option<String> {
        let manager = get_manager();
        let manager = manager.lock().unwrap();
        manager.get_export_settings_schema(format)
    }

    /// Get the settings schema for an import format (returns JSON string or undefined).
    pub fn get_import_settings_schema(format: &str) -> Option<String> {
        let manager = get_manager();
        let manager = manager.lock().unwrap();
        manager.get_import_settings_schema(format)
    }

    /// Fast binary snapshot serialization — bypasses format manager for maximum speed.
    #[wasm_bindgen(js_name = toSnapshot)]
    pub fn to_snapshot(&self) -> Result<Vec<u8>, JsValue> {
        crate::formats::snapshot::to_snapshot(&self.0)
            .map_err(|e| JsValue::from_str(&format!("Snapshot export error: {}", e)))
    }

    /// Fast binary snapshot deserialization — bypasses format manager for maximum speed.
    #[wasm_bindgen(js_name = fromSnapshot)]
    pub fn from_snapshot(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.0 = crate::formats::snapshot::from_snapshot(data)
            .map_err(|e| JsValue::from_str(&format!("Snapshot import error: {}", e)))?;
        Ok(())
    }

    pub fn from_litematic(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.0 = litematic::from_litematic(data)
            .map_err(|e| JsValue::from_str(&format!("Litematic parsing error: {}", e)))?;
        Ok(())
    }

    pub fn to_litematic(&self) -> Result<Vec<u8>, JsValue> {
        litematic::to_litematic(&self.0)
            .map_err(|e| JsValue::from_str(&format!("Litematic conversion error: {}", e)))
    }

    pub fn from_schematic(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.0 = schematic::from_schematic(data)
            .map_err(|e| JsValue::from_str(&format!("Schematic parsing error: {}", e)))?;
        Ok(())
    }

    pub fn to_schematic(&self) -> Result<Vec<u8>, JsValue> {
        schematic::to_schematic(&self.0)
            .map_err(|e| JsValue::from_str(&format!("Schematic conversion error: {}", e)))
    }

    pub fn from_mcstructure(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.0 = mcstructure::from_mcstructure(data)
            .map_err(|e| JsValue::from_str(&format!("McStructure parsing error: {}", e)))?;
        Ok(())
    }

    pub fn to_mcstructure(&self) -> Result<Vec<u8>, JsValue> {
        mcstructure::to_mcstructure(&self.0)
            .map_err(|e| JsValue::from_str(&format!("McStructure conversion error: {}", e)))
    }

    pub fn from_mca(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.0 = world::from_mca(data)
            .map_err(|e| JsValue::from_str(&format!("MCA parsing error: {}", e)))?;
        Ok(())
    }

    pub fn from_mca_bounded(
        &mut self,
        data: &[u8],
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> Result<(), JsValue> {
        self.0 = world::from_mca_bounded(data, min_x, min_y, min_z, max_x, max_y, max_z)
            .map_err(|e| JsValue::from_str(&format!("MCA parsing error: {}", e)))?;
        Ok(())
    }

    pub fn from_world_zip(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.0 = world::from_world_zip(data)
            .map_err(|e| JsValue::from_str(&format!("World zip parsing error: {}", e)))?;
        Ok(())
    }

    pub fn from_world_zip_bounded(
        &mut self,
        data: &[u8],
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> Result<(), JsValue> {
        self.0 = world::from_world_zip_bounded(data, min_x, min_y, min_z, max_x, max_y, max_z)
            .map_err(|e| JsValue::from_str(&format!("World zip parsing error: {}", e)))?;
        Ok(())
    }

    pub fn to_world(&self, options_json: Option<String>) -> Result<JsValue, JsValue> {
        let options = match options_json {
            Some(ref json) => Some(
                serde_json::from_str::<world::WorldExportOptions>(json)
                    .map_err(|e| JsValue::from_str(&format!("Invalid options JSON: {}", e)))?,
            ),
            None => None,
        };
        let files = world::to_world(&self.0, options)
            .map_err(|e| JsValue::from_str(&format!("World export error: {}", e)))?;

        let map = js_sys::Map::new();
        for (path, data) in &files {
            let key = JsValue::from_str(path);
            let value = js_sys::Uint8Array::from(data.as_slice());
            map.set(&key, &value);
        }
        Ok(map.into())
    }

    pub fn to_world_zip(&self, options_json: Option<String>) -> Result<Vec<u8>, JsValue> {
        let options = match options_json {
            Some(ref json) => Some(
                serde_json::from_str::<world::WorldExportOptions>(json)
                    .map_err(|e| JsValue::from_str(&format!("Invalid options JSON: {}", e)))?,
            ),
            None => None,
        };
        world::to_world_zip(&self.0, options)
            .map_err(|e| JsValue::from_str(&format!("World zip export error: {}", e)))
    }

    pub fn to_schematic_version(&self, version: &str) -> Result<Vec<u8>, JsValue> {
        let version =
            schematic::to_schematic_version(&self.0, SchematicVersion::from_str(version).unwrap());
        match version {
            Ok(data) => Ok(data),
            Err(e) => Err(JsValue::from_str(&format!(
                "Schematic version conversion error: {}",
                e
            ))),
        }
    }

    pub fn get_available_schematic_versions(&self) -> Array {
        let versions = SchematicVersion::get_all();
        let js_versions = Array::new();
        for version in versions {
            js_versions.push(&JsValue::from_str(&version.to_string()));
        }
        js_versions
    }

    pub fn get_palette(&self) -> JsValue {
        let merged_region = self.0.get_merged_region();
        let palette = &merged_region.palette;

        let js_palette = Array::new();
        for block_state in palette {
            let obj = Object::new();
            Reflect::set(&obj, &"name".into(), &JsValue::from_str(&block_state.name)).unwrap();

            let properties = Object::new();
            for (key, value) in &block_state.properties {
                Reflect::set(
                    &properties,
                    &JsValue::from_str(key),
                    &JsValue::from_str(value),
                )
                .unwrap();
            }
            Reflect::set(&obj, &"properties".into(), &properties).unwrap();

            js_palette.push(&obj);
        }
        js_palette.into()
    }

    pub fn get_default_region_palette(&self) -> JsValue {
        let palette = self.0.get_default_region_palette();
        let js_palette = Array::new();
        for block_state in palette {
            let obj = Object::new();
            Reflect::set(&obj, &"name".into(), &JsValue::from_str(&block_state.name)).unwrap();

            let properties = Object::new();
            for (key, value) in &block_state.properties {
                Reflect::set(
                    &properties,
                    &JsValue::from_str(key),
                    &JsValue::from_str(value),
                )
                .unwrap();
            }
            Reflect::set(&obj, &"properties".into(), &properties).unwrap();

            js_palette.push(&obj);
        }
        js_palette.into()
    }

    pub fn get_palette_from_region(&self, region_name: &str) -> JsValue {
        let palette = if region_name == "default" || region_name == "Default" {
            &self.0.default_region.palette
        } else {
            match self.0.other_regions.get(region_name) {
                Some(region) => &region.palette,
                None => return JsValue::NULL, // Region not found
            }
        };

        let js_palette = Array::new();
        for block_state in palette {
            let obj = Object::new();
            Reflect::set(&obj, &"name".into(), &JsValue::from_str(&block_state.name)).unwrap();

            let properties = Object::new();
            for (key, value) in &block_state.properties {
                Reflect::set(
                    &properties,
                    &JsValue::from_str(key),
                    &JsValue::from_str(value),
                )
                .unwrap();
            }
            Reflect::set(&obj, &"properties".into(), &properties).unwrap();

            js_palette.push(&obj);
        }
        js_palette.into()
    }

    pub fn get_bounding_box(&self) -> JsValue {
        let bbox = self.0.get_bounding_box();
        let obj = Object::new();
        Reflect::set(
            &obj,
            &"min".into(),
            &Array::of3(
                &JsValue::from(bbox.min.0),
                &JsValue::from(bbox.min.1),
                &JsValue::from(bbox.min.2),
            ),
        )
        .unwrap();
        Reflect::set(
            &obj,
            &"max".into(),
            &Array::of3(
                &JsValue::from(bbox.max.0),
                &JsValue::from(bbox.max.1),
                &JsValue::from(bbox.max.2),
            ),
        )
        .unwrap();
        obj.into()
    }

    pub fn get_region_bounding_box(&self, region_name: &str) -> JsValue {
        let bbox = if region_name == "default" || region_name == "Default" {
            self.0.default_region.get_bounding_box()
        } else {
            match self.0.other_regions.get(region_name) {
                Some(region) => region.get_bounding_box(),
                None => return JsValue::NULL, // Region not found
            }
        };

        let obj = Object::new();
        Reflect::set(
            &obj,
            &"min".into(),
            &Array::of3(
                &JsValue::from(bbox.min.0),
                &JsValue::from(bbox.min.1),
                &JsValue::from(bbox.min.2),
            ),
        )
        .unwrap();
        Reflect::set(
            &obj,
            &"max".into(),
            &Array::of3(
                &JsValue::from(bbox.max.0),
                &JsValue::from(bbox.max.1),
                &JsValue::from(bbox.max.2),
            ),
        )
        .unwrap();
        obj.into()
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block_name: &str) {
        self.0.set_block_str(x, y, z, block_name);
    }

    pub fn set_block_in_region(
        &mut self,
        region_name: &str,
        x: i32,
        y: i32,
        z: i32,
        block_name: &str,
    ) -> bool {
        self.0
            .set_block_in_region_str(region_name, x, y, z, block_name)
    }

    pub fn set_block_from_string(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        block_string: &str,
    ) -> Result<(), JsValue> {
        self.0
            .set_block_from_string(x, y, z, block_string)
            .map(|_| ())
            .map_err(|e| JsValue::from_str(&e))
    }

    pub fn copy_region(
        &mut self,
        from_schematic: &SchematicWrapper,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
        target_x: i32,
        target_y: i32,
        target_z: i32,
        excluded_blocks: &JsValue,
    ) -> Result<(), JsValue> {
        let bounds = BoundingBox::new((min_x, min_y, min_z), (max_x, max_y, max_z));

        let excluded_blocks = if !excluded_blocks.is_undefined() && !excluded_blocks.is_null() {
            let js_array: Array = excluded_blocks
                .clone()
                .dyn_into()
                .map_err(|_| JsValue::from_str("Excluded blocks should be an array"))?;
            let mut rust_vec: Vec<BlockState> = Vec::new();
            for i in 0..js_array.length() {
                let block_string = match js_array.get(i).as_string() {
                    Some(name) => name,
                    None => return Err(JsValue::from_str("Excluded blocks should be strings")),
                };
                let (block_state, _) = UniversalSchematic::parse_block_string(&block_string)
                    .map_err(|e| JsValue::from_str(&format!("Invalid block state: {}", e)))?;
                rust_vec.push(block_state);
            }

            rust_vec
        } else {
            Vec::new() // Return empty vec instead of None
        };

        self.0
            .copy_region(
                &from_schematic.0,
                &bounds,
                (target_x, target_y, target_z),
                &excluded_blocks, // Now we can pass a direct reference to the Vec
            )
            .map_err(|e| JsValue::from_str(&format!("Failed to copy region: {}", e)))
    }

    pub fn set_block_with_properties(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        block_name: &str,
        properties: &JsValue,
    ) -> Result<(), JsValue> {
        // Convert JsValue to Vec<(SmolStr, SmolStr)>
        let mut props = Vec::new();

        if !properties.is_undefined() && !properties.is_null() {
            let obj: Object = properties
                .clone()
                .dyn_into()
                .map_err(|_| JsValue::from_str("Properties should be an object"))?;

            let keys = js_sys::Object::keys(&obj);
            for i in 0..keys.length() {
                let key = keys.get(i);
                let key_str = key
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("Property keys should be strings"))?;

                let value = Reflect::get(&obj, &key)
                    .map_err(|_| JsValue::from_str("Error getting property value"))?;

                let value_str = value
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("Property values should be strings"))?;

                props.push((
                    SmolStr::from(key_str.as_str()),
                    SmolStr::from(value_str.as_str()),
                ));
            }
        }

        // Create BlockState with properties
        let block_state = BlockState {
            name: block_name.into(),
            properties: props,
        };

        // Set the block in the schematic
        self.0.set_block(x, y, z, &block_state);

        Ok(())
    }

    #[wasm_bindgen(js_name = setBlockWithNbt)]
    pub fn set_block_with_nbt(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        block_name: &str,
        nbt_data: &JsValue,
    ) -> Result<(), JsValue> {
        // Convert JsValue to HashMap<String, String>
        let mut nbt = HashMap::new();

        if !nbt_data.is_undefined() && !nbt_data.is_null() {
            let obj: Object = nbt_data
                .clone()
                .dyn_into()
                .map_err(|_| JsValue::from_str("NBT data should be an object"))?;

            let keys = js_sys::Object::keys(&obj);
            for i in 0..keys.length() {
                let key = keys.get(i);
                let key_str = key
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("NBT keys should be strings"))?;

                let value = Reflect::get(&obj, &key)
                    .map_err(|_| JsValue::from_str("Error getting NBT value"))?;

                let value_str = value
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("NBT values should be strings"))?;

                nbt.insert(key_str, value_str);
            }
        }

        self.0
            .set_block_with_nbt(x, y, z, block_name, nbt)
            .map_err(|e| JsValue::from_str(&format!("Error setting block with NBT: {}", e)))?;
        Ok(())
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<String> {
        self.0
            .get_block(x, y, z)
            .map(|block_state| block_state.name.to_string())
    }

    /// Get block as formatted string with properties (e.g., "minecraft:lever[powered=true,facing=north]")
    pub fn get_block_string(&self, x: i32, y: i32, z: i32) -> Option<String> {
        self.0.get_block(x, y, z).map(|bs| bs.to_string())
    }

    pub fn get_block_with_properties(&self, x: i32, y: i32, z: i32) -> Option<BlockStateWrapper> {
        self.0.get_block(x, y, z).cloned().map(BlockStateWrapper)
    }

    /// Batch set blocks at multiple positions to the same block name.
    /// `positions` is a flat Int32Array of [x0,y0,z0, x1,y1,z1, ...].
    /// Returns the number of blocks set.
    pub fn set_blocks(&mut self, positions: &[i32], block_name: &str) -> usize {
        let count = positions.len() / 3;
        if count == 0 {
            return 0;
        }

        // Complex block strings fall back to per-call path
        if block_name.contains('[') || block_name.ends_with('}') {
            let mut set = 0;
            for i in 0..count {
                let (x, y, z) = (positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]);
                if self.0.set_block_str(x, y, z, block_name) {
                    set += 1;
                }
            }
            return set;
        }

        // Pre-expand to fit all positions
        let (mut min_x, mut min_y, mut min_z) = (positions[0], positions[1], positions[2]);
        let (mut max_x, mut max_y, mut max_z) = (min_x, min_y, min_z);
        for i in 1..count {
            let (x, y, z) = (positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]);
            if x < min_x {
                min_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if z < min_z {
                min_z = z;
            }
            if x > max_x {
                max_x = x;
            }
            if y > max_y {
                max_y = y;
            }
            if z > max_z {
                max_z = z;
            }
        }

        let region = &mut self.0.default_region;
        region.ensure_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
        let palette_index = region.get_or_insert_palette_by_name(block_name);

        for i in 0..count {
            let (x, y, z) = (positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]);
            region.set_block_at_index_unchecked(palette_index, x, y, z);
        }

        count
    }

    /// Batch get block names at multiple positions.
    /// `positions` is a flat Int32Array of [x0,y0,z0, x1,y1,z1, ...].
    /// Returns a JS Array of block name strings (null for empty/out-of-bounds).
    pub fn get_blocks(&self, positions: &[i32]) -> Array {
        let count = positions.len() / 3;
        let result = Array::new_with_length(count as u32);
        let region = &self.0.default_region;

        for i in 0..count {
            let (x, y, z) = (positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]);
            let val = if region.is_in_region(x, y, z) {
                region
                    .get_block_name(x, y, z)
                    .map(|s| JsValue::from_str(s))
                    .unwrap_or(JsValue::NULL)
            } else {
                // Fall back to multi-region scan
                self.0
                    .get_block(x, y, z)
                    .map(|bs| JsValue::from_str(&bs.name))
                    .unwrap_or(JsValue::NULL)
            };
            result.set(i as u32, val);
        }

        result
    }

    pub fn get_block_entity(&self, x: i32, y: i32, z: i32) -> JsValue {
        let block_position = BlockPosition { x, y, z };
        if let Some(block_entity) = self.0.get_block_entity(block_position) {
            if block_entity.id.contains("chest") {
                let obj = Object::new();
                Reflect::set(&obj, &"id".into(), &JsValue::from_str(&block_entity.id)).unwrap();

                let position = Array::new();
                position.push(&JsValue::from(block_entity.position.0));
                position.push(&JsValue::from(block_entity.position.1));
                position.push(&JsValue::from(block_entity.position.2));
                Reflect::set(&obj, &"position".into(), &position).unwrap();

                // Use the new to_js_value method
                Reflect::set(&obj, &"nbt".into(), &block_entity.nbt.to_js_value()).unwrap();

                obj.into()
            } else {
                JsValue::NULL
            }
        } else {
            JsValue::NULL
        }
    }

    pub fn get_all_block_entities(&self) -> JsValue {
        let block_entities = self.0.get_block_entities_as_list();
        let js_block_entities = Array::new();
        for block_entity in block_entities {
            let obj = Object::new();
            Reflect::set(&obj, &"id".into(), &JsValue::from_str(&block_entity.id)).unwrap();

            let position = Array::new();
            position.push(&JsValue::from(block_entity.position.0));
            position.push(&JsValue::from(block_entity.position.1));
            position.push(&JsValue::from(block_entity.position.2));
            Reflect::set(&obj, &"position".into(), &position).unwrap();

            // Use the new to_js_value method
            Reflect::set(&obj, &"nbt".into(), &block_entity.nbt.to_js_value()).unwrap();

            js_block_entities.push(&obj);
        }
        js_block_entities.into()
    }

    /// Get the number of mobile entities (not block entities).
    pub fn entity_count(&self) -> usize {
        self.0.default_region.entities.len()
    }

    /// Get all mobile entities as a JS array of objects.
    pub fn get_entities(&self) -> JsValue {
        let entities = &self.0.default_region.entities;
        let js_entities = Array::new();
        for entity in entities {
            let obj = Object::new();
            Reflect::set(&obj, &"id".into(), &JsValue::from_str(&entity.id)).unwrap();

            let position = Array::new();
            position.push(&JsValue::from(entity.position.0));
            position.push(&JsValue::from(entity.position.1));
            position.push(&JsValue::from(entity.position.2));
            Reflect::set(&obj, &"position".into(), &position).unwrap();

            let nbt_json = serde_json::to_string(&entity.nbt).unwrap_or_default();
            Reflect::set(&obj, &"nbt".into(), &JsValue::from_str(&nbt_json)).unwrap();

            js_entities.push(&obj);
        }
        js_entities.into()
    }

    /// Add a mobile entity to the schematic.
    pub fn add_entity(
        &mut self,
        id: &str,
        x: f64,
        y: f64,
        z: f64,
        nbt_json: Option<String>,
    ) -> Result<(), JsValue> {
        let mut entity = crate::entity::Entity::new(id.to_string(), (x, y, z));
        if let Some(ref json) = nbt_json {
            entity.nbt = serde_json::from_str(json)
                .map_err(|e| JsValue::from_str(&format!("Invalid NBT JSON: {}", e)))?;
        }
        self.0.add_entity(entity);
        Ok(())
    }

    /// Remove a mobile entity by index. Returns true if removed.
    pub fn remove_entity(&mut self, index: usize) -> bool {
        self.0.remove_entity(index).is_some()
    }

    pub fn print_schematic(&self) -> String {
        print_schematic(&self.0)
    }

    pub fn debug_info(&self) -> String {
        format!(
            "Schematic name: {}, Regions: {}",
            self.0
                .metadata
                .name
                .as_ref()
                .unwrap_or(&"Unnamed".to_string()),
            self.0.other_regions.len() + 1
        )
    }

    // --- Metadata Accessors ---

    #[wasm_bindgen(js_name = "getName")]
    pub fn get_name(&self) -> Option<String> {
        self.0.metadata.name.clone()
    }

    #[wasm_bindgen(js_name = "setName")]
    pub fn set_name(&mut self, name: &str) {
        self.0.metadata.name = Some(name.to_string());
    }

    #[wasm_bindgen(js_name = "getAuthor")]
    pub fn get_author(&self) -> Option<String> {
        self.0.metadata.author.clone()
    }

    #[wasm_bindgen(js_name = "setAuthor")]
    pub fn set_author(&mut self, author: &str) {
        self.0.metadata.author = Some(author.to_string());
    }

    #[wasm_bindgen(js_name = "getDescription")]
    pub fn get_description(&self) -> Option<String> {
        self.0.metadata.description.clone()
    }

    #[wasm_bindgen(js_name = "setDescription")]
    pub fn set_description(&mut self, description: &str) {
        self.0.metadata.description = Some(description.to_string());
    }

    #[wasm_bindgen(js_name = "getCreated")]
    pub fn get_created(&self) -> Option<f64> {
        self.0.metadata.created.map(|v| v as f64)
    }

    #[wasm_bindgen(js_name = "setCreated")]
    pub fn set_created(&mut self, created: f64) {
        self.0.metadata.created = Some(created as u64);
    }

    #[wasm_bindgen(js_name = "getModified")]
    pub fn get_modified(&self) -> Option<f64> {
        self.0.metadata.modified.map(|v| v as f64)
    }

    #[wasm_bindgen(js_name = "setModified")]
    pub fn set_modified(&mut self, modified: f64) {
        self.0.metadata.modified = Some(modified as u64);
    }

    #[wasm_bindgen(js_name = "getLmVersion")]
    pub fn get_lm_version(&self) -> Option<i32> {
        self.0.metadata.lm_version
    }

    #[wasm_bindgen(js_name = "setLmVersion")]
    pub fn set_lm_version(&mut self, version: i32) {
        self.0.metadata.lm_version = Some(version);
    }

    #[wasm_bindgen(js_name = "getMcVersion")]
    pub fn get_mc_version(&self) -> Option<i32> {
        self.0.metadata.mc_version
    }

    #[wasm_bindgen(js_name = "setMcVersion")]
    pub fn set_mc_version(&mut self, version: i32) {
        self.0.metadata.mc_version = Some(version);
    }

    #[wasm_bindgen(js_name = "getWeVersion")]
    pub fn get_we_version(&self) -> Option<i32> {
        self.0.metadata.we_version
    }

    #[wasm_bindgen(js_name = "setWeVersion")]
    pub fn set_we_version(&mut self, version: i32) {
        self.0.metadata.we_version = Some(version);
    }

    // Add these methods back
    pub fn get_dimensions(&self) -> Vec<i32> {
        // Return tight dimensions by default (actual content size)
        let tight = self.0.get_tight_dimensions();
        if tight != (0, 0, 0) {
            vec![tight.0, tight.1, tight.2]
        } else {
            let (x, y, z) = self.0.get_dimensions();
            vec![x, y, z]
        }
    }

    /// Get the allocated dimensions (full buffer size including pre-allocated space)
    /// Use this if you need to know the internal buffer size
    pub fn get_allocated_dimensions(&self) -> Vec<i32> {
        let (x, y, z) = self.0.get_dimensions();
        vec![x, y, z]
    }

    /// Get the tight dimensions of actual block content (excluding pre-allocated space)
    /// Returns [width, height, length] or [0, 0, 0] if no non-air blocks exist
    pub fn get_tight_dimensions(&self) -> Vec<i32> {
        let (x, y, z) = self.0.get_tight_dimensions();
        vec![x, y, z]
    }

    /// Get the tight bounding box min coordinates [x, y, z]
    /// Returns null if no non-air blocks have been placed
    pub fn get_tight_bounds_min(&self) -> Option<Vec<i32>> {
        self.0
            .get_tight_bounds()
            .map(|bounds| vec![bounds.min.0, bounds.min.1, bounds.min.2])
    }

    /// Get the tight bounding box max coordinates [x, y, z]
    /// Returns null if no non-air blocks have been placed
    pub fn get_tight_bounds_max(&self) -> Option<Vec<i32>> {
        self.0
            .get_tight_bounds()
            .map(|bounds| vec![bounds.max.0, bounds.max.1, bounds.max.2])
    }

    pub fn get_block_count(&self) -> i32 {
        self.0.total_blocks()
    }

    pub fn get_volume(&self) -> i32 {
        self.0.total_volume()
    }

    pub fn get_region_names(&self) -> Vec<String> {
        self.0.get_region_names()
    }

    pub fn blocks(&self) -> Array {
        self.0
            .iter_blocks()
            .map(|(pos, block)| {
                let obj = js_sys::Object::new();
                js_sys::Reflect::set(&obj, &"x".into(), &pos.x.into()).unwrap();
                js_sys::Reflect::set(&obj, &"y".into(), &pos.y.into()).unwrap();
                js_sys::Reflect::set(&obj, &"z".into(), &pos.z.into()).unwrap();
                js_sys::Reflect::set(&obj, &"name".into(), &JsValue::from_str(&block.name))
                    .unwrap();
                let properties = js_sys::Object::new();
                for (key, value) in &block.properties {
                    js_sys::Reflect::set(
                        &properties,
                        &JsValue::from_str(key),
                        &JsValue::from_str(value),
                    )
                    .unwrap();
                }
                js_sys::Reflect::set(&obj, &"properties".into(), &properties).unwrap();
                obj
            })
            .collect::<Array>()
    }

    pub fn chunks(&self, chunk_width: i32, chunk_height: i32, chunk_length: i32) -> Array {
        self.0
            .iter_chunks(
                chunk_width,
                chunk_height,
                chunk_length,
                Some(ChunkLoadingStrategy::BottomUp),
            )
            .map(|chunk| {
                let chunk_obj = js_sys::Object::new();
                js_sys::Reflect::set(&chunk_obj, &"chunk_x".into(), &chunk.chunk_x.into()).unwrap();
                js_sys::Reflect::set(&chunk_obj, &"chunk_y".into(), &chunk.chunk_y.into()).unwrap();
                js_sys::Reflect::set(&chunk_obj, &"chunk_z".into(), &chunk.chunk_z.into()).unwrap();

                let blocks_array = chunk
                    .positions
                    .into_iter()
                    .map(|pos| {
                        let block = self.0.get_block(pos.x, pos.y, pos.z).unwrap();
                        let obj = js_sys::Object::new();
                        js_sys::Reflect::set(&obj, &"x".into(), &pos.x.into()).unwrap();
                        js_sys::Reflect::set(&obj, &"y".into(), &pos.y.into()).unwrap();
                        js_sys::Reflect::set(&obj, &"z".into(), &pos.z.into()).unwrap();
                        js_sys::Reflect::set(&obj, &"name".into(), &JsValue::from_str(&block.name))
                            .unwrap();
                        let properties = js_sys::Object::new();
                        for (key, value) in &block.properties {
                            js_sys::Reflect::set(
                                &properties,
                                &JsValue::from_str(key),
                                &JsValue::from_str(value),
                            )
                            .unwrap();
                        }
                        js_sys::Reflect::set(&obj, &"properties".into(), &properties).unwrap();
                        obj
                    })
                    .collect::<Array>();

                js_sys::Reflect::set(&chunk_obj, &"blocks".into(), &blocks_array).unwrap();
                chunk_obj
            })
            .collect::<Array>()
    }

    pub fn chunks_with_strategy(
        &self,
        chunk_width: i32,
        chunk_height: i32,
        chunk_length: i32,
        strategy: &str,
        camera_x: f32,
        camera_y: f32,
        camera_z: f32,
    ) -> Array {
        // Map the string strategy to enum
        let strategy_enum = match strategy {
            "distance_to_camera" => Some(ChunkLoadingStrategy::DistanceToCamera(
                camera_x, camera_y, camera_z,
            )),
            "top_down" => Some(ChunkLoadingStrategy::TopDown),
            "bottom_up" => Some(ChunkLoadingStrategy::BottomUp),
            "center_outward" => Some(ChunkLoadingStrategy::CenterOutward),
            "random" => Some(ChunkLoadingStrategy::Random),
            _ => None, // Default
        };

        // Use the enhanced iter_chunks method
        self.0
            .iter_chunks(chunk_width, chunk_height, chunk_length, strategy_enum)
            .map(|chunk| {
                let chunk_obj = js_sys::Object::new();
                js_sys::Reflect::set(&chunk_obj, &"chunk_x".into(), &chunk.chunk_x.into()).unwrap();
                js_sys::Reflect::set(&chunk_obj, &"chunk_y".into(), &chunk.chunk_y.into()).unwrap();
                js_sys::Reflect::set(&chunk_obj, &"chunk_z".into(), &chunk.chunk_z.into()).unwrap();

                let blocks_array = chunk
                    .positions
                    .into_iter()
                    .map(|pos| {
                        let block = self.0.get_block(pos.x, pos.y, pos.z).unwrap();
                        let obj = js_sys::Object::new();
                        js_sys::Reflect::set(&obj, &"x".into(), &pos.x.into()).unwrap();
                        js_sys::Reflect::set(&obj, &"y".into(), &pos.y.into()).unwrap();
                        js_sys::Reflect::set(&obj, &"z".into(), &pos.z.into()).unwrap();
                        js_sys::Reflect::set(&obj, &"name".into(), &JsValue::from_str(&block.name))
                            .unwrap();
                        let properties = js_sys::Object::new();
                        for (key, value) in &block.properties {
                            js_sys::Reflect::set(
                                &properties,
                                &JsValue::from_str(key),
                                &JsValue::from_str(value),
                            )
                            .unwrap();
                        }
                        js_sys::Reflect::set(&obj, &"properties".into(), &properties).unwrap();
                        obj
                    })
                    .collect::<Array>();

                js_sys::Reflect::set(&chunk_obj, &"blocks".into(), &blocks_array).unwrap();
                chunk_obj
            })
            .collect::<Array>()
    }

    pub fn get_chunk_blocks(
        &self,
        offset_x: i32,
        offset_y: i32,
        offset_z: i32,
        width: i32,
        height: i32,
        length: i32,
    ) -> js_sys::Array {
        let blocks = self
            .0
            .iter_blocks()
            .filter(|(pos, _)| {
                pos.x >= offset_x
                    && pos.x < offset_x + width
                    && pos.y >= offset_y
                    && pos.y < offset_y + height
                    && pos.z >= offset_z
                    && pos.z < offset_z + length
            })
            .map(|(pos, block)| {
                let obj = js_sys::Object::new();
                js_sys::Reflect::set(&obj, &"x".into(), &pos.x.into()).unwrap();
                js_sys::Reflect::set(&obj, &"y".into(), &pos.y.into()).unwrap();
                js_sys::Reflect::set(&obj, &"z".into(), &pos.z.into()).unwrap();
                js_sys::Reflect::set(&obj, &"name".into(), &JsValue::from_str(&block.name))
                    .unwrap();
                let properties = js_sys::Object::new();
                for (key, value) in &block.properties {
                    js_sys::Reflect::set(
                        &properties,
                        &JsValue::from_str(key),
                        &JsValue::from_str(value),
                    )
                    .unwrap();
                }
                js_sys::Reflect::set(&obj, &"properties".into(), &properties).unwrap();
                obj
            })
            .collect::<js_sys::Array>();

        blocks
    }

    /// Get all palettes once - eliminates repeated string transfers
    /// Returns: { default: [BlockState], regions: { regionName: [BlockState] } }
    pub fn get_all_palettes(&self) -> JsValue {
        let all_palettes = self.0.get_all_palettes();

        let js_object = Object::new();

        // Convert default palette
        let default_palette = Array::new();
        for block_state in &all_palettes.default_palette {
            let block_obj = Object::new();
            Reflect::set(
                &block_obj,
                &"name".into(),
                &JsValue::from_str(&block_state.name),
            )
            .unwrap();

            let properties = Object::new();
            for (key, value) in &block_state.properties {
                Reflect::set(
                    &properties,
                    &JsValue::from_str(key),
                    &JsValue::from_str(value),
                )
                .unwrap();
            }
            Reflect::set(&block_obj, &"properties".into(), &properties).unwrap();
            default_palette.push(&block_obj);
        }
        Reflect::set(&js_object, &"default".into(), &default_palette).unwrap();

        // Convert region palettes
        let regions_obj = Object::new();
        for (region_name, palette) in &all_palettes.region_palettes {
            let region_palette = Array::new();
            for block_state in palette {
                let block_obj = Object::new();
                Reflect::set(
                    &block_obj,
                    &"name".into(),
                    &JsValue::from_str(&block_state.name),
                )
                .unwrap();

                let properties = Object::new();
                for (key, value) in &block_state.properties {
                    Reflect::set(
                        &properties,
                        &JsValue::from_str(key),
                        &JsValue::from_str(value),
                    )
                    .unwrap();
                }
                Reflect::set(&block_obj, &"properties".into(), &properties).unwrap();
                region_palette.push(&block_obj);
            }
            Reflect::set(
                &regions_obj,
                &JsValue::from_str(region_name),
                &region_palette,
            )
            .unwrap();
        }
        Reflect::set(&js_object, &"regions".into(), &regions_obj).unwrap();

        js_object.into()
    }

    /// Optimized chunks iterator that returns palette indices instead of full block data
    /// Returns array of: { chunk_x, chunk_y, chunk_z, blocks: [[x,y,z,palette_index],...] }
    pub fn chunks_indices(&self, chunk_width: i32, chunk_height: i32, chunk_length: i32) -> Array {
        self.0
            .iter_chunks_indices(
                chunk_width,
                chunk_height,
                chunk_length,
                Some(ChunkLoadingStrategy::BottomUp),
            )
            .map(|chunk| {
                let chunk_obj = Object::new();
                Reflect::set(&chunk_obj, &"chunk_x".into(), &chunk.chunk_x.into()).unwrap();
                Reflect::set(&chunk_obj, &"chunk_y".into(), &chunk.chunk_y.into()).unwrap();
                Reflect::set(&chunk_obj, &"chunk_z".into(), &chunk.chunk_z.into()).unwrap();

                // Pack blocks as array of [x, y, z, palette_index] for minimal data transfer
                let blocks_array = Array::new();
                for (pos, palette_index) in chunk.blocks {
                    let block_data = Array::new();
                    block_data.push(&pos.x.into());
                    block_data.push(&pos.y.into());
                    block_data.push(&pos.z.into());
                    block_data.push(&(palette_index as u32).into());
                    blocks_array.push(&block_data);
                }

                Reflect::set(&chunk_obj, &"blocks".into(), &blocks_array).unwrap();
                chunk_obj
            })
            .collect::<Array>()
    }

    /// Optimized chunks with strategy - returns palette indices
    pub fn chunks_indices_with_strategy(
        &self,
        chunk_width: i32,
        chunk_height: i32,
        chunk_length: i32,
        strategy: &str,
        camera_x: f32,
        camera_y: f32,
        camera_z: f32,
    ) -> Array {
        let strategy_enum = match strategy {
            "distance_to_camera" => Some(ChunkLoadingStrategy::DistanceToCamera(
                camera_x, camera_y, camera_z,
            )),
            "top_down" => Some(ChunkLoadingStrategy::TopDown),
            "bottom_up" => Some(ChunkLoadingStrategy::BottomUp),
            "center_outward" => Some(ChunkLoadingStrategy::CenterOutward),
            "random" => Some(ChunkLoadingStrategy::Random),
            _ => None,
        };

        self.0
            .iter_chunks_indices(chunk_width, chunk_height, chunk_length, strategy_enum)
            .map(|chunk| {
                let chunk_obj = Object::new();
                Reflect::set(&chunk_obj, &"chunk_x".into(), &chunk.chunk_x.into()).unwrap();
                Reflect::set(&chunk_obj, &"chunk_y".into(), &chunk.chunk_y.into()).unwrap();
                Reflect::set(&chunk_obj, &"chunk_z".into(), &chunk.chunk_z.into()).unwrap();

                let blocks_array = Array::new();
                for (pos, palette_index) in chunk.blocks {
                    let block_data = Array::new();
                    block_data.push(&pos.x.into());
                    block_data.push(&pos.y.into());
                    block_data.push(&pos.z.into());
                    block_data.push(&(palette_index as u32).into());
                    blocks_array.push(&block_data);
                }

                Reflect::set(&chunk_obj, &"blocks".into(), &blocks_array).unwrap();
                chunk_obj
            })
            .collect::<Array>()
    }

    /// Get specific chunk blocks as palette indices (for lazy loading individual chunks)
    /// Returns array of [x, y, z, palette_index]
    pub fn get_chunk_blocks_indices(
        &self,
        offset_x: i32,
        offset_y: i32,
        offset_z: i32,
        width: i32,
        height: i32,
        length: i32,
    ) -> Array {
        let blocks = self
            .0
            .get_chunk_blocks_indices(offset_x, offset_y, offset_z, width, height, length);

        let blocks_array = Array::new();
        for (pos, palette_index) in blocks {
            let block_data = Array::new();
            block_data.push(&pos.x.into());
            block_data.push(&pos.y.into());
            block_data.push(&pos.z.into());
            block_data.push(&(palette_index as u32).into());
            blocks_array.push(&block_data);
        }

        blocks_array
    }

    /// Get optimized chunk data including blocks and relevant tile entities
    /// Returns { blocks: [[x,y,z,palette_index],...], entities: [{id, position, nbt},...] }
    #[wasm_bindgen(js_name = getChunkData)]
    pub fn get_chunk_data(
        &self,
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
        chunk_width: i32,
        chunk_height: i32,
        chunk_length: i32,
    ) -> JsValue {
        let min_x = chunk_x * chunk_width;
        let min_y = chunk_y * chunk_height;
        let min_z = chunk_z * chunk_length;
        let max_x = min_x + chunk_width;
        let max_y = min_y + chunk_height;
        let max_z = min_z + chunk_length;

        // 1. Get Blocks (indices)
        let blocks = self.0.get_chunk_blocks_indices(
            min_x,
            min_y,
            min_z,
            chunk_width,
            chunk_height,
            chunk_length,
        );

        let result = Object::new();

        // Blocks array - Optimized to Flat Int32Array
        // [x, y, z, palette_index, x, y, z, palette_index, ...]
        let mut flat_blocks = Vec::with_capacity(blocks.len() * 4);
        for (pos, palette_index) in blocks {
            flat_blocks.push(pos.x);
            flat_blocks.push(pos.y);
            flat_blocks.push(pos.z);
            flat_blocks.push(palette_index as i32);
        }
        let blocks_typed_array = js_sys::Int32Array::from(&flat_blocks[..]);

        Reflect::set(&result, &"blocks".into(), &blocks_typed_array).unwrap();

        // 2. Get Entities (Naive filtering)
        // This runs in WASM/Rust so it's faster than JS
        let all_entities = self.0.get_block_entities_as_list();
        let entities_array = Array::new();

        for entity in all_entities {
            // Filter in Rust
            if entity.position.0 >= min_x
                && entity.position.0 < max_x
                && entity.position.1 >= min_y
                && entity.position.1 < max_y
                && entity.position.2 >= min_z
                && entity.position.2 < max_z
            {
                let obj = Object::new();
                Reflect::set(&obj, &"id".into(), &JsValue::from_str(&entity.id)).unwrap();

                let pos_arr = Array::new();
                pos_arr.push(&JsValue::from(entity.position.0));
                pos_arr.push(&JsValue::from(entity.position.1));
                pos_arr.push(&JsValue::from(entity.position.2));
                Reflect::set(&obj, &"position".into(), &pos_arr).unwrap();

                // NBT
                Reflect::set(&obj, &"nbt".into(), &entity.nbt.to_js_value()).unwrap();

                entities_array.push(&obj);
            }
        }
        Reflect::set(&result, &"entities".into(), &entities_array).unwrap();

        result.into()
    }

    /// All blocks as palette indices - for when you need everything at once but efficiently
    /// Returns array of [x, y, z, palette_index]
    pub fn blocks_indices(&self) -> Array {
        self.0
            .iter_blocks_indices()
            .map(|(pos, palette_index)| {
                let block_data = Array::new();
                block_data.push(&pos.x.into());
                block_data.push(&pos.y.into());
                block_data.push(&pos.z.into());
                block_data.push(&(palette_index as u32).into());
                block_data
            })
            .collect::<Array>()
    }

    /// Get optimization stats
    pub fn get_optimization_info(&self) -> JsValue {
        let default_region = &self.0.default_region;
        let total_blocks = default_region.blocks.len();
        let non_air_blocks = default_region
            .blocks
            .iter()
            .filter(|&&idx| idx != 0)
            .count();
        let palette_size = default_region.palette.len();

        let info_obj = Object::new();
        Reflect::set(
            &info_obj,
            &"total_blocks".into(),
            &(total_blocks as u32).into(),
        )
        .unwrap();
        Reflect::set(
            &info_obj,
            &"non_air_blocks".into(),
            &(non_air_blocks as u32).into(),
        )
        .unwrap();
        Reflect::set(
            &info_obj,
            &"palette_size".into(),
            &(palette_size as u32).into(),
        )
        .unwrap();
        Reflect::set(
            &info_obj,
            &"compression_ratio".into(),
            &((total_blocks as f64) / (palette_size as f64)).into(),
        )
        .unwrap();

        info_obj.into()
    }

    pub fn create_lazy_chunk_iterator(
        &self,
        chunk_width: i32,
        chunk_height: i32,
        chunk_length: i32,
        strategy: &str,
        camera_x: f32,
        camera_y: f32,
        camera_z: f32,
    ) -> LazyChunkIterator {
        let mut chunk_coords =
            self.calculate_chunk_coordinates(chunk_width, chunk_height, chunk_length);

        // Sort coordinates by strategy
        match strategy {
            "distance_to_camera" => {
                chunk_coords.sort_by(|a, b| {
                    let a_center_x = (a.0 * chunk_width) as f32 + (chunk_width as f32 / 2.0);
                    let a_center_y = (a.1 * chunk_height) as f32 + (chunk_height as f32 / 2.0);
                    let a_center_z = (a.2 * chunk_length) as f32 + (chunk_length as f32 / 2.0);

                    let b_center_x = (b.0 * chunk_width) as f32 + (chunk_width as f32 / 2.0);
                    let b_center_y = (b.1 * chunk_height) as f32 + (chunk_height as f32 / 2.0);
                    let b_center_z = (b.2 * chunk_length) as f32 + (chunk_length as f32 / 2.0);

                    let a_dist = (a_center_x - camera_x).powi(2)
                        + (a_center_y - camera_y).powi(2)
                        + (a_center_z - camera_z).powi(2);
                    let b_dist = (b_center_x - camera_x).powi(2)
                        + (b_center_y - camera_y).powi(2)
                        + (b_center_z - camera_z).powi(2);

                    a_dist
                        .partial_cmp(&b_dist)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            "bottom_up" => {
                chunk_coords.sort_by(|a, b| a.1.cmp(&b.1));
            }
            _ => {} // Default order
        }

        LazyChunkIterator {
            schematic_wrapper: self.clone(),
            chunk_width,
            chunk_height,
            chunk_length,
            current_chunk_coords: chunk_coords,
            current_index: 0,
        }
    }

    fn calculate_chunk_coordinates(
        &self,
        chunk_width: i32,
        chunk_height: i32,
        chunk_length: i32,
    ) -> Vec<(i32, i32, i32)> {
        use std::collections::HashSet;
        let mut chunk_coords = HashSet::new();

        let get_chunk_coord = |pos: i32, chunk_size: i32| -> i32 {
            let offset = if pos < 0 { chunk_size - 1 } else { 0 };
            (pos - offset) / chunk_size
        };

        // Use iter_blocks_indices to skip air blocks, maintaining consistency with chunk methods
        for (pos, _palette_index) in self.0.iter_blocks_indices() {
            let chunk_x = get_chunk_coord(pos.x, chunk_width);
            let chunk_y = get_chunk_coord(pos.y, chunk_height);
            let chunk_z = get_chunk_coord(pos.z, chunk_length);
            chunk_coords.insert((chunk_x, chunk_y, chunk_z));
        }

        chunk_coords.into_iter().collect()
    }

    // Transformation methods

    /// Flip the schematic along the X axis
    pub fn flip_x(&mut self) {
        self.0.flip_x();
    }

    /// Flip the schematic along the Y axis
    pub fn flip_y(&mut self) {
        self.0.flip_y();
    }

    /// Flip the schematic along the Z axis
    pub fn flip_z(&mut self) {
        self.0.flip_z();
    }

    /// Rotate the schematic around the Y axis (horizontal plane)
    /// Degrees must be 90, 180, or 270
    pub fn rotate_y(&mut self, degrees: i32) {
        self.0.rotate_y(degrees);
    }

    /// Rotate the schematic around the X axis
    /// Degrees must be 90, 180, or 270
    pub fn rotate_x(&mut self, degrees: i32) {
        self.0.rotate_x(degrees);
    }

    /// Rotate the schematic around the Z axis
    /// Degrees must be 90, 180, or 270
    pub fn rotate_z(&mut self, degrees: i32) {
        self.0.rotate_z(degrees);
    }

    /// Flip a specific region along the X axis
    pub fn flip_region_x(&mut self, region_name: &str) -> Result<(), JsValue> {
        self.0
            .flip_region_x(region_name)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Flip a specific region along the Y axis
    pub fn flip_region_y(&mut self, region_name: &str) -> Result<(), JsValue> {
        self.0
            .flip_region_y(region_name)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Flip a specific region along the Z axis
    pub fn flip_region_z(&mut self, region_name: &str) -> Result<(), JsValue> {
        self.0
            .flip_region_z(region_name)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Rotate a specific region around the Y axis
    pub fn rotate_region_y(&mut self, region_name: &str, degrees: i32) -> Result<(), JsValue> {
        self.0
            .rotate_region_y(region_name, degrees)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Rotate a specific region around the X axis
    pub fn rotate_region_x(&mut self, region_name: &str, degrees: i32) -> Result<(), JsValue> {
        self.0
            .rotate_region_x(region_name, degrees)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Rotate a specific region around the Z axis
    pub fn rotate_region_z(&mut self, region_name: &str, degrees: i32) -> Result<(), JsValue> {
        self.0
            .rotate_region_z(region_name, degrees)
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = addDefinitionRegion)]
    pub fn add_definition_region(&mut self, name: String, region: &DefinitionRegionWrapper) {
        self.0.definition_regions.insert(name, region.inner.clone());
    }

    #[wasm_bindgen(js_name = getDefinitionRegion)]
    pub fn get_definition_region(
        &mut self,
        name: String,
    ) -> Result<DefinitionRegionWrapper, JsValue> {
        match self.0.definition_regions.get(&name) {
            Some(region) => Ok(DefinitionRegionWrapper {
                inner: region.clone(),
                schematic_ptr: &mut self.0 as *mut _,
                name: Some(name),
            }),
            None => Err(JsValue::from_str(&format!(
                "Definition region '{}' not found",
                name
            ))),
        }
    }

    #[wasm_bindgen(js_name = removeDefinitionRegion)]
    pub fn remove_definition_region(&mut self, name: String) -> bool {
        self.0.definition_regions.remove(&name).is_some()
    }

    #[wasm_bindgen(js_name = getDefinitionRegionNames)]
    pub fn get_definition_region_names(&self) -> Array {
        let array = Array::new();
        for name in self.0.definition_regions.keys() {
            array.push(&JsValue::from_str(name));
        }
        array
    }

    #[wasm_bindgen(js_name = createDefinitionRegion)]
    pub fn create_definition_region(&mut self, name: String) {
        self.0
            .definition_regions
            .insert(name, DefinitionRegion::new());
    }

    #[wasm_bindgen(js_name = definitionRegionAddBounds)]
    pub fn definition_region_add_bounds(
        &mut self,
        name: String,
        min: BlockPosition,
        max: BlockPosition,
    ) -> Result<(), JsValue> {
        match self.0.definition_regions.get_mut(&name) {
            Some(region) => {
                region.add_bounds((min.x, min.y, min.z), (max.x, max.y, max.z));
                Ok(())
            }
            None => Err(JsValue::from_str(&format!(
                "Definition region '{}' not found",
                name
            ))),
        }
    }

    #[wasm_bindgen(js_name = definitionRegionAddPoint)]
    pub fn definition_region_add_point(
        &mut self,
        name: String,
        x: i32,
        y: i32,
        z: i32,
    ) -> Result<(), JsValue> {
        match self.0.definition_regions.get_mut(&name) {
            Some(region) => {
                region.add_point(x, y, z);
                Ok(())
            }
            None => Err(JsValue::from_str(&format!(
                "Definition region '{}' not found",
                name
            ))),
        }
    }

    #[wasm_bindgen(js_name = definitionRegionSetMetadata)]
    pub fn definition_region_set_metadata(
        &mut self,
        name: String,
        key: String,
        value: String,
    ) -> Result<(), JsValue> {
        match self.0.definition_regions.get_mut(&name) {
            Some(region) => {
                region.metadata.insert(key, value);
                Ok(())
            }
            None => Err(JsValue::from_str(&format!(
                "Definition region '{}' not found",
                name
            ))),
        }
    }

    #[wasm_bindgen(js_name = definitionRegionShift)]
    pub fn definition_region_shift(
        &mut self,
        name: String,
        x: i32,
        y: i32,
        z: i32,
    ) -> Result<(), JsValue> {
        match self.0.definition_regions.get_mut(&name) {
            Some(region) => {
                region.shift(x, y, z);
                Ok(())
            }
            None => Err(JsValue::from_str(&format!(
                "Definition region '{}' not found",
                name
            ))),
        }
    }

    #[wasm_bindgen(js_name = createRegion)]
    pub fn create_region(
        &mut self,
        name: String,
        min: JsValue,
        max: JsValue,
    ) -> Result<DefinitionRegionWrapper, JsValue> {
        let min_x = Reflect::get(&min, &"x".into())?.as_f64().unwrap_or(0.0) as i32;
        let min_y = Reflect::get(&min, &"y".into())?.as_f64().unwrap_or(0.0) as i32;
        let min_z = Reflect::get(&min, &"z".into())?.as_f64().unwrap_or(0.0) as i32;

        let max_x = Reflect::get(&max, &"x".into())?.as_f64().unwrap_or(0.0) as i32;
        let max_y = Reflect::get(&max, &"y".into())?.as_f64().unwrap_or(0.0) as i32;
        let max_z = Reflect::get(&max, &"z".into())?.as_f64().unwrap_or(0.0) as i32;

        let mut region = DefinitionRegion::new();
        region.add_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));

        self.0
            .definition_regions
            .insert(name.clone(), region.clone());

        Ok(DefinitionRegionWrapper {
            inner: region,
            schematic_ptr: &mut self.0 as *mut UniversalSchematic,
            name: Some(name),
        })
    }

    #[wasm_bindgen(js_name = updateRegion)]
    pub fn update_region(&mut self, name: String, region: &DefinitionRegionWrapper) {
        self.0.definition_regions.insert(name, region.inner.clone());
    }

    #[wasm_bindgen(js_name = createDefinitionRegionFromPoint)]
    pub fn create_definition_region_from_point(&mut self, name: String, x: i32, y: i32, z: i32) {
        let mut region = DefinitionRegion::new();
        region.add_point(x, y, z);
        self.0.definition_regions.insert(name, region);
    }

    #[wasm_bindgen(js_name = createDefinitionRegionFromBounds)]
    pub fn create_definition_region_from_bounds(
        &mut self,
        name: String,
        min: BlockPosition,
        max: BlockPosition,
    ) {
        let mut region = DefinitionRegion::new();
        region.add_bounds((min.x, min.y, min.z), (max.x, max.y, max.z));
        self.0.definition_regions.insert(name, region);
    }

    #[cfg(feature = "simulation")]
    #[wasm_bindgen(js_name = createCircuitBuilder)]
    pub fn create_circuit_builder(&self) -> CircuitBuilderWrapper {
        CircuitBuilderWrapper::new(self)
    }

    #[cfg(feature = "simulation")]
    #[wasm_bindgen(js_name = createCircuit)]
    pub fn create_circuit(
        &mut self,
        inputs: JsValue,
        outputs: JsValue,
    ) -> Result<TypedCircuitExecutorWrapper, JsValue> {
        let mut builder = CircuitBuilderWrapper::new(self);

        if !inputs.is_undefined() {
            let inputs_array = Array::from(&inputs);
            for i in 0..inputs_array.length() {
                let input = inputs_array.get(i);
                let name = Reflect::get(&input, &JsValue::from_str("name"))?
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("Input name missing or not a string"))?;

                let bits = Reflect::get(&input, &JsValue::from_str("bits"))?
                    .as_f64()
                    .unwrap_or(1.0) as u32;

                let region_name = Reflect::get(&input, &JsValue::from_str("region"))?
                    .as_string()
                    .ok_or_else(|| {
                        JsValue::from_str("Input region name missing or not a string")
                    })?;

                let region = self.get_definition_region(region_name.clone())?;
                let io_type = crate::simulation::typed_executor::IoType::UnsignedInt {
                    bits: bits as usize,
                };
                let io_type_wrapper = crate::IoTypeWrapper { inner: io_type };

                builder = builder.with_input_auto(name, &io_type_wrapper, &region)?;
            }
        }

        if !outputs.is_undefined() {
            let outputs_array = Array::from(&outputs);
            for i in 0..outputs_array.length() {
                let output = outputs_array.get(i);
                let name = Reflect::get(&output, &JsValue::from_str("name"))?
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("Output name missing or not a string"))?;

                let bits = Reflect::get(&output, &JsValue::from_str("bits"))?
                    .as_f64()
                    .unwrap_or(1.0) as u32;

                let region_name = Reflect::get(&output, &JsValue::from_str("region"))?
                    .as_string()
                    .ok_or_else(|| {
                        JsValue::from_str("Output region name missing or not a string")
                    })?;

                let region = self.get_definition_region(region_name.clone())?;
                let io_type = crate::simulation::typed_executor::IoType::UnsignedInt {
                    bits: bits as usize,
                };
                let io_type_wrapper = crate::IoTypeWrapper { inner: io_type };

                builder = builder.with_output_auto(name, &io_type_wrapper, &region)?;
            }
        }

        Ok(builder.build()?)
    }

    #[cfg(feature = "simulation")]
    #[wasm_bindgen(js_name = buildExecutor)]
    pub fn build_executor(
        &mut self,
        config: JsValue,
    ) -> Result<TypedCircuitExecutorWrapper, JsValue> {
        let mut builder = CircuitBuilderWrapper::new(self);

        // Parse config object
        // Expected format:
        // {
        //   inputs: [ { name: "a", type: "uint", bits: 8, region: "a" }, ... ],
        //   outputs: [ { name: "out", type: "matrix", rows: 4, cols: 4, element: "boolean", region: "c" }, ... ]
        // }

        let inputs = Reflect::get(&config, &JsValue::from_str("inputs"))?;
        if !inputs.is_undefined() {
            let inputs_array = Array::from(&inputs);
            for i in 0..inputs_array.length() {
                let input = inputs_array.get(i);
                let name = Reflect::get(&input, &JsValue::from_str("name"))?
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("Input name missing or not a string"))?;

                let region_name = Reflect::get(&input, &JsValue::from_str("region"))?
                    .as_string()
                    .ok_or_else(|| {
                        JsValue::from_str("Input region name missing or not a string")
                    })?;

                // Get region wrapper
                let region = self.get_definition_region(region_name.clone())?;

                // Create IO Type
                let io_type = parse_js_io_type(&input)?;
                let io_type_wrapper = crate::IoTypeWrapper { inner: io_type };

                // Parse sort strategy
                let sort_str = Reflect::get(&input, &JsValue::from_str("sort"))?.as_string();

                let sort_wrapper = if let Some(s) = sort_str {
                    Some(SortStrategyWrapper {
                        inner: parse_sort_string(&s),
                    })
                } else {
                    None
                };

                if let Some(sort) = sort_wrapper {
                    builder =
                        builder.with_input_auto_sorted(name, &io_type_wrapper, &region, &sort)?;
                } else {
                    builder = builder.with_input_auto(name, &io_type_wrapper, &region)?;
                }
            }
        }

        let outputs = Reflect::get(&config, &JsValue::from_str("outputs"))?;
        if !outputs.is_undefined() {
            let outputs_array = Array::from(&outputs);
            for i in 0..outputs_array.length() {
                let output = outputs_array.get(i);
                let name = Reflect::get(&output, &JsValue::from_str("name"))?
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("Output name missing or not a string"))?;

                let region_name = Reflect::get(&output, &JsValue::from_str("region"))?
                    .as_string()
                    .ok_or_else(|| {
                        JsValue::from_str("Output region name missing or not a string")
                    })?;

                // Get region wrapper
                let region = self.get_definition_region(region_name.clone())?;

                // Create IO Type
                let io_type = parse_js_io_type(&output)?;
                let io_type_wrapper = crate::IoTypeWrapper { inner: io_type };

                // Parse sort strategy
                let sort_str = Reflect::get(&output, &JsValue::from_str("sort"))?.as_string();

                let sort_wrapper = if let Some(s) = sort_str {
                    Some(SortStrategyWrapper {
                        inner: parse_sort_string(&s),
                    })
                } else {
                    None
                };

                if let Some(sort) = sort_wrapper {
                    builder =
                        builder.with_output_auto_sorted(name, &io_type_wrapper, &region, &sort)?;
                } else {
                    builder = builder.with_output_auto(name, &io_type_wrapper, &region)?;
                }
            }
        }

        Ok(builder.build()?)
    }
}

impl Clone for SchematicWrapper {
    fn clone(&self) -> Self {
        SchematicWrapper(self.0.clone())
    }
}
#[wasm_bindgen]
impl LazyChunkIterator {
    /// Get the next chunk on-demand (generates it fresh, doesn't store it)
    pub fn next(&mut self) -> JsValue {
        if self.current_index >= self.current_chunk_coords.len() {
            return JsValue::NULL;
        }

        let (chunk_x, chunk_y, chunk_z) = self.current_chunk_coords[self.current_index];
        self.current_index += 1;

        // Calculate chunk bounds
        let min_x = chunk_x * self.chunk_width;
        let min_y = chunk_y * self.chunk_height;
        let min_z = chunk_z * self.chunk_length;

        // Generate this chunk's data on-demand (only in memory temporarily)
        let blocks = self.schematic_wrapper.0.get_chunk_blocks_indices(
            min_x,
            min_y,
            min_z,
            self.chunk_width,
            self.chunk_height,
            self.chunk_length,
        );

        // Create result object
        let chunk_obj = Object::new();
        Reflect::set(&chunk_obj, &"chunk_x".into(), &chunk_x.into()).unwrap();
        Reflect::set(&chunk_obj, &"chunk_y".into(), &chunk_y.into()).unwrap();
        Reflect::set(&chunk_obj, &"chunk_z".into(), &chunk_z.into()).unwrap();
        Reflect::set(
            &chunk_obj,
            &"index".into(),
            &(self.current_index - 1).into(),
        )
        .unwrap();
        Reflect::set(
            &chunk_obj,
            &"total".into(),
            &self.current_chunk_coords.len().into(),
        )
        .unwrap();

        // Flatten blocks to Int32Array for performance
        let mut flat_blocks = Vec::with_capacity(blocks.len() * 4);
        for (pos, palette_index) in blocks {
            flat_blocks.push(pos.x);
            flat_blocks.push(pos.y);
            flat_blocks.push(pos.z);
            flat_blocks.push(palette_index as i32);
        }
        let blocks_typed_array = js_sys::Int32Array::from(&flat_blocks[..]);

        Reflect::set(&chunk_obj, &"blocks".into(), &blocks_typed_array).unwrap();

        chunk_obj.into()
    }

    pub fn has_next(&self) -> bool {
        self.current_index < self.current_chunk_coords.len()
    }

    pub fn total_chunks(&self) -> u32 {
        self.current_chunk_coords.len() as u32
    }

    pub fn current_position(&self) -> u32 {
        self.current_index as u32
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
    }

    pub fn skip_to(&mut self, index: u32) {
        self.current_index = (index as usize).min(self.current_chunk_coords.len());
    }
}

#[wasm_bindgen]
impl BlockStateWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str) -> Self {
        BlockStateWrapper(BlockState::new(name.to_string()))
    }

    pub fn with_property(&mut self, key: &str, value: &str) {
        self.0 = self
            .0
            .clone()
            .with_property(key.to_string(), value.to_string());
    }

    pub fn name(&self) -> String {
        self.0.name.to_string()
    }

    pub fn properties(&self) -> JsValue {
        let properties = self.0.properties.clone();
        let js_properties = js_sys::Object::new();
        for (key, value) in properties {
            js_sys::Reflect::set(
                &js_properties,
                &JsValue::from_str(key.as_str()),
                &JsValue::from_str(value.as_str()),
            )
            .unwrap();
        }
        js_properties.into()
    }
}

// Standalone functions
#[wasm_bindgen]
pub fn debug_schematic(schematic: &SchematicWrapper) -> String {
    format!(
        "{}\n{}",
        schematic.debug_info(),
        print_schematic(&schematic.0)
    )
}

#[wasm_bindgen]
pub fn debug_json_schematic(schematic: &SchematicWrapper) -> String {
    format!(
        "{}\n{}",
        schematic.debug_info(),
        print_json_schematic(&schematic.0)
    )
}

// ============================================================================
// INSIGN BINDINGS
// ============================================================================

#[wasm_bindgen]
impl SchematicWrapper {
    /// Extract all sign text from the schematic
    /// Returns a JavaScript array of objects: [{pos: [x,y,z], text: "..."}]
    #[wasm_bindgen(js_name = extractSigns)]
    pub fn extract_signs(&self) -> JsValue {
        let signs = crate::insign::extract_signs(&self.0);

        let js_signs = Array::new();
        for sign in signs {
            let obj = Object::new();

            // Create pos array
            let pos_array = Array::new();
            pos_array.push(&JsValue::from_f64(sign.pos[0] as f64));
            pos_array.push(&JsValue::from_f64(sign.pos[1] as f64));
            pos_array.push(&JsValue::from_f64(sign.pos[2] as f64));

            Reflect::set(&obj, &"pos".into(), &pos_array).unwrap();
            Reflect::set(&obj, &"text".into(), &JsValue::from_str(&sign.text)).unwrap();

            js_signs.push(&obj);
        }

        js_signs.into()
    }

    /// Compile Insign annotations from the schematic's signs
    /// Returns a JavaScript object with compiled region metadata
    /// This returns raw Insign data - interpretation is up to the consumer
    #[wasm_bindgen(js_name = compileInsign)]
    pub fn compile_insign(&self) -> Result<JsValue, JsValue> {
        let insign_data = crate::insign::compile_schematic_insign(&self.0)
            .map_err(|e| JsValue::from_str(&format!("Insign compilation error: {}", e)))?;

        // Convert serde_json::Value to JsValue
        serde_wasm_bindgen::to_value(&insign_data)
            .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
    }
}

#[cfg(feature = "simulation")]
fn parse_sort_string(sort_str: &str) -> SortStrategy {
    use crate::simulation::typed_executor::sort_strategy::{Axis, Direction};

    // Handle standard presets
    match sort_str {
        "yxz" => return SortStrategy::YXZ,
        "xyz" => return SortStrategy::XYZ,
        "zyx" => return SortStrategy::ZYX,
        "yDescXZ" => return SortStrategy::YDescXZ,
        "xDescYZ" => return SortStrategy::XDescYZ,
        "zDescYX" => return SortStrategy::ZDescYX,
        "descending" => return SortStrategy::YXZDesc,
        _ => {}
    }

    // Parse custom string like "-y+x-z" or "yxz"
    let mut orders = Vec::new();
    let mut chars = sort_str.chars().peekable();

    while let Some(&c) = chars.peek() {
        let direction = if c == '-' {
            chars.next();
            Direction::Descending
        } else if c == '+' {
            chars.next();
            Direction::Ascending
        } else {
            Direction::Ascending // Default to ascending if no sign
        };

        if let Some(axis_char) = chars.next() {
            let axis = match axis_char.to_ascii_lowercase() {
                'x' => Axis::X,
                'y' => Axis::Y,
                'z' => Axis::Z,
                _ => continue, // Skip invalid chars
            };
            orders.push((axis, direction));
        }
    }

    if orders.is_empty() {
        SortStrategy::YXZ // Fallback
    } else {
        SortStrategy::Custom(orders)
    }
}

#[cfg(feature = "simulation")]
fn parse_js_io_type(config: &JsValue) -> Result<IoType, JsValue> {
    if config.is_string() {
        let type_str = config.as_string().unwrap();
        return match type_str.as_str() {
            "boolean" => Ok(IoType::Boolean),
            "float" => Ok(IoType::Float32),
            "uint" => Ok(IoType::UnsignedInt { bits: 1 }),
            "int" => Ok(IoType::SignedInt { bits: 1 }),
            "hex" => Ok(IoType::UnsignedInt { bits: 4 }),
            _ => Err(JsValue::from_str(&format!(
                "Unknown simple IO type: {}",
                type_str
            ))),
        };
    }

    let type_str = Reflect::get(config, &"type".into())?
        .as_string()
        .unwrap_or_else(|| "uint".to_string());

    match type_str.as_str() {
        "uint" => {
            let bits = Reflect::get(config, &"bits".into())?
                .as_f64()
                .unwrap_or(1.0) as usize;
            Ok(IoType::UnsignedInt { bits })
        }
        "int" => {
            let bits = Reflect::get(config, &"bits".into())?
                .as_f64()
                .unwrap_or(1.0) as usize;
            Ok(IoType::SignedInt { bits })
        }
        "float" => Ok(IoType::Float32),
        "boolean" => Ok(IoType::Boolean),
        "hex" => Ok(IoType::UnsignedInt { bits: 4 }),
        "array" => {
            let length = Reflect::get(config, &"length".into())?
                .as_f64()
                .ok_or_else(|| JsValue::from_str("Array type requires 'length'"))?
                as usize;

            let element_val = Reflect::get(config, &"element".into())?;
            let element_type = parse_js_io_type(&element_val)?;

            Ok(IoType::Array {
                element_type: Box::new(element_type),
                length,
            })
        }
        "matrix" => {
            let rows = Reflect::get(config, &"rows".into())?
                .as_f64()
                .ok_or_else(|| JsValue::from_str("Matrix type requires 'rows'"))?
                as usize;
            let cols = Reflect::get(config, &"cols".into())?
                .as_f64()
                .ok_or_else(|| JsValue::from_str("Matrix type requires 'cols'"))?
                as usize;

            let element_val = Reflect::get(config, &"element".into())?;
            let element_type = parse_js_io_type(&element_val)?;

            Ok(IoType::Matrix {
                element_type: Box::new(element_type),
                rows,
                cols,
            })
        }
        _ => Err(JsValue::from_str(&format!("Unknown IO type: {}", type_str))),
    }
}
