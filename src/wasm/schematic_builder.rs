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

    /// Create from template string
    #[wasm_bindgen(js_name = fromTemplate)]
    pub fn from_template(template: String) -> Result<SchematicBuilderWrapper, JsValue> {
        let builder =
            crate::SchematicBuilder::from_template(&template).map_err(|e| JsValue::from_str(&e))?;
        Ok(Self { inner: builder })
    }
}
