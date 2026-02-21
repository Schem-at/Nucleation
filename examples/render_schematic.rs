//! Headless and interactive wgpu renderer for Nucleation schematics.
//!
//! Supports GPU rendering (PNG/video/interactive) with optional HDRI environment maps,
//! and CPU-only mesh export (GLB). Automatically uses parallel chunked
//! meshing for large schematics.
//!
//! Usage:
//!   cargo run --release --example render_schematic --features meshing -- \
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
use nucleation::meshing::{MeshConfig, MeshLayer, MeshOutput, ResourcePackSource};
use std::sync::Arc;
use std::time::Instant;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

const SHADER_SRC: &str = include_str!("render_shader.wgsl");
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
        // Default video output when orbit is set but no explicit video extension
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
        let data = load_hdri(path);
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
        // Use first chunk for GLB export (multi-chunk GLB not supported from cache)
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
        let pixels = pollster::block_on(render_headless(
            &meshes,
            cfg.width,
            cfg.height,
            &camera,
            hdri_data.as_ref(),
        ));
        println!("  Rendered in {:.1}ms", t3.elapsed().as_secs_f64() * 1000.0);

        let img = image::RgbaImage::from_raw(cfg.width, cfg.height, pixels)
            .expect("Failed to create image");
        img.save(&cfg.output_path).expect("Failed to save PNG");
        println!("Saved to {}", cfg.output_path);
        println!("Total: {:.1}ms", t0.elapsed().as_secs_f64() * 1000.0);
    }
}

// ─── HDRI loading ───────────────────────────────────────────────────────────

struct HdriData {
    width: u32,
    height: u32,
    /// RGBA f32 pixel data (4 floats per pixel)
    pixels_rgba32f: Vec<f32>,
}

fn load_hdri(path: &str) -> HdriData {
    let img = image::open(path).expect("Failed to open HDRI file");
    let rgb32f = img.to_rgb32f();
    let (w, h) = (rgb32f.width(), rgb32f.height());

    // Convert RGB32F → RGBA32F (add alpha=1.0)
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for pixel in rgb32f.pixels() {
        rgba.push(pixel[0]);
        rgba.push(pixel[1]);
        rgba.push(pixel[2]);
        rgba.push(1.0);
    }

    HdriData {
        width: w,
        height: h,
        pixels_rgba32f: rgba,
    }
}

// ─── Camera ─────────────────────────────────────────────────────────────────

struct CameraConfig {
    yaw_deg: f32,
    pitch_deg: f32,
    zoom: f32,
    fov_deg: f32,
}

fn merged_bounds(meshes: &[MeshOutput]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for m in meshes {
        for i in 0..3 {
            min[i] = min[i].min(m.bounds.min[i]);
            max[i] = max[i].max(m.bounds.max[i]);
        }
    }
    (min, max)
}

fn compute_view_proj(
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
    aspect: f32,
    camera: &CameraConfig,
) -> ([[f32; 4]; 4], [[f32; 4]; 4]) {
    let center = [
        (bounds_min[0] + bounds_max[0]) * 0.5,
        (bounds_min[1] + bounds_max[1]) * 0.5,
        (bounds_min[2] + bounds_max[2]) * 0.5,
    ];

    let yaw = camera.yaw_deg.to_radians();
    let pitch = camera.pitch_deg.to_radians();
    let fov = camera.fov_deg.to_radians();

    let dir = normalize3([
        -(pitch.cos() * yaw.sin()),
        -(pitch.sin()),
        -(pitch.cos() * yaw.cos()),
    ]);

    let forward = dir;
    let right = normalize3(cross3(forward, [0.0, 1.0, 0.0]));
    let up = cross3(right, forward);

    let half_fov_y = fov * 0.5;
    let half_fov_x = (half_fov_y.tan() * aspect).atan();

    let corners = [
        [bounds_min[0], bounds_min[1], bounds_min[2]],
        [bounds_max[0], bounds_min[1], bounds_min[2]],
        [bounds_min[0], bounds_max[1], bounds_min[2]],
        [bounds_max[0], bounds_max[1], bounds_min[2]],
        [bounds_min[0], bounds_min[1], bounds_max[2]],
        [bounds_max[0], bounds_min[1], bounds_max[2]],
        [bounds_min[0], bounds_max[1], bounds_max[2]],
        [bounds_max[0], bounds_max[1], bounds_max[2]],
    ];

    let mut max_dist = 1.0f32;
    for c in &corners {
        let rel = sub3(*c, center);
        let proj_right = dot3(rel, right).abs();
        let proj_up = dot3(rel, up).abs();
        let proj_depth = -dot3(rel, forward);
        let dist_h = proj_right / half_fov_x.tan() + proj_depth;
        let dist_v = proj_up / half_fov_y.tan() + proj_depth;
        max_dist = max_dist.max(dist_h).max(dist_v);
    }

    let distance = max_dist * 1.1 * camera.zoom;
    let eye = [
        center[0] - dir[0] * distance,
        center[1] - dir[1] * distance,
        center[2] - dir[2] * distance,
    ];

    let view = look_at(eye, center, [0.0, 1.0, 0.0]);
    let near = distance * 0.01;
    let far = distance * 10.0;
    let proj = perspective(fov, aspect, near, far);
    let view_proj = mat4_mul(proj, view);
    let inv_view_proj = mat4_inverse(view_proj);
    (view_proj, inv_view_proj)
}

fn look_at(eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4] {
    let f = normalize3(sub3(target, eye));
    let s = normalize3(cross3(f, up));
    let u = cross3(s, f);
    [
        [s[0], u[0], -f[0], 0.0],
        [s[1], u[1], -f[1], 0.0],
        [s[2], u[2], -f[2], 0.0],
        [-dot3(s, eye), -dot3(u, eye), dot3(f, eye), 1.0],
    ]
}

fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let f = 1.0 / (fov_y * 0.5).tan();
    let nf = 1.0 / (near - far);
    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, far * nf, -1.0],
        [0.0, 0.0, near * far * nf, 0.0],
    ]
}

fn mat4_mul(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut out = [[0.0f32; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            out[i][j] =
                a[0][j] * b[i][0] + a[1][j] * b[i][1] + a[2][j] * b[i][2] + a[3][j] * b[i][3];
        }
    }
    out
}

/// 4x4 matrix inverse (general, cofactor expansion).
fn mat4_inverse(m: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    // Flatten column-major
    let m00 = m[0][0];
    let m01 = m[0][1];
    let m02 = m[0][2];
    let m03 = m[0][3];
    let m10 = m[1][0];
    let m11 = m[1][1];
    let m12 = m[1][2];
    let m13 = m[1][3];
    let m20 = m[2][0];
    let m21 = m[2][1];
    let m22 = m[2][2];
    let m23 = m[2][3];
    let m30 = m[3][0];
    let m31 = m[3][1];
    let m32 = m[3][2];
    let m33 = m[3][3];

    let a2323 = m22 * m33 - m23 * m32;
    let a1323 = m21 * m33 - m23 * m31;
    let a1223 = m21 * m32 - m22 * m31;
    let a0323 = m20 * m33 - m23 * m30;
    let a0223 = m20 * m32 - m22 * m30;
    let a0123 = m20 * m31 - m21 * m30;
    let a2313 = m12 * m33 - m13 * m32;
    let a1313 = m11 * m33 - m13 * m31;
    let a1213 = m11 * m32 - m12 * m31;
    let a2312 = m12 * m23 - m13 * m22;
    let a1312 = m11 * m23 - m13 * m21;
    let a1212 = m11 * m22 - m12 * m21;
    let a0313 = m10 * m33 - m13 * m30;
    let a0213 = m10 * m32 - m12 * m30;
    let a0312 = m10 * m23 - m13 * m20;
    let a0212 = m10 * m22 - m12 * m20;
    let a0113 = m10 * m31 - m11 * m30;
    let a0112 = m10 * m21 - m11 * m20;

    let det = m00 * (m11 * a2323 - m12 * a1323 + m13 * a1223)
        - m01 * (m10 * a2323 - m12 * a0323 + m13 * a0223)
        + m02 * (m10 * a1323 - m11 * a0323 + m13 * a0123)
        - m03 * (m10 * a1223 - m11 * a0223 + m12 * a0123);

    let inv_det = 1.0 / det;

    [
        [
            inv_det * (m11 * a2323 - m12 * a1323 + m13 * a1223),
            inv_det * -(m01 * a2323 - m02 * a1323 + m03 * a1223),
            inv_det * (m01 * a2313 - m02 * a1313 + m03 * a1213),
            inv_det * -(m01 * a2312 - m02 * a1312 + m03 * a1212),
        ],
        [
            inv_det * -(m10 * a2323 - m12 * a0323 + m13 * a0223),
            inv_det * (m00 * a2323 - m02 * a0323 + m03 * a0223),
            inv_det * -(m00 * a2313 - m02 * a0313 + m03 * a0213),
            inv_det * (m00 * a2312 - m02 * a0312 + m03 * a0212),
        ],
        [
            inv_det * (m10 * a1323 - m11 * a0323 + m13 * a0123),
            inv_det * -(m00 * a1323 - m01 * a0323 + m03 * a0123),
            inv_det * (m00 * a1313 - m01 * a0313 + m03 * a0113),
            inv_det * -(m00 * a1312 - m01 * a0312 + m03 * a0112),
        ],
        [
            inv_det * -(m10 * a1223 - m11 * a0223 + m12 * a0123),
            inv_det * (m00 * a1223 - m01 * a0223 + m02 * a0123),
            inv_det * -(m00 * a1213 - m01 * a0213 + m02 * a0113),
            inv_det * (m00 * a1212 - m01 * a0212 + m02 * a0112),
        ],
    ]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = dot3(v, v).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

// ─── Uniform data ───────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    inv_view_proj: [[f32; 4]; 4],
    params: [f32; 4], // x = alpha_cutoff, y = hdri_enabled, z = hdri_intensity
}

// ─── GPU renderer (reusable across frames) ──────────────────────────────────

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

struct GpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    color_format: wgpu::TextureFormat,
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
    depth_view: wgpu::TextureView,
    width: u32,
    height: u32,
    padded_bytes_per_row: u32,
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
}

impl GpuRenderer {
    /// Headless constructor (unchanged API).
    async fn new(meshes: &[MeshOutput], width: u32, height: u32, hdri: Option<&HdriData>) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        Self::create(meshes, width, height, hdri, &instance, None).await
    }

    /// Windowed constructor — caller provides instance + surface so they share the same instance.
    async fn new_windowed(
        meshes: &[MeshOutput],
        instance: &wgpu::Instance,
        surface: &wgpu::Surface<'_>,
        width: u32,
        height: u32,
        hdri: Option<&HdriData>,
    ) -> Self {
        Self::create(meshes, width, height, hdri, instance, Some(surface)).await
    }

    async fn create(
        meshes: &[MeshOutput],
        width: u32,
        height: u32,
        hdri: Option<&HdriData>,
        instance: &wgpu::Instance,
        surface: Option<&wgpu::Surface<'_>>,
    ) -> Self {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: surface,
                force_fallback_adapter: false,
            })
            .await
            .expect("No GPU adapter found. Use --glb for CPU-only export.");

        println!(
            "  GPU: {} ({:?})",
            adapter.get_info().name,
            adapter.get_info().backend
        );

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
            .expect("Failed to create device");

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

        // --- Color format (surface-aware or headless default) ---
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

        Self {
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
        }
    }

    /// Encode the full render pass (skybox + 3 mesh layers) into `encoder`,
    /// targeting the provided `color_view` and `depth_view`. Does not submit.
    fn render_to_view(
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

    /// Render a single frame with the given camera and return RGBA pixels.
    /// Only works in headless mode (render_target must be Some).
    fn render_frame(&self, camera: &CameraConfig) -> Vec<u8> {
        let render_target = self
            .render_target
            .as_ref()
            .expect("render_frame requires headless mode");
        let render_target_view = self
            .render_target_view
            .as_ref()
            .expect("render_frame requires headless mode");
        let staging_buffer = self
            .staging_buffer
            .as_ref()
            .expect("render_frame requires headless mode");

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

        // Readback
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
        receiver.recv().unwrap().expect("Failed to map buffer");

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
        pixels
    }

    /// Recreate the depth texture for a new window size.
    fn recreate_depth(&mut self, width: u32, height: u32) {
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
    /// Returns RGBA pixels regardless of the surface format.
    fn screenshot(&self, camera: &CameraConfig) -> Vec<u8> {
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

        // Staging buffer for readback
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
        rx.recv().unwrap().expect("Failed to map screenshot buffer");

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

        pixels
    }
}

/// Single-frame convenience wrapper (backwards compat with existing code path).
async fn render_headless(
    meshes: &[MeshOutput],
    width: u32,
    height: u32,
    camera: &CameraConfig,
    hdri: Option<&HdriData>,
) -> Vec<u8> {
    let renderer = GpuRenderer::new(meshes, width, height, hdri).await;
    renderer.render_frame(camera)
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

    let renderer = pollster::block_on(GpuRenderer::new(meshes, width, height, hdri));

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

    // Build ffmpeg command: rawvideo input from stdin → encoded output
    let mut cmd = FfmpegCommand::new();
    cmd.args(["-y"]) // overwrite
        .args(["-f", "rawvideo"])
        .args(["-pix_fmt", "rgba"])
        .args(["-s", &format!("{}x{}", width, height)])
        .args(["-r", &fps.to_string()])
        .args(["-i", "pipe:0"]) // stdin
        .args(["-c:v", encoder])
        .args(["-pix_fmt", "yuv420p"]); // output pixel format

    // CRF / quality — VideoToolbox uses -q:v, software uses -crf
    if hwaccel && (codec == "h264" || codec == "hevc" || codec == "h265") {
        cmd.args(["-q:v", &crf.to_string()]);
    } else if codec == "vp9" {
        cmd.args(["-crf", &crf.to_string()]).args(["-b:v", "0"]); // VPx needs -b:v 0 with crf
    } else if codec == "av1" {
        cmd.args(["-crf", &crf.to_string()]);
    } else {
        cmd.args(["-crf", &crf.to_string()]);
    }

    cmd.args(["-movflags", "+faststart"]) // MP4 streaming
        .arg(output_path);

    // Spawn ffmpeg process
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

        let pixels = renderer.render_frame(&cam);

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

    // Close stdin to signal EOF to ffmpeg
    drop(stdin);
    println!();

    // Wait for ffmpeg to finish
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
    // GPU state (initialized in resumed())
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    renderer: Option<GpuRenderer>,

    // Camera
    camera: CameraConfig,

    // Mouse tracking
    mouse_pressed: bool,
    last_mouse_pos: Option<(f64, f64)>,

    // Init data (consumed by resumed())
    meshes: Vec<MeshOutput>,
    hdri: Option<HdriData>,
    init_width: u32,
    init_height: u32,
    output_path: String,
}

impl ApplicationHandler for InteractiveApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return; // already initialized
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
        ));

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
                        // Reconfigure surface on next resize
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
                                let pixels = renderer.screenshot(&self.camera);
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
