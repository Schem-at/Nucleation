//! Diff demo: load a schematic, make a modified copy (translate + palette swap +
//! add/remove blocks), diff them, and write a markdown report + the marker /
//! added / removed schematics you can load in-game or render.
//!
//!   cargo run --example diff_demo -- [schematic.litematic]   (default: 4bit_adder.litematic)

use nucleation::diff::regions::regions;
use nucleation::diff::{diff, Diff, DiffSpec};
use nucleation::fingerprint::FingerprintSpec;
use nucleation::{BlockState, UniversalSchematic};

/// Set a (2r+1)³ cube of `block` centered on `p` — makes markers bigger than a
/// single block so they're easy to see.
fn place_cube(o: &mut UniversalSchematic, p: (i32, i32, i32), r: i32, block: &BlockState) {
    for dx in -r..=r {
        for dy in -r..=r {
            for dz in -r..=r {
                o.set_block(p.0 + dx, p.1 + dy, p.2 + dz, block);
            }
        }
    }
}

/// Paint the diff as cube markers: added=lime, removed=red, changed=yellow,
/// palette-swapped=light blue (so a repaint is visible, not just collapsed).
fn highlight(o: &mut UniversalSchematic, d: &Diff, r: i32) {
    let lime = BlockState::new("minecraft:lime_stained_glass");
    let red = BlockState::new("minecraft:red_stained_glass");
    let yellow = BlockState::new("minecraft:yellow_stained_glass");
    let blue = BlockState::new("minecraft:light_blue_stained_glass");
    for (p, _) in &d.added {
        place_cube(o, *p, r, &lime);
    }
    for (p, _) in &d.removed {
        place_cube(o, *p, r, &red);
    }
    for (p, _, _) in &d.changed {
        place_cube(o, *p, r, &yellow);
    }
    for (p, _, _) in &d.swapped {
        place_cube(o, *p, r, &blue);
    }
}

/// Just the cube markers (no build).
fn markers_only(d: &Diff, r: i32) -> UniversalSchematic {
    let mut o = UniversalSchematic::new("markers".to_string());
    highlight(&mut o, d, r);
    o
}

#[cfg(feature = "meshing")]
fn mesh_glb(schem: &UniversalSchematic, pack_path: &str) -> Option<Vec<u8>> {
    use nucleation::meshing::{MeshConfig, ResourcePackSource};
    let pack = match ResourcePackSource::from_file(pack_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("resource pack load failed ({pack_path}): {e}");
            return None;
        }
    };
    match schem.to_mesh(&pack, &MeshConfig::default()) {
        Ok(mesh) => match mesh.to_glb() {
            Ok(glb) => Some(glb),
            Err(e) => {
                eprintln!("glb export failed: {e}");
                None
            }
        },
        Err(e) => {
            eprintln!("mesh failed: {e}");
            None
        }
    }
}

#[cfg(feature = "meshing")]
fn write_glb(schem: &UniversalSchematic, pack_path: &str, out: &str) {
    if let Some(glb) = mesh_glb(schem, pack_path) {
        std::fs::write(out, &glb).ok();
        eprintln!("wrote {out} ({} bytes)", glb.len());
    }
}

#[cfg(not(feature = "meshing"))]
fn write_glb(_schem: &UniversalSchematic, _pack_path: &str, out: &str) {
    eprintln!("(skipping {out}: build with --features meshing for GLB export)");
}

/// Build a modified copy of `a`: shift by `shift`, swap the build's dominant
/// block to a contrasting one (a real palette swap), drop every 40th block
/// (removes), and add a few glass blocks (adds). Returns (modified, swap_pair).
fn modify(
    a: &UniversalSchematic,
    shift: (i32, i32, i32),
) -> (UniversalSchematic, (String, String)) {
    use std::collections::HashMap;
    let cells: Vec<((i32, i32, i32), BlockState)> = a
        .iter_blocks()
        .filter(|(_, bs)| bs.get_name() != "minecraft:air")
        .map(|(p, bs)| ((p.x, p.y, p.z), bs.clone()))
        .collect();

    // dominant (most common) block — that's what we'll swap
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for (_, bs) in &cells {
        *counts.entry(bs.get_name()).or_default() += 1;
    }
    let dominant = counts
        .iter()
        .max_by_key(|(_, c)| **c)
        .map(|(n, _)| n.to_string())
        .unwrap_or_else(|| "minecraft:stone".to_string());
    let swap_to = if dominant != "minecraft:white_concrete" {
        "minecraft:white_concrete"
    } else {
        "minecraft:gray_concrete"
    }
    .to_string();
    let swap_block = BlockState::new(swap_to.clone());
    let glass = BlockState::new("minecraft:glass");

    let mut b = UniversalSchematic::new("modified".to_string());
    for (i, ((x, y, z), bs)) in cells.iter().enumerate() {
        if i % 40 == 13 {
            continue; // remove
        }
        let blk = if bs.get_name() == dominant {
            &swap_block // the palette swap
        } else {
            bs
        };
        b.set_block(x + shift.0, y + shift.1, z + shift.2, blk);
    }
    let (mx, _, mz) = shift;
    for k in 0..3 {
        b.set_block(mx - 2 - k, 0, mz, &glass); // additions
    }
    (b, (dominant, swap_to))
}

fn report_section(md: &mut String, label: &str, d: &Diff) {
    use std::fmt::Write as _;
    let regs = regions(d);
    let _ = writeln!(md, "## {label}\n");
    let _ = writeln!(md, "| metric | value |");
    let _ = writeln!(md, "|---|---|");
    let (tx, ty, tz) = d.transform.translate;
    let _ = writeln!(md, "| recovered translate | ({tx}, {ty}, {tz}) |");
    let _ = writeln!(md, "| edit distance | {} |", d.distance);
    let _ = writeln!(md, "| alignment support | {:.2} |", d.support);
    let _ = writeln!(md, "| added | {} |", d.added.len());
    let _ = writeln!(md, "| removed | {} |", d.removed.len());
    let _ = writeln!(md, "| changed | {} |", d.changed.len());
    let _ = writeln!(md, "| swapped cells | {} |", d.swapped.len());
    let _ = writeln!(
        md,
        "| palette swaps | {} |",
        d.palette_swaps
            .iter()
            .map(|(a, b)| format!("{a}→{b}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let _ = writeln!(md, "| change regions | {} |\n", regs.len());
    if !regs.is_empty() {
        let _ = writeln!(md, "| region | kind | bbox | cells |");
        let _ = writeln!(md, "|---|---|---|---|");
        for (i, r) in regs.iter().take(10).enumerate() {
            let _ = writeln!(
                md,
                "| {} | {:?} | ({},{},{})–({},{},{}) | {} |",
                i + 1,
                r.kind,
                r.min.0,
                r.min.1,
                r.min.2,
                r.max.0,
                r.max.1,
                r.max.2,
                r.count
            );
        }
        let _ = writeln!(md);
    }
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "4bit_adder.litematic".to_string());
    let a = match UniversalSchematic::open(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("could not open {path}: {e}");
            std::process::exit(1);
        }
    };
    let (b, (from, to)) = modify(&a, (7, 0, 4));
    let swapped_cells = a
        .iter_blocks()
        .filter(|(_, bs)| bs.get_name() == from)
        .count();

    let mut md = format!(
        "# Diff demo — `{path}`\n\nB = A shifted by (7,0,4); `{from}` → `{to}` ({swapped_cells} cells); ~1/40 blocks removed; 3 glass added.\n\n"
    );

    // exact: counts material — the swap collapses {swapped_cells} changes into 1 op
    let d_exact = diff(&a, &b, &DiffSpec::from_preset(FingerprintSpec::exact()));
    report_section(&mut md, "Preset `exact` (material-sensitive)", &d_exact);

    // redstone_computational: functional — material swaps & orientation are free
    let d_redstone = diff(
        &a,
        &b,
        &DiffSpec::from_preset(FingerprintSpec::redstone_computational()),
    );
    report_section(
        &mut md,
        "Preset `redstone_computational` (functional)",
        &d_redstone,
    );

    md.push_str("\n## JSON (exact)\n\n```json\n");
    md.push_str(&d_exact.to_json());
    md.push_str("\n```\n");

    std::fs::write("diff_report.md", &md).expect("write report");
    d_exact
        .markers()
        .save("diff_markers.schem", None)
        .expect("save markers");
    d_exact
        .added()
        .save("diff_added.schem", None)
        .expect("save added");
    d_exact
        .removed()
        .save("diff_removed.schem", None)
        .expect("save removed");

    eprintln!(
        "exact: dist={} (translate {:?}, swaps={}, +{} -{} ~{})",
        d_exact.distance,
        d_exact.transform.translate,
        d_exact.palette_swaps.len(),
        d_exact.added.len(),
        d_exact.removed.len(),
        d_exact.changed.len()
    );
    // GLB visualisations (needs --features meshing + a resource pack)
    let pack = std::env::var("PACK").unwrap_or_else(|_| "render_work/pack.zip".to_string());
    write_glb(&a, &pack, "diff_before.glb"); // original
    write_glb(&b, &pack, "diff_after.glb"); // modified
    write_glb(&markers_only(&d_exact, 0), &pack, "diff_markers.glb"); // glass markers (in-game)
                                                                      // Overlay: the textured after-build with glowing emissive boxes on top.
    #[cfg(feature = "meshing")]
    if let Some(after) = mesh_glb(&b, &pack) {
        let opts = nucleation::diff::OverlayOptions {
            inflate: 0.08,
            ..Default::default()
        };
        match d_exact.to_overlay_glb(&after, &opts) {
            Ok(glb) => {
                std::fs::write("diff_overlay.glb", &glb).ok();
                eprintln!(
                    "wrote diff_overlay.glb ({} bytes, glowing markers)",
                    glb.len()
                );
            }
            Err(e) => eprintln!("glow overlay failed: {e}"),
        }
    }

    eprintln!("redstone_computational: dist={}", d_redstone.distance);
    println!("wrote diff_report.md + diff_*.schem (+ diff_*.glb if meshing enabled)");
}
