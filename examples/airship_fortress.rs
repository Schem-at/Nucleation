//! Voxelize the Mario Kart DS "Airship Fortress" course into a .schem.
//!
//! Prepare the model first (downloads the rip, filters it, converts to GLB):
//!
//!     python3 tools/readme-media/prep_airship.py
//!     cargo run --release --example airship_fortress --features voxelize
//!
//! Writes target/readme-models/airship-fortress.schem.

use nucleation::building::brushes::BlockPalette;
use nucleation::voxelize::{voxelize_textured, MeshModel, MeshShape};
use nucleation::BlockState;

const GLB: &str = "target/readme-models/mkds-airship-fortress.glb";
const OUT: &str = "target/readme-models/airship-fortress.schem";

/// The course is a stone-brick keep with a dark wooden airship moored to it:
/// measured mean texture colors run grey (120,116,107), near-black (58,63,60),
/// wood brown (131,86,6) and a blue-grey (63,85,95). MK64_PALETTE's neon
/// wools/concretes are entirely wrong here, so this is a masonry-and-timber
/// set instead.
const PALETTE: &[&str] = &[
    // Stonework: the keep, towers, ramparts and road surface.
    "minecraft:stone",
    "minecraft:cobblestone",
    "minecraft:stone_bricks",
    "minecraft:mossy_stone_bricks",
    "minecraft:cracked_stone_bricks",
    "minecraft:andesite",
    "minecraft:polished_andesite",
    "minecraft:smooth_stone",
    "minecraft:gray_concrete",
    "minecraft:light_gray_concrete",
    "minecraft:gray_terracotta",
    "minecraft:light_gray_terracotta",
    // The darkest faces: shadowed masonry and the airship's underside.
    "minecraft:deepslate",
    "minecraft:deepslate_bricks",
    "minecraft:polished_deepslate",
    "minecraft:blackstone",
    "minecraft:polished_blackstone_bricks",
    "minecraft:black_concrete",
    "minecraft:black_terracotta",
    // Timber: hull planking, decking, scaffolding and the tower roofs.
    "minecraft:oak_planks",
    "minecraft:spruce_planks",
    "minecraft:dark_oak_planks",
    "minecraft:spruce_log",
    "minecraft:dark_oak_log",
    "minecraft:brown_concrete",
    "minecraft:brown_terracotta",
    "minecraft:packed_mud",
    "minecraft:mud_bricks",
    "minecraft:terracotta",
    // Mid-tone greys. Without these the palette has a hole between deepslate
    // (80,80,83) and stone (126,126,126), and the snap fills it with
    // dark_prismarine (52,92,76) and cyan_terracotta (87,91,91) — whose
    // blockpedia colors are near-neutral even though both read as teal in
    // game. That put ~35% of the build in green/teal on the first pass.
    "minecraft:tuff",
    "minecraft:cobbled_deepslate",
    "minecraft:mossy_cobblestone",
    "minecraft:gray_wool",
];

/// Largest extent of the filtered course is 438 model units, so fitting to
/// 438 lands at ~1 block per model unit. That puts the road at roughly the
/// same 8-9 blocks wide the MK64 scene calibrates to, and keeps the keep's
/// window and crenellation detail legible instead of aliasing away.
const TARGET_SIZE: f32 = 438.0;

/// Surface-only. Airship Fortress is not a closed solid: the course is an
/// open ribbon of road wrapped around a hollow tower, with the airship deck
/// as a separate open shell. A parity interior test fills the tower and the
/// arcs under the road as "enclosed volume"; a negative shell skips parity
/// and keeps a one-block skin.
const SHELL: f32 = -1.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = std::fs::read(GLB)
        .map_err(|e| format!("{GLB}: {e}\nRun: python3 tools/readme-media/prep_airship.py"))?;

    let palette = BlockPalette::from_block_ids(PALETTE.iter().copied());
    println!(
        "palette: {} of {} ids known to blockpedia",
        palette.len(),
        PALETTE.len()
    );

    let mut model = MeshModel::from_glb_bytes(&data)?;
    model.fit(TARGET_SIZE);
    let shape = MeshShape::new(model).with_surface_shell(-SHELL);

    let mut schematic = voxelize_textured(&shape, &palette, "airship_fortress");
    let bb = schematic.get_bounding_box();
    println!(
        "voxelized: {:?} blocks in {:?}",
        schematic.total_blocks(),
        bb.get_dimensions()
    );

    // The rip still carries a few detached scraps (loose scaffolding planks,
    // the odd stray quad). Keep only the largest 26-connected component.
    let occupied: std::collections::HashSet<(i32, i32, i32)> = schematic
        .iter_blocks()
        .filter(|(_, b)| b.name != "minecraft:air")
        .map(|(p, _)| (p.x, p.y, p.z))
        .collect();
    println!("occupied: {}", occupied.len());
    {
        let (mut mn, mut mx) = ([i32::MAX; 3], [i32::MIN; 3]);
        for &(x, y, z) in &occupied {
            for (k, v) in [x, y, z].iter().enumerate() {
                mn[k] = mn[k].min(*v);
                mx[k] = mx[k].max(*v);
            }
        }
        println!(
            "occupied bounds: min {:?} max {:?} extent {:?}",
            mn,
            mx,
            [mx[0] - mn[0], mx[1] - mn[1], mx[2] - mn[2]]
        );
    }

    let mut seen: std::collections::HashSet<(i32, i32, i32)> = Default::default();
    let mut components: Vec<Vec<(i32, i32, i32)>> = Vec::new();
    for &start in &occupied {
        if !seen.insert(start) {
            continue;
        }
        let (mut stack, mut comp) = (vec![start], vec![start]);
        while let Some((x, y, z)) = stack.pop() {
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        let q = (x + dx, y + dy, z + dz);
                        if occupied.contains(&q) && seen.insert(q) {
                            stack.push(q);
                            comp.push(q);
                        }
                    }
                }
            }
        }
        components.push(comp);
    }
    components.sort_by_key(|c| std::cmp::Reverse(c.len()));
    let kept = components.first().map(|c| c.len()).unwrap_or(0);
    println!(
        "components: {} (largest {}, dropping {} blocks)",
        components.len(),
        kept,
        occupied.len() - kept
    );

    let air = BlockState::new("minecraft:air".to_string());
    for comp in components.iter().skip(1) {
        for &(x, y, z) in comp {
            schematic.set_block(x, y, z, &air);
        }
    }

    let bytes = schematic.to_schematic()?;
    std::fs::write(OUT, &bytes)?;
    println!("wrote {OUT} ({} bytes)", bytes.len());

    // Round-trip: .schem normalises to a non-negative origin, so compare
    // counts and extents rather than absolute coordinates.
    let reloaded = nucleation::UniversalSchematic::from_schematic(&bytes)?;
    let solid: Vec<_> = reloaded
        .iter_blocks()
        .filter(|(_, b)| b.name != "minecraft:air")
        .collect();
    let (mut mn, mut mx) = ([i32::MAX; 3], [i32::MIN; 3]);
    let mut histogram: std::collections::HashMap<&str, usize> = Default::default();
    for (p, b) in &solid {
        for (k, v) in [p.x, p.y, p.z].iter().enumerate() {
            mn[k] = mn[k].min(*v);
            mx[k] = mx[k].max(*v);
        }
        *histogram.entry(b.name.as_str()).or_default() += 1;
    }
    println!(
        "reloaded: {} blocks, extent {:?}",
        solid.len(),
        [mx[0] - mn[0], mx[1] - mn[1], mx[2] - mn[2]]
    );
    let mut top: Vec<_> = histogram.into_iter().collect();
    top.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    for (name, n) in top.iter().take(12) {
        println!("  {:>7}  {}", n, name);
    }
    Ok(())
}
