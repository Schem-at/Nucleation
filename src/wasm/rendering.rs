//! Rendering WASM bindings
//!
//! GPU-accelerated rendering of schematics to images via WebGPU.
//! WASM rendering uses wasm_bindgen_futures for async GPU operations.

use js_sys::{Promise, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use crate::rendering::{self, RenderConfig, RenderError};

use super::meshing::ResourcePackWrapper;
use super::schematic::SchematicWrapper;

fn render_err_to_js(e: RenderError) -> JsValue {
    JsValue::from_str(&e.to_string())
}

/// Configuration for GPU rendering.
#[wasm_bindgen]
pub struct RenderConfigWrapper {
    inner: RenderConfig,
}

#[wasm_bindgen]
impl RenderConfigWrapper {
    /// Create a new RenderConfig with default settings.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RenderConfig::default(),
        }
    }

    #[wasm_bindgen(js_name = setWidth)]
    pub fn set_width(&mut self, width: u32) {
        self.inner.width = width;
    }

    #[wasm_bindgen(js_name = setHeight)]
    pub fn set_height(&mut self, height: u32) {
        self.inner.height = height;
    }

    #[wasm_bindgen(js_name = setYaw)]
    pub fn set_yaw(&mut self, yaw: f32) {
        self.inner.yaw = yaw;
    }

    #[wasm_bindgen(js_name = setPitch)]
    pub fn set_pitch(&mut self, pitch: f32) {
        self.inner.pitch = pitch;
    }

    #[wasm_bindgen(js_name = setZoom)]
    pub fn set_zoom(&mut self, zoom: f32) {
        self.inner.zoom = zoom;
    }

    #[wasm_bindgen(js_name = setFov)]
    pub fn set_fov(&mut self, fov: f32) {
        self.inner.fov = fov;
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.inner.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.inner.height
    }

    #[wasm_bindgen(getter)]
    pub fn yaw(&self) -> f32 {
        self.inner.yaw
    }

    #[wasm_bindgen(getter)]
    pub fn pitch(&self) -> f32 {
        self.inner.pitch
    }

    #[wasm_bindgen(getter)]
    pub fn zoom(&self) -> f32 {
        self.inner.zoom
    }

    #[wasm_bindgen(getter)]
    pub fn fov(&self) -> f32 {
        self.inner.fov
    }

    /// Set an explicit orbit target (world-space coordinates). When set,
    /// the camera aims at this point instead of the model's bounding-box
    /// centroid; yaw / pitch / zoom continue to orbit around it.
    #[wasm_bindgen(js_name = setTarget)]
    pub fn set_target(&mut self, x: f32, y: f32, z: f32) {
        self.inner.target = Some([x, y, z]);
    }

    /// Clear any custom orbit target — camera reverts to aiming at the
    /// model's bounding-box centroid.
    #[wasm_bindgen(js_name = clearTarget)]
    pub fn clear_target(&mut self) {
        self.inner.target = None;
    }

    /// Returns the current target as `[x, y, z]`, or `null` if none.
    #[wasm_bindgen(getter, js_name = target)]
    pub fn target(&self) -> Option<Vec<f32>> {
        self.inner.target.map(|t| t.to_vec())
    }

    /// Set a solid RGBA clear color (linear 0.0–1.0). An alpha below 1.0
    /// produces a transparent PNG. Ignored when HDRI is enabled.
    #[wasm_bindgen(js_name = setBackground)]
    pub fn set_background(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.inner.background = Some([r, g, b, a]);
    }

    /// Clear the custom background — revert to the default sky / HDRI.
    #[wasm_bindgen(js_name = clearBackground)]
    pub fn clear_background(&mut self) {
        self.inner.background = None;
    }

    /// Current background as `[r, g, b, a]`, or `null` if none.
    #[wasm_bindgen(getter, js_name = background)]
    pub fn background(&self) -> Option<Vec<f32>> {
        self.inner.background.map(|c| c.to_vec())
    }

    /// Enable or disable orthographic projection (default: perspective).
    #[wasm_bindgen(js_name = setOrthographic)]
    pub fn set_orthographic(&mut self, value: bool) {
        self.inner.projection = if value {
            crate::rendering::Projection::Orthographic
        } else {
            crate::rendering::Projection::Perspective
        };
    }

    /// Whether orthographic projection is enabled.
    #[wasm_bindgen(getter, js_name = orthographic)]
    pub fn orthographic(&self) -> bool {
        matches!(
            self.inner.projection,
            crate::rendering::Projection::Orthographic
        )
    }

    /// Preset for a true isometric view: orthographic at yaw 45° / pitch ≈35.264°.
    #[wasm_bindgen(js_name = isometric)]
    pub fn isometric() -> RenderConfigWrapper {
        RenderConfigWrapper {
            inner: crate::rendering::RenderConfig::isometric(),
        }
    }
}

// Rendering methods on SchematicWrapper

#[wasm_bindgen]
impl SchematicWrapper {
    /// Render the schematic to RGBA pixel bytes.
    /// Returns a Promise<Uint8Array>.
    #[wasm_bindgen(js_name = render)]
    pub fn render_wasm(&self, pack: &ResourcePackWrapper, config: &RenderConfigWrapper) -> Promise {
        // Clone data needed for the async block
        let mesh_config = crate::meshing::MeshConfig::default();
        let mesh_result = self.0.to_mesh(&pack.inner, &mesh_config);
        let render_config = RenderConfig {
            width: config.inner.width,
            height: config.inner.height,
            yaw: config.inner.yaw,
            pitch: config.inner.pitch,
            zoom: config.inner.zoom,
            fov: config.inner.fov,
            target: config.inner.target,
            background: config.inner.background,
            projection: config.inner.projection,
        };

        future_to_promise(async move {
            let mesh = mesh_result.map_err(|e| JsValue::from_str(&e.to_string()))?;
            let meshes = vec![mesh];
            let pixels = rendering::render_meshes_async(&meshes, &render_config, None)
                .await
                .map_err(render_err_to_js)?;
            Ok(Uint8Array::from(pixels.as_slice()).into())
        })
    }

    /// Render the schematic to PNG bytes.
    /// Returns a Promise<Uint8Array>.
    #[wasm_bindgen(js_name = renderPng)]
    pub fn render_png_wasm(
        &self,
        pack: &ResourcePackWrapper,
        config: &RenderConfigWrapper,
    ) -> Promise {
        let mesh_config = crate::meshing::MeshConfig::default();
        let mesh_result = self.0.to_mesh(&pack.inner, &mesh_config);
        let render_config = RenderConfig {
            width: config.inner.width,
            height: config.inner.height,
            yaw: config.inner.yaw,
            pitch: config.inner.pitch,
            zoom: config.inner.zoom,
            fov: config.inner.fov,
            target: config.inner.target,
            background: config.inner.background,
            projection: config.inner.projection,
        };

        future_to_promise(async move {
            let mesh = mesh_result.map_err(|e| JsValue::from_str(&e.to_string()))?;
            let meshes = vec![mesh];
            let pixels = rendering::render_meshes_async(&meshes, &render_config, None)
                .await
                .map_err(render_err_to_js)?;
            let png = rendering::encode_png(&pixels, render_config.width, render_config.height)
                .map_err(render_err_to_js)?;
            Ok(Uint8Array::from(png.as_slice()).into())
        })
    }
}
