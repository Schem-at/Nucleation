//! README illustration: a build assembling itself, bottom to top.
//!
//! This is the canonical shape for every README animation. Each illustration is
//! ONE self-contained script that:
//!   1. builds a schematic with the public API (so the script doubles as an
//!      API-usability test — if this reads badly, the API needs work),
//!   2. saves it as `.schem` *and* `.litematic` (the download beside the image),
//!   3. renders deterministic transparent frames,
//!   4. emits `anchors.json` — per-frame pixel positions of a few labelled
//!      blocks, so a compositor can draw callouts without any text-in-Rust.
//!
//!     cargo run --release --example readme_assemble --features rendering -- <pack.zip> [out_dir]
//!
//! Assemble the frames with:
//!     ffmpeg -i 'out/f%04d.png' -vf "split[a][b];[a]palettegen=reserve_transparent=1[p];[b][p]paletteuse=alpha_threshold=128" out.gif

use nucleation::animation::{presets, Axis, BuildAnimator, Grouping, Order, Stagger, Target};
use nucleation::formats::litematic;
use nucleation::meshing::{MeshConfig, ResourcePackSource};
use nucleation::rendering::{
    animation_view_projs, camera::project_point, render_animation_to_files, RenderConfig,
};
use nucleation::UniversalSchematic;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let pack_path = args.next().unwrap_or_else(|| {
        eprintln!("usage: readme_assemble <pack.zip> [out_dir]");
        std::process::exit(2);
    });
    let out = args
        .next()
        .unwrap_or_else(|| "render_work/assemble".to_string());
    std::fs::create_dir_all(&out)?;

    // ── 1. Build ────────────────────────────────────────────────────────────
    // A small stepped plinth. Plain coordinates and block strings — this is the
    // whole point of the API, and the script is the readability test.
    let mut schem = UniversalSchematic::new("assemble".to_string());
    for x in 0..5 {
        for z in 0..5 {
            schem.set_block_from_string(x, 0, z, "minecraft:stone_bricks")?;
        }
    }
    for x in 1..4 {
        for z in 1..4 {
            schem.set_block_from_string(x, 1, z, "minecraft:polished_andesite")?;
        }
    }
    schem.set_block_from_string(2, 2, 2, "minecraft:sea_lantern")?;

    // ── 2. Save the downloads ───────────────────────────────────────────────
    std::fs::write(format!("{out}/assemble.schem"), schem.to_schematic()?)?;
    std::fs::write(
        format!("{out}/assemble.litematic"),
        litematic::to_litematic(&schem)?,
    )?;

    // ── 3. Animate ──────────────────────────────────────────────────────────
    let mut anim = BuildAnimator::from_schematic(&schem, Grouping::PerBlock);
    anim.timeline_mut().add_staggered(
        presets::drop_and_pop(420.0, 6.0),
        &Stagger::each(Order::Axis(Axis::Y, true), 60.0),
        0.0,
    );
    let spin = presets::turntable(anim.duration_ms().max(1.0));
    anim.timeline_mut().add(spin, Target::Camera, 0.0);

    let frames = anim.frames(20.0);

    // ── 4. Render ───────────────────────────────────────────────────────────
    let pack = ResourcePackSource::from_file(&pack_path)?;
    let meshes = schem.mesh_groups(&pack, &MeshConfig::default(), anim.groups())?;

    let mut rc = RenderConfig::isometric();
    rc.width = 480;
    rc.height = 400;
    rc.sphere_fit = true;
    rc.background = Some([0.0, 0.0, 0.0, 0.0]);

    render_animation_to_files(&meshes, &frames, &rc, None, &format!("{out}/f"))?;

    // ── 5. Overlay anchors ──────────────────────────────────────────────────
    // Where does the lantern sit on screen, each frame? A compositor turns this
    // into a leader line and a "sea_lantern @ (2,2,2)" caption. Block centre,
    // so the label points at the middle of the cube.
    let view_projs = animation_view_projs(&meshes, &frames, &rc);
    let lantern = [2.5, 2.5, 2.5];
    let anchors: Vec<serde_json::Value> = view_projs
        .iter()
        .enumerate()
        .map(
            |(i, vp)| match project_point(vp, lantern, rc.width, rc.height) {
                Some((px, py)) => {
                    serde_json::json!({ "frame": i, "x": px, "y": py, "visible": true })
                }
                None => serde_json::json!({ "frame": i, "visible": false }),
            },
        )
        .collect();
    std::fs::write(
        format!("{out}/anchors.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "label": "sea_lantern @ (2,2,2)",
            "width": rc.width, "height": rc.height,
            "anchors": anchors,
        }))?,
    )?;

    println!(
        "{} blocks, {} frames -> {out}/ (.schem, .litematic, f####.png, anchors.json)",
        anim.groups().len(),
        frames.len()
    );
    Ok(())
}
