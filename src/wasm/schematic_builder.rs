//! Schematic Builder WASM bindings
//!
//! ASCII art and template-based schematic construction.

use super::SchematicWrapper;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

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

    /// Add multiple layers (array of arrays of strings)
    #[wasm_bindgen(js_name = layers)]
    pub fn layers(mut self, layers: JsValue) -> Result<SchematicBuilderWrapper, JsValue> {
        let layers_array: js_sys::Array = layers
            .dyn_into()
            .map_err(|_| JsValue::from_str("Expected an array of layers"))?;

        let mut rust_layers: Vec<Vec<String>> = Vec::new();
        for i in 0..layers_array.length() {
            let layer: js_sys::Array = layers_array
                .get(i)
                .dyn_into()
                .map_err(|_| JsValue::from_str("Each layer should be an array of strings"))?;
            let mut layer_strings = Vec::new();
            for j in 0..layer.length() {
                let s = layer
                    .get(j)
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("Each row should be a string"))?;
                layer_strings.push(s);
            }
            rust_layers.push(layer_strings);
        }

        let layer_refs: Vec<Vec<&str>> = rust_layers
            .iter()
            .map(|layer| layer.iter().map(|s| s.as_str()).collect())
            .collect();
        let layer_slice_refs: Vec<&[&str]> = layer_refs.iter().map(|v| v.as_slice()).collect();
        self.inner = self.inner.layers(&layer_slice_refs);
        Ok(self)
    }

    /// Build the schematic
    #[wasm_bindgen(js_name = build)]
    pub fn build(self) -> Result<SchematicWrapper, JsValue> {
        let schematic = self.inner.build().map_err(|e| JsValue::from_str(&e))?;
        Ok(SchematicWrapper(schematic))
    }

    /// Append a single layer (`string[]` of rows). Equivalent to
    /// `layers([rows])` but doesn't require nesting for a single layer.
    #[wasm_bindgen(js_name = layer)]
    pub fn layer(mut self, rows: JsValue) -> Result<SchematicBuilderWrapper, JsValue> {
        let arr: js_sys::Array = rows
            .dyn_into()
            .map_err(|_| JsValue::from_str("Expected an array of strings"))?;
        let mut row_strings = Vec::new();
        for i in 0..arr.length() {
            let s = arr
                .get(i)
                .as_string()
                .ok_or_else(|| JsValue::from_str("Each row should be a string"))?;
            row_strings.push(s);
        }
        let row_refs: Vec<&str> = row_strings.iter().map(|s| s.as_str()).collect();
        self.inner = self.inner.layer(&row_refs);
        Ok(self)
    }

    /// Bulk version of `map`. Pass either an `Object` literal
    /// (`{ c: "minecraft:stone", " ": "minecraft:air" }`) or an array
    /// of `[char, block]` pairs.
    #[wasm_bindgen(js_name = palette)]
    pub fn palette(mut self, mappings: JsValue) -> Result<SchematicBuilderWrapper, JsValue> {
        let mut pairs: Vec<(char, String)> = Vec::new();
        if let Ok(arr) = mappings.clone().dyn_into::<js_sys::Array>() {
            for i in 0..arr.length() {
                let entry: js_sys::Array = arr
                    .get(i)
                    .dyn_into()
                    .map_err(|_| JsValue::from_str("palette: each entry must be [char, block]"))?;
                let key = entry
                    .get(0)
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("palette: char key must be a string"))?;
                let val = entry
                    .get(1)
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("palette: block must be a string"))?;
                let ch = key.chars().next().ok_or_else(|| {
                    JsValue::from_str("palette: char key must be a non-empty string")
                })?;
                pairs.push((ch, val));
            }
        } else if mappings.is_object() {
            let obj: js_sys::Object = mappings
                .dyn_into()
                .map_err(|_| JsValue::from_str("palette: expected an object or array"))?;
            let entries = js_sys::Object::entries(&obj);
            for i in 0..entries.length() {
                let kv: js_sys::Array = entries
                    .get(i)
                    .dyn_into()
                    .map_err(|_| JsValue::from_str("palette: malformed object entry"))?;
                let key = kv
                    .get(0)
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("palette: key must be a string"))?;
                let val = kv
                    .get(1)
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("palette: value must be a string"))?;
                let ch = key
                    .chars()
                    .next()
                    .ok_or_else(|| JsValue::from_str("palette: key must be a non-empty string"))?;
                pairs.push((ch, val));
            }
        } else {
            return Err(JsValue::from_str(
                "palette: expected an object or an array of [char, block] pairs",
            ));
        }
        let pairs_ref: Vec<(char, &str)> = pairs.iter().map(|(c, b)| (*c, b.as_str())).collect();
        self.inner = self.inner.palette(&pairs_ref);
        Ok(self)
    }

    /// Set the world offset of the resulting schematic.
    #[wasm_bindgen(js_name = offset)]
    pub fn offset(mut self, x: i32, y: i32, z: i32) -> Self {
        self.inner = self.inner.offset(x, y, z);
        self
    }

    /// Apply the standard palette (gray concrete, air, plus the
    /// named-direction characters used in the canonical examples).
    #[wasm_bindgen(js_name = useStandardPalette)]
    pub fn use_standard_palette(mut self) -> Self {
        self.inner = self.inner.use_standard_palette();
        self
    }

    /// Apply the minimal palette (`c` and space only).
    #[wasm_bindgen(js_name = useMinimalPalette)]
    pub fn use_minimal_palette(mut self) -> Self {
        self.inner = self.inner.use_minimal_palette();
        self
    }

    /// Apply the compact palette (single-glyph redstone shapes).
    #[wasm_bindgen(js_name = useCompactPalette)]
    pub fn use_compact_palette(mut self) -> Self {
        self.inner = self.inner.use_compact_palette();
        self
    }

    /// Run pre-build validation. Throws if the layered template is
    /// malformed (missing palette mapping, ragged rows, etc.).
    #[wasm_bindgen(js_name = validate)]
    pub fn validate(&self) -> Result<(), JsValue> {
        self.inner.validate().map_err(|e| JsValue::from_str(&e))
    }

    /// Serialize the builder back into the canonical template format.
    #[wasm_bindgen(js_name = toTemplate)]
    pub fn to_template(&self) -> String {
        self.inner.to_template()
    }

    /// Create from template string
    #[wasm_bindgen(js_name = fromTemplate)]
    pub fn from_template(template: String) -> Result<SchematicBuilderWrapper, JsValue> {
        let builder =
            crate::SchematicBuilder::from_template(&template).map_err(|e| JsValue::from_str(&e))?;
        Ok(Self { inner: builder })
    }
}
