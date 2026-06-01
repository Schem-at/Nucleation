//! Fingerprint WASM bindings
//!
//! Content-addressable fingerprints and structural signatures of schematics,
//! plus footprint-distance and duplicate detection. Exposed as methods on
//! [`SchematicWrapper`].

use wasm_bindgen::prelude::*;

use crate::fingerprint::{
    fingerprint as core_fingerprint, footprint_distance as core_footprint_distance,
    is_duplicate as core_is_duplicate, signature as core_signature, FingerprintSpec,
};

use super::schematic::SchematicWrapper;

/// Resolve a preset name (defaulting to `"exact"`) into a [`FingerprintSpec`].
fn resolve_spec(preset: Option<String>) -> Result<FingerprintSpec, JsValue> {
    let preset = preset.unwrap_or_else(|| "exact".to_string());
    FingerprintSpec::from_preset(&preset)
        .ok_or_else(|| JsValue::from_str(&format!("unknown fingerprint preset: {}", preset)))
}

#[wasm_bindgen]
impl SchematicWrapper {
    /// Compute the content fingerprint as a hex string.
    ///
    /// `preset` defaults to `"exact"`. Unknown preset → Err.
    #[wasm_bindgen]
    pub fn fingerprint(&self, preset: Option<String>) -> Result<String, JsValue> {
        let spec = resolve_spec(preset)?;
        Ok(core_fingerprint(&self.0, &spec).to_hex())
    }

    /// Compute the structural signature as a JSON string.
    ///
    /// `preset` defaults to `"exact"`. Unknown preset → Err.
    #[wasm_bindgen]
    pub fn signature(&self, preset: Option<String>) -> Result<String, JsValue> {
        let spec = resolve_spec(preset)?;
        Ok(core_signature(&self.0, &spec).to_json())
    }

    /// Footprint distance between this schematic and `other` (0.0 = identical).
    ///
    /// `preset` defaults to `"exact"`. Unknown preset → Err.
    #[wasm_bindgen(js_name = footprintDistance)]
    pub fn footprint_distance(
        &self,
        other: &SchematicWrapper,
        preset: Option<String>,
    ) -> Result<f32, JsValue> {
        let spec = resolve_spec(preset)?;
        Ok(core_footprint_distance(&self.0, &other.0, &spec))
    }

    /// Whether this schematic is a duplicate of `other` under `preset`.
    ///
    /// `preset` defaults to `"exact"`. Unknown preset → Err.
    #[wasm_bindgen(js_name = isDuplicateOf)]
    pub fn is_duplicate_of(
        &self,
        other: &SchematicWrapper,
        preset: Option<String>,
    ) -> Result<bool, JsValue> {
        let spec = resolve_spec(preset)?;
        Ok(core_is_duplicate(&self.0, &other.0, &spec))
    }
}
