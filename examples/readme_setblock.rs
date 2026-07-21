//! README illustration: placing blocks, with the code highlighted in sync.
//!
//! Emits transparent animation frames plus `timing.json` (which code line is
//! "live" at each frame). The compositor `tools/readme-media/compose_code.py`
//! turns those into a side-by-side GIF: code on the left, build on the right,
//! the active line lighting up as its block drops in.
//!
//!     cargo run --release --example readme_setblock --features rendering -- <pack.zip> [out_dir]

use nucleation::animation::{presets, Axis, BuildAnimator, Grouping, Order, Stagger};
use nucleation::formats::litematic;
use nucleation::meshing::{MeshConfig, ResourcePackSource};
use nucleation::rendering::{render_animation_to_files, RenderConfig};
use nucleation::UniversalSchematic;

// The Python a reader would write, paired with the block index each line places
// (None = a line that places nothing: the constructor and the save).
const CODE: &[(&str, Option<usize>)] = &[
    ("s = Schematic(\"pillar\")", None),
    ("s.set_block(0, 0, 0, \"minecraft:stone\")", Some(0)),
    ("s.set_block(0, 1, 0, \"minecraft:cobblestone\")", Some(1)),
    (
        "s.set_block(0, 2, 0, \"minecraft:mossy_cobblestone\")",
        Some(2),
    ),
    ("s.set_block(0, 3, 0, \"minecraft:sea_lantern\")", Some(3)),
    ("s.save_to_file(\"pillar.schem\")", None),
];
const BLOCKS: &[&str] = &[
    "minecraft:stone",
    "minecraft:cobblestone",
    "minecraft:mossy_cobblestone",
    "minecraft:sea_lantern",
];

const FPS: f64 = 20.0;
const INTRO_MS: f32 = 500.0;
const EACH_MS: f32 = 460.0;
const CLIP_MS: f32 = 460.0;
const OUTRO_MS: f32 = 1000.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let pack_path = args.next().unwrap_or_else(|| {
        eprintln!("usage: readme_setblock <pack.zip> [out_dir]");
        std::process::exit(2);
    });
    let out = args
        .next()
        .unwrap_or_else(|| "render_work/setblock".to_string());
    std::fs::create_dir_all(&out)?;

    // Build the pillar the code describes, then save the downloads.
    let mut s = UniversalSchematic::new("pillar".to_string());
    for (y, block) in BLOCKS.iter().enumerate() {
        s.set_block_from_string(0, y as i32, 0, block)?;
    }
    std::fs::write(format!("{out}/pillar.schem"), s.to_schematic()?)?;
    std::fs::write(
        format!("{out}/pillar.litematic"),
        litematic::to_litematic(&s)?,
    )?;

    // Blocks drop in bottom-to-top — the order the code places them — after a
    // short intro so the empty scene reads first.
    let mut anim = BuildAnimator::from_schematic(&s, Grouping::PerBlock);
    anim.timeline_mut().add_staggered(
        presets::drop_and_pop(CLIP_MS, 5.0),
        &Stagger::each(Order::Axis(Axis::Y, true), EACH_MS),
        INTRO_MS,
    );

    let n = BLOCKS.len() as f32;
    let outro_start = INTRO_MS + (n - 1.0) * EACH_MS + CLIP_MS;
    let total = outro_start + OUTRO_MS;
    let frame_count = (total as f64 / 1000.0 * FPS).round() as usize;

    // active_line for each frame: constructor during intro, each set_block line
    // while its block drops, the save line once the build stands.
    let save_line = CODE.len() - 1;
    let active: Vec<usize> = (0..frame_count)
        .map(|i| {
            let t = i as f32 * 1000.0 / FPS as f32;
            if t < INTRO_MS {
                0
            } else if t >= outro_start {
                save_line
            } else {
                let bi = (((t - INTRO_MS) / EACH_MS).floor() as usize).min(BLOCKS.len() - 1);
                1 + bi // set_block lines start at index 1
            }
        })
        .collect();

    let frames: Vec<_> = (0..frame_count)
        .map(|i| anim.frame_at(i as f32 * 1000.0 / FPS as f32))
        .collect();

    let pack = ResourcePackSource::from_file(&pack_path)?;
    let meshes = s.mesh_groups(&pack, &MeshConfig::default(), anim.groups())?;
    let mut rc = RenderConfig::isometric();
    rc.width = 360;
    rc.height = 420;
    rc.sphere_fit = true;
    rc.background = Some([0.0, 0.0, 0.0, 0.0]);
    render_animation_to_files(&meshes, &frames, &rc, None, &format!("{out}/f"))?;

    // The compositor reads this: the code lines and, per frame, the live line.
    let code: Vec<&str> = CODE.iter().map(|(l, _)| *l).collect();
    std::fs::write(
        format!("{out}/timing.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "title": "Placing blocks",
            "code": code,
            "anim_w": rc.width, "anim_h": rc.height,
            "active": active,
        }))?,
    )?;

    println!(
        "{} frames -> {out}/ (f####.png, timing.json, .schem, .litematic)",
        frame_count
    );
    Ok(())
}
