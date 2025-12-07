//! DefinitionRegion WASM bindings

use crate::block_position::BlockPosition;
use crate::definition_region::DefinitionRegion;
use crate::UniversalSchematic;
use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;

use super::SchematicWrapper;

#[wasm_bindgen]
pub struct DefinitionRegionWrapper {
    pub(crate) inner: DefinitionRegion,
    pub(crate) schematic_ptr: *mut UniversalSchematic,
    pub(crate) name: Option<String>,
}

#[wasm_bindgen]
impl DefinitionRegionWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: DefinitionRegion::new(),
            schematic_ptr: std::ptr::null_mut(),
            name: None,
        }
    }

    fn sync(&self) {
        if let Some(name) = &self.name {
            if !self.schematic_ptr.is_null() {
                unsafe {
                    (*self.schematic_ptr)
                        .definition_regions
                        .insert(name.clone(), self.inner.clone());
                }
            }
        }
    }

    #[wasm_bindgen(js_name = addBounds)]
    pub fn add_bounds(
        mut self,
        min: JsValue,
        max: JsValue,
    ) -> Result<DefinitionRegionWrapper, JsValue> {
        let min_x = Reflect::get(&min, &"x".into())?.as_f64().unwrap_or(0.0) as i32;
        let min_y = Reflect::get(&min, &"y".into())?.as_f64().unwrap_or(0.0) as i32;
        let min_z = Reflect::get(&min, &"z".into())?.as_f64().unwrap_or(0.0) as i32;

        let max_x = Reflect::get(&max, &"x".into())?.as_f64().unwrap_or(0.0) as i32;
        let max_y = Reflect::get(&max, &"y".into())?.as_f64().unwrap_or(0.0) as i32;
        let max_z = Reflect::get(&max, &"z".into())?.as_f64().unwrap_or(0.0) as i32;

        self.inner
            .add_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
        self.sync();
        Ok(self)
    }

    #[wasm_bindgen(js_name = fromBounds)]
    pub fn from_bounds(min: BlockPosition, max: BlockPosition) -> Self {
        Self {
            inner: DefinitionRegion::from_bounds((min.x, min.y, min.z), (max.x, max.y, max.z)),
            schematic_ptr: std::ptr::null_mut(),
            name: None,
        }
    }

    #[wasm_bindgen(js_name = setMetadata)]
    pub fn set_metadata(mut self, key: String, value: String) -> Self {
        self.inner.with_metadata(key, value);
        self.sync();
        self
    }

    #[wasm_bindgen(js_name = setColor)]
    pub fn set_color(mut self, color: u32) -> Self {
        let hex = format!("#{:06x}", color);
        self.inner.set_color(color);
        self.sync();
        self
    }

    #[wasm_bindgen(js_name = addFilter)]
    pub fn add_filter(mut self, filter: String) -> Result<DefinitionRegionWrapper, JsValue> {
        if !self.schematic_ptr.is_null() {
            unsafe {
                let sch = &*self.schematic_ptr;
                self.inner.filter_by_block(sch, &filter);
            }
            self.sync();
            Ok(self)
        } else {
            // Fallback for detached regions
            let current = self
                .inner
                .metadata
                .get("filter")
                .cloned()
                .unwrap_or_default();
            let new_val = if current.is_empty() {
                filter
            } else {
                format!("{},{}", current, filter)
            };
            self.inner.set_metadata("filter".to_string(), new_val);
            Ok(self)
        }
    }

    #[wasm_bindgen(js_name = excludeBlock)]
    pub fn exclude_block(mut self, block_name: String) -> Result<DefinitionRegionWrapper, JsValue> {
        if !self.schematic_ptr.is_null() {
            unsafe {
                let sch = &*self.schematic_ptr;
                self.inner.exclude_block(sch, &block_name);
            }
            self.sync();
            Ok(self)
        } else {
            Err(JsValue::from_str(
                "Cannot exclude block: Region is not attached to a schematic",
            ))
        }
    }

    #[wasm_bindgen(js_name = addPoint)]
    pub fn add_point(mut self, x: i32, y: i32, z: i32) -> Self {
        self.inner.add_point(x, y, z);
        self.sync();
        self
    }

    #[wasm_bindgen(js_name = merge)]
    pub fn merge(mut self, other: &DefinitionRegionWrapper) -> Self {
        self.inner.merge(&other.inner);
        self.sync();
        self
    }

    #[wasm_bindgen(js_name = filterByBlock)]
    pub fn filter_by_block(
        &self,
        schematic: &SchematicWrapper,
        block_name: String,
    ) -> DefinitionRegionWrapper {
        // Create a clone and filter it (since filter_by_block now mutates)
        let mut filtered = self.inner.clone();
        filtered.filter_by_block(&schematic.0, &block_name);
        DefinitionRegionWrapper {
            inner: filtered,
            schematic_ptr: std::ptr::null_mut(),
            name: None,
        }
    }

    // ========================================================================
    // Boolean Operations
    // ========================================================================

    /// Subtract another region from this one (removes points present in `other`)
    #[wasm_bindgen(js_name = subtract)]
    pub fn subtract(mut self, other: &DefinitionRegionWrapper) -> Self {
        self.inner.subtract(&other.inner);
        self.sync();
        self
    }

    /// Keep only points present in both regions (intersection)
    #[wasm_bindgen(js_name = intersect)]
    pub fn intersect(mut self, other: &DefinitionRegionWrapper) -> Self {
        self.inner.intersect(&other.inner);
        self.sync();
        self
    }

    /// Create a new region that is the union of this region and another
    #[wasm_bindgen(js_name = union)]
    pub fn union(&self, other: &DefinitionRegionWrapper) -> DefinitionRegionWrapper {
        DefinitionRegionWrapper {
            inner: self.inner.union(&other.inner),
            schematic_ptr: std::ptr::null_mut(),
            name: None,
        }
    }

    // ========================================================================
    // Geometric Transformations
    // ========================================================================

    /// Translate all boxes by the given offset
    #[wasm_bindgen(js_name = shift)]
    pub fn shift(mut self, x: i32, y: i32, z: i32) -> Self {
        self.inner.shift(x, y, z);
        self.sync();
        self
    }

    /// Expand all boxes by the given amounts in each direction
    #[wasm_bindgen(js_name = expand)]
    pub fn expand(mut self, x: i32, y: i32, z: i32) -> Self {
        self.inner.expand(x, y, z);
        self.sync();
        self
    }

    /// Contract all boxes by the given amount uniformly
    #[wasm_bindgen(js_name = contract)]
    pub fn contract(mut self, amount: i32) -> Self {
        self.inner.contract(amount);
        self.sync();
        self
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
            schematic_ptr: std::ptr::null_mut(),
            name: None,
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
            schematic_ptr: std::ptr::null_mut(),
            name: None,
        }
    }

    /// Create a new region with only points in both (immutable)
    #[wasm_bindgen(js_name = intersected)]
    pub fn intersected(&self, other: &DefinitionRegionWrapper) -> DefinitionRegionWrapper {
        DefinitionRegionWrapper {
            inner: self.inner.intersected(&other.inner),
            schematic_ptr: std::ptr::null_mut(),
            name: None,
        }
    }
    /// Add all points from another region to this one (mutating union)
    #[wasm_bindgen(js_name = unionInto)]
    pub fn union_into(mut self, other: &DefinitionRegionWrapper) -> Self {
        self.inner.union_into(&other.inner);
        self.sync();
        self
    }

    /// Simplify the region by merging adjacent/overlapping boxes
    #[wasm_bindgen(js_name = simplify)]
    pub fn simplify(mut self) -> Self {
        self.inner.simplify();
        self.sync();
        self
    }

    // ========================================================================
    // Box Access (for Rendering)
    // ========================================================================

    /// Create a DefinitionRegion from multiple bounding boxes
    ///
    /// Takes an array of {min: [x,y,z], max: [x,y,z]} objects.
    /// Unlike fromPositions which merges adjacent points, this keeps boxes as provided.
    #[wasm_bindgen(js_name = fromBoundingBoxes)]
    pub fn from_bounding_boxes(boxes: &JsValue) -> Result<DefinitionRegionWrapper, JsValue> {
        let array: Array = boxes
            .clone()
            .dyn_into()
            .map_err(|_| JsValue::from_str("Expected an array of bounding boxes"))?;

        let mut box_list = Vec::new();

        for i in 0..array.length() {
            let item = array.get(i);
            let obj: Object = item
                .dyn_into()
                .map_err(|_| JsValue::from_str("Each box should be an object"))?;

            let min_val = Reflect::get(&obj, &"min".into())
                .map_err(|_| JsValue::from_str("Box missing 'min' property"))?;
            let max_val = Reflect::get(&obj, &"max".into())
                .map_err(|_| JsValue::from_str("Box missing 'max' property"))?;

            let min_arr: Array = min_val
                .dyn_into()
                .map_err(|_| JsValue::from_str("'min' should be an array"))?;
            let max_arr: Array = max_val
                .dyn_into()
                .map_err(|_| JsValue::from_str("'max' should be an array"))?;

            let min = (
                min_arr
                    .get(0)
                    .as_f64()
                    .ok_or_else(|| JsValue::from_str("Invalid min x"))? as i32,
                min_arr
                    .get(1)
                    .as_f64()
                    .ok_or_else(|| JsValue::from_str("Invalid min y"))? as i32,
                min_arr
                    .get(2)
                    .as_f64()
                    .ok_or_else(|| JsValue::from_str("Invalid min z"))? as i32,
            );
            let max = (
                max_arr
                    .get(0)
                    .as_f64()
                    .ok_or_else(|| JsValue::from_str("Invalid max x"))? as i32,
                max_arr
                    .get(1)
                    .as_f64()
                    .ok_or_else(|| JsValue::from_str("Invalid max y"))? as i32,
                max_arr
                    .get(2)
                    .as_f64()
                    .ok_or_else(|| JsValue::from_str("Invalid max z"))? as i32,
            );

            box_list.push((min, max));
        }

        Ok(DefinitionRegionWrapper {
            inner: DefinitionRegion::from_bounding_boxes(box_list),
            schematic_ptr: std::ptr::null_mut(),
            name: None,
        })
    }

    /// Create a DefinitionRegion from an array of positions
    ///
    /// Takes an array of [x, y, z] arrays. Adjacent points will be merged into boxes.
    #[wasm_bindgen(js_name = fromPositions)]
    pub fn from_positions(positions: &JsValue) -> Result<DefinitionRegionWrapper, JsValue> {
        let array: Array = positions
            .clone()
            .dyn_into()
            .map_err(|_| JsValue::from_str("Expected an array of positions"))?;

        let mut pos_list = Vec::new();

        for i in 0..array.length() {
            let pos_arr: Array = array
                .get(i)
                .dyn_into()
                .map_err(|_| JsValue::from_str("Each position should be an array"))?;

            let x = pos_arr
                .get(0)
                .as_f64()
                .ok_or_else(|| JsValue::from_str("Invalid x coordinate"))?
                as i32;
            let y = pos_arr
                .get(1)
                .as_f64()
                .ok_or_else(|| JsValue::from_str("Invalid y coordinate"))?
                as i32;
            let z = pos_arr
                .get(2)
                .as_f64()
                .ok_or_else(|| JsValue::from_str("Invalid z coordinate"))?
                as i32;

            pos_list.push((x, y, z));
        }

        Ok(DefinitionRegionWrapper {
            inner: DefinitionRegion::from_positions(&pos_list),
            schematic_ptr: std::ptr::null_mut(),
            name: None,
        })
    }

    /// Get the number of bounding boxes in this region
    #[wasm_bindgen(js_name = boxCount)]
    pub fn box_count(&self) -> usize {
        self.inner.box_count()
    }

    /// Get a specific bounding box by index
    ///
    /// Returns {min: [x,y,z], max: [x,y,z]} or null if index is out of bounds
    #[wasm_bindgen(js_name = getBox)]
    pub fn get_box(&self, index: usize) -> JsValue {
        match self.inner.get_box(index) {
            Some((min, max)) => {
                let obj = Object::new();
                let min_arr = Array::new();
                min_arr.push(&JsValue::from(min.0));
                min_arr.push(&JsValue::from(min.1));
                min_arr.push(&JsValue::from(min.2));
                let max_arr = Array::new();
                max_arr.push(&JsValue::from(max.0));
                max_arr.push(&JsValue::from(max.1));
                max_arr.push(&JsValue::from(max.2));
                Reflect::set(&obj, &"min".into(), &min_arr).unwrap();
                Reflect::set(&obj, &"max".into(), &max_arr).unwrap();
                obj.into()
            }
            None => JsValue::NULL,
        }
    }

    /// Get all bounding boxes in this region
    ///
    /// Returns an array of {min: [x,y,z], max: [x,y,z]} objects.
    /// Useful for rendering each box separately.
    #[wasm_bindgen(js_name = getBoxes)]
    pub fn get_boxes(&self) -> Array {
        let array = Array::new();
        for (min, max) in self.inner.get_boxes() {
            let obj = Object::new();
            let min_arr = Array::new();
            min_arr.push(&JsValue::from(min.0));
            min_arr.push(&JsValue::from(min.1));
            min_arr.push(&JsValue::from(min.2));
            let max_arr = Array::new();
            max_arr.push(&JsValue::from(max.0));
            max_arr.push(&JsValue::from(max.1));
            max_arr.push(&JsValue::from(max.2));
            Reflect::set(&obj, &"min".into(), &min_arr).unwrap();
            Reflect::set(&obj, &"max".into(), &max_arr).unwrap();
            array.push(&obj);
        }
        array
    }

    // ========================================================================
    // Metadata Access
    // ========================================================================

    /// Get a metadata value by key
    ///
    /// Returns the value string or null if not found
    #[wasm_bindgen(js_name = getMetadata)]
    pub fn get_metadata(&self, key: &str) -> JsValue {
        match self.inner.get_metadata(key) {
            Some(value) => JsValue::from_str(value),
            None => JsValue::NULL,
        }
    }

    /// Get all metadata as a JS object
    #[wasm_bindgen(js_name = getAllMetadata)]
    pub fn get_all_metadata(&self) -> JsValue {
        let obj = Object::new();
        for (key, value) in self.inner.metadata_ref() {
            Reflect::set(&obj, &JsValue::from_str(key), &JsValue::from_str(value)).unwrap();
        }
        obj.into()
    }

    /// Get all metadata keys
    #[wasm_bindgen(js_name = metadataKeys)]
    pub fn metadata_keys(&self) -> Array {
        let array = Array::new();
        for key in self.inner.metadata_keys() {
            array.push(&JsValue::from_str(key));
        }
        array
    }

    // ========================================================================
    // Geometry Helpers (for Rendering)
    // ========================================================================

    /// Get the dimensions (width, height, length) of the overall bounding box
    ///
    /// Returns [width, height, length] or [0, 0, 0] if empty
    #[wasm_bindgen(js_name = dimensions)]
    pub fn dimensions(&self) -> Array {
        let (w, h, l) = self.inner.dimensions();
        let arr = Array::new();
        arr.push(&JsValue::from(w));
        arr.push(&JsValue::from(h));
        arr.push(&JsValue::from(l));
        arr
    }

    /// Get the center point of the region (integer coordinates)
    ///
    /// Returns [x, y, z] or null if empty
    #[wasm_bindgen(js_name = center)]
    pub fn center(&self) -> JsValue {
        match self.inner.center() {
            Some((x, y, z)) => {
                let arr = Array::new();
                arr.push(&JsValue::from(x));
                arr.push(&JsValue::from(y));
                arr.push(&JsValue::from(z));
                arr.into()
            }
            None => JsValue::NULL,
        }
    }

    /// Get the center point of the region as f32 (for rendering)
    ///
    /// Returns [x, y, z] as floats or null if empty
    #[wasm_bindgen(js_name = centerF32)]
    pub fn center_f32(&self) -> JsValue {
        match self.inner.center_f32() {
            Some((x, y, z)) => {
                let arr = Array::new();
                arr.push(&JsValue::from(x));
                arr.push(&JsValue::from(y));
                arr.push(&JsValue::from(z));
                arr.into()
            }
            None => JsValue::NULL,
        }
    }

    /// Check if this region intersects with a bounding box
    ///
    /// Useful for frustum culling in renderers.
    #[wasm_bindgen(js_name = intersectsBounds)]
    pub fn intersects_bounds(
        &self,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> bool {
        self.inner
            .intersects_bounds((min_x, min_y, min_z), (max_x, max_y, max_z))
    }

    // ========================================================================
    // Immutable Geometric Transformations
    // ========================================================================

    /// Create a new region shifted by the given offset (immutable)
    #[wasm_bindgen(js_name = shifted)]
    pub fn shifted(&self, x: i32, y: i32, z: i32) -> DefinitionRegionWrapper {
        DefinitionRegionWrapper {
            inner: self.inner.shifted(x, y, z),
            schematic_ptr: std::ptr::null_mut(),
            name: None,
        }
    }

    /// Create a new region expanded by the given amounts (immutable)
    #[wasm_bindgen(js_name = expanded)]
    pub fn expanded(&self, x: i32, y: i32, z: i32) -> DefinitionRegionWrapper {
        DefinitionRegionWrapper {
            inner: self.inner.expanded(x, y, z),
            schematic_ptr: std::ptr::null_mut(),
            name: None,
        }
    }

    /// Create a new region contracted by the given amount (immutable)
    #[wasm_bindgen(js_name = contracted)]
    pub fn contracted(&self, amount: i32) -> DefinitionRegionWrapper {
        DefinitionRegionWrapper {
            inner: self.inner.contracted(amount),
            schematic_ptr: std::ptr::null_mut(),
            name: None,
        }
    }

    /// Create a deep copy of this region
    #[wasm_bindgen(js_name = copy)]
    pub fn copy(&self) -> DefinitionRegionWrapper {
        DefinitionRegionWrapper {
            inner: self.inner.copy(),
            schematic_ptr: std::ptr::null_mut(),
            name: None,
        }
    }

    /// Clone this region (alias for copy)
    #[wasm_bindgen(js_name = clone)]
    pub fn clone_region(&self) -> DefinitionRegionWrapper {
        self.copy()
    }

    #[wasm_bindgen(js_name = getBlocks)]
    pub fn get_blocks(&self) -> Result<Array, JsValue> {
        if self.schematic_ptr.is_null() {
            return Err(JsValue::from_str("Region is not attached to a schematic"));
        }

        let sch = unsafe { &*self.schematic_ptr };
        let arr = Array::new();

        for (x, y, z) in self.inner.iter_positions() {
            let obj = Object::new();
            Reflect::set(&obj, &"x".into(), &JsValue::from(x))?;
            Reflect::set(&obj, &"y".into(), &JsValue::from(y))?;
            Reflect::set(&obj, &"z".into(), &JsValue::from(z))?;

            if let Some(block) = sch.get_block(x, y, z) {
                Reflect::set(&obj, &"block".into(), &JsValue::from(block.name.clone()))?;
            }

            arr.push(&obj);
        }

        Ok(arr)
    }
}

impl Default for DefinitionRegionWrapper {
    fn default() -> Self {
        Self::new()
    }
}
