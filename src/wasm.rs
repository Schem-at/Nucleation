// src/wasm.rs

use crate::bounding_box::BoundingBox;
use crate::schematic::SchematicVersion;
use crate::universal_schematic::ChunkLoadingStrategy;
use crate::{
    block_position::BlockPosition,
    formats::{litematic, schematic},
    print_utils::{
        format_json_schematic as print_json_schematic, format_schematic as print_schematic,
    },
    BlockState, UniversalSchematic,
};
use js_sys::{self, Array, Object, Reflect};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::console;

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

#[wasm_bindgen(start)]
pub fn start() {
    console::log_1(&"Initializing schematic utilities".into());
}

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

    pub fn from_data(&mut self, data: &[u8]) -> Result<(), JsValue> {
        if litematic::is_litematic(data) {
            console::log_1(&"Parsing litematic data".into());
            self.from_litematic(data)
        } else if schematic::is_schematic(data) {
            console::log_1(&"Parsing schematic data".into());
            self.from_schematic(data)
        } else {
            Err(JsValue::from_str("Unknown or unsupported schematic format"))
        }
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
        // Convert JsValue to HashMap<String, String>
        let mut props = HashMap::new();

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

                props.insert(key_str, value_str);
            }
        }

        // Create BlockState with properties
        let block_state = BlockState {
            name: block_name.to_string(),
            properties: props,
        };

        // Set the block in the schematic
        self.0.set_block(x, y, z, block_state);

        Ok(())
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<String> {
        self.0
            .get_block(x, y, z)
            .map(|block_state| block_state.name.clone())
    }

    /// Get block as formatted string with properties (e.g., "minecraft:lever[powered=true,facing=north]")
    pub fn get_block_string(&self, x: i32, y: i32, z: i32) -> Option<String> {
        self.0.get_block(x, y, z).map(|bs| bs.to_string())
    }

    pub fn get_block_with_properties(&self, x: i32, y: i32, z: i32) -> Option<BlockStateWrapper> {
        self.0.get_block(x, y, z).cloned().map(BlockStateWrapper)
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

    // Add these methods back
    pub fn get_dimensions(&self) -> Vec<i32> {
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
        let blocks = self.schematic_wrapper.get_chunk_blocks_indices(
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

        // Blocks are already in the right format: [[x,y,z,palette_index], ...]
        Reflect::set(&chunk_obj, &"blocks".into(), &blocks).unwrap();

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
        self.0.name.clone()
    }

    pub fn properties(&self) -> JsValue {
        let properties = self.0.properties.clone();
        let js_properties = js_sys::Object::new();
        for (key, value) in properties {
            js_sys::Reflect::set(&js_properties, &key.into(), &value.into()).unwrap();
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

// ============================================================================
// SIMULATION (MCHPRS) BINDINGS
// ============================================================================

#[cfg(feature = "simulation")]
use crate::simulation::{generate_truth_table, BlockPos, MchprsWorld, SimulationOptions};

#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct SimulationOptionsWrapper {
    inner: SimulationOptions,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl SimulationOptionsWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: SimulationOptions::default(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn optimize(&self) -> bool {
        self.inner.optimize
    }

    #[wasm_bindgen(setter)]
    pub fn set_optimize(&mut self, value: bool) {
        self.inner.optimize = value;
    }

    #[wasm_bindgen(getter)]
    pub fn io_only(&self) -> bool {
        self.inner.io_only
    }

    #[wasm_bindgen(setter)]
    pub fn set_io_only(&mut self, value: bool) {
        self.inner.io_only = value;
    }

    /// Adds a position to the custom IO list
    #[wasm_bindgen(js_name = addCustomIo)]
    pub fn add_custom_io(&mut self, x: i32, y: i32, z: i32) {
        self.inner.custom_io.push(BlockPos::new(x, y, z));
    }

    /// Clears the custom IO list
    #[wasm_bindgen(js_name = clearCustomIo)]
    pub fn clear_custom_io(&mut self) {
        self.inner.custom_io.clear();
    }
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct MchprsWorldWrapper {
    world: MchprsWorld,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl SchematicWrapper {
    /// Creates a simulation world for this schematic with default options
    ///
    /// This allows you to simulate redstone circuits and interact with them.
    pub fn create_simulation_world(&self) -> Result<MchprsWorldWrapper, JsValue> {
        MchprsWorldWrapper::new(self)
    }

    /// Creates a simulation world for this schematic with custom options
    ///
    /// This allows you to configure simulation behavior like wire state tracking.
    pub fn create_simulation_world_with_options(
        &self,
        options: &SimulationOptionsWrapper,
    ) -> Result<MchprsWorldWrapper, JsValue> {
        MchprsWorldWrapper::with_options(self, options)
    }
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl MchprsWorldWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(schematic: &SchematicWrapper) -> Result<MchprsWorldWrapper, JsValue> {
        let world = MchprsWorld::new(schematic.0.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to create MchprsWorld: {}", e)))?;

        Ok(MchprsWorldWrapper { world })
    }

    /// Creates a simulation world with custom options
    pub fn with_options(
        schematic: &SchematicWrapper,
        options: &SimulationOptionsWrapper,
    ) -> Result<MchprsWorldWrapper, JsValue> {
        let world = MchprsWorld::with_options(schematic.0.clone(), options.inner.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to create MchprsWorld: {}", e)))?;

        Ok(MchprsWorldWrapper { world })
    }

    /// Simulates a right-click on a block (typically a lever)
    pub fn on_use_block(&mut self, x: i32, y: i32, z: i32) {
        self.world.on_use_block(BlockPos::new(x, y, z));
    }

    /// Advances the simulation by the specified number of ticks
    pub fn tick(&mut self, number_of_ticks: u32) {
        self.world.tick(number_of_ticks);
    }

    /// Flushes pending changes from the compiler to the world
    pub fn flush(&mut self) {
        self.world.flush();
    }

    /// Checks if a redstone lamp is lit at the given position
    pub fn is_lit(&self, x: i32, y: i32, z: i32) -> bool {
        self.world.is_lit(BlockPos::new(x, y, z))
    }

    /// Gets the power state of a lever
    pub fn get_lever_power(&self, x: i32, y: i32, z: i32) -> bool {
        self.world.get_lever_power(BlockPos::new(x, y, z))
    }

    /// Gets the redstone power level at a position
    pub fn get_redstone_power(&self, x: i32, y: i32, z: i32) -> u8 {
        self.world.get_redstone_power(BlockPos::new(x, y, z))
    }

    /// Sets the signal strength at a specific block position (for custom IO nodes)
    #[wasm_bindgen(js_name = setSignalStrength)]
    pub fn set_signal_strength(&mut self, x: i32, y: i32, z: i32, strength: u8) {
        self.world
            .set_signal_strength(BlockPos::new(x, y, z), strength);
    }

    /// Gets the signal strength at a specific block position (for custom IO nodes)
    #[wasm_bindgen(js_name = getSignalStrength)]
    pub fn get_signal_strength(&self, x: i32, y: i32, z: i32) -> u8 {
        self.world.get_signal_strength(BlockPos::new(x, y, z))
    }

    /// Check for custom IO state changes and queue them
    /// Call this after tick() or setSignalStrength() to detect changes
    #[wasm_bindgen(js_name = checkCustomIoChanges)]
    pub fn check_custom_io_changes(&mut self) {
        self.world.check_custom_io_changes();
    }

    /// Get and clear all custom IO changes since last poll
    /// Returns an array of change objects with {x, y, z, oldPower, newPower}
    #[wasm_bindgen(js_name = pollCustomIoChanges)]
    pub fn poll_custom_io_changes(&mut self) -> JsValue {
        let changes = self.world.poll_custom_io_changes();
        let array = js_sys::Array::new();

        for change in changes {
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("x"),
                &JsValue::from_f64(change.x as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("y"),
                &JsValue::from_f64(change.y as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("z"),
                &JsValue::from_f64(change.z as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("oldPower"),
                &JsValue::from_f64(change.old_power as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("newPower"),
                &JsValue::from_f64(change.new_power as f64),
            )
            .unwrap();
            array.push(&obj);
        }

        array.into()
    }

    /// Get custom IO changes without clearing the queue
    #[wasm_bindgen(js_name = peekCustomIoChanges)]
    pub fn peek_custom_io_changes(&self) -> JsValue {
        let changes = self.world.peek_custom_io_changes();
        let array = js_sys::Array::new();

        for change in changes {
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("x"),
                &JsValue::from_f64(change.x as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("y"),
                &JsValue::from_f64(change.y as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("z"),
                &JsValue::from_f64(change.z as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("oldPower"),
                &JsValue::from_f64(change.old_power as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("newPower"),
                &JsValue::from_f64(change.new_power as f64),
            )
            .unwrap();
            array.push(&obj);
        }

        array.into()
    }

    /// Clear all queued custom IO changes
    #[wasm_bindgen(js_name = clearCustomIoChanges)]
    pub fn clear_custom_io_changes(&mut self) {
        self.world.clear_custom_io_changes();
    }

    /// Generates a truth table for the circuit
    ///
    /// Returns an array of objects with keys like "Input 0", "Output 0", etc.
    pub fn get_truth_table(&self) -> JsValue {
        let truth_table = generate_truth_table(&self.world.schematic);

        // Create a JavaScript array to hold the results
        let result = js_sys::Array::new();

        // Convert each row in the truth table to a JavaScript object
        for row in truth_table {
            let row_obj = js_sys::Object::new();

            // Add each entry in the row to the object
            for (key, value) in row {
                js_sys::Reflect::set(
                    &row_obj,
                    &JsValue::from_str(&key),
                    &JsValue::from_bool(value),
                )
                .unwrap();
            }

            result.push(&row_obj);
        }

        result.into()
    }

    /// Syncs the current simulation state back to the underlying schematic
    ///
    /// Call this after running simulation to update block states (redstone power, lever states, etc.)
    pub fn sync_to_schematic(&mut self) {
        self.world.sync_to_schematic();
    }

    /// Gets a copy of the underlying schematic
    ///
    /// Note: Call sync_to_schematic() first if you want the latest simulation state
    pub fn get_schematic(&self) -> SchematicWrapper {
        SchematicWrapper(self.world.get_schematic().clone())
    }

    /// Consumes the simulation world and returns the schematic with simulation state
    ///
    /// This automatically syncs before returning
    pub fn into_schematic(mut self) -> SchematicWrapper {
        SchematicWrapper(self.world.into_schematic())
    }
}
