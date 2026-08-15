//! Executable Rust source for docs/features/meshing-and-rendering.md.

use std::error::Error;
use std::fs;

use nucleation::UniversalSchematic;

fn main() -> Result<(), Box<dyn Error>> {
    // --8<-- [start:build]
    let mut scene = UniversalSchematic::new("render_lab".into());
    scene.fill_cuboid_str((-5, 0, -4), (5, 0, 4), "minecraft:polished_deepslate");
    scene.fill_cuboid_str((-4, 1, -3), (4, 1, 3), "minecraft:dark_prismarine");
    for y in 1..5 {
        for x in -5..6 {
            scene.set_block_from_string(x, y, -4, "minecraft:light_blue_stained_glass")?;
            scene.set_block_from_string(x, y, 4, "minecraft:light_blue_stained_glass")?;
        }
        for z in -3..4 {
            scene.set_block_from_string(-5, y, z, "minecraft:light_blue_stained_glass")?;
            scene.set_block_from_string(5, y, z, "minecraft:light_blue_stained_glass")?;
        }
    }
    for y in 1..4 {
        scene.set_block_from_string(0, y, 0, "minecraft:sea_lantern")?;
    }
    scene.set_block_from_string(-3, 1, 0, "minecraft:azalea_leaves[persistent=true]")?;
    scene.set_block_from_string(3, 1, 0, "minecraft:azalea_leaves[persistent=true]")?;
    // --8<-- [end:build]

    let pack_path = std::env::var("NUCLEATION_PACK")
        .unwrap_or_else(|_| "render_work/pack.zip".into());

    // --8<-- [start:mesh]
    use nucleation::meshing::{MeshConfig, ResourcePackSource};

    let pack = ResourcePackSource::from_file(&pack_path)?;
    let mut config = MeshConfig::default();
    config.biome = Some("lush_caves".into());
    let mesh = scene.to_mesh(&pack, &config)?;
    let glb = mesh.to_glb()?;
    assert_eq!(&glb[..4], b"glTF");
    assert!(mesh.has_transparency());
    println!("{} {}", mesh.total_vertices(), mesh.total_triangles());
    // --8<-- [end:mesh]

    // --8<-- [start:render]
    use nucleation::rendering::RenderConfig;

    let mut view = RenderConfig::isometric();
    view.width = 640;
    view.height = 440;
    view.sphere_fit = true;
    scene.render_to_file(&pack, "render-lab.png", &view)?;
    // --8<-- [end:render]

    let glb_out = std::env::var("MESH_RENDER_GLB_OUT")
        .unwrap_or_else(|_| "render-lab.glb".into());
    let schem_out = std::env::var("MESH_RENDER_SCHEM_OUT")
        .unwrap_or_else(|_| "render-lab.schem".into());
    let png_out = std::env::var("MESH_RENDER_PNG_OUT")
        .unwrap_or_else(|_| "render-lab.png".into());
    fs::write(glb_out, glb)?;
    fs::write(schem_out, scene.to_schematic()?)?;
    if png_out != "render-lab.png" {
        scene.render_to_file(&pack, &png_out, &view)?;
    }
    println!(
        "Meshing Rust example: OK ({} vertices, {} triangles)",
        mesh.total_vertices(), mesh.total_triangles()
    );
    Ok(())
}
