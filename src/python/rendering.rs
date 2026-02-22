//! Rendering Python bindings
//!
//! GPU-accelerated rendering of schematics to PNG images.

use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::rendering::{self, RenderConfig, RenderError};

fn render_err_to_py(e: RenderError) -> PyErr {
    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
}

/// Configuration for GPU rendering.
#[pyclass(name = "RenderConfig")]
#[derive(Clone)]
pub struct PyRenderConfig {
    pub(crate) inner: RenderConfig,
}

#[pymethods]
impl PyRenderConfig {
    /// Create a new RenderConfig with default settings.
    #[new]
    #[pyo3(signature = (width=1024, height=1024, yaw=45.0, pitch=30.0, zoom=1.0, fov=45.0))]
    pub fn new(width: u32, height: u32, yaw: f32, pitch: f32, zoom: f32, fov: f32) -> Self {
        Self {
            inner: RenderConfig {
                width,
                height,
                yaw,
                pitch,
                zoom,
                fov,
            },
        }
    }

    #[getter]
    pub fn width(&self) -> u32 {
        self.inner.width
    }

    #[setter]
    pub fn set_width(&mut self, value: u32) {
        self.inner.width = value;
    }

    #[getter]
    pub fn height(&self) -> u32 {
        self.inner.height
    }

    #[setter]
    pub fn set_height(&mut self, value: u32) {
        self.inner.height = value;
    }

    #[getter]
    pub fn yaw(&self) -> f32 {
        self.inner.yaw
    }

    #[setter]
    pub fn set_yaw(&mut self, value: f32) {
        self.inner.yaw = value;
    }

    #[getter]
    pub fn pitch(&self) -> f32 {
        self.inner.pitch
    }

    #[setter]
    pub fn set_pitch(&mut self, value: f32) {
        self.inner.pitch = value;
    }

    #[getter]
    pub fn zoom(&self) -> f32 {
        self.inner.zoom
    }

    #[setter]
    pub fn set_zoom(&mut self, value: f32) {
        self.inner.zoom = value;
    }

    #[getter]
    pub fn fov(&self) -> f32 {
        self.inner.fov
    }

    #[setter]
    pub fn set_fov(&mut self, value: f32) {
        self.inner.fov = value;
    }

    fn __repr__(&self) -> String {
        format!(
            "<RenderConfig {}x{} yaw={} pitch={} zoom={} fov={}>",
            self.inner.width,
            self.inner.height,
            self.inner.yaw,
            self.inner.pitch,
            self.inner.zoom,
            self.inner.fov
        )
    }
}

// Rendering methods on PySchematic are in src/python/schematic.rs
// (PyO3 requires all #[pymethods] blocks for a type to be in the same file)
