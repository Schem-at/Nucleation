//! Diff Python bindings
//!
//! Wraps the [`crate::diff`] engine: structural diffing between two schematics,
//! producing a [`PyDiff`] with change sets, regions, and overlay export.

use pyo3::prelude::*;

use super::schematic::PySchematic;
use crate::diff::{Diff, DiffSpec, SpecOverrides};
use crate::fingerprint::symmetry::Symmetry;

/// Build a [`SpecOverrides`] from optional kwargs, parsing `symmetry` by name.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_overrides(
    cost_add: Option<u32>,
    cost_delete: Option<u32>,
    cost_change: Option<u32>,
    cost_swap: Option<u32>,
    swap_dominance_pct: Option<u32>,
    symmetry: Option<&str>,
) -> PyResult<SpecOverrides> {
    let symmetry = match symmetry {
        Some(name) => Some(Symmetry::from_name(name).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("unknown symmetry: '{}'", name))
        })?),
        None => None,
    };
    Ok(SpecOverrides {
        cost_add,
        cost_delete,
        cost_change,
        cost_swap,
        swap_dominance_pct,
        symmetry,
    })
}

/// Resolve a [`DiffSpec`] from a preset name plus overrides, raising
/// `ValueError` on an unknown preset.
pub(crate) fn resolve_spec(preset: &str, ov: &SpecOverrides) -> PyResult<DiffSpec> {
    DiffSpec::resolve(preset, ov).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("unknown preset: '{}'", preset))
    })
}

/// Result of diffing two schematics.
///
/// Wraps a [`Diff`] with change distance, support, and accessors that
/// materialize the added/removed/changed/swapped cells as schematics.
#[pyclass(name = "Diff")]
pub struct PyDiff {
    pub(crate) inner: Diff,
}

#[pymethods]
impl PyDiff {
    /// Edit distance between the two schematics under the cost model.
    #[getter]
    pub fn distance(&self) -> u64 {
        self.inner.distance
    }

    /// Alignment support (fraction of cells explained by the transform).
    #[getter]
    pub fn support(&self) -> f32 {
        self.inner.support
    }

    /// The recovered rigid transform (rotation + translation) as JSON.
    pub fn transform_json(&self) -> String {
        serde_json::json!({
            "rotate": serde_json::to_value(&self.inner.transform.rotate)
                .unwrap_or(serde_json::Value::Null),
            "translate": [
                self.inner.transform.translate.0,
                self.inner.transform.translate.1,
                self.inner.transform.translate.2,
            ],
        })
        .to_string()
    }

    /// Lossless JSON serialization of the full diff.
    pub fn to_json(&self) -> String {
        self.inner.to_json()
    }

    /// Compact human summary (counts + regions + swaps), no per-cell data.
    pub fn summary_json(&self) -> String {
        self.inner.summary_json()
    }

    /// Reconstruct a Diff from lossless [`to_json`](Self::to_json) output.
    #[staticmethod]
    pub fn from_json(s: &str) -> PyResult<Self> {
        let inner = Diff::from_json(s)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(Self { inner })
    }

    /// A schematic containing only the added blocks.
    pub fn added(&self) -> PySchematic {
        PySchematic::from_inner(self.inner.added())
    }

    /// A schematic containing only the removed blocks.
    pub fn removed(&self) -> PySchematic {
        PySchematic::from_inner(self.inner.removed())
    }

    /// A schematic containing only the changed blocks (new values).
    pub fn changed(&self) -> PySchematic {
        PySchematic::from_inner(self.inner.changed())
    }

    /// A schematic containing only the swapped blocks (new values).
    pub fn swapped(&self) -> PySchematic {
        PySchematic::from_inner(self.inner.swapped())
    }

    /// A schematic of marker blocks for all change cells.
    pub fn markers(&self) -> PySchematic {
        PySchematic::from_inner(self.inner.markers())
    }

    /// Connected change regions as JSON: a list of
    /// `{min:[x,y,z], max:[x,y,z], kind:"...", count:n}`.
    pub fn regions_json(&self) -> String {
        let regs = crate::diff::regions::regions(&self.inner);
        let arr: Vec<serde_json::Value> = regs
            .iter()
            .map(|r| {
                serde_json::json!({
                    "min": [r.min.0, r.min.1, r.min.2],
                    "max": [r.max.0, r.max.1, r.max.2],
                    "kind": format!("{:?}", r.kind),
                    "count": r.count,
                })
            })
            .collect();
        serde_json::Value::Array(arr).to_string()
    }

    /// Inject translucent change markers onto an already-meshed GLB,
    /// returning a new GLB byte buffer.
    #[cfg(feature = "meshing")]
    pub fn to_overlay_glb<'py>(
        &self,
        py: Python<'py>,
        after_glb: Vec<u8>,
    ) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
        let opts = crate::diff::OverlayOptions::default();
        let data = self
            .inner
            .to_overlay_glb(&after_glb, &opts)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(pyo3::types::PyBytes::new(py, &data))
    }

    fn __repr__(&self) -> String {
        format!(
            "<Diff distance={} support={:.3} added={} removed={} changed={} swapped={}>",
            self.inner.distance,
            self.inner.support,
            self.inner.added.len(),
            self.inner.removed.len(),
            self.inner.changed.len(),
            self.inner.swapped.len(),
        )
    }
}
