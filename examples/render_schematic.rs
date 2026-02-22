//! Headless and interactive wgpu renderer for Nucleation schematics.
//!
//! Supports GPU rendering (PNG/video/interactive) with optional HDRI environment maps,
//! and CPU-only mesh export (GLB). Automatically uses parallel chunked
//! meshing for large schematics.
//!
//! Usage:
//!   cargo run --release --example render_schematic --features rendering -- \
//!       <resource_pack> <schematic> [output] [--flags...]
//!
//! Flags:
//!   --glb            Force GLB export (CPU only, no GPU required)
//!   --interactive    Open real-time 3D viewer window
//!   --hdri=path.hdr  HDRI environment map for skybox + lighting
//!   --yaw=45         Camera horizontal angle in degrees (default: 45)
//!   --pitch=30       Camera elevation angle in degrees (default: 30)
//!   --zoom=1.0       Camera distance multiplier (default: 1.0)
//!   --fov=45         Field of view in degrees (default: 45)
//!   --width=1024     Output width in pixels (default: 1024)
//!   --height=1024    Output height in pixels (default: 1024)
//!   --chunk=N        Override chunk size (default: auto)
//!   --threads=N      Max parallel meshing threads (default: num CPUs)
//!
//! Video / orbit flags:
//!   --orbit=N        Render N-frame orbit video (360° rotation)
//!   --fps=30         Video frame rate (default: 30)
//!   --codec=h264     Video codec: h264, hevc, vp9, av1 (default: h264)
//!   --hwaccel        Use hardware encoder (VideoToolbox on macOS)
//!   --crf=23         Constant rate factor / quality (default: 23)
//!
//! Interactive controls:
//!   Left-drag = orbit, Scroll = zoom, S = screenshot, Esc/Q = quit

use nucleation::formats::manager::get_manager;
use nucleation::meshing::{MeshConfig, MeshOutput, ResourcePackSource};
use nucleation::rendering::camera::CameraConfig;
use nucleation::rendering::gpu::GpuRenderer;
use nucleation::rendering::hdri::{load_hdri, HdriData};
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

const CHUNK_VOLUME_THRESHOLD: i64 = 500_000;

// ─── CLI parsing ────────────────────────────────────────────────────────────

struct Config {
    pack_path: String,
    schem_path: String,
    output_path: String,
    width: u32,
    height: u32,
    glb_mode: bool,
    interactive: bool,
    hdri_path: Option<String>,
    yaw_deg: f32,
    pitch_deg: f32,
    zoom: f32,
    fov_deg: f32,
    chunk_override: Option<i32>,
    thread_count: usize,
    // Video / orbit
    orbit_frames: Option<u32>,
    fps: u32,
    codec: String,
    hwaccel: bool,
    crf: u32,
    // Cache
    cache_path: Option<String>,
    // NUCM export
    nucm_path: Option<String>,
}

fn parse_args() -> Config {
    let args: Vec<String> = std::env::args().collect();

    let mut positional = Vec::new();
    let mut glb_mode = false;
    let mut interactive = false;
    let mut hdri_path: Option<String> = None;
    let mut yaw_deg: f32 = 45.0;
    let mut pitch_deg: f32 = 30.0;
    let mut zoom: f32 = 1.0;
    let mut fov_deg: f32 = 45.0;
    let mut width: u32 = 1024;
    let mut height: u32 = 1024;
    let mut chunk_override: Option<i32> = None;
    let mut thread_count: usize = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let mut orbit_frames: Option<u32> = None;
    let mut fps: u32 = 30;
    let mut codec = "h264".to_string();
    let mut hwaccel = false;
    let mut crf: u32 = 23;
    let mut cache_path: Option<String> = None;
    let mut nucm_path: Option<String> = None;

    for arg in &args[1..] {
        if arg == "--glb" {
            glb_mode = true;
        } else if arg == "--interactive" {
            interactive = true;
        } else if arg == "--hwaccel" {
            hwaccel = true;
        } else if let Some(v) = arg.strip_prefix("--hdri=") {
            hdri_path = Some(v.to_string());
        } else if let Some(v) = arg.strip_prefix("--yaw=") {
            yaw_deg = v.parse().unwrap_or(45.0);
        } else if let Some(v) = arg.strip_prefix("--pitch=") {
            pitch_deg = v.parse().unwrap_or(30.0);
        } else if let Some(v) = arg.strip_prefix("--zoom=") {
            zoom = v.parse().unwrap_or(1.0);
        } else if let Some(v) = arg.strip_prefix("--fov=") {
            fov_deg = v.parse().unwrap_or(45.0);
        } else if let Some(v) = arg.strip_prefix("--width=") {
            width = v.parse().unwrap_or(1024);
        } else if let Some(v) = arg.strip_prefix("--height=") {
            height = v.parse().unwrap_or(1024);
        } else if let Some(v) = arg.strip_prefix("--chunk=") {
            chunk_override = v.parse().ok();
        } else if let Some(v) = arg.strip_prefix("--threads=") {
            if let Ok(t) = v.parse() {
                thread_count = t;
            }
        } else if let Some(v) = arg.strip_prefix("--orbit=") {
            orbit_frames = v.parse().ok();
        } else if let Some(v) = arg.strip_prefix("--fps=") {
            fps = v.parse().unwrap_or(30);
        } else if let Some(v) = arg.strip_prefix("--codec=") {
            codec = v.to_string();
        } else if let Some(v) = arg.strip_prefix("--crf=") {
            crf = v.parse().unwrap_or(23);
        } else if let Some(v) = arg.strip_prefix("--cache=") {
            cache_path = Some(v.to_string());
        } else if let Some(v) = arg.strip_prefix("--nucm=") {
            nucm_path = Some(v.to_string());
        } else {
            positional.push(arg.clone());
        }
    }

    if positional.len() < 2 && cache_path.is_none() {
        eprintln!(
            "Usage: {} <resource_pack> <schematic> [output] [flags...]",
            args[0]
        );
        eprintln!(
            "       {} --cache=file.nucm --interactive [flags...]",
            args[0]
        );
        eprintln!();
        eprintln!("Flags:");
        eprintln!("  --glb            Export GLB (CPU only, no GPU)");
        eprintln!("  --nucm=path      Export .nucm mesh cache (CPU only, no GPU)");
        eprintln!("  --interactive    Open real-time 3D viewer window");
        eprintln!("  --cache=path     Load pre-meshed .nucm cache (skip meshing)");
        eprintln!("  --hdri=path.hdr  HDRI environment map");
        eprintln!("  --yaw=45         Horizontal angle (degrees)");
        eprintln!("  --pitch=30       Elevation angle (degrees)");
        eprintln!("  --zoom=1.0       Distance multiplier");
        eprintln!("  --fov=45         Field of view (degrees)");
        eprintln!("  --width=1024     Output width");
        eprintln!("  --height=1024    Output height");
        eprintln!("  --chunk=N        Chunk size override");
        eprintln!("  --threads=N      Parallel meshing threads");
        eprintln!("  --orbit=N        Orbit video with N frames (360°)");
        eprintln!("  --fps=30         Video frame rate");
        eprintln!("  --codec=h264     Codec: h264, hevc, vp9, av1");
        eprintln!("  --hwaccel        Hardware encoder (VideoToolbox)");
        eprintln!("  --crf=23         Quality (lower = better)");
        std::process::exit(1);
    }

    let mut output_path = positional.get(2).cloned().unwrap_or_else(|| {
        if glb_mode {
            "/tmp/nucleation_render.glb".to_string()
        } else {
            "/tmp/nucleation_render.png".to_string()
        }
    });

    // Auto-detect GLB from extension
    if output_path.ends_with(".glb") {
        glb_mode = true;
    }

    // Auto-detect video from extension or orbit flag
    if orbit_frames.is_some()
        && !output_path.ends_with(".mp4")
        && !output_path.ends_with(".mkv")
        && !output_path.ends_with(".webm")
        && !output_path.ends_with(".glb")
        && !output_path.ends_with(".png")
    {
        output_path = output_path.replace(".png", ".mp4");
        if !output_path.ends_with(".mp4") {
            output_path = "/tmp/nucleation_orbit.mp4".to_string();
        }
    }

    Config {
        pack_path: positional.first().cloned().unwrap_or_default(),
        schem_path: positional.get(1).cloned().unwrap_or_default(),
        output_path,
        width,
        height,
        glb_mode,
        interactive,
        hdri_path,
        yaw_deg,
        pitch_deg,
        zoom,
        fov_deg,
        chunk_override,
        thread_count,
        orbit_frames,
        fps,
        codec,
        hwaccel,
        crf,
        cache_path,
        nucm_path,
    }
}

// ─── Main ───────────────────────────────────────────────────────────────────

fn main() {
    let cfg = parse_args();
    let t0 = Instant::now();

    // --- Load HDRI (if provided) ---
    let hdri_data = cfg.hdri_path.as_ref().map(|path| {
        let t = Instant::now();
        println!("Loading HDRI: {}", path);
        let data = load_hdri(path).expect("Failed to load HDRI");
        println!(
            "  {}x{} loaded in {:.1}ms",
            data.width,
            data.height,
            t.elapsed().as_secs_f64() * 1000.0
        );
        data
    });

    // --- Load meshes: either from .nucm cache or by meshing a schematic ---
    let meshes: Vec<MeshOutput> = if let Some(ref cache_path) = cfg.cache_path {
        let t = Instant::now();
        println!("Loading mesh cache: {}", cache_path);
        let meshes = nucleation::meshing::cache::load_cached_mesh(std::path::Path::new(cache_path))
            .expect("Failed to load .nucm cache");
        let total_verts: usize = meshes.iter().map(|m| m.total_vertices()).sum();
        let total_tris: usize = meshes.iter().map(|m| m.total_triangles()).sum();
        println!(
            "  {} chunks, {} vertices, {} triangles in {:.1}s",
            meshes.len(),
            total_verts,
            total_tris,
            t.elapsed().as_secs_f64()
        );
        meshes
    } else {
        // --- Load schematic ---
        println!("Loading schematic: {}", cfg.schem_path);
        let schem_data = std::fs::read(&cfg.schem_path).expect("Failed to read schematic file");
        let manager = get_manager();
        let manager = manager.lock().unwrap();
        let schematic = manager
            .read(&schem_data)
            .expect("Failed to parse schematic");

        let bb = schematic.default_region.get_bounding_box();
        let dims = (
            (bb.max.0 - bb.min.0 + 1) as i64,
            (bb.max.1 - bb.min.1 + 1) as i64,
            (bb.max.2 - bb.min.2 + 1) as i64,
        );
        let volume = dims.0 * dims.1 * dims.2;
        println!(
            "  Dimensions: {}x{}x{} ({} blocks)",
            dims.0, dims.1, dims.2, volume
        );
        println!("  Loaded in {:.1}ms", t0.elapsed().as_secs_f64() * 1000.0);

        // --- Load resource pack ---
        let t1 = Instant::now();
        println!("Loading resource pack: {}", cfg.pack_path);
        let pack =
            ResourcePackSource::from_file(&cfg.pack_path).expect("Failed to load resource pack");
        let stats = pack.stats();
        println!(
            "  {} blockstates, {} models, {} textures",
            stats.blockstate_count, stats.model_count, stats.texture_count
        );
        println!("  Loaded in {:.1}ms", t1.elapsed().as_secs_f64() * 1000.0);

        // --- Mesh ---
        let mesh_config = MeshConfig::default();
        let use_chunked = volume > CHUNK_VOLUME_THRESHOLD;
        let chunk_size = cfg
            .chunk_override
            .unwrap_or(if volume > 10_000_000 { 32 } else { 64 });

        let t2 = Instant::now();
        let meshes: Vec<MeshOutput> = if use_chunked {
            println!(
                "Meshing (parallel, chunk_size={}, threads={})...",
                chunk_size, cfg.thread_count
            );
            schematic
                .mesh_chunks_parallel(&pack, &mesh_config, chunk_size, cfg.thread_count)
                .expect("Failed to mesh schematic")
        } else {
            println!("Meshing (single)...");
            let mesh = schematic
                .to_mesh(&pack, &mesh_config)
                .expect("Failed to mesh schematic");
            vec![mesh]
        };

        let total_verts: usize = meshes.iter().map(|m| m.total_vertices()).sum();
        let total_tris: usize = meshes.iter().map(|m| m.total_triangles()).sum();
        println!(
            "  {} chunks, {} vertices, {} triangles in {:.1}ms",
            meshes.len(),
            total_verts,
            total_tris,
            t2.elapsed().as_secs_f64() * 1000.0
        );

        meshes
    };

    // --- Output ---
    let camera = CameraConfig {
        yaw_deg: cfg.yaw_deg,
        pitch_deg: cfg.pitch_deg,
        zoom: cfg.zoom,
        fov_deg: cfg.fov_deg,
    };

    if cfg.interactive {
        println!("Controls: Left-drag = orbit, Scroll = zoom, S = screenshot, Esc/Q = quit");
        let event_loop = EventLoop::new().unwrap();
        let mut app = InteractiveApp {
            window: None,
            surface: None,
            surface_config: None,
            renderer: None,
            camera,
            mouse_pressed: false,
            last_mouse_pos: None,
            meshes,
            hdri: hdri_data,
            init_width: cfg.width,
            init_height: cfg.height,
            output_path: cfg.output_path.clone(),
        };
        event_loop.run_app(&mut app).unwrap();
        println!("Total: {:.1}ms", t0.elapsed().as_secs_f64() * 1000.0);
    } else if let Some(ref nucm_out) = cfg.nucm_path {
        println!("Exporting NUCM ({} chunks)...", meshes.len());
        let t = Instant::now();
        nucleation::meshing::cache::save_cached_mesh(&meshes, std::path::Path::new(nucm_out))
            .expect("Failed to write .nucm");
        let size = std::fs::metadata(nucm_out).unwrap().len();
        println!(
            "  Wrote {} ({:.1} KB) in {:.1}ms",
            nucm_out,
            size as f64 / 1024.0,
            t.elapsed().as_secs_f64() * 1000.0
        );
        println!("Total: {:.1}ms", t0.elapsed().as_secs_f64() * 1000.0);
    } else if cfg.glb_mode {
        println!("Exporting GLB (CPU only)...");
        let glb = meshes[0].to_glb().expect("Failed to export GLB");
        std::fs::write(&cfg.output_path, &glb).expect("Failed to write GLB");
        println!("  Wrote {} bytes to {}", glb.len(), cfg.output_path);
        println!("Total: {:.1}ms", t0.elapsed().as_secs_f64() * 1000.0);
    } else if let Some(total_frames) = cfg.orbit_frames {
        println!(
            "Rendering orbit video: {}x{} @ {}fps, {} frames, codec={} ...",
            cfg.width, cfg.height, cfg.fps, total_frames, cfg.codec
        );
        let t3 = Instant::now();
        render_orbit_video(
            &meshes,
            cfg.width,
            cfg.height,
            &camera,
            hdri_data.as_ref(),
            total_frames,
            cfg.fps,
            &cfg.codec,
            cfg.hwaccel,
            cfg.crf,
            &cfg.output_path,
        );
        println!("  Total render+encode: {:.1}s", t3.elapsed().as_secs_f64());
        println!("Saved to {}", cfg.output_path);
        println!("Total: {:.1}ms", t0.elapsed().as_secs_f64() * 1000.0);
    } else {
        println!(
            "Rendering {}x{} (yaw={}, pitch={}, zoom={}, fov={}) ...",
            cfg.width, cfg.height, camera.yaw_deg, camera.pitch_deg, camera.zoom, camera.fov_deg
        );
        let t3 = Instant::now();
        let renderer = pollster::block_on(GpuRenderer::new(
            &meshes,
            cfg.width,
            cfg.height,
            hdri_data.as_ref(),
        ))
        .expect("Failed to create GPU renderer");
        let pixels = renderer
            .render_frame(&camera)
            .expect("Failed to render frame");
        println!("  Rendered in {:.1}ms", t3.elapsed().as_secs_f64() * 1000.0);

        let img = image::RgbaImage::from_raw(cfg.width, cfg.height, pixels)
            .expect("Failed to create image");
        img.save(&cfg.output_path).expect("Failed to save PNG");
        println!("Saved to {}", cfg.output_path);
        println!("Total: {:.1}ms", t0.elapsed().as_secs_f64() * 1000.0);
    }
}

// ─── Orbit video rendering ──────────────────────────────────────────────────

fn render_orbit_video(
    meshes: &[MeshOutput],
    width: u32,
    height: u32,
    base_camera: &CameraConfig,
    hdri: Option<&HdriData>,
    total_frames: u32,
    fps: u32,
    codec: &str,
    hwaccel: bool,
    crf: u32,
    output_path: &str,
) {
    use ffmpeg_sidecar::command::FfmpegCommand;

    let renderer = pollster::block_on(GpuRenderer::new(meshes, width, height, hdri))
        .expect("Failed to create GPU renderer");

    // Resolve encoder name
    let encoder = if hwaccel {
        match codec {
            "h264" => "h264_videotoolbox",
            "hevc" | "h265" => "hevc_videotoolbox",
            _ => {
                eprintln!(
                    "  Warning: no hwaccel encoder for '{}', using software",
                    codec
                );
                resolve_sw_encoder(codec)
            }
        }
    } else {
        resolve_sw_encoder(codec)
    };

    println!(
        "  Encoding: {} frames @ {}fps, codec={}, crf={}, output={}",
        total_frames, fps, encoder, crf, output_path
    );

    let mut cmd = FfmpegCommand::new();
    cmd.args(["-y"])
        .args(["-f", "rawvideo"])
        .args(["-pix_fmt", "rgba"])
        .args(["-s", &format!("{}x{}", width, height)])
        .args(["-r", &fps.to_string()])
        .args(["-i", "pipe:0"])
        .args(["-c:v", encoder])
        .args(["-pix_fmt", "yuv420p"]);

    if hwaccel && (codec == "h264" || codec == "hevc" || codec == "h265") {
        cmd.args(["-q:v", &crf.to_string()]);
    } else if codec == "vp9" {
        cmd.args(["-crf", &crf.to_string()]).args(["-b:v", "0"]);
    } else {
        cmd.args(["-crf", &crf.to_string()]);
    }

    cmd.args(["-movflags", "+faststart"]).arg(output_path);

    let mut child = cmd
        .spawn()
        .expect("Failed to spawn ffmpeg. Is ffmpeg installed?");

    let stdin = child.take_stdin().expect("Failed to get ffmpeg stdin");
    let mut stdin = std::io::BufWriter::new(stdin);

    let t_render = Instant::now();

    for frame_idx in 0..total_frames {
        let yaw = base_camera.yaw_deg + (frame_idx as f32 / total_frames as f32) * 360.0;
        let cam = CameraConfig {
            yaw_deg: yaw,
            pitch_deg: base_camera.pitch_deg,
            zoom: base_camera.zoom,
            fov_deg: base_camera.fov_deg,
        };

        let pixels = renderer.render_frame(&cam).expect("Failed to render frame");

        use std::io::Write;
        stdin
            .write_all(&pixels)
            .expect("Failed to write frame to ffmpeg");

        if (frame_idx + 1) % 10 == 0 || frame_idx + 1 == total_frames {
            let elapsed = t_render.elapsed().as_secs_f64();
            let fps_actual = (frame_idx + 1) as f64 / elapsed;
            print!(
                "\r  Frame {}/{} ({:.1} fps, {:.1}s elapsed)",
                frame_idx + 1,
                total_frames,
                fps_actual,
                elapsed
            );
            use std::io::Write as _;
            std::io::stdout().flush().ok();
        }
    }

    drop(stdin);
    println!();

    let status = child.wait().expect("Failed to wait for ffmpeg");
    if !status.success() {
        eprintln!("  ffmpeg exited with: {}", status);
    }

    let file_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);
    println!(
        "  Video: {} frames in {:.1}s, {:.1} MB",
        total_frames,
        t_render.elapsed().as_secs_f64(),
        file_size as f64 / 1_048_576.0
    );
}

fn resolve_sw_encoder(codec: &str) -> &'static str {
    match codec {
        "h264" => "libx264",
        "hevc" | "h265" => "libx265",
        "vp9" => "libvpx-vp9",
        "av1" => "libsvtav1",
        _ => {
            eprintln!("  Unknown codec '{}', falling back to libx264", codec);
            "libx264"
        }
    }
}

// ─── Interactive viewer (winit 0.30) ────────────────────────────────────────

struct InteractiveApp {
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    renderer: Option<GpuRenderer>,
    camera: CameraConfig,
    mouse_pressed: bool,
    last_mouse_pos: Option<(f64, f64)>,
    meshes: Vec<MeshOutput>,
    hdri: Option<HdriData>,
    init_width: u32,
    init_height: u32,
    output_path: String,
}

impl ApplicationHandler for InteractiveApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_title("Nucleation Viewer")
            .with_inner_size(winit::dpi::PhysicalSize::new(
                self.init_width,
                self.init_height,
            ));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();

        let renderer = pollster::block_on(GpuRenderer::new_windowed(
            &self.meshes,
            &instance,
            &surface,
            self.init_width,
            self.init_height,
            self.hdri.as_ref(),
        ))
        .expect("Failed to create windowed renderer");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: renderer.color_format,
            width: self.init_width,
            height: self.init_height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&renderer.device, &config);

        self.window = Some(window);
        self.surface = Some(surface);
        self.surface_config = Some(config);
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                let surface = self.surface.as_ref().unwrap();
                let renderer = self.renderer.as_ref().unwrap();

                let output = match surface.get_current_texture() {
                    Ok(t) => t,
                    Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                        return;
                    }
                    Err(e) => {
                        eprintln!("Surface error: {:?}", e);
                        return;
                    }
                };

                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder =
                    renderer
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("interactive_encoder"),
                        });

                renderer.render_to_view(
                    &mut encoder,
                    &view,
                    &renderer.depth_view,
                    &self.camera,
                    renderer.width,
                    renderer.height,
                );

                renderer.queue.submit(std::iter::once(encoder.finish()));
                output.present();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let (Some(surface), Some(config), Some(renderer)) = (
                        self.surface.as_ref(),
                        self.surface_config.as_mut(),
                        self.renderer.as_mut(),
                    ) {
                        config.width = size.width;
                        config.height = size.height;
                        surface.configure(&renderer.device, config);
                        renderer.recreate_depth(size.width, size.height);
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                if self.mouse_pressed {
                    if let Some((lx, ly)) = self.last_mouse_pos {
                        let dx = position.x - lx;
                        let dy = position.y - ly;
                        self.camera.yaw_deg += dx as f32 * 0.3;
                        self.camera.pitch_deg =
                            (self.camera.pitch_deg + dy as f32 * 0.3).clamp(-89.0, 89.0);
                    }
                    self.last_mouse_pos = Some((position.x, position.y));
                }
            }

            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.mouse_pressed = state == ElementState::Pressed;
                if !self.mouse_pressed {
                    self.last_mouse_pos = None;
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y as f64,
                    MouseScrollDelta::PixelDelta(pos) => pos.y / 30.0,
                };
                self.camera.zoom = (self.camera.zoom * (1.0 - scroll as f32 * 0.1)).max(0.1);
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    match event.logical_key.as_ref() {
                        Key::Named(NamedKey::Escape) => {
                            event_loop.exit();
                        }
                        Key::Character("q") => {
                            event_loop.exit();
                        }
                        Key::Character("s") => {
                            if let Some(renderer) = &self.renderer {
                                println!("Saving screenshot...");
                                let pixels = renderer
                                    .screenshot(&self.camera)
                                    .expect("Failed to capture screenshot");
                                let img = image::RgbaImage::from_raw(
                                    renderer.width,
                                    renderer.height,
                                    pixels,
                                )
                                .expect("Failed to create screenshot image");
                                img.save(&self.output_path)
                                    .expect("Failed to save screenshot");
                                println!("  Screenshot saved to {}", self.output_path);
                            }
                        }
                        _ => {}
                    }
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
