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
