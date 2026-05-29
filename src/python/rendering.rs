//! Rendering Python bindings
//!
//! GPU-accelerated rendering of schematics to PNG images.

use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::rendering::{self, Projection, RenderConfig, RenderError};

fn render_err_to_py(e: RenderError) -> PyErr {
    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
}

/// Camera projection mode.
#[pyclass(name = "Projection", eq, eq_int)]
#[derive(Clone, Copy, PartialEq)]
pub enum PyProjection {
    Perspective,
    Orthographic,
}

impl From<PyProjection> for Projection {
    fn from(p: PyProjection) -> Self {
        match p {
            PyProjection::Perspective => Projection::Perspective,
            PyProjection::Orthographic => Projection::Orthographic,
        }
    }
}

impl From<Projection> for PyProjection {
    fn from(p: Projection) -> Self {
        match p {
            Projection::Perspective => PyProjection::Perspective,
            Projection::Orthographic => PyProjection::Orthographic,
        }
    }
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
    #[pyo3(signature = (width=1024, height=1024, yaw=45.0, pitch=30.0, zoom=1.0, fov=45.0, target=None, background=None, projection=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        width: u32,
        height: u32,
        yaw: f32,
        pitch: f32,
        zoom: f32,
        fov: f32,
        target: Option<(f32, f32, f32)>,
        background: Option<(f32, f32, f32, f32)>,
        projection: Option<PyProjection>,
    ) -> Self {
        Self {
            inner: RenderConfig {
                width,
                height,
                yaw,
                pitch,
                zoom,
                fov,
                target: target.map(|(x, y, z)| [x, y, z]),
                background: background.map(|(r, g, b, a)| [r, g, b, a]),
                projection: projection.unwrap_or(PyProjection::Perspective).into(),
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

    /// Optional explicit orbit target (world coords). ``None`` aims at the
    /// model's bounding-box centroid.
    #[getter]
    pub fn target(&self) -> Option<(f32, f32, f32)> {
        self.inner.target.map(|t| (t[0], t[1], t[2]))
    }

    #[setter]
    pub fn set_target(&mut self, value: Option<(f32, f32, f32)>) {
        self.inner.target = value.map(|(x, y, z)| [x, y, z]);
    }

    /// Clear any custom orbit target — camera reverts to aiming at the
    /// model's bounding-box centroid. Equivalent to ``self.target = None``.
    pub fn clear_target(&mut self) {
        self.inner.target = None;
    }

    /// Current background as `(r, g, b, a)` in linear 0.0–1.0, or `None`.
    #[getter]
    pub fn background(&self) -> Option<(f32, f32, f32, f32)> {
        self.inner.background.map(|c| (c[0], c[1], c[2], c[3]))
    }

    /// Set a solid RGBA clear color (linear 0.0–1.0). An alpha below 1.0
    /// produces a transparent PNG. Ignored when HDRI is enabled.
    pub fn set_background(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.inner.background = Some([r, g, b, a]);
    }

    /// Clear the custom background — revert to the default sky / HDRI.
    pub fn clear_background(&mut self) {
        self.inner.background = None;
    }

    /// Current projection mode.
    #[getter]
    pub fn projection(&self) -> PyProjection {
        self.inner.projection.into()
    }

    #[setter]
    pub fn set_projection(&mut self, value: PyProjection) {
        self.inner.projection = value.into();
    }

    /// Preset for a true isometric view: orthographic at yaw 45° / pitch ≈35.264°.
    #[staticmethod]
    #[pyo3(signature = (width=1024, height=1024))]
    pub fn isometric(width: u32, height: u32) -> Self {
        let mut inner = RenderConfig::isometric();
        inner.width = width;
        inner.height = height;
        Self { inner }
    }

    fn __repr__(&self) -> String {
        let proj = match self.inner.projection {
            Projection::Perspective => "perspective",
            Projection::Orthographic => "orthographic",
        };
        format!(
            "<RenderConfig {}x{} yaw={} pitch={} zoom={} fov={} projection={} background={:?}>",
            self.inner.width,
            self.inner.height,
            self.inner.yaw,
            self.inner.pitch,
            self.inner.zoom,
            self.inner.fov,
            proj,
            self.inner.background
        )
    }
}

// Rendering methods on PySchematic are in src/python/schematic.rs
// (PyO3 requires all #[pymethods] blocks for a type to be in the same file)
