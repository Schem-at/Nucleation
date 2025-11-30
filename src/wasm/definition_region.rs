//! DefinitionRegion WASM bindings

use crate::block_position::BlockPosition;
use crate::definition_region::DefinitionRegion;
use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;

use super::SchematicWrapper;

#[wasm_bindgen]
pub struct DefinitionRegionWrapper {
    pub(crate) inner: DefinitionRegion,
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

impl Default for DefinitionRegionWrapper {
    fn default() -> Self {
        Self::new()
    }
}
