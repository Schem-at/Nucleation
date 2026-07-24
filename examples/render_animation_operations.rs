//! Render a compact proof of region transforms, stamping, temporal meshes, and gizmos.
//!
//! cargo run --release --example render_animation_operations --features rendering -- <pack.zip> [out]

use nucleation::animation::{AnimationEffect, BuildAnimation};
use nucleation::meshing::{MeshConfig, ResourcePackSource};
use nucleation::rendering::{render_animation_to_files, RenderConfig};
use nucleation::UniversalSchematic;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let pack_path = args.next().ok_or("missing resource-pack ZIP")?;
    let out = args
        .next()
        .unwrap_or_else(|| "render_work/animation-operations".to_string());
    std::fs::create_dir_all(&out)?;

    let mut animation = BuildAnimation::new("operation-proof");
    animation.set_default_effect(AnimationEffect::new(0.0));
    animation
        .schematic_mut()
        .create_region("wing".into(), (0, 0, 0), (65, 0, 65));
    for x in 30..34 {
        animation.set_block_in_region("wing", x, 0, 32, "minecraft:oak_stairs[facing=east]")?;
        animation.set_block_in_region("wing", x, 0, 33, "minecraft:copper_block")?;
    }
    animation.rotate_region_y("wing", 90, 900.0)?;

    let mut source = UniversalSchematic::new("module".into());
    source.create_region("module".into(), (30, 0, 32), (31, 0, 32));
    source.try_set_block_in_region_str("module", 30, 0, 32, "minecraft:sea_lantern")?;
    source.try_set_block_in_region_str("module", 31, 0, 32, "minecraft:gold_block")?;
    animation.stamp_region(
        &source,
        "module",
        (36, 0, 32),
        &["minecraft:gold_block".into()],
        700.0,
    )?;

    let pack = ResourcePackSource::from_file(pack_path)?;
    let meshes = animation.mesh_outputs(&pack, &MeshConfig::default())?;
    let frames = animation.frames(12.0, 300.0);
    let mut config = RenderConfig::isometric();
    config.width = 640;
    config.height = 480;
    config.sphere_fit = true;
    config.background = Some([0.025, 0.035, 0.055, 1.0]);
    let paths = render_animation_to_files(&meshes, &frames, &config, None, &format!("{out}/f"))?;
    std::fs::write(
        format!("{out}/receipts.json"),
        serde_json::to_string_pretty(&animation.operation_receipts())?,
    )?;
    println!("rendered {} frames to {}", paths.len(), out);
    Ok(())
}
