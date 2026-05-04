//! Schematic Builder Python bindings
//!
//! ASCII art and template-based schematic construction.

use pyo3::prelude::*;

use super::PySchematic;

#[pyclass(name = "SchematicBuilder")]
pub struct PySchematicBuilder {
    inner: crate::SchematicBuilder,
}

#[pymethods]
impl PySchematicBuilder {
    #[new]
    fn new() -> Self {
        Self {
            inner: crate::SchematicBuilder::new(),
        }
    }

    /// Set the name of the schematic
    fn name<'py>(mut slf: PyRefMut<'py, Self>, name: String) -> PyRefMut<'py, Self> {
        let old_builder = std::mem::replace(&mut slf.inner, crate::SchematicBuilder::new());
        slf.inner = old_builder.name(name);
        slf
    }

    /// Map a character to a block string
    fn map<'py>(mut slf: PyRefMut<'py, Self>, ch: char, block: String) -> PyRefMut<'py, Self> {
        let old_builder = std::mem::replace(&mut slf.inner, crate::SchematicBuilder::new());
        slf.inner = old_builder.map(ch, &block);
        slf
    }

    /// Add multiple layers (list of list of strings)
    fn layers<'py>(mut slf: PyRefMut<'py, Self>, layers: Vec<Vec<String>>) -> PyRefMut<'py, Self> {
        // Convert Vec<Vec<String>> to Vec<&[&str]>
        let layer_refs: Vec<Vec<&str>> = layers
            .iter()
            .map(|layer| layer.iter().map(|s| s.as_str()).collect())
            .collect();
        let layer_slice_refs: Vec<&[&str]> = layer_refs.iter().map(|v| v.as_slice()).collect();
        let old_builder = std::mem::replace(&mut slf.inner, crate::SchematicBuilder::new());
        slf.inner = old_builder.layers(&layer_slice_refs);
        slf
    }

    /// Build the schematic
    fn build(mut slf: PyRefMut<'_, Self>) -> PyResult<PySchematic> {
        let builder = std::mem::replace(&mut slf.inner, crate::SchematicBuilder::new());
        let schematic = builder
            .build()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        Ok(PySchematic::from_inner(schematic))
    }

    /// Append a single layer. Equivalent to ``layers([rows])`` but doesn't
    /// require nesting when you only have one layer.
    fn layer<'py>(mut slf: PyRefMut<'py, Self>, rows: Vec<String>) -> PyRefMut<'py, Self> {
        let row_refs: Vec<&str> = rows.iter().map(|s| s.as_str()).collect();
        let old_builder = std::mem::replace(&mut slf.inner, crate::SchematicBuilder::new());
        slf.inner = old_builder.layer(&row_refs);
        slf
    }

    /// Bulk version of :py:meth:`map` — register many ``(char, block)``
    /// pairs in one call.
    fn palette<'py>(
        mut slf: PyRefMut<'py, Self>,
        mappings: Vec<(char, String)>,
    ) -> PyRefMut<'py, Self> {
        let pairs: Vec<(char, &str)> = mappings.iter().map(|(c, b)| (*c, b.as_str())).collect();
        let old_builder = std::mem::replace(&mut slf.inner, crate::SchematicBuilder::new());
        slf.inner = old_builder.palette(&pairs);
        slf
    }

    /// Set the world offset of the resulting schematic.
    fn offset<'py>(mut slf: PyRefMut<'py, Self>, x: i32, y: i32, z: i32) -> PyRefMut<'py, Self> {
        let old_builder = std::mem::replace(&mut slf.inner, crate::SchematicBuilder::new());
        slf.inner = old_builder.offset(x, y, z);
        slf
    }

    /// Apply the standard palette: ``c`` → gray concrete, `` `` → air, and
    /// the named-direction characters used in the canonical examples.
    fn use_standard_palette<'py>(mut slf: PyRefMut<'py, Self>) -> PyRefMut<'py, Self> {
        let old_builder = std::mem::replace(&mut slf.inner, crate::SchematicBuilder::new());
        slf.inner = old_builder.use_standard_palette();
        slf
    }

    /// Apply the minimal palette (``c`` and `` `` only).
    fn use_minimal_palette<'py>(mut slf: PyRefMut<'py, Self>) -> PyRefMut<'py, Self> {
        let old_builder = std::mem::replace(&mut slf.inner, crate::SchematicBuilder::new());
        slf.inner = old_builder.use_minimal_palette();
        slf
    }

    /// Apply the compact palette (single-glyph blocks for redstone shapes).
    fn use_compact_palette<'py>(mut slf: PyRefMut<'py, Self>) -> PyRefMut<'py, Self> {
        let old_builder = std::mem::replace(&mut slf.inner, crate::SchematicBuilder::new());
        slf.inner = old_builder.use_compact_palette();
        slf
    }

    /// Run pre-build validation. Raises ``ValueError`` if the layered
    /// template is malformed (palette character missing, ragged rows, …).
    fn validate(&self) -> PyResult<()> {
        self.inner
            .validate()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
    }

    /// Serialize the builder back into the canonical template format.
    /// Useful for round-tripping and for showing users what they're
    /// actually about to build.
    fn to_template(&self) -> String {
        self.inner.to_template()
    }

    /// Create from template string
    #[staticmethod]
    fn from_template(template: String) -> PyResult<PySchematicBuilder> {
        let builder = crate::SchematicBuilder::from_template(&template)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        Ok(Self { inner: builder })
    }

    fn __repr__(&self) -> String {
        "<SchematicBuilder>".to_string()
    }
}
