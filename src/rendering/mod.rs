//! GPU rendering module for Nucleation schematics.
//!
//! Renders schematics to RGBA pixels or PNG images using wgpu. Supports
//! optional HDRI environment maps for skybox and image-based lighting.
//!
//! # Feature gate
//! This module requires the `rendering` feature, which implies `meshing`.
//!
//! # Platform notes
//! - Native (Linux/macOS/Windows): synchronous API via `pollster`
//! - WASM: async API returning `Promise<Uint8Array>`

pub mod camera;
pub mod gpu;
pub mod hdri;

pub use camera::CameraConfig;
pub use camera::Projection;
pub use gpu::GpuRenderer;
pub use hdri::HdriData;

use crate::meshing::MeshOutput;

/// Render configuration.
#[derive(Clone)]
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub fov: f32,
    /// Optional explicit orbit target in world coordinates. When ``None``
    /// the camera aims at the model's bounding-box centroid.
    pub target: Option<[f32; 3]>,
    /// Optional solid RGBA clear color (linear 0.0–1.0). `None` keeps the
    /// default sky-blue clear (or the HDRI sky when HDRI is enabled). An
    /// alpha below 1.0 produces a transparent PNG. Ignored when HDRI is on.
    pub background: Option<[f32; 4]>,
    /// Camera projection mode (default `Perspective`).
    pub projection: Projection,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 1024,
            yaw: 45.0,
            pitch: 30.0,
            zoom: 1.0,
            fov: 45.0,
            target: None,
            background: None,
            projection: Projection::Perspective,
        }
    }
}

impl RenderConfig {
    fn to_camera(&self) -> CameraConfig {
        CameraConfig {
            yaw_deg: self.yaw,
            pitch_deg: self.pitch,
            zoom: self.zoom,
            fov_deg: self.fov,
            target: self.target,
            projection: self.projection,
            background: self.background,
        }
    }

    /// A config preset for a true isometric view: orthographic projection at
    /// yaw 45° and pitch ≈35.264° (`arctan(1/√2)`).
    pub fn isometric() -> Self {
        Self {
            yaw: 45.0,
            pitch: 35.264,
            projection: Projection::Orthographic,
            ..Self::default()
        }
    }
}

/// Errors that can occur during rendering.
#[derive(Debug)]
pub enum RenderError {
    /// No GPU adapter found (neither hardware nor software fallback).
    NoGpuAdapter,
    /// Failed to create GPU device.
    DeviceCreation(String),
    /// Render pass or readback failed.
    RenderFailed(String),
    /// PNG encoding failed.
    PngEncode(String),
    /// I/O error.
    Io(std::io::Error),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoGpuAdapter => write!(
                f,
                "No GPU adapter found. Neither hardware nor software rendering is available."
            ),
            Self::DeviceCreation(e) => write!(f, "Failed to create GPU device: {}", e),
            Self::RenderFailed(e) => write!(f, "Render failed: {}", e),
            Self::PngEncode(e) => write!(f, "PNG encoding failed: {}", e),
            Self::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for RenderError {}

// ─── Core async API ─────────────────────────────────────────────────────────

/// Render meshes to RGBA pixels (async, works on all platforms).
pub async fn render_meshes_async(
    meshes: &[MeshOutput],
    config: &RenderConfig,
    hdri: Option<&HdriData>,
) -> Result<Vec<u8>, RenderError> {
    let renderer = GpuRenderer::new(meshes, config.width, config.height, hdri).await?;
    let camera = config.to_camera();
    // Use render_frame on native, which does sync readback
    #[cfg(not(target_arch = "wasm32"))]
    {
        renderer.render_frame(&camera)
    }
    #[cfg(target_arch = "wasm32")]
    {
        Err(RenderError::RenderFailed(
            "Use render_meshes_async with wasm_bindgen_futures on WASM".into(),
        ))
    }
}

// ─── Sync wrappers (native only) ────────────────────────────────────────────

/// Render meshes to RGBA pixels (synchronous, native only).
#[cfg(not(target_arch = "wasm32"))]
pub fn render_meshes(
    meshes: &[MeshOutput],
    config: &RenderConfig,
    hdri: Option<&HdriData>,
) -> Result<Vec<u8>, RenderError> {
    pollster::block_on(render_meshes_async(meshes, config, hdri))
}

/// Render meshes to PNG bytes (synchronous, native only).
#[cfg(not(target_arch = "wasm32"))]
pub fn render_meshes_png(
    meshes: &[MeshOutput],
    config: &RenderConfig,
    hdri: Option<&HdriData>,
) -> Result<Vec<u8>, RenderError> {
    let pixels = render_meshes(meshes, config, hdri)?;
    encode_png(&pixels, config.width, config.height)
}

/// Encode RGBA pixels to PNG bytes.
pub fn encode_png(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>, RenderError> {
    let img = image::RgbaImage::from_raw(width, height, pixels.to_vec())
        .ok_or_else(|| RenderError::PngEncode("Failed to create image from pixels".into()))?;
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| RenderError::PngEncode(e.to_string()))?;
    Ok(buf.into_inner())
}

// ─── High-level API on UniversalSchematic ───────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
impl crate::UniversalSchematic {
    /// Render this schematic to RGBA pixels.
    pub fn render(
        &self,
        pack: &crate::meshing::ResourcePackSource,
        config: &RenderConfig,
    ) -> Result<Vec<u8>, RenderError> {
        let mesh_config = crate::meshing::MeshConfig::default();
        let meshes = self
            .mesh_chunks_parallel(&pack, &mesh_config, 64, num_cpus())
            .map_err(|e| RenderError::RenderFailed(e.to_string()))?;
        render_meshes(&meshes, config, None)
    }

    /// Render this schematic to PNG bytes.
    pub fn render_png(
        &self,
        pack: &crate::meshing::ResourcePackSource,
        config: &RenderConfig,
    ) -> Result<Vec<u8>, RenderError> {
        let pixels = self.render(pack, config)?;
        encode_png(&pixels, config.width, config.height)
    }

    /// Render this schematic and save as a PNG file.
    pub fn render_to_file(
        &self,
        pack: &crate::meshing::ResourcePackSource,
        path: &str,
        config: &RenderConfig,
    ) -> Result<(), RenderError> {
        let png = self.render_png(pack, config)?;
        std::fs::write(path, &png).map_err(RenderError::Io)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use crate::rendering::camera::Projection;

    #[test]
    fn default_config_is_perspective_no_background() {
        let c = RenderConfig::default();
        assert_eq!(c.projection, Projection::Perspective);
        assert!(c.background.is_none());
    }

    #[test]
    fn isometric_sets_ortho_and_angles() {
        let c = RenderConfig::isometric();
        assert_eq!(c.projection, Projection::Orthographic);
        assert!((c.yaw - 45.0).abs() < 1e-4);
        assert!((c.pitch - 35.264).abs() < 1e-3);
    }

    #[test]
    fn to_camera_propagates_projection_and_background() {
        let mut c = RenderConfig::default();
        c.projection = Projection::Orthographic;
        c.background = Some([1.0, 0.0, 0.0, 0.5]);
        let cam = c.to_camera();
        assert_eq!(cam.projection, Projection::Orthographic);
        assert_eq!(cam.background, Some([1.0, 0.0, 0.0, 0.5]));
    }
}
