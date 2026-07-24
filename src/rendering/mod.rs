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
#[cfg(not(target_arch = "wasm32"))]
pub mod video;

#[cfg(not(target_arch = "wasm32"))]
pub use video::{VideoCodec, VideoConfig};

pub use camera::CameraConfig;
pub use camera::Projection;
pub use gpu::GpuRenderer;
pub use hdri::HdriData;

use crate::meshing::MeshOutput;

/// A world-space reference grid drawn on a horizontal plane, optionally with
/// coloured X/Y/Z axis lines through the origin. Makes block coordinates
/// legible — the point of it is documentation, not decoration.
#[derive(Clone, Copy, Debug)]
pub struct GridConfig {
    /// The grid spans `-half_extent..=half_extent` blocks on each axis unless
    /// `fit_to_bounds` is enabled.
    pub half_extent: i32,
    /// Fit the grid to the rendered block bounds instead of an origin-centred square.
    pub fit_to_bounds: bool,
    /// Whole-block margin around fitted bounds.
    pub margin: i32,
    /// A grid line every `spacing` blocks (clamped to at least 1).
    pub spacing: i32,
    /// Height of the grid plane (usually the build's floor, `0`).
    pub plane_y: f32,
    /// Draw red/green/blue lines along +X/+Y/+Z from the origin.
    pub show_axes: bool,
    /// Grid line colour, linear RGBA. Alpha below 1 blends over the scene.
    pub line_rgba: [f32; 4],
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            half_extent: 16,
            fit_to_bounds: false,
            margin: 1,
            spacing: 1,
            plane_y: 0.0,
            show_axes: true,
            line_rgba: [0.5, 0.5, 0.55, 0.5],
        }
    }
}

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
    /// Rotation-invariant framing: fit the bounding sphere instead of the
    /// yaw-dependent silhouette, so orbiting cameras (turntables) keep a
    /// constant distance. Default `false`.
    pub sphere_fit: bool,
    /// Optional world-space reference grid. `None` (the default) draws nothing
    /// and leaves rendering bit-for-bit identical to before this existed.
    pub grid: Option<GridConfig>,
    /// World-space direction from the surface toward the directional light.
    pub light_direction: [f32; 3],
    /// Directional-light multiplier. Must be finite and non-negative.
    pub directional_intensity: f32,
    /// Unlit floor for non-HDRI rendering, in `0..=1`.
    pub ambient_light: f32,
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
            sphere_fit: false,
            grid: None,
            light_direction: [0.3, 1.0, 0.5],
            directional_intensity: 1.0,
            ambient_light: 0.4,
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
            sphere_fit: self.sphere_fit,
            light_direction: self.light_direction,
            directional_intensity: self.directional_intensity,
            ambient_light: self.ambient_light,
        }
    }

    pub fn set_directional_light(
        &mut self,
        direction: [f32; 3],
        intensity: f32,
    ) -> Result<(), String> {
        let length_squared = direction.iter().map(|value| value * value).sum::<f32>();
        if !direction.iter().all(|value| value.is_finite())
            || !length_squared.is_finite()
            || length_squared <= f32::EPSILON
            || !intensity.is_finite()
            || intensity < 0.0
        {
            return Err(
                "direction must be finite and non-zero; intensity must be finite and non-negative"
                    .to_string(),
            );
        }
        self.light_direction = direction;
        self.directional_intensity = intensity;
        Ok(())
    }

    pub fn set_ambient_light(&mut self, ambient: f32) -> Result<(), String> {
        if !ambient.is_finite() || !(0.0..=1.0).contains(&ambient) {
            return Err("ambient light must be finite and in 0..=1".to_string());
        }
        self.ambient_light = ambient;
        Ok(())
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
    /// Video encoder setup, streaming, or completion failed.
    VideoEncode(String),
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
            Self::VideoEncode(e) => write!(f, "Video encoding failed: {}", e),
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
    renderer.set_grid(config.grid);
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

/// Render an animation to a sequence of RGBA frames.
///
/// `meshes[i]` is posed by the animation group with id `i`, so callers building
/// a per-block animation must mesh per block. Meshes with no matching group
/// hold the identity pose.
///
/// The GPU renderer, atlas and geometry buffers are built **once** and reused
/// for every frame — only the per-draw uniform buffer is rewritten. That is the
/// difference between rendering an animation and re-rendering a still N times.
///
/// Frame times come from [`crate::animation::Timeline::frame_times`], so the
/// output is deterministic and regenerating it is byte-identical.
#[cfg(not(target_arch = "wasm32"))]
pub fn render_animation_stream(
    meshes: &[MeshOutput],
    frames: &[crate::animation::Frame],
    config: &RenderConfig,
    hdri: Option<&HdriData>,
    mut consume: impl FnMut(usize, &[u8]) -> Result<(), RenderError>,
) -> Result<(), RenderError> {
    pollster::block_on(async {
        let renderer = GpuRenderer::new(meshes, config.width, config.height, hdri).await?;
        renderer.set_grid(config.grid);
        let base = config.to_camera();
        let mut poses = vec![crate::animation::Pose::IDENTITY; meshes.len()];

        for (index, frame) in frames.iter().enumerate() {
            poses
                .iter_mut()
                .for_each(|pose| *pose = crate::animation::Pose::IDENTITY);
            for (id, pose) in &frame.poses {
                if let Some(slot) = poses.get_mut(*id as usize) {
                    *slot = *pose;
                }
            }
            renderer.set_poses(&poses);
            renderer.set_gizmos(&frame.gizmos);

            let camera = match &frame.camera {
                Some(camera_pose) => {
                    let mut frame_config = config.clone();
                    frame_config.yaw += camera_pose.yaw;
                    frame_config.pitch += camera_pose.pitch;
                    frame_config.zoom *= camera_pose.zoom;
                    frame_config.to_camera()
                }
                None => base.clone(),
            };
            let pixels = renderer.render_frame(&camera)?;
            consume(index, &pixels)?;
        }
        Ok(())
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn render_animation(
    meshes: &[MeshOutput],
    frames: &[crate::animation::Frame],
    config: &RenderConfig,
    hdri: Option<&HdriData>,
) -> Result<Vec<Vec<u8>>, RenderError> {
    let mut output = Vec::with_capacity(frames.len());
    render_animation_stream(meshes, frames, config, hdri, |_, pixels| {
        output.push(pixels.to_vec());
        Ok(())
    })?;
    Ok(output)
}

/// Render an animation straight to numbered PNG files (`{prefix}{i:04}.png`).
///
/// The naming matches what `ffmpeg -i 'f%04d.png'` expects, which is how the
/// README media pipeline assembles GIFs.
#[cfg(not(target_arch = "wasm32"))]
pub fn render_animation_to_files(
    meshes: &[MeshOutput],
    frames: &[crate::animation::Frame],
    config: &RenderConfig,
    hdri: Option<&HdriData>,
    prefix: &str,
) -> Result<Vec<String>, RenderError> {
    let mut paths = Vec::with_capacity(frames.len());
    render_animation_stream(meshes, frames, config, hdri, |index, pixels| {
        let path = format!("{prefix}{index:04}.png");
        let png = encode_png(pixels, config.width, config.height)?;
        std::fs::write(&path, &png).map_err(RenderError::Io)?;
        paths.push(path);
        Ok(())
    })?;
    Ok(paths)
}

/// Render and stream an animation directly to FFmpeg without retaining frames.
#[cfg(not(target_arch = "wasm32"))]
pub fn render_animation_to_video(
    meshes: &[MeshOutput],
    frames: &[crate::animation::Frame],
    config: &RenderConfig,
    hdri: Option<&HdriData>,
    video: &VideoConfig,
    output: &std::path::Path,
) -> Result<(), RenderError> {
    let mut encoder = video::VideoEncoder::start(video, config.width, config.height, output)?;
    render_animation_stream(meshes, frames, config, hdri, |_, pixels| {
        encoder.write_frame(pixels)
    })?;
    encoder.finish()
}

/// The exact view-projection matrix used for each frame of an animation.
///
/// GPU-free: the matrices are pure maths over the mesh bounds and per-frame
/// camera. Pair with [`camera::project_point`] to place overlay labels — the
/// compositor asks "where is block P at frame i" and gets a pixel anchor from
/// `project_point(&view_projs[i], p, w, h)`. Because it uses the *same* camera
/// derivation as [`render_animation`], the anchors line up with the pixels.
pub fn animation_view_projs(
    meshes: &[MeshOutput],
    frames: &[crate::animation::Frame],
    config: &RenderConfig,
) -> Vec<[[f32; 4]; 4]> {
    let (bmin, bmax) = camera::merged_bounds(meshes);
    let aspect = config.width as f32 / config.height.max(1) as f32;
    frames
        .iter()
        .map(|frame| {
            let mut cam = config.to_camera();
            if let Some(c) = &frame.camera {
                cam.yaw_deg += c.yaw;
                cam.pitch_deg += c.pitch;
                cam.zoom *= c.zoom;
            }
            camera::compute_view_proj(bmin, bmax, aspect, &cam).0
        })
        .collect()
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

/// Encode deterministic RGBA animation frames as an infinitely looping GIF.
pub fn encode_animation_gif(
    frames: &[Vec<u8>],
    width: u32,
    height: u32,
    fps: f64,
) -> Result<Vec<u8>, RenderError> {
    use image::codecs::gif::{GifEncoder, Repeat as GifRepeat};
    use image::{Delay, Frame};

    let mut bytes = Vec::new();
    {
        let mut encoder = GifEncoder::new(&mut bytes);
        encoder
            .set_repeat(GifRepeat::Infinite)
            .map_err(|e| RenderError::PngEncode(e.to_string()))?;
        let fps = fps.max(1.0).round() as u32;
        for pixels in frames {
            let image =
                image::RgbaImage::from_raw(width, height, pixels.clone()).ok_or_else(|| {
                    RenderError::PngEncode("Failed to create GIF frame from pixels".into())
                })?;
            encoder
                .encode_frame(Frame::from_parts(
                    image,
                    0,
                    0,
                    Delay::from_numer_denom_ms(1000, fps),
                ))
                .map_err(|e| RenderError::PngEncode(e.to_string()))?;
        }
    }
    Ok(bytes)
}

/// Write deterministic RGBA animation frames as an infinitely looping GIF.
pub fn write_animation_gif(
    frames: &[Vec<u8>],
    width: u32,
    height: u32,
    fps: f64,
    path: &str,
) -> Result<(), RenderError> {
    let gif = encode_animation_gif(frames, width, height, fps)?;
    std::fs::write(path, gif).map_err(RenderError::Io)
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
            .mesh_chunks_parallel(pack, &mesh_config, 64, num_cpus())
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
    fn lighting_defaults_preserve_the_legacy_renderer() {
        let c = RenderConfig::default();
        assert_eq!(c.light_direction, [0.3, 1.0, 0.5]);
        assert!((c.directional_intensity - 1.0).abs() < f32::EPSILON);
        assert!((c.ambient_light - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn lighting_setters_validate_physical_inputs() {
        let mut c = RenderConfig::default();
        c.set_directional_light([0.75, 0.4, 0.55], 1.25).unwrap();
        c.set_ambient_light(0.18).unwrap();
        assert_eq!(c.light_direction, [0.75, 0.4, 0.55]);
        assert!((c.directional_intensity - 1.25).abs() < f32::EPSILON);
        assert!((c.ambient_light - 0.18).abs() < f32::EPSILON);

        assert!(c.set_directional_light([0.0; 3], 1.0).is_err());
        assert!(c.set_directional_light([1.0, 0.0, 0.0], -0.1).is_err());
        assert!(c.set_ambient_light(-0.1).is_err());
        assert!(c.set_ambient_light(1.1).is_err());
    }

    #[test]
    fn prores_4444_video_config_emits_an_alpha_preserving_raw_rgba_pipeline() {
        let config = VideoConfig::prores_4444(20.0).unwrap();
        let args = config.ffmpeg_args(840, 840, std::path::Path::new("earth.mov"));
        assert!(args
            .windows(2)
            .any(|pair| pair == ["-pixel_format", "rgba"]));
        assert!(args.windows(2).any(|pair| pair == ["-c:v", "prores_ks"]));
        assert!(args.windows(2).any(|pair| pair == ["-profile:v", "4"]));
        assert!(args
            .windows(2)
            .any(|pair| pair == ["-pix_fmt", "yuva444p10le"]));
    }

    #[test]
    fn video_config_rejects_invalid_rate_and_container() {
        assert!(VideoConfig::prores_4444(0.0).is_err());
        assert!(VideoConfig::h264(0.5).is_err());
        assert!(VideoConfig::h264(f64::NAN).is_err());
        let config = VideoConfig::prores_4444(20.0).unwrap();
        assert!(config
            .validate_output(std::path::Path::new("earth.mp4"))
            .is_err());
    }

    #[test]
    fn animation_gif_encoder_produces_a_looping_gif() {
        let black = vec![0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255];
        let white = vec![255; 16];
        let gif = encode_animation_gif(&[black, white], 2, 2, 20.0).unwrap();
        assert!(gif.starts_with(b"GIF"));
        assert!(gif.windows(11).any(|w| w == b"NETSCAPE2.0"));
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
