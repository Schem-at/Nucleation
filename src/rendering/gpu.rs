//! GPU renderer using wgpu.
//!
//! Extracted from `examples/render_schematic.rs`. Supports headless (offscreen)
//! and windowed rendering with optional HDRI environment maps.

use wgpu::util::DeviceExt;

use crate::meshing::{MeshLayer, MeshOutput};

use super::camera::{compute_view_proj, merged_bounds, CameraConfig};
use super::hdri::HdriData;
use super::RenderError;

const SHADER_SRC: &str = include_str!("shader.wgsl");

/// Choose the render-pass clear color. A custom `background` (linear RGBA)
/// always wins; otherwise black when an HDRI sky is drawn, else the default
/// sky-blue.
fn clear_color_for(background: Option<[f32; 4]>, hdri_enabled: bool) -> wgpu::Color {
    if let Some(bg) = background {
        wgpu::Color {
            r: bg[0] as f64,
            g: bg[1] as f64,
            b: bg[2] as f64,
            a: bg[3] as f64,
        }
    } else if hdri_enabled {
        wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    } else {
        wgpu::Color {
            r: 0.529,
            g: 0.808,
            b: 0.922,
            a: 1.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    inv_view_proj: [[f32; 4]; 4],
    params: [f32; 4], // x = alpha_cutoff, y = hdri_enabled, z = hdri_intensity
}

/// Per-draw animation state, mirroring `DrawUniforms` in the shader.
///
/// WGSL aligns each `mat3x3` column to 16 bytes, so the normal matrix is stored
/// as three padded `vec4`s rather than three `vec3`s.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct DrawUniforms {
    model: [[f32; 4]; 4],
    normal_mat: [[f32; 4]; 3],
    tint: [f32; 4],
    emissive: [f32; 4],
}

impl DrawUniforms {
    /// The no-op value: identity transform, neutral tint, no emission.
    fn identity() -> Self {
        DrawUniforms {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            normal_mat: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ],
            tint: [1.0, 1.0, 1.0, 1.0],
            emissive: [0.0; 4],
        }
    }

    fn from_pose(p: &crate::animation::Pose) -> Self {
        let n = p.normal_matrix();
        let mut tint = p.tint;
        // Opacity folds into tint alpha so the shader has one alpha source.
        tint[3] *= p.opacity;
        DrawUniforms {
            model: p.to_matrix(),
            normal_mat: [
                [n[0][0], n[0][1], n[0][2], 0.0],
                [n[1][0], n[1][1], n[1][2], 0.0],
                [n[2][0], n[2][1], n[2][2], 0.0],
            ],
            tint,
            emissive: p.emissive,
        }
    }
}

/// Interleaved `[x, y, z, r, g, b, a]` line vertices for a reference grid and,
/// optionally, coloured axes through the origin.
fn build_grid_vertices(
    cfg: &super::GridConfig,
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
) -> Vec<f32> {
    let mut v = Vec::new();
    let mut push = |p: [f32; 3], c: [f32; 4]| {
        v.extend_from_slice(&[p[0], p[1], p[2], c[0], c[1], c[2], c[3]]);
    };
    let ext = cfg.half_extent.max(1);
    let margin = cfg.margin.max(0) as f32;
    // Block models are centred on integer schematic coordinates, so their
    // boundaries — and therefore world-grid lines — live on half-integers.
    let (min_x, max_x, min_z, max_z) = if cfg.fit_to_bounds {
        (
            bounds_min[0].floor() - margin - 0.5,
            bounds_max[0].ceil() + margin - 0.5,
            bounds_min[2].floor() - margin - 0.5,
            bounds_max[2].ceil() + margin - 0.5,
        )
    } else {
        let e = ext as f32;
        (-e - 0.5, e + 0.5, -e - 0.5, e + 0.5)
    };
    let step = cfg.spacing.max(1) as f32;
    let y = cfg.plane_y;
    let col = cfg.line_rgba;

    let mut z = min_z;
    while z <= max_z + f32::EPSILON {
        push([min_x, y, z], col);
        push([max_x, y, z], col);
        z += step;
    }
    let mut x = min_x;
    while x <= max_x + f32::EPSILON {
        push([x, y, min_z], col);
        push([x, y, max_z], col);
        x += step;
    }

    if cfg.show_axes {
        let axis_extent = max_x
            .abs()
            .max(min_x.abs())
            .max(max_z.abs())
            .max(min_z.abs());
        // Blocks are centred on integer coordinates, so the world-coordinate
        // origin lies at the minimum X/Z corner of block (0, 0, 0). Keep the
        // marker and coloured axes on the same half-integer lines as the grid.
        let origin = -0.5;
        push([origin, y, origin], [0.9, 0.2, 0.2, 1.0]);
        push([axis_extent, y, origin], [0.9, 0.2, 0.2, 1.0]);
        push([origin, y, origin], [0.2, 0.8, 0.3, 1.0]);
        push([origin, y + axis_extent, origin], [0.2, 0.8, 0.3, 1.0]);
        push([origin, y, origin], [0.3, 0.5, 0.95, 1.0]);
        push([origin, y, axis_extent], [0.3, 0.5, 0.95, 1.0]);
    }
    v
}

#[cfg(test)]
mod grid_tests {
    use super::*;

    #[test]
    fn fitted_grid_uses_half_integer_block_boundaries_with_a_small_margin() {
        let cfg = super::super::GridConfig {
            fit_to_bounds: true,
            margin: 1,
            show_axes: false,
            ..Default::default()
        };
        // Mesher metadata uses [min block coordinate, max coordinate + 1], while
        // the actual models are centred on integer coordinates and therefore
        // occupy n-0.5..n+0.5.
        let vertices = build_grid_vertices(&cfg, [-3.0, 0.0, -2.0], [3.0, 2.0, 3.0]);
        let points: Vec<[f32; 3]> = vertices
            .chunks_exact(7)
            .map(|v| [v[0], v[1], v[2]])
            .collect();
        assert_eq!(
            points.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min),
            -4.5
        );
        assert_eq!(
            points
                .iter()
                .map(|p| p[0])
                .fold(f32::NEG_INFINITY, f32::max),
            3.5
        );
        assert_eq!(
            points.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min),
            -3.5
        );
        assert_eq!(
            points
                .iter()
                .map(|p| p[2])
                .fold(f32::NEG_INFINITY, f32::max),
            3.5
        );
    }

    #[test]
    fn grid_vertex_count_matches_the_line_count() {
        let cfg = super::super::GridConfig {
            half_extent: 4,
            spacing: 1,
            plane_y: -0.02,
            show_axes: true,
            line_rgba: [1.0; 4],
            ..Default::default()
        };
        let v = build_grid_vertices(&cfg, [-4.0, 0.0, -4.0], [4.0, 1.0, 4.0]);
        // 7 floats per vertex, 2 vertices per line.
        assert_eq!(v.len() % 7, 0, "each vertex is 7 floats");
        let verts = v.len() / 7;
        // Centres -4..=4 occupy cells bounded by -4.5..=4.5: 10 X-lines
        // and 10 Z-lines, plus 3 axes = 23 lines = 46 vertices.
        assert_eq!(verts, 46);
    }

    #[test]
    fn axes_start_on_the_origin_block_boundary() {
        let cfg = super::super::GridConfig {
            half_extent: 2,
            spacing: 1,
            plane_y: -0.502,
            show_axes: true,
            line_rgba: [1.0; 4],
            ..Default::default()
        };
        let vertices = build_grid_vertices(&cfg, [-1.0, 0.0, -1.0], [2.0, 2.0, 2.0]);
        let points: Vec<[f32; 3]> = vertices
            .chunks_exact(7)
            .map(|v| [v[0], v[1], v[2]])
            .collect();
        let axes = &points[points.len() - 6..];

        // Block (0, 0, 0) occupies -0.5..0.5 on X/Z. The origin marker and
        // all three positive axes must meet at its minimum corner, on actual
        // grid lines, rather than piercing the block centre at (0, 0).
        let origin = [-0.5, cfg.plane_y, -0.5];
        assert_eq!(axes[0], origin);
        assert_eq!(axes[2], origin);
        assert_eq!(axes[4], origin);
        assert_eq!(axes[1], [2.5, cfg.plane_y, -0.5]);
        assert_eq!(axes[3], [-0.5, cfg.plane_y + 2.5, -0.5]);
        assert_eq!(axes[5], [-0.5, cfg.plane_y, 2.5]);
    }

    #[test]
    fn axes_can_be_disabled_and_spacing_thins_the_grid() {
        let base = super::super::GridConfig {
            half_extent: 6,
            spacing: 1,
            plane_y: 0.0,
            show_axes: false,
            line_rgba: [1.0; 4],
            ..Default::default()
        };
        let bounds = ([-6.0, 0.0, -6.0], [6.0, 1.0, 6.0]);
        let dense = build_grid_vertices(&base, bounds.0, bounds.1).len();
        let sparse = build_grid_vertices(
            &super::super::GridConfig { spacing: 3, ..base },
            bounds.0,
            bounds.1,
        )
        .len();
        assert!(sparse < dense, "larger spacing draws fewer lines");
    }

    #[test]
    fn degenerate_spacing_does_not_hang() {
        // spacing 0 is clamped to 1 rather than looping forever.
        let v = build_grid_vertices(
            &super::super::GridConfig {
                half_extent: 2,
                spacing: 0,
                plane_y: 0.0,
                show_axes: false,
                line_rgba: [1.0; 4],
                ..Default::default()
            },
            [-2.0, 0.0, -2.0],
            [2.0, 1.0, 2.0],
        );
        assert!(!v.is_empty());
    }
}

struct LayerBuffers {
    positions: wgpu::Buffer,
    normals: wgpu::Buffer,
    uvs: wgpu::Buffer,
    colors: wgpu::Buffer,
    indices: wgpu::Buffer,
    index_count: u32,
}

struct ChunkGpuData {
    texture_bg: wgpu::BindGroup,
    opaque: Option<LayerBuffers>,
    cutout: Option<LayerBuffers>,
    transparent: Option<LayerBuffers>,
}

/// GPU renderer for schematics. Supports both headless and windowed modes.
///
/// For headless rendering, use [`GpuRenderer::new`]. For windowed rendering
/// (interactive viewer), use [`GpuRenderer::new_windowed`].
pub struct GpuRenderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub color_format: wgpu::TextureFormat,
    uniform_bgl: wgpu::BindGroupLayout,
    opaque_pipeline: wgpu::RenderPipeline,
    cutout_pipeline: wgpu::RenderPipeline,
    transparent_pipeline: wgpu::RenderPipeline,
    sky_pipeline: Option<wgpu::RenderPipeline>,
    hdri_bg: wgpu::BindGroup,
    hdri_enabled: bool,
    dummy_atlas_bg: wgpu::BindGroup,
    // Per-draw animation state, addressed with a dynamic offset per chunk.
    draw_buf: wgpu::Buffer,
    draw_bg: wgpu::BindGroup,
    draw_stride: u32,
    // Optional world-space reference grid.
    line_pipeline: wgpu::RenderPipeline,
    grid: std::cell::Cell<Option<super::GridConfig>>,
    chunks_gpu: Vec<ChunkGpuData>,
    // Headless-only (None in windowed mode)
    render_target: Option<wgpu::Texture>,
    render_target_view: Option<wgpu::TextureView>,
    staging_buffer: Option<wgpu::Buffer>,
    pub depth_view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
    padded_bytes_per_row: u32,
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
}

impl GpuRenderer {
    /// Headless constructor — creates its own wgpu instance with graceful GPU fallback.
    pub async fn new(
        meshes: &[MeshOutput],
        width: u32,
        height: u32,
        hdri: Option<&HdriData>,
    ) -> Result<Self, RenderError> {
        // wgpu 30: `InstanceDescriptor` is passed by value and has no `Default`
        // (its `display` field is a boxed trait object), so spell it out.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });
        Self::create(meshes, width, height, hdri, &instance, None).await
    }

    /// Windowed constructor — caller provides instance + surface.
    pub async fn new_windowed(
        meshes: &[MeshOutput],
        instance: &wgpu::Instance,
        surface: &wgpu::Surface<'_>,
        width: u32,
        height: u32,
        hdri: Option<&HdriData>,
    ) -> Result<Self, RenderError> {
        Self::create(meshes, width, height, hdri, instance, Some(surface)).await
    }

    async fn create(
        meshes: &[MeshOutput],
        width: u32,
        height: u32,
        hdri: Option<&HdriData>,
        instance: &wgpu::Instance,
        surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Self, RenderError> {
        // Graceful GPU fallback: hardware → software → error
        // wgpu 30: `request_adapter` returns `Result`, not `Option`.
        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: surface,
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
        {
            Ok(a) => a,
            Err(_) => {
                // Try software fallback
                instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::LowPower,
                        compatible_surface: surface,
                        force_fallback_adapter: true,
                        ..Default::default()
                    })
                    .await
                    .map_err(|_| RenderError::NoGpuAdapter)?
            }
        };

        // wgpu 30: single-argument `request_device` (the trace path moved into
        // the descriptor), and `DeviceDescriptor` has no `Default`.
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("render_device"),
                required_features: wgpu::Features::FLOAT32_FILTERABLE,
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|e| RenderError::DeviceCreation(e.to_string()))?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("render_shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        // --- Bind group layouts ---
        let uniform_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let texture_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let hdri_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("hdri_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Per-draw uniforms: one aligned slot per mesh, selected with a dynamic
        // offset so the whole scene needs a single buffer and bind group.
        let draw_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("draw_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<DrawUniforms>() as u64
                    ),
                },
                count: None,
            }],
        });

        let mesh_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh_pipeline_layout"),
            // wgpu 30: layouts are `Option`-wrapped (gaps allowed) and
            // `push_constant_ranges` was replaced by immediates.
            bind_group_layouts: &[
                Some(&uniform_bgl),
                Some(&texture_bgl),
                Some(&hdri_bgl),
                Some(&draw_bgl),
            ],
            ..Default::default()
        });

        let sky_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky_pipeline_layout"),
            // wgpu 30: layouts are `Option`-wrapped (gaps allowed) and
            // `push_constant_ranges` was replaced by immediates.
            bind_group_layouts: &[Some(&uniform_bgl), Some(&texture_bgl), Some(&hdri_bgl)],
            ..Default::default()
        });

        // --- HDRI texture (or 1x1 dummy) ---
        let hdri_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("hdri_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let (hdri_tex_view, hdri_enabled) = if let Some(hdr) = hdri {
            let size = wgpu::Extent3d {
                width: hdr.width,
                height: hdr.height,
                depth_or_array_layers: 1,
            };
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("hdri"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let pixel_bytes: &[u8] = bytemuck::cast_slice(&hdr.pixels_rgba32f);
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                pixel_bytes,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(hdr.width * 16),
                    rows_per_image: Some(hdr.height),
                },
                size,
            );
            (
                tex.create_view(&wgpu::TextureViewDescriptor::default()),
                true,
            )
        } else {
            let size = wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            };
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("hdri_dummy"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                bytemuck::cast_slice(&[0.0f32, 0.0, 0.0, 1.0]),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(16),
                    rows_per_image: Some(1),
                },
                size,
            );
            (
                tex.create_view(&wgpu::TextureViewDescriptor::default()),
                false,
            )
        };

        let hdri_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hdri_bg"),
            layout: &hdri_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&hdri_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&hdri_sampler),
                },
            ],
        });

        // --- Vertex buffer layout ---
        // wgpu 30: vertex buffer slots are optional, so each layout is `Some(..)`.
        let vertex_buffer_layouts = [
            Some(wgpu::VertexBufferLayout {
                array_stride: 12,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                }],
            }),
            Some(wgpu::VertexBufferLayout {
                array_stride: 12,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 1,
                }],
            }),
            Some(wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 2,
                }],
            }),
            Some(wgpu::VertexBufferLayout {
                array_stride: 16,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3,
                }],
            }),
        ];

        // --- Color format ---
        let color_format = if let Some(s) = surface {
            let caps = s.get_capabilities(&adapter);
            caps.formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(caps.formats[0])
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };
        let depth_format = wgpu::TextureFormat::Depth32Float;

        // --- Render targets (headless only) ---
        let (render_target, render_target_view) = if surface.is_none() {
            let rt = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("render_target"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: color_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let rtv = rt.create_view(&wgpu::TextureViewDescriptor::default());
            (Some(rt), Some(rtv))
        } else {
            (None, None)
        };

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: depth_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // --- Pipelines ---
        let make_mesh_pipeline =
            |label: &str, blend: Option<wgpu::BlendState>, depth_write: bool, cull_back: bool| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(&mesh_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &vertex_buffer_layouts,
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: color_format,
                            blend,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: if cull_back {
                            Some(wgpu::Face::Back)
                        } else {
                            None
                        },
                        ..Default::default()
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: depth_format,
                        depth_write_enabled: Some(depth_write),
                        depth_compare: Some(wgpu::CompareFunction::Less),
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                    multisample: Default::default(),
                    multiview_mask: None,
                    cache: None,
                })
            };

        let opaque_pipeline = make_mesh_pipeline("opaque", None, true, true);
        let cutout_pipeline = make_mesh_pipeline("cutout", None, true, true);
        let transparent_pipeline = make_mesh_pipeline(
            "transparent",
            Some(wgpu::BlendState::ALPHA_BLENDING),
            false,
            false,
        );

        let sky_pipeline = if hdri_enabled {
            Some(
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("skybox"),
                    layout: Some(&sky_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_sky"),
                        buffers: &[],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_sky"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: color_format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        ..Default::default()
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: depth_format,
                        depth_write_enabled: Some(false),
                        depth_compare: Some(wgpu::CompareFunction::LessEqual),
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                    multisample: Default::default(),
                    multiview_mask: None,
                    cache: None,
                }),
            )
        } else {
            None
        };

        // --- Grid/axis line pipeline (group 0 = view-projection only) ---
        let line_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("line_pipeline_layout"),
            bind_group_layouts: &[Some(&uniform_bgl)],
            ..Default::default()
        });
        let line_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("grid_lines"),
            layout: Some(&line_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_line"),
                // Interleaved position(3) + colour(4) = 28-byte stride.
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 28,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 12,
                            shader_location: 1,
                        },
                    ],
                })],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_line"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            // Depth-tested so blocks occlude grid lines behind them, but no
            // depth write so the grid does not disturb transparent sorting.
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });

        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("atlas_sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        // --- Upload chunk data ---
        let upload_layer = |layer: &MeshLayer, label: &str| -> Option<LayerBuffers> {
            if layer.is_empty() {
                return None;
            }
            Some(LayerBuffers {
                positions: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{}_pos", label)),
                    contents: layer.positions_bytes(),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
                normals: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{}_norm", label)),
                    contents: layer.normals_bytes(),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
                uvs: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{}_uv", label)),
                    contents: layer.uvs_bytes(),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
                colors: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{}_col", label)),
                    contents: layer.colors_bytes(),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
                indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{}_idx", label)),
                    contents: layer.indices_bytes(),
                    usage: wgpu::BufferUsages::INDEX,
                }),
                index_count: layer.indices.len() as u32,
            })
        };

        let chunks_gpu: Vec<ChunkGpuData> = meshes
            .iter()
            .enumerate()
            .map(|(i, mesh)| {
                let atlas_size = wgpu::Extent3d {
                    width: mesh.atlas.width,
                    height: mesh.atlas.height,
                    depth_or_array_layers: 1,
                };
                let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(&format!("atlas_{}", i)),
                    size: atlas_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &atlas_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &mesh.atlas.pixels,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * mesh.atlas.width),
                        rows_per_image: Some(mesh.atlas.height),
                    },
                    atlas_size,
                );
                let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
                let texture_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("tex_bg_{}", i)),
                    layout: &texture_bgl,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&atlas_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                        },
                    ],
                });

                let label = format!("c{}", i);
                ChunkGpuData {
                    texture_bg,
                    opaque: upload_layer(&mesh.opaque, &format!("{}_opaque", label)),
                    cutout: upload_layer(&mesh.cutout, &format!("{}_cutout", label)),
                    transparent: upload_layer(&mesh.transparent, &format!("{}_transparent", label)),
                }
            })
            .collect();

        // Dummy atlas bind group for skybox pass
        let dummy_atlas_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("dummy_atlas"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let dummy_atlas_view = dummy_atlas_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let dummy_atlas_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("dummy_atlas_bg"),
            layout: &texture_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&dummy_atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
            ],
        });

        // Staging buffer (headless only)
        let (staging_buffer, padded_bytes_per_row) = if surface.is_none() {
            let bytes_per_pixel = 4u32;
            let unpadded_bytes_per_row = bytes_per_pixel * width;
            let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let padded = unpadded_bytes_per_row.div_ceil(align) * align;
            let sb = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("staging"),
                size: (padded * height) as u64,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            (Some(sb), padded)
        } else {
            (None, 0)
        };

        let (bounds_min, bounds_max) = merged_bounds(meshes);

        // One aligned slot per mesh. Alignment comes from the device rather
        // than a hardcoded 256, so this is correct on every backend.
        let align = device.limits().min_uniform_buffer_offset_alignment;
        let draw_stride = {
            let sz = std::mem::size_of::<DrawUniforms>() as u32;
            sz.div_ceil(align) * align
        };
        let slots = chunks_gpu.len().max(1) as u64;
        let identity = DrawUniforms::identity();
        let mut init = vec![0u8; (draw_stride as u64 * slots) as usize];
        for i in 0..slots as usize {
            let off = i * draw_stride as usize;
            init[off..off + std::mem::size_of::<DrawUniforms>()]
                .copy_from_slice(bytemuck::bytes_of(&identity));
        }
        let draw_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("draw_uniforms"),
            contents: &init,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let draw_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("draw_bg"),
            layout: &draw_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &draw_buf,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<DrawUniforms>() as u64),
                }),
            }],
        });

        Ok(Self {
            device,
            queue,
            color_format,
            uniform_bgl,
            opaque_pipeline,
            cutout_pipeline,
            transparent_pipeline,
            sky_pipeline,
            hdri_bg,
            hdri_enabled,
            dummy_atlas_bg,
            draw_buf,
            draw_bg,
            draw_stride,
            line_pipeline,
            grid: std::cell::Cell::new(None),
            chunks_gpu,
            render_target,
            render_target_view,
            staging_buffer,
            depth_view,
            width,
            height,
            padded_bytes_per_row,
            bounds_min,
            bounds_max,
        })
    }

    /// Encode the full render pass into `encoder`.
    pub fn render_to_view(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        camera: &CameraConfig,
        width: u32,
        height: u32,
    ) {
        let aspect = width as f32 / height as f32;
        let (view_proj, inv_view_proj) =
            compute_view_proj(self.bounds_min, self.bounds_max, aspect, camera);

        let hdri_intensity = if self.hdri_enabled { 2.5f32 } else { 0.0 };
        let hdri_flag = if self.hdri_enabled { 1.0f32 } else { 0.0 };

        let make_uniforms = |alpha_cutoff: f32| -> Uniforms {
            Uniforms {
                view_proj,
                inv_view_proj,
                params: [alpha_cutoff, hdri_flag, hdri_intensity, 0.0],
            }
        };

        let make_ub = |uniforms: &Uniforms, label: &str| -> (wgpu::Buffer, wgpu::BindGroup) {
            let buf = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    contents: bytemuck::bytes_of(uniforms),
                    usage: wgpu::BufferUsages::UNIFORM,
                });
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &self.uniform_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buf.as_entire_binding(),
                }],
            });
            (buf, bg)
        };

        let opaque_u = make_uniforms(0.0);
        let cutout_u = make_uniforms(0.5);
        let transparent_u = make_uniforms(0.0);
        let sky_u = make_uniforms(0.0);

        let (_ob, opaque_bg) = make_ub(&opaque_u, "opaque_ub");
        let (_cb, cutout_bg) = make_ub(&cutout_u, "cutout_ub");
        let (_tb, transparent_bg) = make_ub(&transparent_u, "transparent_ub");
        let (_sb, sky_bg) = make_ub(&sky_u, "sky_ub");

        let clear_color = clear_color_for(camera.background, self.hdri_enabled);

        // Build the grid vertex buffer before the pass so it outlives it.
        let grid_lines = self.grid.get().map(|cfg| {
            let verts = build_grid_vertices(&cfg, self.bounds_min, self.bounds_max);
            let count = (verts.len() / 7) as u32;
            let buf = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("grid_lines"),
                    contents: bytemuck::cast_slice(&verts),
                    usage: wgpu::BufferUsages::VERTEX,
                });
            (buf, count)
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                depth_slice: None,
                view: color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });

        // 1) Skybox
        if let Some(ref sky_pl) = self.sky_pipeline {
            pass.set_pipeline(sky_pl);
            pass.set_bind_group(0, &sky_bg, &[]);
            pass.set_bind_group(1, &self.dummy_atlas_bg, &[]);
            pass.set_bind_group(2, &self.hdri_bg, &[]);
            pass.draw(0..3, 0..1);
        }

        // 2) Mesh layers
        let draw_layer = |pass: &mut wgpu::RenderPass,
                          pipeline: &wgpu::RenderPipeline,
                          uniform_bg: &wgpu::BindGroup,
                          texture_bg: &wgpu::BindGroup,
                          bufs: &Option<LayerBuffers>,
                          chunk_index: usize| {
            if let Some(b) = bufs {
                let draw_offset = (chunk_index as u32) * self.draw_stride;
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, uniform_bg, &[]);
                pass.set_bind_group(1, texture_bg, &[]);
                pass.set_bind_group(2, &self.hdri_bg, &[]);
                pass.set_bind_group(3, &self.draw_bg, &[draw_offset]);
                pass.set_vertex_buffer(0, b.positions.slice(..));
                pass.set_vertex_buffer(1, b.normals.slice(..));
                pass.set_vertex_buffer(2, b.uvs.slice(..));
                pass.set_vertex_buffer(3, b.colors.slice(..));
                pass.set_index_buffer(b.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..b.index_count, 0, 0..1);
            }
        };

        for (i, chunk) in self.chunks_gpu.iter().enumerate() {
            draw_layer(
                &mut pass,
                &self.opaque_pipeline,
                &opaque_bg,
                &chunk.texture_bg,
                &chunk.opaque,
                i,
            );
        }
        for (i, chunk) in self.chunks_gpu.iter().enumerate() {
            draw_layer(
                &mut pass,
                &self.cutout_pipeline,
                &cutout_bg,
                &chunk.texture_bg,
                &chunk.cutout,
                i,
            );
        }
        // Grid after the opaque/cutout depth is laid down (so blocks occlude it)
        // but before transparent geometry (so glass blends over it).
        if let Some((buf, count)) = &grid_lines {
            pass.set_pipeline(&self.line_pipeline);
            pass.set_bind_group(0, &opaque_bg, &[]);
            pass.set_vertex_buffer(0, buf.slice(..));
            pass.draw(0..*count, 0..1);
        }

        for (i, chunk) in self.chunks_gpu.iter().enumerate() {
            draw_layer(
                &mut pass,
                &self.transparent_pipeline,
                &transparent_bg,
                &chunk.texture_bg,
                &chunk.transparent,
                i,
            );
        }
    }

    /// Render a single frame and return RGBA pixels. Headless mode only.
    #[cfg(not(target_arch = "wasm32"))]
    /// Number of independently posable meshes in this renderer.
    pub fn mesh_count(&self) -> usize {
        self.chunks_gpu.len()
    }

    /// Set the per-draw pose of every mesh.
    ///
    /// `poses[i]` applies to the *i*-th [`MeshOutput`] this renderer was built
    /// from, so a caller animating per block must mesh per block (one
    /// `MeshOutput` per animation group). Extra poses are ignored; meshes past
    /// the end of `poses` keep the identity pose.
    ///
    /// Cheap enough to call once per frame — it writes one uniform buffer and
    /// touches no geometry, which is what makes rendering an animation from a
    /// single [`GpuRenderer`] worthwhile.
    pub fn set_poses(&self, poses: &[crate::animation::Pose]) {
        if self.chunks_gpu.is_empty() {
            return;
        }
        let slot = std::mem::size_of::<DrawUniforms>();
        let mut bytes = vec![0u8; self.draw_stride as usize * self.chunks_gpu.len()];
        for i in 0..self.chunks_gpu.len() {
            let u = poses
                .get(i)
                .map(DrawUniforms::from_pose)
                .unwrap_or_else(DrawUniforms::identity);
            let off = i * self.draw_stride as usize;
            bytes[off..off + slot].copy_from_slice(bytemuck::bytes_of(&u));
        }
        self.queue.write_buffer(&self.draw_buf, 0, &bytes);
    }

    /// Reset every mesh to the identity pose.
    pub fn clear_poses(&self) {
        self.set_poses(&[]);
    }

    /// Set (or clear, with `None`) the world-space reference grid drawn under
    /// the scene. Cheap: the geometry is rebuilt per frame from this config.
    pub fn set_grid(&self, grid: Option<super::GridConfig>) {
        self.grid.set(grid);
    }

    pub fn render_frame(&self, camera: &CameraConfig) -> Result<Vec<u8>, RenderError> {
        let render_target = self.render_target.as_ref().ok_or_else(|| {
            RenderError::RenderFailed("render_frame requires headless mode".into())
        })?;
        let render_target_view = self.render_target_view.as_ref().unwrap();
        let staging_buffer = self.staging_buffer.as_ref().unwrap();

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame_encoder"),
            });

        self.render_to_view(
            &mut encoder,
            render_target_view,
            &self.depth_view,
            camera,
            self.width,
            self.height,
        );

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: render_target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.padded_bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
        receiver
            .recv()
            .unwrap()
            .map_err(|e| RenderError::RenderFailed(format!("Failed to map buffer: {}", e)))?;

        // wgpu 30: `get_mapped_range` is fallible.
        let data = buffer_slice
            .get_mapped_range()
            .map_err(|e| RenderError::RenderFailed(format!("Failed to read mapped buffer: {e}")))?;
        let bytes_per_pixel = 4u32;
        let unpadded_bytes_per_row = bytes_per_pixel * self.width;
        let mut pixels = Vec::with_capacity((self.width * self.height * bytes_per_pixel) as usize);
        for row in 0..self.height {
            let start = (row * self.padded_bytes_per_row) as usize;
            let end = start + unpadded_bytes_per_row as usize;
            pixels.extend_from_slice(&data[start..end]);
        }

        drop(data);
        staging_buffer.unmap();
        Ok(pixels)
    }

    /// Recreate the depth texture for a new window size.
    pub fn recreate_depth(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        self.depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
    }

    /// Capture a screenshot from the current camera. Works in any mode.
    pub fn screenshot(&self, camera: &CameraConfig) -> Result<Vec<u8>, RenderError> {
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("screenshot_target"),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.color_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let tex_view = tex.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("screenshot_encoder"),
            });

        self.render_to_view(
            &mut encoder,
            &tex_view,
            &self.depth_view,
            camera,
            self.width,
            self.height,
        );

        let bpp = 4u32;
        let unpadded = bpp * self.width;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded = unpadded.div_ceil(align) * align;

        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("screenshot_staging"),
            size: (padded * self.height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            tx.send(r).unwrap();
        });
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv().unwrap().map_err(|e| {
            RenderError::RenderFailed(format!("Failed to map screenshot buffer: {}", e))
        })?;

        // wgpu 30: `get_mapped_range` is fallible.
        let data = slice.get_mapped_range().map_err(|e| {
            RenderError::RenderFailed(format!("Failed to read screenshot buffer: {e}"))
        })?;
        let mut pixels = Vec::with_capacity((self.width * self.height * bpp) as usize);
        for row in 0..self.height {
            let start = (row * padded) as usize;
            let end = start + unpadded as usize;
            pixels.extend_from_slice(&data[start..end]);
        }
        drop(data);
        staging.unmap();

        // BGRA → RGBA swap if needed
        if self.color_format == wgpu::TextureFormat::Bgra8UnormSrgb
            || self.color_format == wgpu::TextureFormat::Bgra8Unorm
        {
            for chunk in pixels.chunks_exact_mut(4) {
                chunk.swap(0, 2);
            }
        }

        Ok(pixels)
    }
}

#[cfg(test)]
mod clear_color_tests {
    use super::*;

    #[test]
    fn custom_background_used_verbatim() {
        let c = clear_color_for(Some([0.2, 0.4, 0.6, 0.0]), false);
        assert!((c.r - 0.2).abs() < 1e-6);
        assert!((c.g - 0.4).abs() < 1e-6);
        assert!((c.b - 0.6).abs() < 1e-6);
        assert!((c.a - 0.0).abs() < 1e-6); // transparent
    }

    #[test]
    fn custom_background_wins_over_hdri() {
        let c = clear_color_for(Some([1.0, 1.0, 1.0, 1.0]), true);
        assert!((c.r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn default_no_hdri_is_sky_blue() {
        let c = clear_color_for(None, false);
        assert!((c.r - 0.529).abs() < 1e-3);
        assert!((c.g - 0.808).abs() < 1e-3);
        assert!((c.b - 0.922).abs() < 1e-3);
        assert!((c.a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn default_with_hdri_is_black() {
        let c = clear_color_for(None, true);
        assert!((c.r) < 1e-6 && (c.g) < 1e-6 && (c.b) < 1e-6);
        assert!((c.a - 1.0).abs() < 1e-6);
    }
}
