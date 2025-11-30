//! Schematic Builder WASM bindings
//!
//! ASCII art and template-based schematic construction.

use super::SchematicWrapper;
use wasm_bindgen::prelude::*;

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
