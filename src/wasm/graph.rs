//! WASM bindings for the redstone-graph analysis surface.
//!
//! Mirrors the Python `RedstoneGraph` API: `nodes`/`edges` (plain JS arrays of
//! objects), `nodeCount`/`edgeCount`, `features`/`featuresJson`, `fingerprint`,
//! and JSON round-trip (`toJson`/`fromJson`). The node/edge object shape is the
//! same across all bindings (produced by the shared core `RedstoneGraph`
//! `nodes_json`/`edges_json` helpers).

use crate::simulation::fingerprint::GraphFingerprintSpec;
use crate::simulation::graph::RedstoneGraph;
use wasm_bindgen::prelude::*;

/// A wrapper around an extracted [`RedstoneGraph`] for JS consumers.
#[wasm_bindgen]
pub struct RedstoneGraphWrapper {
    pub(crate) inner: RedstoneGraph,
}

/// Parse a JSON string into a `JsValue` (plain JS objects/arrays).
fn json_to_js(json: &str) -> Result<JsValue, JsValue> {
    js_sys::JSON::parse(json).map_err(|_| JsValue::from_str("failed to parse graph JSON"))
}

#[wasm_bindgen]
impl RedstoneGraphWrapper {
    /// The nodes as an array of plain objects (kind-specific fields inlined).
    #[wasm_bindgen(getter)]
    pub fn nodes(&self) -> Result<JsValue, JsValue> {
        let json = self.inner.nodes_json().map_err(|e| JsValue::from_str(&e))?;
        json_to_js(&json)
    }

    /// The directed edges as an array of `{source, target, kind, strength}`.
    #[wasm_bindgen(getter)]
    pub fn edges(&self) -> Result<JsValue, JsValue> {
        let json = self.inner.edges_json().map_err(|e| JsValue::from_str(&e))?;
        json_to_js(&json)
    }

    /// Number of nodes in the graph.
    #[wasm_bindgen(getter, js_name = nodeCount)]
    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    /// Total number of directed edges in the graph.
    #[wasm_bindgen(getter, js_name = edgeCount)]
    pub fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }

    /// Computed graph features as a plain object.
    pub fn features(&self) -> Result<JsValue, JsValue> {
        let json = self
            .inner
            .features()
            .to_json()
            .map_err(|e| JsValue::from_str(&e))?;
        json_to_js(&json)
    }

    /// The computed graph features serialized as a JSON string.
    #[wasm_bindgen(js_name = featuresJson)]
    pub fn features_json(&self) -> Result<String, JsValue> {
        self.inner
            .features()
            .to_json()
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Compute the structural/functional/exact fingerprint as a hex string.
    ///
    /// `preset` defaults to `"structural"`; throws for an unknown preset.
    pub fn fingerprint(&self, preset: Option<String>) -> Result<String, JsValue> {
        let preset = preset.unwrap_or_else(|| "structural".to_string());
        let spec = GraphFingerprintSpec::from_preset(&preset).ok_or_else(|| {
            JsValue::from_str(&format!(
                "unknown fingerprint preset: {preset:?} (expected \"structural\", \"functional\", or \"exact\")"
            ))
        })?;
        Ok(self.inner.fingerprint(&spec).to_hex())
    }

    /// Serialize the graph to JSON.
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> Result<String, JsValue> {
        self.inner.to_json().map_err(|e| JsValue::from_str(&e))
    }

    /// Deserialize a graph from JSON.
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(s: &str) -> Result<RedstoneGraphWrapper, JsValue> {
        RedstoneGraph::from_json(s)
            .map(|inner| RedstoneGraphWrapper { inner })
            .map_err(|e| JsValue::from_str(&e))
    }
}
