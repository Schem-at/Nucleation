//! Diff WASM bindings
//!
//! Structural diff between two schematics: alignment transform, edit distance,
//! and per-cell added/removed/changed/swapped sets. Exposes [`DiffWrapper`]
//! plus a `diff()` method on [`SchematicWrapper`].

#[cfg(feature = "meshing")]
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

use crate::diff::regions::{regions, RegionKind};
use crate::diff::{diff as core_diff, Diff, DiffSpec, SpecOverrides};
use crate::fingerprint::symmetry::Symmetry;

use super::schematic::SchematicWrapper;

/// Parse the optional `options` JS object into [`SpecOverrides`].
///
/// Accepts an object `{cost_add?, cost_delete?, cost_change?, cost_swap?,
/// swap_dominance_pct?, symmetry?}`. `null`/`undefined` yields default (empty)
/// overrides. Unknown symmetry name → Err.
fn parse_overrides(options: JsValue) -> Result<SpecOverrides, JsValue> {
    if options.is_null() || options.is_undefined() {
        return Ok(SpecOverrides::default());
    }
    let obj: js_sys::Object = options
        .dyn_into()
        .map_err(|_| JsValue::from_str("diff options must be an object"))?;

    let read_u32 = |key: &str| -> Result<Option<u32>, JsValue> {
        let v = js_sys::Reflect::get(&obj, &JsValue::from_str(key))
            .map_err(|_| JsValue::from_str("failed to read diff options"))?;
        if v.is_null() || v.is_undefined() {
            Ok(None)
        } else {
            v.as_f64().map(|f| f as u32).map(Some).ok_or_else(|| {
                JsValue::from_str(&format!("diff option '{}' must be a number", key))
            })
        }
    };

    let symmetry = {
        let v = js_sys::Reflect::get(&obj, &JsValue::from_str("symmetry"))
            .map_err(|_| JsValue::from_str("failed to read diff options"))?;
        if v.is_null() || v.is_undefined() {
            None
        } else {
            let name = v
                .as_string()
                .ok_or_else(|| JsValue::from_str("diff option 'symmetry' must be a string"))?;
            Some(
                Symmetry::from_name(&name)
                    .ok_or_else(|| JsValue::from_str(&format!("unknown symmetry: {}", name)))?,
            )
        }
    };

    Ok(SpecOverrides {
        cost_add: read_u32("cost_add")?,
        cost_delete: read_u32("cost_delete")?,
        cost_change: read_u32("cost_change")?,
        cost_swap: read_u32("cost_swap")?,
        swap_dominance_pct: read_u32("swap_dominance_pct")?,
        symmetry,
    })
}

/// WASM wrapper for a [`Diff`] between two schematics.
#[wasm_bindgen]
pub struct DiffWrapper(pub(crate) Diff);

#[wasm_bindgen]
impl DiffWrapper {
    /// Total edit distance (sum of weighted add/delete/change/swap costs).
    ///
    /// Surfaced as a JS `number` (f64). Edit distances stay well within f64's
    /// exact-integer range; the core value is `u64`.
    #[wasm_bindgen(getter)]
    pub fn distance(&self) -> f64 {
        self.0.distance as f64
    }

    /// Match support score (fraction of cells explained by the alignment).
    #[wasm_bindgen(getter)]
    pub fn support(&self) -> f32 {
        self.0.support
    }

    /// Serialize the full diff to a JSON string.
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> String {
        self.0.to_json()
    }

    /// Serialize a compact summary (counts + transform) to a JSON string.
    #[wasm_bindgen(js_name = summaryJson)]
    pub fn summary_json(&self) -> String {
        self.0.summary_json()
    }

    /// Reconstruct a diff from a JSON string produced by `toJson()`.
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(s: &str) -> Result<DiffWrapper, JsValue> {
        Diff::from_json(s)
            .map(DiffWrapper)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Schematic containing only the added blocks.
    pub fn added(&self) -> SchematicWrapper {
        SchematicWrapper(self.0.added())
    }

    /// Schematic containing only the removed blocks.
    pub fn removed(&self) -> SchematicWrapper {
        SchematicWrapper(self.0.removed())
    }

    /// Schematic containing only the changed blocks (post-change state).
    pub fn changed(&self) -> SchematicWrapper {
        SchematicWrapper(self.0.changed())
    }

    /// Schematic containing only the palette-swapped blocks.
    pub fn swapped(&self) -> SchematicWrapper {
        SchematicWrapper(self.0.swapped())
    }

    /// Schematic of visualization markers for all change classes.
    pub fn markers(&self) -> SchematicWrapper {
        SchematicWrapper(self.0.markers())
    }

    /// Connected change regions as a JSON array.
    ///
    /// Each region: `{min:[x,y,z], max:[x,y,z], kind:"added|removed|changed|swapped|mixed", count}`.
    #[wasm_bindgen(js_name = regionsJson)]
    pub fn regions_json(&self) -> String {
        let regs = regions(&self.0);
        let mut out = String::from("[");
        for (i, r) in regs.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let kind = match r.kind {
                RegionKind::Added => "added",
                RegionKind::Removed => "removed",
                RegionKind::Changed => "changed",
                RegionKind::Swapped => "swapped",
                RegionKind::Mixed => "mixed",
            };
            out.push_str(&format!(
                "{{\"min\":[{},{},{}],\"max\":[{},{},{}],\"kind\":\"{}\",\"count\":{}}}",
                r.min.0, r.min.1, r.min.2, r.max.0, r.max.1, r.max.2, kind, r.count
            ));
        }
        out.push(']');
        out
    }

    /// Generate a colored overlay GLB on top of an already-meshed "after" GLB.
    #[cfg(feature = "meshing")]
    #[wasm_bindgen(js_name = toOverlayGlb)]
    pub fn to_overlay_glb(&self, after_glb: &[u8]) -> Result<Uint8Array, JsValue> {
        let opts = crate::diff::OverlayOptions::default();
        let data = self
            .0
            .to_overlay_glb(after_glb, &opts)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Uint8Array::from(data.as_slice()))
    }
}

#[wasm_bindgen]
impl SchematicWrapper {
    /// Structurally diff this schematic against `other`.
    ///
    /// `preset` defaults to `"exact"`. `options` is an optional object
    /// `{cost_add?, cost_delete?, cost_change?, cost_swap?, swap_dominance_pct?,
    /// symmetry?}`. Returns a [`DiffWrapper`]. Unknown preset/symmetry → Err.
    #[wasm_bindgen]
    pub fn diff(
        &self,
        other: &SchematicWrapper,
        preset: Option<String>,
        options: JsValue,
    ) -> Result<DiffWrapper, JsValue> {
        let preset = preset.unwrap_or_else(|| "exact".to_string());
        let overrides = parse_overrides(options)?;
        let spec = DiffSpec::resolve(&preset, &overrides)
            .ok_or_else(|| JsValue::from_str(&format!("unknown diff preset: {}", preset)))?;
        Ok(DiffWrapper(core_diff(&self.0, &other.0, &spec)))
    }
}
