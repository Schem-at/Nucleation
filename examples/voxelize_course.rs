//! Generic course voxelizer: GLB -> textured .schem.
//!
//!     cargo run --release --example voxelize_course --features voxelize -- \
//!         <in.glb> <out.schem> <name> [target_size] [shell]
//!
//! Generalized from examples/airship_fortress.rs, which documents why the
//! defaults are what they are. Unlike that one this builds its palette from
//! all of blockpedia rather than a hand-picked list -- see `palette()`.

use nucleation::building::brushes::BlockPalette;
use nucleation::voxelize::{voxelize_textured, MeshModel, MeshShape};
use nucleation::BlockState;

/// Every full opaque cube blockpedia knows a color for, minus the ones that
/// would misbehave in a pasted build. A wide palette is the point: colour
/// matching is only as good as its densest neighbourhood, and the failure we
/// hit hand-picking 30 blocks for Airship Fortress was a *gap* -- mid-greys
/// were absent, so near-neutral oddballs like cyan_terracotta won them.
///
/// Exclusions, all for behaviour rather than colour:
///   * gravity blocks (sand, gravel, concrete powder) collapse on paste,
///   * light emitters bake glow into what should be flat masonry,
///   * block entities and technical blocks (spawner, command, barrier, jigsaw)
///     carry NBT or are non-survival,
///   * infested blocks look like stone but are a trap,
///   * TNT, because a 600k-block course should not be a bomb.
fn palette() -> BlockPalette {
    const GRAVITY: &[&str] = &["sand", "gravel", "concrete_powder", "anvil", "scaffolding"];
    const TECHNICAL: &[&str] = &[
        "spawner",
        "command_block",
        "barrier",
        "jigsaw",
        "structure_block",
        "structure_void",
        "light",
        "budding_amethyst",
        "reinforced_deepslate",
        "infested",
        "tnt",
        "slime_block",
        "honey_block",
        "sculk_",
    ];
    let ids: Vec<&'static str> = nucleation::blockpedia::all_blocks()
        .filter(|f| f.full_cube && !f.transparent)
        .filter(|f| f.emit_light == 0 && !f.has_block_entity)
        .filter(|f| f.extras.color.is_some())
        .filter(|f| {
            let name = f.id.strip_prefix("minecraft:").unwrap_or(f.id);
            !GRAVITY.iter().any(|g| name.contains(g)) && !TECHNICAL.iter().any(|t| name.contains(t))
        })
        .map(|f| f.id)
        .collect();
    println!("palette: {} blocks", ids.len());
    BlockPalette::from_block_ids(ids.into_iter())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() < 3 {
        eprintln!("usage: voxelize_course <in.glb> <out.schem> <name> [target_size] [shell]");
        std::process::exit(2);
    }
    let (glb_path, out_path, name) = (&a[0], &a[1], &a[2]);
    let target_size: f32 = a.get(3).map_or(Ok(400.0), |s| s.parse())?;
    // Negative = surface-only (skip the parity interior fill). Courses are
    // open ribbons that dip and cross over themselves; parity floods those
    // arcs as enclosed volume.
    let shell: f32 = a.get(4).map_or(Ok(-1.0), |s| s.parse())?;

    let data = std::fs::read(glb_path).map_err(|e| format!("{glb_path}: {e}"))?;
    let mut model = MeshModel::from_glb_bytes(&data)?;
    model.fit(target_size);
    let mesh = MeshShape::new(model);
    let shape = if shell < 0.0 {
        mesh.with_surface_shell(-shell)
    } else {
        mesh.with_shell(shell)
    };

    let mut schematic = voxelize_textured(&shape, &palette(), name);

    let occupied: std::collections::HashSet<(i32, i32, i32)> = schematic
        .iter_blocks()
        .filter(|(_, b)| b.name != "minecraft:air")
        .map(|(p, _)| (p.x, p.y, p.z))
        .collect();
    if occupied.is_empty() {
        return Err(format!("{name}: voxelized to nothing").into());
    }

    // Drop 26-connected components below MIN_COMPONENT blocks. Keeping only
    // the largest (what airship_fortress.rs does, where the rest was 0.03%)
    // does not generalize: Waluigi Pinball's bumpers, signage and the balls
    // themselves are legitimately detached, and largest-only threw away 4.2%
    // of the course. The prep step already removed off-course billboards
    // geometrically, so anything sizeable left here is real.
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
    const MIN_COMPONENT: usize = 64;
    components.sort_by_key(|c| std::cmp::Reverse(c.len()));
    let (keep, junk): (Vec<_>, Vec<_>) = components.iter().partition(|c| c.len() >= MIN_COMPONENT);
    let kept: usize = keep.iter().map(|c| c.len()).sum();
    let air = BlockState::new("minecraft:air".to_string());
    for comp in &junk {
        for &(x, y, z) in comp.iter() {
            schematic.set_block(x, y, z, &air);
        }
    }

    let (mut mn, mut mx) = ([i32::MAX; 3], [i32::MIN; 3]);
    for &(x, y, z) in keep.iter().flat_map(|c| c.iter()) {
        for (k, v) in [x, y, z].iter().enumerate() {
            mn[k] = mn[k].min(*v);
            mx[k] = mx[k].max(*v);
        }
    }
    let extent = [mx[0] - mn[0] + 1, mx[1] - mn[1] + 1, mx[2] - mn[2] + 1];

    let bytes = schematic.to_schematic()?;
    std::fs::write(out_path, &bytes)?;

    // Round-trip before declaring success.
    let reloaded = nucleation::UniversalSchematic::from_schematic(&bytes)?;
    let back = reloaded
        .iter_blocks()
        .filter(|(_, b)| b.name != "minecraft:air")
        .count();

    println!(
        "RESULT\t{name}\t{kept}\t{}\t{}x{}x{}\t{}/{}\t{}\t{}",
        occupied.len() - kept,
        extent[0],
        extent[1],
        extent[2],
        keep.len(),
        components.len(),
        bytes.len(),
        if back == kept {
            "roundtrip-ok"
        } else {
            "ROUNDTRIP-MISMATCH"
        }
    );
    Ok(())
}
