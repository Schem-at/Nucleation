// src/wasm.rs

use crate::bounding_box::BoundingBox;
use crate::definition_region::DefinitionRegion;
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
pub struct DefinitionRegionWrapper {
    inner: DefinitionRegion,
}

#[wasm_bindgen]
impl DefinitionRegionWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: DefinitionRegion::new(),
        }
    }

    #[wasm_bindgen(js_name = addBounds)]
    pub fn add_bounds(&mut self, min: BlockPosition, max: BlockPosition) {
        self.inner
            .add_bounds((min.x, min.y, min.z), (max.x, max.y, max.z));
    }

    #[wasm_bindgen(js_name = fromBounds)]
    pub fn from_bounds(min: BlockPosition, max: BlockPosition) -> Self {
        Self {
            inner: DefinitionRegion::from_bounds((min.x, min.y, min.z), (max.x, max.y, max.z)),
        }
    }

    #[wasm_bindgen(js_name = setMetadata)]
    pub fn set_metadata(mut self, key: String, value: String) -> Self {
        self.inner = self.inner.with_metadata(key, value);
        self
    }

    #[wasm_bindgen(js_name = addPoint)]
    pub fn add_point(&mut self, x: i32, y: i32, z: i32) {
        self.inner.add_point(x, y, z);
    }

    #[wasm_bindgen(js_name = merge)]
    pub fn merge(&mut self, other: &DefinitionRegionWrapper) {
        self.inner.merge(&other.inner);
    }

    #[wasm_bindgen(js_name = filterByBlock)]
    pub fn filter_by_block(
        &self,
        schematic: &SchematicWrapper,
        block_name: String,
    ) -> DefinitionRegionWrapper {
        DefinitionRegionWrapper {
            inner: self.inner.filter_by_block(&schematic.0, &block_name),
        }
    }

    // ========================================================================
    // Boolean Operations
    // ========================================================================

    /// Subtract another region from this one (removes points present in `other`)
    #[wasm_bindgen(js_name = subtract)]
    pub fn subtract(&mut self, other: &DefinitionRegionWrapper) {
        self.inner.subtract(&other.inner);
    }

    /// Keep only points present in both regions (intersection)
    #[wasm_bindgen(js_name = intersect)]
    pub fn intersect(&mut self, other: &DefinitionRegionWrapper) {
        self.inner.intersect(&other.inner);
    }

    /// Create a new region that is the union of this region and another
    #[wasm_bindgen(js_name = union)]
    pub fn union(&self, other: &DefinitionRegionWrapper) -> DefinitionRegionWrapper {
        DefinitionRegionWrapper {
            inner: self.inner.union(&other.inner),
        }
    }

    // ========================================================================
    // Geometric Transformations
    // ========================================================================

    /// Translate all boxes by the given offset
    #[wasm_bindgen(js_name = shift)]
    pub fn shift(&mut self, x: i32, y: i32, z: i32) {
        self.inner.shift(x, y, z);
    }

    /// Expand all boxes by the given amounts in each direction
    #[wasm_bindgen(js_name = expand)]
    pub fn expand(&mut self, x: i32, y: i32, z: i32) {
        self.inner.expand(x, y, z);
    }

    /// Contract all boxes by the given amount uniformly
    #[wasm_bindgen(js_name = contract)]
    pub fn contract(&mut self, amount: i32) {
        self.inner.contract(amount);
    }

    /// Get the overall bounding box encompassing all boxes in this region
    /// Returns an object with {min: [x,y,z], max: [x,y,z]} or null if empty
    #[wasm_bindgen(js_name = getBounds)]
    pub fn get_bounds(&self) -> JsValue {
        match self.inner.get_bounds() {
            Some(bbox) => {
                let obj = Object::new();
                let min = Array::new();
                min.push(&JsValue::from(bbox.min.0));
                min.push(&JsValue::from(bbox.min.1));
                min.push(&JsValue::from(bbox.min.2));
                let max = Array::new();
                max.push(&JsValue::from(bbox.max.0));
                max.push(&JsValue::from(bbox.max.1));
                max.push(&JsValue::from(bbox.max.2));
                Reflect::set(&obj, &"min".into(), &min).unwrap();
                Reflect::set(&obj, &"max".into(), &max).unwrap();
                obj.into()
            }
            None => JsValue::NULL,
        }
    }

    // ========================================================================
    // Connectivity Analysis
    // ========================================================================

    /// Check if all points in the region are connected (6-connectivity)
    #[wasm_bindgen(js_name = isContiguous)]
    pub fn is_contiguous(&self) -> bool {
        self.inner.is_contiguous()
    }

    /// Get the number of connected components in this region
    #[wasm_bindgen(js_name = connectedComponents)]
    pub fn connected_components(&self) -> usize {
        self.inner.connected_components()
    }

    // ========================================================================
    // Filtering
    // ========================================================================

    /// Filter positions by block state properties (JS object)
    /// Only keeps positions where the block has ALL specified properties matching
    #[wasm_bindgen(js_name = filterByProperties)]
    pub fn filter_by_properties(
        &self,
        schematic: &SchematicWrapper,
        properties: &JsValue,
    ) -> Result<DefinitionRegionWrapper, JsValue> {
        let mut props = std::collections::HashMap::new();

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

        Ok(DefinitionRegionWrapper {
            inner: self.inner.filter_by_properties(&schematic.0, &props),
        })
    }

    // ========================================================================
    // Utility Methods
    // ========================================================================

    /// Check if the region is empty
    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Check if the region contains a specific point
    #[wasm_bindgen(js_name = contains)]
    pub fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        self.inner.contains(x, y, z)
    }

    /// Get total volume (number of blocks) covered by all boxes
    #[wasm_bindgen(js_name = volume)]
    pub fn volume(&self) -> u32 {
        self.inner.volume() as u32
    }

    /// Get a list of all positions as an array of [x, y, z] arrays
    #[wasm_bindgen(js_name = positions)]
    pub fn positions(&self) -> Array {
        let array = Array::new();
        for (x, y, z) in self.inner.iter_positions() {
            let pos = Array::new();
            pos.push(&JsValue::from(x));
            pos.push(&JsValue::from(y));
            pos.push(&JsValue::from(z));
            array.push(&pos);
        }
        array
    }

    /// Get positions in globally sorted order (Y, then X, then Z)
    ///
    /// This provides **deterministic bit ordering** for circuits regardless of
    /// how the region was constructed. Use this for IO bit assignment.
    #[wasm_bindgen(js_name = positionsSorted)]
    pub fn positions_sorted(&self) -> Array {
        let array = Array::new();
        for (x, y, z) in self.inner.iter_positions_sorted() {
            let pos = Array::new();
            pos.push(&JsValue::from(x));
            pos.push(&JsValue::from(y));
            pos.push(&JsValue::from(z));
            array.push(&pos);
        }
        array
    }

    // ========================================================================
    // Boolean Operations (Immutable variants)
    // ========================================================================

    /// Create a new region with points from `other` removed (immutable)
    #[wasm_bindgen(js_name = subtracted)]
    pub fn subtracted(&self, other: &DefinitionRegionWrapper) -> DefinitionRegionWrapper {
        DefinitionRegionWrapper {
            inner: self.inner.subtracted(&other.inner),
        }
    }

    /// Create a new region with only points in both (immutable)
    #[wasm_bindgen(js_name = intersected)]
    pub fn intersected(&self, other: &DefinitionRegionWrapper) -> DefinitionRegionWrapper {
        DefinitionRegionWrapper {
            inner: self.inner.intersected(&other.inner),
        }
    }

    /// Add all points from another region to this one (mutating union)
    #[wasm_bindgen(js_name = unionInto)]
    pub fn union_into(&mut self, other: &DefinitionRegionWrapper) {
        self.inner.union_into(&other.inner);
    }

    /// Simplify the region by merging adjacent/overlapping boxes
    #[wasm_bindgen(js_name = simplify)]
    pub fn simplify(&mut self) {
        self.inner.simplify();
    }
}

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

// =============================================================================
// TYPED CIRCUIT EXECUTOR BINDINGS
// =============================================================================

#[cfg(feature = "simulation")]
use crate::simulation::typed_executor::{
    ExecutionMode, ExecutionResult, IoLayout, IoLayoutBuilder, IoMapping, IoType, LayoutFunction,
    OutputCondition, StateMode, TypedCircuitExecutor, Value,
};

/// JavaScript-compatible Value wrapper
#[cfg(feature = "simulation")]
#[wasm_bindgen]
#[derive(Clone)]
pub struct ValueWrapper {
    inner: Value,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl ValueWrapper {
    /// Create a U32 value
    #[wasm_bindgen(js_name = fromU32)]
    pub fn from_u32(value: u32) -> Self {
        Self {
            inner: Value::U32(value),
        }
    }

    /// Create an I32 value
    #[wasm_bindgen(js_name = fromI32)]
    pub fn from_i32(value: i32) -> Self {
        Self {
            inner: Value::I32(value),
        }
    }

    /// Create an F32 value
    #[wasm_bindgen(js_name = fromF32)]
    pub fn from_f32(value: f32) -> Self {
        Self {
            inner: Value::F32(value),
        }
    }

    /// Create a Bool value
    #[wasm_bindgen(js_name = fromBool)]
    pub fn from_bool(value: bool) -> Self {
        Self {
            inner: Value::Bool(value),
        }
    }

    /// Create a String value
    #[wasm_bindgen(js_name = fromString)]
    pub fn from_string(value: String) -> Self {
        Self {
            inner: Value::String(value),
        }
    }

    /// Convert to JavaScript value
    #[wasm_bindgen(js_name = toJs)]
    pub fn to_js(&self) -> JsValue {
        match &self.inner {
            Value::U32(v) => JsValue::from_f64(*v as f64),
            Value::I32(v) => JsValue::from_f64(*v as f64),
            Value::U64(v) => JsValue::from_f64(*v as f64),
            Value::I64(v) => JsValue::from_f64(*v as f64),
            Value::F32(v) => JsValue::from_f64(*v as f64),
            Value::Bool(v) => JsValue::from_bool(*v),
            Value::String(v) => JsValue::from_str(v),
            Value::Array(_) => JsValue::from_str("[Array]"),
            Value::Struct(_) => JsValue::from_str("[Struct]"),
            Value::BitArray(_) => JsValue::from_str("[BitArray]"),
            Value::Bytes(_) => JsValue::from_str("[Bytes]"),
        }
    }

    /// Get type name
    #[wasm_bindgen(js_name = typeName)]
    pub fn type_name(&self) -> String {
        match &self.inner {
            Value::U32(_) => "U32".to_string(),
            Value::I32(_) => "I32".to_string(),
            Value::U64(_) => "U64".to_string(),
            Value::I64(_) => "I64".to_string(),
            Value::F32(_) => "F32".to_string(),
            Value::Bool(_) => "Bool".to_string(),
            Value::String(_) => "String".to_string(),
            Value::Array(_) => "Array".to_string(),
            Value::Struct(_) => "Struct".to_string(),
            Value::BitArray(_) => "BitArray".to_string(),
            Value::Bytes(_) => "Bytes".to_string(),
        }
    }
}

/// IoType builder for JavaScript
#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct IoTypeWrapper {
    inner: IoType,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl IoTypeWrapper {
    /// Create an unsigned integer type
    #[wasm_bindgen(js_name = unsignedInt)]
    pub fn unsigned_int(bits: usize) -> Self {
        Self {
            inner: IoType::UnsignedInt { bits },
        }
    }

    /// Create a signed integer type
    #[wasm_bindgen(js_name = signedInt)]
    pub fn signed_int(bits: usize) -> Self {
        Self {
            inner: IoType::SignedInt { bits },
        }
    }

    /// Create a Float32 type
    #[wasm_bindgen(js_name = float32)]
    pub fn float32() -> Self {
        Self {
            inner: IoType::Float32,
        }
    }

    /// Create a Boolean type
    #[wasm_bindgen(js_name = boolean)]
    pub fn boolean() -> Self {
        Self {
            inner: IoType::Boolean,
        }
    }

    /// Create an ASCII string type
    #[wasm_bindgen(js_name = ascii)]
    pub fn ascii(chars: usize) -> Self {
        Self {
            inner: IoType::Ascii { chars },
        }
    }
}

/// LayoutFunction builder for JavaScript
#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct LayoutFunctionWrapper {
    inner: LayoutFunction,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl LayoutFunctionWrapper {
    /// One bit per position (0 or 15)
    #[wasm_bindgen(js_name = oneToOne)]
    pub fn one_to_one() -> Self {
        Self {
            inner: LayoutFunction::OneToOne,
        }
    }

    /// Four bits per position (0-15)
    #[wasm_bindgen(js_name = packed4)]
    pub fn packed4() -> Self {
        Self {
            inner: LayoutFunction::Packed4,
        }
    }

    /// Custom bit-to-position mapping
    #[wasm_bindgen(js_name = custom)]
    pub fn custom(mapping: Vec<usize>) -> Self {
        Self {
            inner: LayoutFunction::Custom(mapping),
        }
    }

    /// Row-major 2D layout
    #[wasm_bindgen(js_name = rowMajor)]
    pub fn row_major(rows: usize, cols: usize, bits_per_element: usize) -> Self {
        Self {
            inner: LayoutFunction::RowMajor {
                rows,
                cols,
                bits_per_element,
            },
        }
    }

    /// Column-major 2D layout
    #[wasm_bindgen(js_name = columnMajor)]
    pub fn column_major(rows: usize, cols: usize, bits_per_element: usize) -> Self {
        Self {
            inner: LayoutFunction::ColumnMajor {
                rows,
                cols,
                bits_per_element,
            },
        }
    }

    /// Scanline layout for screens
    #[wasm_bindgen(js_name = scanline)]
    pub fn scanline(width: usize, height: usize, bits_per_pixel: usize) -> Self {
        Self {
            inner: LayoutFunction::Scanline {
                width,
                height,
                bits_per_pixel,
            },
        }
    }
}

/// OutputCondition for conditional execution
#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct OutputConditionWrapper {
    inner: OutputCondition,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl OutputConditionWrapper {
    /// Output equals a value
    #[wasm_bindgen(js_name = equals)]
    pub fn equals(value: &ValueWrapper) -> Self {
        Self {
            inner: OutputCondition::Equals(value.inner.clone()),
        }
    }

    /// Output not equals a value
    #[wasm_bindgen(js_name = notEquals)]
    pub fn not_equals(value: &ValueWrapper) -> Self {
        Self {
            inner: OutputCondition::NotEquals(value.inner.clone()),
        }
    }

    /// Output greater than a value
    #[wasm_bindgen(js_name = greaterThan)]
    pub fn greater_than(value: &ValueWrapper) -> Self {
        Self {
            inner: OutputCondition::GreaterThan(value.inner.clone()),
        }
    }

    /// Output less than a value
    #[wasm_bindgen(js_name = lessThan)]
    pub fn less_than(value: &ValueWrapper) -> Self {
        Self {
            inner: OutputCondition::LessThan(value.inner.clone()),
        }
    }

    /// Bitwise AND with mask
    #[wasm_bindgen(js_name = bitwiseAnd)]
    pub fn bitwise_and(mask: u32) -> Self {
        Self {
            inner: OutputCondition::BitwiseAnd(mask as u64),
        }
    }
}

/// ExecutionMode for circuit execution
#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct ExecutionModeWrapper {
    inner: ExecutionMode,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl ExecutionModeWrapper {
    /// Run for a fixed number of ticks
    #[wasm_bindgen(js_name = fixedTicks)]
    pub fn fixed_ticks(ticks: u32) -> Self {
        Self {
            inner: ExecutionMode::FixedTicks { ticks },
        }
    }

    /// Run until an output meets a condition
    #[wasm_bindgen(js_name = untilCondition)]
    pub fn until_condition(
        output_name: String,
        condition: &OutputConditionWrapper,
        max_ticks: u32,
        check_interval: u32,
    ) -> Self {
        Self {
            inner: ExecutionMode::UntilCondition {
                output_name,
                condition: condition.inner.clone(),
                max_ticks,
                check_interval,
            },
        }
    }

    /// Run until any output changes
    #[wasm_bindgen(js_name = untilChange)]
    pub fn until_change(max_ticks: u32, check_interval: u32) -> Self {
        Self {
            inner: ExecutionMode::UntilChange {
                max_ticks,
                check_interval,
            },
        }
    }

    /// Run until outputs are stable
    #[wasm_bindgen(js_name = untilStable)]
    pub fn until_stable(stable_ticks: u32, max_ticks: u32) -> Self {
        Self {
            inner: ExecutionMode::UntilStable {
                stable_ticks,
                max_ticks,
            },
        }
    }
}

/// IoLayoutBuilder for JavaScript
#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct IoLayoutBuilderWrapper {
    inner: IoLayoutBuilder,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl IoLayoutBuilderWrapper {
    /// Create a new IO layout builder
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: IoLayoutBuilder::new(),
        }
    }

    /// Add an input
    #[wasm_bindgen(js_name = addInput)]
    pub fn add_input(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        positions: Vec<JsValue>,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        // Convert positions from JsValue array to Vec<(i32, i32, i32)>
        let mut pos_vec = Vec::new();
        for pos in positions {
            let array = js_sys::Array::from(&pos);
            if array.length() != 3 {
                return Err(JsValue::from_str("Position must be [x, y, z]"));
            }
            let x = array.get(0).as_f64().ok_or("Invalid x")? as i32;
            let y = array.get(1).as_f64().ok_or("Invalid y")? as i32;
            let z = array.get(2).as_f64().ok_or("Invalid z")? as i32;
            pos_vec.push((x, y, z));
        }

        self.inner = self
            .inner
            .add_input(name, io_type.inner.clone(), layout.inner.clone(), pos_vec)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(self)
    }

    /// Add an output
    #[wasm_bindgen(js_name = addOutput)]
    pub fn add_output(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        positions: Vec<JsValue>,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        // Convert positions
        let mut pos_vec = Vec::new();
        for pos in positions {
            let array = js_sys::Array::from(&pos);
            if array.length() != 3 {
                return Err(JsValue::from_str("Position must be [x, y, z]"));
            }
            let x = array.get(0).as_f64().ok_or("Invalid x")? as i32;
            let y = array.get(1).as_f64().ok_or("Invalid y")? as i32;
            let z = array.get(2).as_f64().ok_or("Invalid z")? as i32;
            pos_vec.push((x, y, z));
        }

        self.inner = self
            .inner
            .add_output(name, io_type.inner.clone(), layout.inner.clone(), pos_vec)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(self)
    }

    /// Add an input with automatic layout inference
    #[wasm_bindgen(js_name = addInputAuto)]
    pub fn add_input_auto(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        positions: Vec<JsValue>,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        // Convert positions
        let mut pos_vec = Vec::new();
        for pos in positions {
            let array = js_sys::Array::from(&pos);
            if array.length() != 3 {
                return Err(JsValue::from_str("Position must be [x, y, z]"));
            }
            let x = array.get(0).as_f64().ok_or("Invalid x")? as i32;
            let y = array.get(1).as_f64().ok_or("Invalid y")? as i32;
            let z = array.get(2).as_f64().ok_or("Invalid z")? as i32;
            pos_vec.push((x, y, z));
        }

        self.inner = self
            .inner
            .add_input_auto(name, io_type.inner.clone(), pos_vec)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(self)
    }

    /// Add an input defined by a region (bounding box)
    /// Iterates Y (layers), then X (rows), then Z (columns)
    #[wasm_bindgen(js_name = addInputRegion)]
    pub fn add_input_region(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        min: BlockPosition,
        max: BlockPosition,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        let mut positions = Vec::new();

        let min_x = std::cmp::min(min.x, max.x);
        let max_x = std::cmp::max(min.x, max.x);
        let min_y = std::cmp::min(min.y, max.y);
        let max_y = std::cmp::max(min.y, max.y);
        let min_z = std::cmp::min(min.z, max.z);
        let max_z = std::cmp::max(min.z, max.z);

        // Standard redstone order: Y-axis first (layers), then X (rows), then Z (columns)
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                for z in min_z..=max_z {
                    positions.push((x, y, z));
                }
            }
        }

        self.inner = self
            .inner
            .add_input(name, io_type.inner.clone(), layout.inner.clone(), positions)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(self)
    }

    /// Add an input defined by a DefinitionRegion
    #[wasm_bindgen(js_name = addInputFromRegion)]
    pub fn add_input_from_region(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        region: &DefinitionRegionWrapper,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .add_input_from_region(
                name,
                io_type.inner.clone(),
                layout.inner.clone(),
                region.inner.clone(),
            )
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Add an input defined by a region with automatic layout inference
    #[wasm_bindgen(js_name = addInputRegionAuto)]
    pub fn add_input_region_auto(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        min: BlockPosition,
        max: BlockPosition,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        let mut positions = Vec::new();

        let min_x = std::cmp::min(min.x, max.x);
        let max_x = std::cmp::max(min.x, max.x);
        let min_y = std::cmp::min(min.y, max.y);
        let max_y = std::cmp::max(min.y, max.y);
        let min_z = std::cmp::min(min.z, max.z);
        let max_z = std::cmp::max(min.z, max.z);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                for z in min_z..=max_z {
                    positions.push((x, y, z));
                }
            }
        }

        self.inner = self
            .inner
            .add_input_auto(name, io_type.inner.clone(), positions)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(self)
    }

    /// Add an input defined by a DefinitionRegion with automatic layout inference
    #[wasm_bindgen(js_name = addInputFromRegionAuto)]
    pub fn add_input_from_region_auto(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        region: &DefinitionRegionWrapper,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .add_input_from_region_auto(name, io_type.inner.clone(), region.inner.clone())
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Add an output with automatic layout inference
    #[wasm_bindgen(js_name = addOutputAuto)]
    pub fn add_output_auto(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        positions: Vec<JsValue>,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        // Convert positions
        let mut pos_vec = Vec::new();
        for pos in positions {
            let array = js_sys::Array::from(&pos);
            if array.length() != 3 {
                return Err(JsValue::from_str("Position must be [x, y, z]"));
            }
            let x = array.get(0).as_f64().ok_or("Invalid x")? as i32;
            let y = array.get(1).as_f64().ok_or("Invalid y")? as i32;
            let z = array.get(2).as_f64().ok_or("Invalid z")? as i32;
            pos_vec.push((x, y, z));
        }

        self.inner = self
            .inner
            .add_output_auto(name, io_type.inner.clone(), pos_vec)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(self)
    }

    /// Add an output defined by a region (bounding box)
    /// Iterates Y (layers), then X (rows), then Z (columns)
    #[wasm_bindgen(js_name = addOutputRegion)]
    pub fn add_output_region(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        min: BlockPosition,
        max: BlockPosition,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        let mut positions = Vec::new();

        let min_x = std::cmp::min(min.x, max.x);
        let max_x = std::cmp::max(min.x, max.x);
        let min_y = std::cmp::min(min.y, max.y);
        let max_y = std::cmp::max(min.y, max.y);
        let min_z = std::cmp::min(min.z, max.z);
        let max_z = std::cmp::max(min.z, max.z);

        // Standard redstone order: Y-axis first (layers), then X (rows), then Z (columns)
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                for z in min_z..=max_z {
                    positions.push((x, y, z));
                }
            }
        }

        self.inner = self
            .inner
            .add_output(name, io_type.inner.clone(), layout.inner.clone(), positions)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(self)
    }

    /// Add an output defined by a DefinitionRegion
    #[wasm_bindgen(js_name = addOutputFromRegion)]
    pub fn add_output_from_region(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        region: &DefinitionRegionWrapper,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .add_output_from_region(
                name,
                io_type.inner.clone(),
                layout.inner.clone(),
                region.inner.clone(),
            )
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Add an output defined by a region with automatic layout inference
    #[wasm_bindgen(js_name = addOutputRegionAuto)]
    pub fn add_output_region_auto(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        min: BlockPosition,
        max: BlockPosition,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        let mut positions = Vec::new();

        let min_x = std::cmp::min(min.x, max.x);
        let max_x = std::cmp::max(min.x, max.x);
        let min_y = std::cmp::min(min.y, max.y);
        let max_y = std::cmp::max(min.y, max.y);
        let min_z = std::cmp::min(min.z, max.z);
        let max_z = std::cmp::max(min.z, max.z);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                for z in min_z..=max_z {
                    positions.push((x, y, z));
                }
            }
        }

        self.inner = self
            .inner
            .add_output_auto(name, io_type.inner.clone(), positions)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(self)
    }

    /// Add an output defined by a DefinitionRegion with automatic layout inference
    #[wasm_bindgen(js_name = addOutputFromRegionAuto)]
    pub fn add_output_from_region_auto(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        region: &DefinitionRegionWrapper,
    ) -> Result<IoLayoutBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .add_output_from_region_auto(name, io_type.inner.clone(), region.inner.clone())
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Build the IO layout
    pub fn build(self) -> IoLayoutWrapper {
        IoLayoutWrapper {
            inner: self.inner.build(),
        }
    }
}

/// IoLayout wrapper for JavaScript
#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct IoLayoutWrapper {
    inner: IoLayout,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl IoLayoutWrapper {
    /// Get input names
    #[wasm_bindgen(js_name = inputNames)]
    pub fn input_names(&self) -> Vec<String> {
        self.inner
            .input_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get output names
    #[wasm_bindgen(js_name = outputNames)]
    pub fn output_names(&self) -> Vec<String> {
        self.inner
            .output_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }
}

/// TypedCircuitExecutor wrapper for JavaScript
#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct TypedCircuitExecutorWrapper {
    inner: TypedCircuitExecutor,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl TypedCircuitExecutorWrapper {
    /// Create executor from world and layout
    #[wasm_bindgen(js_name = fromLayout)]
    pub fn from_layout(
        world: MchprsWorldWrapper,
        layout: IoLayoutWrapper,
    ) -> Result<TypedCircuitExecutorWrapper, JsValue> {
        Ok(Self {
            inner: TypedCircuitExecutor::from_layout(world.world, layout.inner),
        })
    }

    /// Create executor from world, layout, and options
    #[wasm_bindgen(js_name = fromLayoutWithOptions)]
    pub fn from_layout_with_options(
        world: MchprsWorldWrapper,
        layout: IoLayoutWrapper,
        options: &SimulationOptionsWrapper,
    ) -> Result<TypedCircuitExecutorWrapper, JsValue> {
        Ok(Self {
            inner: TypedCircuitExecutor::from_layout_with_options(
                world.world,
                layout.inner,
                options.inner.clone(),
            ),
        })
    }

    /// Create executor from Insign annotations in schematic
    #[wasm_bindgen(js_name = fromInsign)]
    pub fn from_insign(
        schematic: &SchematicWrapper,
    ) -> Result<TypedCircuitExecutorWrapper, JsValue> {
        use crate::simulation::typed_executor::create_executor_from_insign;

        let executor = create_executor_from_insign(&schematic.0).map_err(|e| {
            JsValue::from_str(&format!("Failed to create executor from Insign: {}", e))
        })?;

        Ok(Self { inner: executor })
    }

    /// Create executor from Insign annotations with custom simulation options
    #[wasm_bindgen(js_name = fromInsignWithOptions)]
    pub fn from_insign_with_options(
        schematic: &SchematicWrapper,
        options: &SimulationOptionsWrapper,
    ) -> Result<TypedCircuitExecutorWrapper, JsValue> {
        use crate::simulation::typed_executor::create_executor_from_insign_with_options;

        let executor =
            create_executor_from_insign_with_options(&schematic.0, options.inner.clone()).map_err(
                |e| JsValue::from_str(&format!("Failed to create executor from Insign: {}", e)),
            )?;

        Ok(Self { inner: executor })
    }

    /// Set state mode
    #[wasm_bindgen(js_name = setStateMode)]
    pub fn set_state_mode(&mut self, mode: &str) -> Result<(), JsValue> {
        let state_mode = match mode {
            "stateless" => StateMode::Stateless,
            "stateful" => StateMode::Stateful,
            "manual" => StateMode::Manual,
            _ => {
                return Err(JsValue::from_str(
                    "Invalid state mode. Use 'stateless', 'stateful', or 'manual'",
                ))
            }
        };
        self.inner.set_state_mode(state_mode);
        Ok(())
    }

    /// Reset the simulation
    pub fn reset(&mut self) -> Result<(), JsValue> {
        self.inner.reset().map_err(|e| JsValue::from_str(&e))
    }

    /// Execute the circuit
    pub fn execute(
        &mut self,
        inputs: JsValue,
        mode: &ExecutionModeWrapper,
    ) -> Result<JsValue, JsValue> {
        // Convert inputs from JS object to HashMap<String, Value>
        let mut input_map = std::collections::HashMap::new();
        let obj = js_sys::Object::from(inputs);
        let entries = js_sys::Object::entries(&obj);

        for i in 0..entries.length() {
            let entry = js_sys::Array::from(&entries.get(i));
            let key = entry.get(0).as_string().ok_or("Invalid input key")?;
            let value_js = entry.get(1);

            // Try to convert JsValue to Value
            let value = if let Some(b) = value_js.as_bool() {
                Value::Bool(b)
            } else if let Some(n) = value_js.as_f64() {
                // Assume integers for now
                Value::U32(n as u32)
            } else if let Some(s) = value_js.as_string() {
                Value::String(s)
            } else {
                return Err(JsValue::from_str("Unsupported input value type"));
            };

            input_map.insert(key, value);
        }

        // Execute
        let result = self
            .inner
            .execute(input_map, mode.inner.clone())
            .map_err(|e| JsValue::from_str(&e))?;

        // Convert result to JS object
        let result_obj = js_sys::Object::new();

        // Add outputs
        let outputs_obj = js_sys::Object::new();
        for (name, value) in result.outputs {
            let value_wrapper = ValueWrapper { inner: value };
            js_sys::Reflect::set(
                &outputs_obj,
                &JsValue::from_str(&name),
                &value_wrapper.to_js(),
            )
            .unwrap();
        }
        js_sys::Reflect::set(&result_obj, &JsValue::from_str("outputs"), &outputs_obj).unwrap();

        // Add ticks_elapsed
        js_sys::Reflect::set(
            &result_obj,
            &JsValue::from_str("ticksElapsed"),
            &JsValue::from_f64(result.ticks_elapsed as f64),
        )
        .unwrap();

        // Add condition_met
        js_sys::Reflect::set(
            &result_obj,
            &JsValue::from_str("conditionMet"),
            &JsValue::from_bool(result.condition_met),
        )
        .unwrap();

        Ok(result_obj.into())
    }

    /// Sync the simulation state back to the schematic
    ///
    /// Call this after execute() to update the schematic with the current simulation state.
    /// Returns the updated schematic.
    #[wasm_bindgen(js_name = syncToSchematic)]
    pub fn sync_to_schematic(&mut self) -> SchematicWrapper {
        let schematic = self.inner.sync_and_get_schematic();
        SchematicWrapper(schematic.clone())
    }

    /// Manually advance the simulation by a specified number of ticks
    ///
    /// This is useful for manual state control when using 'manual' state mode.
    /// Unlike execute(), this does not set any inputs or read outputs.
    #[wasm_bindgen(js_name = tick)]
    pub fn tick(&mut self, ticks: u32) {
        self.inner.tick(ticks);
    }

    /// Manually flush the simulation state
    ///
    /// Ensures all pending changes are propagated through the redstone network.
    #[wasm_bindgen(js_name = flush)]
    pub fn flush(&mut self) {
        self.inner.flush();
    }

    /// Set a single input value without executing
    #[wasm_bindgen(js_name = setInput)]
    pub fn set_input(&mut self, name: String, value: &ValueWrapper) -> Result<(), JsValue> {
        self.inner
            .set_input(&name, &value.inner)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Read a single output value without executing
    #[wasm_bindgen(js_name = readOutput)]
    pub fn read_output(&mut self, name: String) -> Result<ValueWrapper, JsValue> {
        let value = self
            .inner
            .read_output(&name)
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(ValueWrapper { inner: value })
    }

    /// Get all input names
    #[wasm_bindgen(js_name = inputNames)]
    pub fn input_names(&self) -> Vec<String> {
        self.inner
            .input_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get all output names
    #[wasm_bindgen(js_name = outputNames)]
    pub fn output_names(&self) -> Vec<String> {
        self.inner
            .output_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get detailed layout information for debugging and visualization
    ///
    /// Returns a JS object with the structure:
    /// ```javascript
    /// {
    ///   inputs: {
    ///     "name": {
    ///       ioType: "UnsignedInt { bits: 8 }",
    ///       positions: [[x, y, z], ...],  // In bit order (LSB first)
    ///       bitCount: 8
    ///     }
    ///   },
    ///   outputs: { ... }
    /// }
    /// ```
    #[wasm_bindgen(js_name = getLayoutInfo)]
    pub fn get_layout_info(&self) -> JsValue {
        let layout_info = self.inner.get_layout_info();

        let result = Object::new();
        let inputs_obj = Object::new();
        let outputs_obj = Object::new();

        // Convert inputs
        for (name, info) in &layout_info.inputs {
            let io_obj = Object::new();
            Reflect::set(&io_obj, &"ioType".into(), &JsValue::from_str(&info.io_type)).unwrap();
            Reflect::set(
                &io_obj,
                &"bitCount".into(),
                &JsValue::from(info.bit_count as u32),
            )
            .unwrap();

            let positions_arr = Array::new();
            for (x, y, z) in &info.positions {
                let pos = Array::new();
                pos.push(&JsValue::from(*x));
                pos.push(&JsValue::from(*y));
                pos.push(&JsValue::from(*z));
                positions_arr.push(&pos);
            }
            Reflect::set(&io_obj, &"positions".into(), &positions_arr).unwrap();
            Reflect::set(&inputs_obj, &name.into(), &io_obj).unwrap();
        }

        // Convert outputs
        for (name, info) in &layout_info.outputs {
            let io_obj = Object::new();
            Reflect::set(&io_obj, &"ioType".into(), &JsValue::from_str(&info.io_type)).unwrap();
            Reflect::set(
                &io_obj,
                &"bitCount".into(),
                &JsValue::from(info.bit_count as u32),
            )
            .unwrap();

            let positions_arr = Array::new();
            for (x, y, z) in &info.positions {
                let pos = Array::new();
                pos.push(&JsValue::from(*x));
                pos.push(&JsValue::from(*y));
                pos.push(&JsValue::from(*z));
                positions_arr.push(&pos);
            }
            Reflect::set(&io_obj, &"positions".into(), &positions_arr).unwrap();
            Reflect::set(&outputs_obj, &name.into(), &io_obj).unwrap();
        }

        Reflect::set(&result, &"inputs".into(), &inputs_obj).unwrap();
        Reflect::set(&result, &"outputs".into(), &outputs_obj).unwrap();

        result.into()
    }
}

// --- CircuitBuilder Support ---

#[cfg(feature = "simulation")]
use crate::simulation::CircuitBuilder;

/// CircuitBuilder wrapper for JavaScript
/// Provides a fluent API for creating TypedCircuitExecutor instances
#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct CircuitBuilderWrapper {
    inner: CircuitBuilder,
}

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl CircuitBuilderWrapper {
    /// Create a new CircuitBuilder from a schematic
    #[wasm_bindgen(constructor)]
    pub fn new(schematic: &SchematicWrapper) -> Self {
        Self {
            inner: CircuitBuilder::new(schematic.0.clone()),
        }
    }

    /// Create a CircuitBuilder pre-populated from Insign annotations
    #[wasm_bindgen(js_name = fromInsign)]
    pub fn from_insign(schematic: &SchematicWrapper) -> Result<CircuitBuilderWrapper, JsValue> {
        let builder =
            CircuitBuilder::from_insign(schematic.0.clone()).map_err(|e| JsValue::from_str(&e))?;
        Ok(Self { inner: builder })
    }

    /// Add an input with full control
    #[wasm_bindgen(js_name = withInput)]
    pub fn with_input(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        region: &DefinitionRegionWrapper,
    ) -> Result<CircuitBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .with_input(
                name,
                io_type.inner.clone(),
                layout.inner.clone(),
                region.inner.clone(),
            )
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Add an input with automatic layout inference
    #[wasm_bindgen(js_name = withInputAuto)]
    pub fn with_input_auto(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        region: &DefinitionRegionWrapper,
    ) -> Result<CircuitBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .with_input_auto(name, io_type.inner.clone(), region.inner.clone())
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Add an output with full control
    #[wasm_bindgen(js_name = withOutput)]
    pub fn with_output(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        layout: &LayoutFunctionWrapper,
        region: &DefinitionRegionWrapper,
    ) -> Result<CircuitBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .with_output(
                name,
                io_type.inner.clone(),
                layout.inner.clone(),
                region.inner.clone(),
            )
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Add an output with automatic layout inference
    #[wasm_bindgen(js_name = withOutputAuto)]
    pub fn with_output_auto(
        mut self,
        name: String,
        io_type: &IoTypeWrapper,
        region: &DefinitionRegionWrapper,
    ) -> Result<CircuitBuilderWrapper, JsValue> {
        self.inner = self
            .inner
            .with_output_auto(name, io_type.inner.clone(), region.inner.clone())
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self)
    }

    /// Set simulation options
    #[wasm_bindgen(js_name = withOptions)]
    pub fn with_options(mut self, options: &SimulationOptionsWrapper) -> Self {
        self.inner = self.inner.with_options(options.inner.clone());
        self
    }

    /// Set state mode: 'stateless', 'stateful', or 'manual'
    #[wasm_bindgen(js_name = withStateMode)]
    pub fn with_state_mode(mut self, mode: &str) -> Result<CircuitBuilderWrapper, JsValue> {
        let state_mode = match mode {
            "stateless" => StateMode::Stateless,
            "stateful" => StateMode::Stateful,
            "manual" => StateMode::Manual,
            _ => {
                return Err(JsValue::from_str(
                    "Invalid state mode. Use 'stateless', 'stateful', or 'manual'",
                ))
            }
        };
        self.inner = self.inner.with_state_mode(state_mode);
        Ok(self)
    }

    /// Validate the circuit configuration
    #[wasm_bindgen(js_name = validate)]
    pub fn validate(&self) -> Result<(), JsValue> {
        self.inner
            .validate()
            .map(|_| ())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Build the TypedCircuitExecutor
    #[wasm_bindgen(js_name = build)]
    pub fn build(self) -> Result<TypedCircuitExecutorWrapper, JsValue> {
        let executor = self.inner.build().map_err(|e| JsValue::from_str(&e))?;
        Ok(TypedCircuitExecutorWrapper { inner: executor })
    }

    /// Build with validation (convenience method)
    #[wasm_bindgen(js_name = buildValidated)]
    pub fn build_validated(self) -> Result<TypedCircuitExecutorWrapper, JsValue> {
        let executor = self
            .inner
            .build_validated()
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(TypedCircuitExecutorWrapper { inner: executor })
    }

    /// Get the current number of inputs
    #[wasm_bindgen(js_name = inputCount)]
    pub fn input_count(&self) -> usize {
        self.inner.input_count()
    }

    /// Get the current number of outputs
    #[wasm_bindgen(js_name = outputCount)]
    pub fn output_count(&self) -> usize {
        self.inner.output_count()
    }

    /// Get the names of defined inputs
    #[wasm_bindgen(js_name = inputNames)]
    pub fn input_names(&self) -> Vec<String> {
        self.inner
            .input_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get the names of defined outputs
    #[wasm_bindgen(js_name = outputNames)]
    pub fn output_names(&self) -> Vec<String> {
        self.inner
            .output_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }
}

// --- State Mode Constants ---

/// State mode constants for JavaScript
#[cfg(feature = "simulation")]
#[wasm_bindgen]
pub struct StateModeConstants;

#[cfg(feature = "simulation")]
#[wasm_bindgen]
impl StateModeConstants {
    /// Always reset before execution (default)
    #[wasm_bindgen(getter = STATELESS)]
    pub fn stateless() -> String {
        "stateless".to_string()
    }

    /// Preserve state between executions
    #[wasm_bindgen(getter = STATEFUL)]
    pub fn stateful() -> String {
        "stateful".to_string()
    }

    /// Manual state control
    #[wasm_bindgen(getter = MANUAL)]
    pub fn manual() -> String {
        "manual".to_string()
    }
}

// --- SchematicBuilder Support ---

/// SchematicBuilder for creating schematics from ASCII art
#[wasm_bindgen]
pub struct SchematicBuilderWrapper {
    inner: crate::SchematicBuilder,
}

#[wasm_bindgen]
impl SchematicBuilderWrapper {
    /// Create a new schematic builder with standard palette
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: crate::SchematicBuilder::new(),
        }
    }

    /// Set the name of the schematic
    #[wasm_bindgen(js_name = name)]
    pub fn name(mut self, name: String) -> Self {
        self.inner = self.inner.name(name);
        self
    }

    /// Map a character to a block string
    #[wasm_bindgen(js_name = map)]
    pub fn map(mut self, ch: char, block: String) -> Self {
        self.inner = self.inner.map(ch, &block);
        self
    }

    /// Build the schematic
    #[wasm_bindgen(js_name = build)]
    pub fn build(self) -> Result<SchematicWrapper, JsValue> {
        let schematic = self.inner.build().map_err(|e| JsValue::from_str(&e))?;
        Ok(SchematicWrapper(schematic))
    }

    /// Create from template string
    #[wasm_bindgen(js_name = fromTemplate)]
    pub fn from_template(template: String) -> Result<SchematicBuilderWrapper, JsValue> {
        let builder =
            crate::SchematicBuilder::from_template(&template).map_err(|e| JsValue::from_str(&e))?;
        Ok(Self { inner: builder })
    }
}
