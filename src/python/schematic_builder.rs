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
        Ok(PySchematic { inner: schematic })
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
