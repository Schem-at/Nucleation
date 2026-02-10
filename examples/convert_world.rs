use nucleation::formats::world;

fn main() {
    let world_path = std::env::args()
        .nth(1)
        .expect("Usage: convert_world <world.zip> [resource_pack.jar]");
    let pack_path = std::env::args().nth(2);

    let data = std::fs::read(&world_path).expect("Failed to read world file");
    println!("Loading world: {} ({} bytes)", world_path, data.len());

    let schematic = world::from_world_zip(&data).expect("Failed to import world");
    let bb = schematic.default_region.get_bounding_box();
    println!(
        "Loaded: {} blocks, {} entities, {} block entities",
        {
            let (min_x, min_y, min_z) = bb.min;
            let (max_x, max_y, max_z) = bb.max;
            let mut count = 0usize;
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    for x in min_x..=max_x {
                        if let Some(b) = schematic.default_region.get_block(x, y, z) {
                            if b.name != "minecraft:air" {
                                count += 1;
                            }
                        }
                    }
                }
            }
            count
        },
        schematic.default_region.entities.len(),
        schematic.default_region.block_entities.len()
    );

    // Export to Litematic
    let litematic_data = nucleation::formats::litematic::to_litematic(&schematic)
        .expect("Failed to export litematic");
    let litematic_path = world_path
        .replace(".zip", ".litematic")
        .replace(".mca", ".litematic");
    std::fs::write(&litematic_path, &litematic_data).expect("Failed to write litematic");
    println!(
        "Saved litematic: {} ({} bytes)",
        litematic_path,
        litematic_data.len()
    );

    // Export to GLB (requires resource pack)
    if let Some(pack_path) = pack_path {
        #[cfg(feature = "meshing")]
        {
            use nucleation::meshing::{MeshConfig, ResourcePackSource};

            println!("Loading resource pack: {}", pack_path);
            let pack =
                ResourcePackSource::from_file(&pack_path).expect("Failed to load resource pack");

            let config = MeshConfig::new()
                .with_culling(true)
                .with_ambient_occlusion(true)
                .with_greedy_meshing(true);

            println!("Generating mesh...");
            let result = schematic
                .to_mesh(&pack, &config)
                .expect("Failed to generate mesh");

            let glb_path = world_path.replace(".zip", ".glb").replace(".mca", ".glb");
            std::fs::write(&glb_path, &result.glb_data).expect("Failed to write GLB");
            println!(
                "Saved GLB: {} ({} bytes, {} vertices, {} triangles)",
                glb_path,
                result.glb_data.len(),
                result.vertex_count,
                result.triangle_count
            );
        }

        #[cfg(not(feature = "meshing"))]
        {
            let _ = pack_path;
            eprintln!("Error: meshing feature not enabled. Build with --features meshing");
        }
    } else {
        println!("Skipping GLB export (no resource pack provided)");
        println!("  To export GLB: convert_world <world.zip> <client.jar>");
    }
}
