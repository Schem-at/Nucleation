//! .nucm cache test: mesh → save, then load → verify → export GLB.
//!
//! Usage:
//!   # Full pipeline: mesh schematic, save .nucm, then load and verify
//!   cargo run --release --example nucm_size_test --features meshing
//!
//!   # Load-only: just load an existing .nucm file
//!   cargo run --release --example nucm_size_test --features meshing -- --load /tmp/IRIS_B.nucm

use nucleation::meshing::cache;
use std::path::Path;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // If --load is passed, skip meshing and go straight to loading
    if let Some(pos) = args.iter().position(|a| a == "--load") {
        let nucm_path = args.get(pos + 1).expect("--load requires a path");
        load_and_verify(Path::new(nucm_path));
        return;
    }

    // Otherwise: full pipeline (mesh → save → load)
    full_pipeline(&args);
}

fn load_and_verify(nucm_path: &Path) {
    let file_size = std::fs::metadata(nucm_path).expect("stat nucm file").len();
    println!(
        "Loading {} ({:.1} MB)...",
        nucm_path.display(),
        file_size as f64 / 1_048_576.0
    );

    let t = Instant::now();
    let meshes = cache::load_cached_mesh(nucm_path).expect("load .nucm");
    let load_time = t.elapsed();

    let total_verts: usize = meshes.iter().map(|m| m.total_vertices()).sum();
    let total_tris: usize = meshes.iter().map(|m| m.total_triangles()).sum();
    let non_empty: usize = meshes.iter().filter(|m| !m.is_empty()).count();

    println!("Loaded in {:.1}s", load_time.as_secs_f64());
    println!("  {} chunks ({} non-empty)", meshes.len(), non_empty);
    println!("  {} vertices, {} triangles", total_verts, total_tris);

    // Per-layer breakdown
    let opaque_v: usize = meshes.iter().map(|m| m.opaque.vertex_count()).sum();
    let cutout_v: usize = meshes.iter().map(|m| m.cutout.vertex_count()).sum();
    let trans_v: usize = meshes.iter().map(|m| m.transparent.vertex_count()).sum();
    println!(
        "  Opaque: {} verts, Cutout: {} verts, Transparent: {} verts",
        opaque_v, cutout_v, trans_v
    );

    // Atlas stats
    let atlas_sizes: Vec<(u32, u32)> = meshes
        .iter()
        .map(|m| (m.atlas.width, m.atlas.height))
        .collect();
    let unique_sizes: std::collections::HashSet<_> = atlas_sizes.iter().collect();
    println!(
        "  Atlas sizes: {} unique across {} chunks",
        unique_sizes.len(),
        meshes.len()
    );
    for size in &unique_sizes {
        let count = atlas_sizes.iter().filter(|s| s == size).count();
        println!("    {}x{}: {} chunks", size.0, size.1, count);
    }

    // Animated texture count
    let anim_count: usize = meshes.iter().map(|m| m.animated_textures.len()).sum();
    println!("  Animated textures: {}", anim_count);

    // Try exporting the largest chunk as GLB to prove mesh data is valid
    if let Some(biggest) = meshes.iter().max_by_key(|m| m.total_vertices()) {
        if !biggest.is_empty() {
            println!("\nExporting largest chunk as GLB...");
            println!(
                "  Chunk coord: {:?}, {} verts, {} tris",
                biggest.chunk_coord,
                biggest.total_vertices(),
                biggest.total_triangles()
            );
            let t = Instant::now();
            match biggest.to_glb() {
                Ok(glb) => {
                    let glb_path = "/tmp/nucm_test_chunk.glb";
                    std::fs::write(glb_path, &glb).expect("write GLB");
                    println!(
                        "  Exported {:.1} KB GLB in {:.0}ms → {}",
                        glb.len() as f64 / 1024.0,
                        t.elapsed().as_millis(),
                        glb_path
                    );
                }
                Err(e) => {
                    println!("  GLB export failed: {}", e);
                }
            }
        }
    }
}

fn full_pipeline(args: &[String]) {
    use nucleation::formats::manager::get_manager;
    use nucleation::meshing::{MeshConfig, ResourcePackSource};

    let schem_path = args
        .get(1)
        .expect("Usage: nucm_size_test <schematic_path> <resource_pack_path>\n  Or: nucm_size_test --load <nucm_path>");
    let pack_env = std::env::var("MINECRAFT_RESOURCE_PACK").ok();
    let pack_path = args.get(2).map(|s| s.as_str())
        .or(pack_env.as_deref())
        .expect("Usage: nucm_size_test <schematic_path> <resource_pack_path>\n  Or set MINECRAFT_RESOURCE_PACK env var");

    // Load schematic
    let t = Instant::now();
    let manager = get_manager();
    let manager = manager.lock().unwrap();
    let schem_bytes = std::fs::read(schem_path).expect("read schematic");
    let schematic = manager.read(&schem_bytes).expect("import schematic");
    println!("Loaded schematic in {:.1}ms", t.elapsed().as_millis());

    let dims = schematic.get_dimensions();
    let volume = dims.0 as i64 * dims.1 as i64 * dims.2 as i64;
    println!(
        "Dimensions: {}x{}x{} ({} blocks)",
        dims.0, dims.1, dims.2, volume
    );
    println!(
        "Schematic file size: {:.1} MB",
        schem_bytes.len() as f64 / 1_048_576.0
    );

    // Load resource pack
    let t = Instant::now();
    let pack = ResourcePackSource::from_file(pack_path).expect("load pack");
    println!("Loaded resource pack in {:.1}ms", t.elapsed().as_millis());

    // Mesh
    let config = MeshConfig::default();
    let chunk_size = 32;
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    println!(
        "Meshing with chunk_size={}, threads={}...",
        chunk_size, threads
    );
    let t = Instant::now();
    let meshes = schematic
        .mesh_chunks_parallel(&pack, &config, chunk_size, threads)
        .expect("mesh");
    let mesh_time = t.elapsed();

    let total_verts: usize = meshes.iter().map(|m| m.total_vertices()).sum();
    let total_tris: usize = meshes.iter().map(|m| m.total_triangles()).sum();
    println!(
        "Meshed {} chunks in {:.1}s ({} verts, {} tris)",
        meshes.len(),
        mesh_time.as_secs_f64(),
        total_verts,
        total_tris
    );

    // Save
    let out_path = "/tmp/IRIS_B.nucm";
    let t = Instant::now();
    cache::save_cached_mesh(&meshes, Path::new(out_path)).expect("save nucm");
    let save_time = t.elapsed();
    let file_size = std::fs::metadata(out_path).unwrap().len();
    println!(
        "Saved to {} ({:.1} MB) in {:.1}s",
        out_path,
        file_size as f64 / 1_048_576.0,
        save_time.as_secs_f64()
    );

    println!("\n--- Now loading back from cache ---\n");
    drop(manager);
    load_and_verify(Path::new(out_path));
}
