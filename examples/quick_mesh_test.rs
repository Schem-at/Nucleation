use nucleation::formats::manager::get_manager;
use nucleation::meshing::{MeshConfig, ResourcePackSource};

fn main() {
    // Load a sample schematic
    let schem_path = std::env::args().nth(2).unwrap_or_else(|| {
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/samples/and.schem").to_string()
    });
    let schem_data = std::fs::read(schem_path).expect("Failed to read schematic");

    let manager = get_manager();
    let manager = manager.lock().unwrap();
    let schematic = manager
        .read(&schem_data)
        .expect("Failed to parse schematic");

    let bb = schematic.default_region.get_bounding_box();
    println!("Schematic bounding box: {:?}", bb);

    // Load resource pack
    let pack_path = std::env::args().nth(1).or_else(|| std::env::var("MINECRAFT_RESOURCE_PACK").ok())
        .expect("Usage: quick_mesh_test <resource_pack_path> [schematic_path]\n  Or set MINECRAFT_RESOURCE_PACK env var");
    println!("Loading resource pack: {}", pack_path);
    let pack = ResourcePackSource::from_file(&pack_path).expect("Failed to load resource pack");
    let stats = pack.stats();
    println!(
        "Resource pack: {} blockstates, {} models, {} textures",
        stats.blockstate_count, stats.model_count, stats.texture_count
    );

    // Mesh
    let config = MeshConfig::default();
    println!("Meshing...");
    let mesh = schematic
        .to_mesh(&pack, &config)
        .expect("Failed to mesh schematic");

    println!(
        "Mesh result: {} vertices, {} triangles",
        mesh.total_vertices(),
        mesh.total_triangles()
    );
    println!("  Opaque:      {} verts", mesh.opaque.vertex_count());
    println!("  Cutout:      {} verts", mesh.cutout.vertex_count());
    println!("  Transparent: {} verts", mesh.transparent.vertex_count());

    // Export GLB
    let glb = mesh.to_glb().expect("Failed to export GLB");
    let out_path = "/tmp/nucleation_test.glb";
    std::fs::write(out_path, &glb).expect("Failed to write GLB");
    println!("\nWrote {} bytes to {}", glb.len(), out_path);
    println!("Open with: open {}", out_path);
}
