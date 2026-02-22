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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    inv_view_proj: [[f32; 4]; 4],
    params: [f32; 4], // x = alpha_cutoff, y = hdri_enabled, z = hdri_intensity
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
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
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
        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: surface,
                force_fallback_adapter: false,
            })
            .await
        {
            Some(a) => a,
            None => {
                // Try software fallback
                instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::LowPower,
                        compatible_surface: surface,
                        force_fallback_adapter: true,
                    })
                    .await
                    .ok_or(RenderError::NoGpuAdapter)?
            }
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("render_device"),
                    required_features: wgpu::Features::FLOAT32_FILTERABLE,
                    required_limits: wgpu::Limits::default(),
                    ..Default::default()
                },
                None,
            )
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

        let mesh_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh_pipeline_layout"),
            bind_group_layouts: &[&uniform_bgl, &texture_bgl, &hdri_bgl],
            push_constant_ranges: &[],
        });

        let sky_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky_pipeline_layout"),
            bind_group_layouts: &[&uniform_bgl, &texture_bgl, &hdri_bgl],
            push_constant_ranges: &[],
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
        let vertex_buffer_layouts = [
            wgpu::VertexBufferLayout {
                array_stride: 12,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                }],
            },
            wgpu::VertexBufferLayout {
                array_stride: 12,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 1,
                }],
            },
            wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 2,
                }],
            },
            wgpu::VertexBufferLayout {
                array_stride: 16,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3,
                }],
            },
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
                        depth_write_enabled: depth_write,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                    multisample: Default::default(),
                    multiview: None,
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
                        depth_write_enabled: false,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                    multisample: Default::default(),
                    multiview: None,
                    cache: None,
                }),
            )
        } else {
            None
        };

        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("atlas_sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
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
            let padded = (unpadded_bytes_per_row + align - 1) / align * align;
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

        let clear_color = if self.hdri_enabled {
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
        };

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
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
                          bufs: &Option<LayerBuffers>| {
            if let Some(b) = bufs {
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, uniform_bg, &[]);
                pass.set_bind_group(1, texture_bg, &[]);
                pass.set_bind_group(2, &self.hdri_bg, &[]);
                pass.set_vertex_buffer(0, b.positions.slice(..));
                pass.set_vertex_buffer(1, b.normals.slice(..));
                pass.set_vertex_buffer(2, b.uvs.slice(..));
                pass.set_vertex_buffer(3, b.colors.slice(..));
                pass.set_index_buffer(b.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..b.index_count, 0, 0..1);
            }
        };

        for chunk in &self.chunks_gpu {
            draw_layer(
                &mut pass,
                &self.opaque_pipeline,
                &opaque_bg,
                &chunk.texture_bg,
                &chunk.opaque,
            );
        }
        for chunk in &self.chunks_gpu {
            draw_layer(
                &mut pass,
                &self.cutout_pipeline,
                &cutout_bg,
                &chunk.texture_bg,
                &chunk.cutout,
            );
        }
        for chunk in &self.chunks_gpu {
            draw_layer(
                &mut pass,
                &self.transparent_pipeline,
                &transparent_bg,
                &chunk.texture_bg,
                &chunk.transparent,
            );
        }
    }

    /// Render a single frame and return RGBA pixels. Headless mode only.
    #[cfg(not(target_arch = "wasm32"))]
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
        self.device.poll(wgpu::Maintain::Wait);
        receiver
            .recv()
            .unwrap()
            .map_err(|e| RenderError::RenderFailed(format!("Failed to map buffer: {}", e)))?;

        let data = buffer_slice.get_mapped_range();
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
        let padded = (unpadded + align - 1) / align * align;

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
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().map_err(|e| {
            RenderError::RenderFailed(format!("Failed to map screenshot buffer: {}", e))
        })?;

        let data = slice.get_mapped_range();
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
