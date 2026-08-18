//! Mesh a small schematic containing both blocks and entities, and export
//! the result as GLB so entity rendering can be inspected visually.
//!
//! Run with: `cargo run --features meshing --example entity_mesh_test <pack.zip>`
//!
//! The resource pack is not in the repo, so it must be supplied: either as the
//! first argument, or via the `NUCLEATION_PACK` environment variable.
//! Output: /tmp/entity_mesh_test.glb

use nucleation::meshing::{MeshConfig, ResourcePackSource};
use nucleation::{BlockState, Entity, UniversalSchematic};

const OUT_PATH: &str = "/tmp/entity_mesh_test.glb";

fn main() {
    let pack_path = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("NUCLEATION_PACK").ok())
        .unwrap_or_else(|| {
            eprintln!(
                "no resource pack given.\n\
                 Pass one as the first argument, or set NUCLEATION_PACK to a pack .zip:\n\
                 \x20 cargo run --features meshing --example entity_mesh_test -- /path/to/pack.zip"
            );
            std::process::exit(1);
        });

    let mut s = UniversalSchematic::new("entity_mesh_test".to_string());
    // A small 3x1x3 stone platform so entities stand on something.
    for x in 0..3 {
        for z in 0..3 {
            s.set_block(x, 0, z, &BlockState::new("minecraft:stone".to_string()));
        }
    }
    // Entities above the platform. Positions are block-centered at y=1.
    s.add_entity(Entity::new(
        "minecraft:creeper".to_string(),
        (0.5, 1.0, 0.5),
    ));
    s.add_entity(Entity::new(
        "minecraft:villager".to_string(),
        (2.5, 1.0, 0.5),
    ));
    s.add_entity(Entity::new("minecraft:sheep".to_string(), (1.5, 1.0, 2.5)));

    println!("loading pack: {}", pack_path);
    let pack = ResourcePackSource::from_file(&pack_path).expect("load pack");
    let stats = pack.stats();
    println!(
        "pack: {} blockstates, {} models, {} textures",
        stats.blockstate_count, stats.model_count, stats.texture_count
    );

    let config = MeshConfig::default();
    println!("meshing...");
    let mesh = s.to_mesh(&pack, &config).expect("mesh");
    println!(
        "mesh: {} verts ({} opaque, {} cutout, {} transparent), {} tris",
        mesh.total_vertices(),
        mesh.opaque.vertex_count(),
        mesh.cutout.vertex_count(),
        mesh.transparent.vertex_count(),
        mesh.total_triangles()
    );

    let glb = mesh.to_glb().expect("glb");
    std::fs::write(OUT_PATH, &glb).expect("write");
    println!("\nwrote {} bytes to {}", glb.len(), OUT_PATH);
    println!("open with: open {}", OUT_PATH);
}
