//! README illustration: a compact workshop assembled with highlighted Python.
//!
//! Emits transparent animation frames plus `timing.json`; the compositor turns
//! them into a side-by-side GIF whose active line follows each scene group.
//!
//!     cargo run --release --example readme_setblock --features rendering -- <pack.zip> [out_dir]

use nucleation::animation::{presets, BuildAnimator, Grouping, Order, Stagger};
use nucleation::formats::litematic;
use nucleation::meshing::{MeshConfig, ResourcePackSource};
use nucleation::rendering::{render_animation_to_files, GridConfig, RenderConfig};
use nucleation::{ArmorStandEquipment, Entity, UniversalSchematic};

// Python shown in the editor. The optional number maps a line to one animation
// group: floor, furnace, crafting table, chest, then armoured stand.
const CODE: &[(&str, Option<usize>)] = &[
    ("from nucleation import Nbt, Schematic", None),
    ("s = Schematic.create(\"workshop\")", None),
    ("for x in range(-3, 3):", None),
    ("    for z in range(-2, 3):", None),
    (
        "        s.set_block(x, 0, z, \"minecraft:oak_planks\")",
        Some(0),
    ),
    ("s.set_block_from_string(-2, 1, -1,", None),
    (
        "    \"minecraft:furnace[facing=south]{BurnTime:200s}\")",
        Some(1),
    ),
    (
        "s.set_block(-1, 1, -1, \"minecraft:crafting_table\")",
        Some(2),
    ),
    (
        "loot = Nbt.chest_build('[{\"id\":\"minecraft:diamond\",\"count\":3}]', \"\")",
        Some(3),
    ),
    (
        "s.set_block_from_string(1, 1, -1, f\"minecraft:chest[facing=south]{loot}\")",
        None,
    ),
    ("s.add_armor_stand(0.5, 1, -0.5, 0, \"diamond\")", Some(4)),
    ("s.save_to_file(\"workshop.schem\")", None),
];

fn workshop_floor() -> Vec<(i32, i32, i32)> {
    let mut floor = Vec::with_capacity(30);
    for x in -3..=2 {
        for z in -2..=2 {
            floor.push((x, 0, z));
        }
    }
    floor
}

fn workshop_blocks() -> Vec<((i32, i32, i32), &'static str)> {
    let mut blocks: Vec<_> = workshop_floor()
        .into_iter()
        .map(|pos| (pos, "minecraft:oak_planks"))
        .collect();
    blocks.extend([
        (
            (-2, 1, -1),
            "minecraft:furnace[facing=south]{BurnTime:200s,CookTime:80s}",
        ),
        ((-1, 1, -1), "minecraft:crafting_table"),
        (
            (1, 1, -1),
            "minecraft:chest[facing=south]{Items:[{Slot:0b,id:\"minecraft:diamond\",Count:3b},{Slot:1b,id:\"minecraft:bread\",Count:6b}]}",
        ),
    ]);
    blocks
}

fn armored_stand() -> Entity {
    Entity::armor_stand(
        (0.5, 1.0, -0.5),
        0.0,
        ArmorStandEquipment::full_set("diamond"),
    )
}

const FPS: f64 = 18.0;
const INTRO_MS: f32 = 400.0;
const EACH_MS: f32 = 600.0;
const CLIP_MS: f32 = 480.0;
const OUTRO_MS: f32 = 900.0;

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

    let blocks = workshop_blocks();
    let mut s = UniversalSchematic::new("workshop".to_string());
    for &(pos, block) in &blocks {
        s.set_block_from_string(pos.0, pos.1, pos.2, block)?;
    }
    s.add_entity(armored_stand());
    std::fs::write(format!("{out}/workshop.schem"), s.to_schematic()?)?;
    std::fs::write(
        format!("{out}/workshop.litematic"),
        litematic::to_litematic(&s)?,
    )?;

    // Group the floor's 30 blocks into one code-level operation, then animate each
    // metadata-bearing helper object as its own unit.
    let floor = workshop_floor();
    let groups = vec![
        floor.clone(),
        vec![(-2, 1, -1)],
        vec![(-1, 1, -1)],
        vec![(1, 1, -1)],
        vec![(0, 1, -1)],
    ];
    let positions: Vec<_> = groups.iter().flatten().copied().collect();
    let mut anim = BuildAnimator::from_positions(&positions, Grouping::Custom(groups));
    anim.timeline_mut().add_staggered(
        presets::drop_and_pop(CLIP_MS, 4.5),
        &Stagger::each(Order::Index, EACH_MS),
        INTRO_MS,
    );

    let group_count = anim.groups().len();
    let outro_start = INTRO_MS + (group_count as f32 - 1.0) * EACH_MS + CLIP_MS;
    let total = outro_start + OUTRO_MS;
    let frame_count = (total as f64 / 1000.0 * FPS).round() as usize;

    let constructor_line = CODE
        .iter()
        .position(|(line, _)| line.contains("Schematic.create("))
        .expect("CODE must construct a schematic");
    let group_lines: Vec<usize> = (0..group_count)
        .map(|group| {
            CODE.iter()
                .position(|(_, mapped)| *mapped == Some(group))
                .expect("every animation group must map to a code line")
        })
        .collect();
    let save_line = CODE.len() - 1;
    let active: Vec<usize> = (0..frame_count)
        .map(|i| {
            let t = i as f32 * 1000.0 / FPS as f32;
            if t < INTRO_MS {
                constructor_line
            } else if t >= outro_start {
                save_line
            } else {
                let group = (((t - INTRO_MS) / EACH_MS).floor() as usize).min(group_count - 1);
                group_lines[group]
            }
        })
        .collect();
    let frames: Vec<_> = (0..frame_count)
        .map(|i| anim.frame_at(i as f32 * 1000.0 / FPS as f32))
        .collect();

    // Group meshing now reads the real schematic entity and its typed equipment.
    let pack = ResourcePackSource::from_file(&pack_path)?;
    let meshes = s.mesh_groups(&pack, &MeshConfig::default(), anim.groups())?;
    let mut rc = RenderConfig::isometric();
    rc.width = 420;
    rc.height = 420;
    rc.sphere_fit = true;
    rc.background = Some([0.0, 0.0, 0.0, 0.0]);
    rc.grid = Some(GridConfig {
        half_extent: 5,
        fit_to_bounds: true,
        margin: 1,
        spacing: 1,
        plane_y: -0.502,
        show_axes: false,
        line_rgba: [0.42, 0.52, 0.60, 0.38],
    });
    render_animation_to_files(&meshes, &frames, &rc, None, &format!("{out}/f"))?;

    let code: Vec<&str> = CODE.iter().map(|(line, _)| *line).collect();
    std::fs::write(
        format!("{out}/timing.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "title": "Metadata workshop",
            "filename": "workshop.py",
            "code": code,
            "anim_w": rc.width,
            "anim_h": rc.height,
            "active": active,
        }))?,
    )?;

    println!(
        "{} frames -> {out}/ (f####.png, timing.json, workshop.schem, workshop.litematic)",
        frame_count
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workshop_floor_is_a_six_by_five_double_loop_surface() {
        let floor = workshop_floor();

        assert_eq!(floor.len(), 30);
        assert!(floor.contains(&(-3, 0, -2)));
        assert!(floor.contains(&(2, 0, 2)));
        assert!(floor.iter().all(|(_, y, _)| *y == 0));
    }

    #[test]
    fn workshop_contains_metadata_blocks_and_a_south_facing_armored_stand() {
        let blocks = workshop_blocks();
        let stand = armored_stand();

        assert!(blocks
            .iter()
            .any(|(pos, block)| *pos == (-2, 1, -1) && block.starts_with("minecraft:furnace")));
        assert!(blocks
            .iter()
            .any(|(pos, block)| *pos == (-1, 1, -1) && *block == "minecraft:crafting_table"));
        assert!(blocks
            .iter()
            .any(|(pos, block)| *pos == (1, 1, -1) && block.starts_with("minecraft:chest")));
        assert_eq!(stand.position, (0.5, 1.0, -0.5));
        assert_eq!(
            format!("{:?}", stand.nbt.get("Rotation")),
            "Some(List([Float(0.0), Float(0.0)]))"
        );
    }

    #[test]
    fn displayed_python_showcases_loops_metadata_container_helpers_and_entity_nbt() {
        let source = CODE
            .iter()
            .map(|(line, _)| *line)
            .collect::<Vec<_>>()
            .join("\n");

        assert!(source.contains("Nbt, Schematic"));
        assert!(source.contains("for x in range(-3, 3)"));
        assert!(source.contains("for z in range(-2, 3)"));
        assert!(source.contains("BurnTime"));
        assert!(source.contains("Nbt.chest_build("));
        assert!(source.contains("add_armor_stand"));
        assert!(source.contains("\"diamond\""));
        assert!(!source.contains("nbt_json"));
        assert!(source.contains("s.save_to_file(\"workshop.schem\")"));
    }
}
