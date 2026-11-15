//! Native, textured Circuit Atlas thumbnails, without advancing simulation.
//!
//! Build on the designated rendering/build host:
//! cargo build --locked --release --example render_archive_thumbnails --features rendering
//! render_archive_thumbnails PACK.zip JOBS.jsonl RESULTS.jsonl
//!
//! Each job is {"id":"...","input":"/path/build.litematic","output":"/path/id.png"}.
//! The pack is loaded once; every nonempty input line produces an immediately
//! flushed result. Existing outputs are regenerated, so callers can implement
//! resume using the result manifest and pack/renderer revision they selected.
//! Errors do not produce placeholder PNGs or stop subsequent jobs. GPU device
//! creation remains per job: the public renderer does not support replacing its
//! meshes. Run separate batches/processes if isolation from driver faults is needed.

use anyhow::{bail, Context, Result};
use nucleation::formats::limits::DecodeLimits;
use nucleation::meshing::{MeshConfig, ResourcePackSource};
use nucleation::rendering::{render_meshes_png, RenderConfig};
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::time::Instant;

const WIDTH: u32 = 768;
const HEIGHT: u32 = 576;
const MAX_VOLUME: usize = 32_000_000;
const MAX_NON_AIR: usize = 2_000_000;
const SINGLE_MESH_VOLUME: u64 = 500_000;

#[derive(Debug, Deserialize)]
struct Job {
    id: String,
    input: PathBuf,
    output: PathBuf,
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        bail!("usage: render_archive_thumbnails PACK.zip JOBS.jsonl RESULTS.jsonl");
    }
    let pack_path = Path::new(&args[0]);
    let jobs_path = Path::new(&args[1]);
    let results_path = Path::new(&args[2]);
    if jobs_path == results_path {
        bail!("jobs and results must be different files");
    }
    let pack_started = Instant::now();
    let pack = ResourcePackSource::from_file(pack_path).context("load resource pack")?;
    let pack_blake3 = if pack_path.is_file() {
        Some(blake3::hash(&fs::read(pack_path)?).to_hex().to_string())
    } else {
        None
    };
    eprintln!("Resource pack ready in {:.2}s", pack_started.elapsed().as_secs_f64());
    create_parent(results_path)?;
    let jobs = BufReader::new(File::open(jobs_path).context("open job list")?);
    let mut results = OpenOptions::new()
        .create(true)
        .append(true)
        .open(results_path)
        .context("open result manifest")?;
    let limits = DecodeLimits {
        max_input_bytes: 128 * 1024 * 1024,
        max_decompressed_bytes: 512 * 1024 * 1024,
        max_volume: MAX_VOLUME,
        max_regions: 4096,
        max_entities: 100_000,
        max_block_entities: MAX_NON_AIR,
        max_nbt_collection_items: 64_000_000,
        max_nbt_nodes: 16_000_000,
        ..DecodeLimits::default()
    };
    let config = RenderConfig {
        width: WIDTH,
        height: HEIGHT,
        // Renderer clear colours are linear; Atlas #202925 is sRGB.
        background: Some([
            srgb_to_linear(0x20),
            srgb_to_linear(0x29),
            srgb_to_linear(0x25),
            1.0,
        ]),
        ..RenderConfig::isometric()
    };
    let mut succeeded = 0usize;
    let mut failed = 0usize;
    for (line_number, line) in jobs.lines().enumerate() {
        let line = line.context("read job list")?;
        if line.trim().is_empty() {
            continue;
        }
        let started = Instant::now();
        let mut result = match serde_json::from_str::<Job>(&line) {
            Ok(job) => {
                eprintln!("Rendering {}: {}", job.id, job.input.display());
                let outcome = catch_unwind(AssertUnwindSafe(|| {
                    render_job(&job, &pack, &config, &limits)
                }));
                let mut result = match outcome {
                    Ok(Ok(result)) => result,
                    Ok(Err(error)) => json!({"status": "error", "error": format!("{error:#}")}),
                    Err(payload) => {
                        let message = payload.downcast_ref::<String>().cloned()
                            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                            .unwrap_or_else(|| "unknown panic".to_string());
                        json!({"status": "error", "error": format!("render job panicked: {message}")})
                    }
                };
                result["id"] = json!(job.id);
                result["input"] = json!(job.input);
                result["output"] = json!(job.output);
                result
            }
            Err(error) => json!({
                "status": "error", "id": null,
                "error": format!("invalid job JSON: {error}")
            }),
        };
        result["job_line"] = json!(line_number + 1);
        result["duration_ms"] = json!(started.elapsed().as_secs_f64() * 1000.0);
        result["renderer"] = json!("nucleation-native-isometric-v1");
        result["pack_blake3"] = json!(pack_blake3);
        result["width"] = json!(WIDTH);
        result["height"] = json!(HEIGHT);
        if result["status"] == "ok" {
            succeeded += 1;
        } else {
            failed += 1;
        }
        serde_json::to_writer(&mut results, &result)?;
        results.write_all(b"\n")?;
        results.flush()?;
        println!("{result}");
        std::io::stdout().flush()?;
    }
    eprintln!("Finished: {succeeded} rendered, {failed} failed; {}", results_path.display());
    // Job-level failures are data in the manifest, not a failed batch process.
    Ok(())
}

fn render_job(
    job: &Job,
    pack: &ResourcePackSource,
    config: &RenderConfig,
    limits: &DecodeLimits,
) -> Result<Value> {
    if job.input == job.output {
        bail!("input and output must differ");
    }
    if job.output.extension().and_then(|s| s.to_str()) != Some("png") {
        bail!("output must have a .png extension");
    }
    let import_started = Instant::now();
    let metadata = fs::metadata(&job.input).context("inspect input")?;
    if metadata.len() > limits.max_input_bytes as u64 {
        bail!("input exceeds {} byte limit", limits.max_input_bytes);
    }
    let bytes = fs::read(&job.input).context("read input")?;
    let input_blake3 = blake3::hash(&bytes).to_hex().to_string();
    let manager = nucleation::formats::manager::get_manager();
    let (format, schematic) = manager
        .lock()
        // Importers only borrow this registry. A malformed prior job panicking
        // during a read does not mutate it or prevent the next job's import.
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .read_bounded_with_format(&bytes, limits)
        .context("import schematic with production format reader")?;
    drop(bytes);
    let regions = std::iter::once(&schematic.default_region)
        .chain(schematic.other_regions.values());
    let (mut cells, mut non_air, mut entities) = (0u64, 0usize, 0usize);
    for region in regions {
        cells = cells.saturating_add(region.volume() as u64);
        non_air = non_air.saturating_add(region.count_non_air_blocks());
        entities = entities.saturating_add(region.entities.len());
    }
    if cells > MAX_VOLUME as u64 || non_air > MAX_NON_AIR {
        bail!("meshing limit exceeded: {cells} cells / {non_air} non-air blocks; limits {MAX_VOLUME} / {MAX_NON_AIR}");
    }
    if non_air == 0 && entities == 0 {
        bail!("schematic has no blocks or entities to render");
    }
    let import_ms = import_started.elapsed().as_secs_f64() * 1000.0;
    let mesh_started = Instant::now();
    let chunked = cells > SINGLE_MESH_VOLUME;
    let mut meshes = if chunked {
        schematic.mesh_chunks_parallel(pack, &MeshConfig::default(), 64, 4)
    } else {
        schematic.to_mesh(pack, &MeshConfig::default()).map(|m| vec![m])
    }.context("mesh schematic using resource-pack models and textures")?;
    // Chunk collection uses a HashMap; keep renderer mesh order stable.
    meshes.sort_by_key(|mesh| mesh.chunk_coord);
    let triangle_count: usize = meshes.iter().map(|mesh| mesh.total_triangles()).sum();
    if triangle_count == 0 {
        bail!("mesher produced no triangles");
    }
    let mesh_ms = mesh_started.elapsed().as_secs_f64() * 1000.0;
    let render_started = Instant::now();
    let png = render_meshes_png(&meshes, config, None)
        .context("render native GPU thumbnail")?;
    // Validate the actual PNG before atomically exposing it to the web server.
    let decoded = image::load_from_memory_with_format(&png, image::ImageFormat::Png)
        .context("validate rendered PNG")?;
    if decoded.width() != WIDTH || decoded.height() != HEIGHT {
        bail!("renderer returned unexpected PNG dimensions");
    }
    let png_blake3 = blake3::hash(&png).to_hex().to_string();
    create_parent(&job.output)?;
    let temporary = job.output.with_extension(format!("png.{}.tmp", std::process::id()));
    fs::write(&temporary, &png).context("write rendered PNG")?;
    if let Err(error) = fs::rename(&temporary, &job.output) {
        let _ = fs::remove_file(&temporary);
        return Err(error).context("publish rendered PNG");
    }
    let render_ms = render_started.elapsed().as_secs_f64() * 1000.0;
    let mut warnings = Vec::new();
    if entities > 0 {
        warnings.push("Authored entities use the mesher's supported models at floored block positions; not all entity appearances are supported.");
        if chunked {
            warnings.push("Chunked meshing omits entities sharing a cell with a block or another entity.");
        }
    }
    Ok(json!({
        "status": "ok", "format": format, "input_blake3": input_blake3,
        "png_blake3": png_blake3, "png_bytes": png.len(),
        "cells": cells, "non_air_blocks": non_air, "source_entities": entities,
        "chunks": meshes.len(), "triangles": triangle_count,
        "mesh_mode": if chunked { "chunks64_threads4" } else { "single" },
        "import_ms": import_ms, "mesh_ms": mesh_ms, "render_ms": render_ms,
        "simulation_ticks": 0, "warnings": warnings,
        "camera": {"projection": "orthographic", "yaw": config.yaw,
            "pitch": config.pitch, "zoom": config.zoom, "background_srgb": "#202925"}
    }))
}

fn create_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn srgb_to_linear(byte: u8) -> f32 {
    let srgb = f32::from(byte) / 255.0;
    if srgb <= 0.04045 { srgb / 12.92 } else { ((srgb + 0.055) / 1.055).powf(2.4) }
}
