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
