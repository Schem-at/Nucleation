//! Generate .nucm test files from real schematics for the browser viewer.
//!
//! Usage:
//!   cargo run --release --example gen_test_nucm --features meshing

use nucleation::formats::manager::get_manager;
use nucleation::meshing::cache;
use nucleation::meshing::{MeshConfig, ResourcePackSource};
use std::path::Path;
use std::time::Instant;

const SCHEMATICS: &[(&str, &str)] = &[
    ("cutecounter", "tests/samples/cutecounter.schem"),
    ("Evaluator", "tests/samples/Evaluator.schem"),
    ("uss-texas", "tests/samples/uss-texas.schem"),
    ("large_schematic", "tests/samples/large_schematic.schem"),
];

fn main() {
    let pack_path = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("MINECRAFT_RESOURCE_PACK").ok())
        .expect(
            "Usage: gen_test_nucm <resource_pack_path>\n  Or set MINECRAFT_RESOURCE_PACK env var",
        );

    let out_dir = Path::new("examples/nucm-viewer/test-files");
    std::fs::create_dir_all(out_dir).unwrap();

    // Load resource pack once
    println!("Loading resource pack...");
    let t = Instant::now();
    let pack = ResourcePackSource::from_file(&pack_path).expect("Failed to load resource pack");
    println!("  Loaded in {:.1}s", t.elapsed().as_secs_f64());

    let manager = get_manager();
    let manager = manager.lock().unwrap();
    let mesh_config = MeshConfig::default();

    for (name, schem_path) in SCHEMATICS {
        println!("\n--- {} ---", name);

        // Load schematic
        let t = Instant::now();
        let schem_bytes = match std::fs::read(schem_path) {
            Ok(b) => b,
            Err(e) => {
                println!("  Skipping: {}", e);
                continue;
            }
        };
        let schematic = manager.read(&schem_bytes).expect("parse schematic");
        let dims = schematic.get_dimensions();
        let volume = dims.0 as i64 * dims.1 as i64 * dims.2 as i64;
        println!(
            "  {}x{}x{} ({} blocks), loaded in {:.0}ms",
            dims.0,
            dims.1,
            dims.2,
            volume,
            t.elapsed().as_secs_f64() * 1000.0
        );

        // Mesh
        let t = Instant::now();
        let meshes = if volume > 500_000 {
            let chunk_size = if volume > 10_000_000 { 32 } else { 64 };
            let threads = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4);
            println!(
                "  Meshing (parallel, chunk_size={}, threads={})...",
                chunk_size, threads
            );
            schematic
                .mesh_chunks_parallel(&pack, &mesh_config, chunk_size, threads)
                .expect("mesh")
        } else {
            println!("  Meshing (single)...");
            let mesh = schematic.to_mesh(&pack, &mesh_config).expect("mesh");
            vec![mesh]
        };

        let total_verts: usize = meshes.iter().map(|m| m.total_vertices()).sum();
        let total_tris: usize = meshes.iter().map(|m| m.total_triangles()).sum();
        println!(
            "  {} chunks, {} verts, {} tris in {:.1}s",
            meshes.len(),
            total_verts,
            total_tris,
            t.elapsed().as_secs_f64()
        );

        // Save .nucm
        let out_path = out_dir.join(format!("{}.nucm", name));
        let t = Instant::now();
        cache::save_cached_mesh(&meshes, &out_path).expect("save nucm");
        let file_size = std::fs::metadata(&out_path).unwrap().len();
        println!(
            "  Saved {} ({:.1} KB) in {:.1}s",
            out_path.display(),
            file_size as f64 / 1024.0,
            t.elapsed().as_secs_f64()
        );
    }

    println!("\nDone! Test files in {}", out_dir.display());
}
