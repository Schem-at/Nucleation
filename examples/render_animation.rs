//! End-to-end animation render: schematic -> groups -> timeline -> PNG frames.
//!
//!     cargo run --release --example render_animation --features rendering -- <pack.zip> <out_dir>
//!
//! Writes `f0000.png`, `f0001.png`, ... ready for
//! `ffmpeg -i 'f%04d.png' out.gif`. Deterministic: rerunning produces
//! byte-identical frames.

use nucleation::animation::{presets, Axis, BuildAnimator, Grouping, Order, Stagger, Target};
use nucleation::meshing::{MeshConfig, ResourcePackSource};
use nucleation::rendering::{render_animation_to_files, RenderConfig};
use nucleation::UniversalSchematic;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let pack_path = args.next().unwrap_or_else(|| {
        eprintln!("usage: render_animation <pack.zip> <out_dir>");
        std::process::exit(2);
    });
    let out_dir = args
        .next()
        .unwrap_or_else(|| "render_work/anim".to_string());
    // `--transparent` clears to alpha 0 so the frames drop into a README on
    // either a light or a dark background.
    let transparent = std::env::args().any(|a| a == "--transparent");
    std::fs::create_dir_all(&out_dir)?;

    let pack = ResourcePackSource::from_file(&pack_path)?;
    let schem = build_scene();

    // 1. Group the build. One group per block, so every block animates alone.
    let mut anim = BuildAnimator::from_schematic(&schem, Grouping::PerBlock);
    println!("groups: {}", anim.groups().len());

    // 2. Blocks drop in and pop to size, bottom-up, as a wave from the centre.
    anim.timeline_mut().add_staggered(
        presets::drop_and_pop(420.0, 6.0),
        &Stagger::each(Order::Axis(Axis::Y, true), 55.0),
        0.0,
    );
    // 3. The camera orbits on the same clock.
    let spin = presets::turntable(anim.duration_ms().max(1.0));
    anim.timeline_mut().add(spin, Target::Camera, 0.0);

    let frames = anim.frames(24.0);
    println!(
        "duration {:.0}ms -> {} frames @24fps",
        anim.duration_ms(),
        frames.len()
    );

    // 4. One MeshOutput per group, index-aligned with the groups — the
    //    contract `render_animation` relies on.
    let cfg = MeshConfig::default();
    let meshes = schem.mesh_groups(&pack, &cfg, anim.groups())?;
    println!("meshes: {}", meshes.len());

    // 5. Render. Camera framing is fixed to the finished build so the shot does
    //    not drift while blocks are still arriving.
    let mut rc = RenderConfig::isometric();
    rc.width = 480;
    rc.height = 360;
    rc.sphere_fit = true;
    if transparent {
        rc.background = Some([0.0, 0.0, 0.0, 0.0]);
    }

    let paths = render_animation_to_files(&meshes, &frames, &rc, None, &format!("{out_dir}/f"))?;
    println!("wrote {} frames to {out_dir}/", paths.len());
    println!("assemble: ffmpeg -y -i '{out_dir}/f%04d.png' {out_dir}/anim.gif");
    Ok(())
}

fn build_scene() -> UniversalSchematic {
    let mut s = UniversalSchematic::new("anim".to_string());
    for x in 0..5 {
        for z in 0..5 {
            s.set_block_from_string(x, 0, z, "minecraft:stone").ok();
        }
    }
    for x in 1..4 {
        for z in 1..4 {
            s.set_block_from_string(x, 1, z, "minecraft:oak_planks")
                .ok();
        }
    }
    s.set_block_from_string(2, 2, 2, "minecraft:glowstone").ok();
    s
}
