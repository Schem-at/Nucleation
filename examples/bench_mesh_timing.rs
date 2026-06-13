// One-shot meshing timer: parse a .schem and run the core to_mesh() pipeline.
// Usage: bench_mesh_timing <resource_pack.zip> <schem_file>
// Prints a TSV line on success: <path>\t<parse_ms>\t<mesh_ms>\t<glb_bytes>
use std::time::Instant;

use nucleation::meshing::{MeshConfig, ResourcePackSource};
use nucleation::schematic::from_schematic;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: bench_mesh_timing <pack.zip> <schem>");
        std::process::exit(2);
    }
    let pack_path = &args[1];
    let schem_path = &args[2];

    let pack = ResourcePackSource::from_file(pack_path).expect("load resource pack");
    let data = std::fs::read(schem_path).expect("read schem file");

    let t_parse = Instant::now();
    let schem = from_schematic(&data).expect("parse schem");
    let parse_ms = t_parse.elapsed().as_millis();

    let config = if std::env::var("BENCH_GREEDY").is_ok() {
        MeshConfig::default().with_greedy_meshing(true)
    } else {
        MeshConfig::default()
    };

    // Correctness mode: emit geometry invariants that must be preserved across
    // optimizations (counts + order-independent position checksum). Vertex order
    // may legitimately change (e.g. parallel meshing), so we sum positions and
    // index values in an order-insensitive way.
    if std::env::var("BENCH_CHECK").is_ok() {
        let raw = schem.to_raw_mesh(&pack, &config).expect("to_raw_mesh");
        let pos = raw.positions_flat();
        let idx = raw.indices();
        let pos_sum: f64 = pos.iter().map(|v| *v as f64).sum();
        let pos_absum: f64 = pos.iter().map(|v| (*v as f64).abs()).sum();
        let idx_sum: u64 = idx.iter().map(|v| *v as u64).sum();
        println!(
            "CHECK\tverts={}\tindices={}\tpos_sum={:.3}\tpos_absum={:.3}\tidx_sum={}",
            raw.vertex_count(),
            idx.len(),
            pos_sum,
            pos_absum,
            idx_sum
        );
        return;
    }

    // GLB export mode: write a .glb to the path in BENCH_GLB for visual checking,
    // timing to_mesh and to_glb separately.
    if let Ok(out_path) = std::env::var("BENCH_GLB") {
        let t_mesh = Instant::now();
        let mesh = schem.to_mesh(&pack, &config).expect("to_mesh");
        let mesh_ms = t_mesh.elapsed().as_millis();
        let t_glb = Instant::now();
        let glb = mesh.to_glb().expect("to_glb");
        let glb_ms = t_glb.elapsed().as_millis();
        std::fs::write(&out_path, &glb).expect("write glb");
        println!(
            "wrote {} ({} bytes)  to_mesh={}ms  to_glb={}ms",
            out_path,
            glb.len(),
            mesh_ms,
            glb_ms
        );
        return;
    }

    let t_mesh = Instant::now();
    let _mesh = schem.to_mesh(&pack, &config).expect("to_mesh");
    let mesh_ms = t_mesh.elapsed().as_millis();

    // Emit core meshing time to stderr immediately so it's captured even if the
    // process is killed later. GLB export is intentionally excluded here — we are
    // benchmarking the CORE to_mesh() pipeline only.
    eprintln!("MESH_MS={}", mesh_ms);
    println!("{}\t{}\t{}", schem_path, parse_ms, mesh_ms);
}
