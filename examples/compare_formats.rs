//! Compare .nucm vs .glb file sizes for the same schematics.
//!
//! Usage:
//!   cargo run --release --example compare_formats --features meshing

use nucleation::formats::manager::get_manager;
use nucleation::meshing::cache;
use nucleation::meshing::{MeshConfig, ResourcePackSource};
use std::time::Instant;

const SCHEMATICS: &[(&str, &str)] = &[
    ("cutecounter", "tests/samples/cutecounter.schem"),
    ("Evaluator", "tests/samples/Evaluator.schem"),
    ("uss-texas", "tests/samples/uss-texas.schem"),
];

fn main() {
    let pack_path = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("MINECRAFT_RESOURCE_PACK").ok())
        .expect(
            "Usage: compare_formats <resource_pack_path>\n  Or set MINECRAFT_RESOURCE_PACK env var",
        );
    let pack = ResourcePackSource::from_file(&pack_path).expect("load pack");
    let manager = get_manager();
    let manager = manager.lock().unwrap();
    let config = MeshConfig::default();

    println!(
        "{:<20} {:>10} {:>10} {:>10} {:>8}",
        "Schematic", "Verts", "GLB", "NUCM", "Ratio"
    );
    println!("{}", "-".repeat(62));

    for (name, path) in SCHEMATICS {
        let bytes = std::fs::read(path).expect("read");
        let schematic = manager.read(&bytes).expect("parse");
        let mesh = schematic.to_mesh(&pack, &config).expect("mesh");

        let verts = mesh.total_vertices();

        // GLB
        let t = Instant::now();
        let glb = mesh.to_glb().expect("glb");
        let glb_time = t.elapsed();

        // NUCM
        let t = Instant::now();
        let nucm = cache::serialize_meshes(&[mesh]);
        let nucm_time = t.elapsed();

        let ratio = nucm.len() as f64 / glb.len() as f64;
        println!(
            "{:<20} {:>10} {:>9.1}K {:>9.1}K {:>7.2}x",
            name,
            verts,
            glb.len() as f64 / 1024.0,
            nucm.len() as f64 / 1024.0,
            ratio,
        );
        println!(
            "{:<20} {:>10} {:>8.0}ms {:>8.0}ms",
            "",
            "",
            glb_time.as_secs_f64() * 1000.0,
            nucm_time.as_secs_f64() * 1000.0,
        );
    }
}
