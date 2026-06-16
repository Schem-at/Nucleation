//! Python bindings for the redstone-graph analysis surface.
//!
//! Exposes the extracted [`RedstoneGraph`] to Python as plain data
//! (lists of dicts for nodes/edges), plus the fast Rust `features()` and
//! `fingerprint()` analyses. The design goal is to let Python perform
//! arbitrary graph analysis (networkx / pandas / json friendly) without
//! reaching into Rust per node.

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::simulation::fingerprint::GraphFingerprintSpec;
use crate::simulation::graph::{ComparatorMode, LinkKind, RedstoneGraph, RedstoneNodeKind};

/// Python-facing wrapper around a [`RedstoneGraph`].
#[pyclass(name = "RedstoneGraph")]
pub struct PyRedstoneGraph {
    pub(crate) inner: RedstoneGraph,
}

fn node_kind_name(kind: &RedstoneNodeKind) -> &'static str {
    match kind {
        RedstoneNodeKind::Repeater { .. } => "Repeater",
        RedstoneNodeKind::Comparator { .. } => "Comparator",
        RedstoneNodeKind::Torch => "Torch",
        RedstoneNodeKind::Lamp => "Lamp",
        RedstoneNodeKind::Button => "Button",
        RedstoneNodeKind::Lever => "Lever",
        RedstoneNodeKind::PressurePlate => "PressurePlate",
        RedstoneNodeKind::Trapdoor => "Trapdoor",
        RedstoneNodeKind::Wire => "Wire",
        RedstoneNodeKind::Constant => "Constant",
        RedstoneNodeKind::NoteBlock => "NoteBlock",
    }
}

fn comparator_mode_name(mode: &ComparatorMode) -> &'static str {
    match mode {
        ComparatorMode::Compare => "Compare",
        ComparatorMode::Subtract => "Subtract",
    }
}

fn link_kind_name(kind: &LinkKind) -> &'static str {
    match kind {
        LinkKind::Default => "Default",
        LinkKind::Side => "Side",
    }
}

/// Convert a `serde_json::Value` into a Python object.
fn json_to_py(py: Python<'_>, value: &serde_json::Value) -> PyResult<PyObject> {
    Ok(match value {
        serde_json::Value::Null => py.None(),
        serde_json::Value::Bool(b) => b.into_pyobject(py)?.to_owned().into_any().unbind(),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into_pyobject(py)?.into_any().unbind()
            } else if let Some(u) = n.as_u64() {
                u.into_pyobject(py)?.into_any().unbind()
            } else if let Some(f) = n.as_f64() {
                f.into_pyobject(py)?.into_any().unbind()
            } else {
                py.None()
            }
        }
        serde_json::Value::String(s) => s.into_pyobject(py)?.into_any().unbind(),
        serde_json::Value::Array(arr) => {
            let list = PyList::empty(py);
            for item in arr {
                list.append(json_to_py(py, item)?)?;
            }
            list.into_any().unbind()
        }
        serde_json::Value::Object(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map {
                dict.set_item(k, json_to_py(py, v)?)?;
            }
            dict.into_any().unbind()
        }
    })
}

#[pymethods]
impl PyRedstoneGraph {
    /// The nodes of the graph as a list of plain dicts.
    ///
    /// Each dict has keys: `id`, `kind`, `delay` (Repeater only),
    /// `comparator_mode` / `comparator_far_input` (Comparator only),
    /// `pos`, `powered`, `repeater_locked`, `output_strength`,
    /// `facing_diode`.
    #[getter]
    fn nodes(&self, py: Python<'_>) -> PyResult<PyObject> {
        let list = PyList::empty(py);
        for node in &self.inner.nodes {
            let dict = PyDict::new(py);
            dict.set_item("id", node.id)?;
            dict.set_item("kind", node_kind_name(&node.kind))?;

            let (delay, comparator_mode, comparator_far_input) = match &node.kind {
                RedstoneNodeKind::Repeater { delay } => (Some(*delay), None, None),
                RedstoneNodeKind::Comparator { mode, far_input } => {
                    (None, Some(comparator_mode_name(mode)), *far_input)
                }
                _ => (None, None, None),
            };
            dict.set_item("delay", delay)?;
            dict.set_item("comparator_mode", comparator_mode)?;
            dict.set_item("comparator_far_input", comparator_far_input)?;

            dict.set_item("pos", node.pos)?;
            dict.set_item("aliased_blocks", node.aliased_blocks.clone())?;
            dict.set_item("powered", node.powered)?;
            dict.set_item("repeater_locked", node.repeater_locked)?;
            dict.set_item("output_strength", node.output_strength)?;
            dict.set_item("facing_diode", node.facing_diode)?;

            list.append(dict)?;
        }
        Ok(list.into_any().unbind())
    }

    /// The directed edges of the graph as a list of plain dicts.
    ///
    /// Each dict has keys: `source`, `target`, `kind` (`"Default"`/`"Side"`),
    /// `strength`. Edges are oriented source -> target along signal flow
    /// (each incoming link of node `n` becomes `link.from -> n.id`).
    #[getter]
    fn edges(&self, py: Python<'_>) -> PyResult<PyObject> {
        let list = PyList::empty(py);
        for node in &self.inner.nodes {
            for link in &node.inputs {
                let dict = PyDict::new(py);
                dict.set_item("source", link.from)?;
                dict.set_item("target", node.id)?;
                dict.set_item("kind", link_kind_name(&link.kind))?;
                dict.set_item("strength", link.strength)?;
                list.append(dict)?;
            }
        }
        Ok(list.into_any().unbind())
    }

    /// Number of nodes in the graph.
    #[getter]
    fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    /// Total number of directed edges in the graph.
    #[getter]
    fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }

    /// Computed graph features as a plain dict (see `GraphFeatures`).
    fn features(&self, py: Python<'_>) -> PyResult<PyObject> {
        let features = self.inner.features();
        let json = features.to_json().map_err(PyRuntimeError::new_err)?;
        let value: serde_json::Value =
            serde_json::from_str(&json).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        json_to_py(py, &value)
    }

    /// The computed graph features serialized as a JSON string.
    fn features_json(&self) -> PyResult<String> {
        self.inner
            .features()
            .to_json()
            .map_err(PyRuntimeError::new_err)
    }

    /// Compute the structural/functional/exact fingerprint as a hex string.
    ///
    /// `preset` defaults to `"structural"`; raises `ValueError` for an
    /// unknown preset.
    #[pyo3(signature = (preset=None))]
    fn fingerprint(&self, preset: Option<&str>) -> PyResult<String> {
        let preset = preset.unwrap_or("structural");
        let spec = GraphFingerprintSpec::from_preset(preset).ok_or_else(|| {
            PyValueError::new_err(format!(
                "unknown fingerprint preset: {preset:?} (expected \"structural\", \"functional\", or \"exact\")"
            ))
        })?;
        Ok(self.inner.fingerprint(&spec).to_hex())
    }

    /// Serialize the graph to JSON.
    fn to_json(&self) -> PyResult<String> {
        self.inner.to_json().map_err(PyRuntimeError::new_err)
    }

    /// Deserialize a graph from JSON.
    #[staticmethod]
    fn from_json(s: &str) -> PyResult<PyRedstoneGraph> {
        RedstoneGraph::from_json(s)
            .map(|inner| PyRedstoneGraph { inner })
            .map_err(PyRuntimeError::new_err)
    }

    fn __repr__(&self) -> String {
        format!(
            "<RedstoneGraph nodes={} edges={}>",
            self.inner.node_count(),
            self.inner.edge_count()
        )
    }
}
