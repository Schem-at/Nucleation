//! README media generator for regions, rigid transforms, stamping, and precedence.
//!
//! cargo run --release --example readme_regions --features rendering -- <pack.zip> [out]

use nucleation::animation::{AnimationEffect, BuildAnimation, Easing, Power, Property};
use nucleation::meshing::{MeshConfig, ResourcePackSource};
use nucleation::rendering::{render_animation_to_files, GridConfig, RenderConfig};
use nucleation::{Region, UniversalSchematic};
use std::path::Path;

const FPS: f64 = 20.0;

fn assembly_effect() -> AnimationEffect {
    AnimationEffect::new(620.0)
        .tween(Property::Y, 3.0, 0.0, Easing::Out(Power::Cubic))
        .tween(Property::ScaleUniform, 0.72, 1.0, Easing::OutBack(1.25))
        .tween(Property::Opacity, 0.0, 1.0, Easing::Out(Power::Quad))
}

fn begin_region(animation: &mut BuildAnimation) -> Result<(), String> {
    animation.begin_group(None)
}

fn end_region(animation: &mut BuildAnimation) -> Result<(), String> {
    animation.end_group().map(|_| ())
}

fn set(
    animation: &mut BuildAnimation,
    region: &str,
    x: i32,
    y: i32,
    z: i32,
    block: &str,
) -> Result<(), String> {
    animation
        .set_block_in_region(region, x, y, z, block)
        .map(|_| ())
}

fn allocate(
    animation: &mut BuildAnimation,
    name: &str,
    position: (i32, i32, i32),
    size: (i32, i32, i32),
) {
    animation
        .schematic_mut()
        .add_region(Region::new(name.to_string(), position, size));
}

fn build_gatehouse() -> Result<BuildAnimation, String> {
    let mut animation = BuildAnimation::new("regions-gatehouse");
    animation.set_default_effect(assembly_effect());
    animation.set_step_ms(470.0);
    allocate(&mut animation, "Main", (4, 0, 1), (5, 5, 5));
    allocate(&mut animation, "west_wing", (0, 0, 2), (4, 3, 3));
    allocate(&mut animation, "east_wing", (9, 0, 2), (4, 3, 3));

    begin_region(&mut animation)?;
    for x in 4..=8 {
        for z in 1..=5 {
            set(&mut animation, "Main", x, 0, z, "minecraft:stone_bricks")?;
        }
    }
    for y in 1..=4 {
        for x in 4..=8 {
            for z in 1..=5 {
                let wall = x == 4 || x == 8 || z == 1 || z == 5;
                let doorway = z == 1 && x == 6 && y <= 2;
                if wall && !doorway {
                    let block = if y == 4 {
                        "minecraft:chiseled_stone_bricks"
                    } else {
                        "minecraft:stone_bricks"
                    };
                    set(&mut animation, "Main", x, y, z, block)?;
                }
            }
        }
    }
    set(&mut animation, "Main", 5, 2, 1, "minecraft:lantern")?;
    set(&mut animation, "Main", 7, 2, 1, "minecraft:lantern")?;
    end_region(&mut animation)?;

    begin_region(&mut animation)?;
    for x in 0..=3 {
        for z in 2..=4 {
            set(&mut animation, "west_wing", x, 0, z, "minecraft:cut_copper")?;
            if x == 0 || z == 2 || z == 4 {
                set(
                    &mut animation,
                    "west_wing",
                    x,
                    1,
                    z,
                    "minecraft:oxidized_cut_copper",
                )?;
            }
        }
    }
    for x in 0..=2 {
        set(
            &mut animation,
            "west_wing",
            x,
            2,
            2,
            "minecraft:oxidized_cut_copper_stairs[facing=south]",
        )?;
    }
    set(
        &mut animation,
        "west_wing",
        0,
        2,
        4,
        "minecraft:lightning_rod",
    )?;
    end_region(&mut animation)?;

    begin_region(&mut animation)?;
    for x in 9..=12 {
        for z in 2..=4 {
            set(
                &mut animation,
                "east_wing",
                x,
                0,
                z,
                "minecraft:dark_prismarine",
            )?;
            if x == 12 || z == 2 || z == 4 {
                set(
                    &mut animation,
                    "east_wing",
                    x,
                    1,
                    z,
                    "minecraft:prismarine_bricks",
                )?;
            }
        }
    }
    set(
        &mut animation,
        "east_wing",
        12,
        2,
        2,
        "minecraft:sea_lantern",
    )?;
    end_region(&mut animation)?;

    animation.rotate_region_y("west_wing", 90, 1_450.0)?;
    animation.rotate_all_y(90, 1_550.0)?;

    let mut merged = animation.schematic().get_merged_region();
    merged.name = "Main".to_string();
    let mut clone_source = UniversalSchematic::new("gatehouse-clone".into());
    clone_source.add_region(merged);
    let bounds = clone_source.get_bounding_box();
    animation.stamp_box(&clone_source, bounds, (8, 0, 1), &[], 1_450.0)?;
    Ok(animation)
}

fn market_stall_source() -> Result<UniversalSchematic, String> {
    let mut source = UniversalSchematic::new("market-stall".into());
    source.add_region(Region::new("stall".into(), (0, 0, 0), (5, 4, 3)));
    for x in 0..=4 {
        for z in 0..=2 {
            source.try_set_block_in_region_str("stall", x, 0, z, "minecraft:spruce_planks")?;
        }
    }
    for &(x, z) in &[(0, 0), (4, 0), (0, 2), (4, 2)] {
        source.try_set_block_in_region_str("stall", x, 1, z, "minecraft:stripped_spruce_log")?;
        source.try_set_block_in_region_str("stall", x, 2, z, "minecraft:stripped_spruce_log")?;
    }
    for x in 0..=4 {
        let awning = if x % 2 == 0 {
            "minecraft:red_wool"
        } else {
            "minecraft:white_wool"
        };
        for z in 0..=2 {
            source.try_set_block_in_region_str("stall", x, 3, z, awning)?;
        }
    }
    source.try_set_block_in_region_str("stall", 2, 1, 1, "minecraft:barrel")?;
    source.try_set_block_in_region_str("stall", 0, 0, 0, "minecraft:gold_block")?;
    Ok(source)
}

fn build_stamping() -> Result<BuildAnimation, String> {
    let source = market_stall_source()?;
    let mut animation = BuildAnimation::new("one-module-many-placements");
    animation.set_default_effect(assembly_effect());
    animation.set_step_ms(380.0);
    allocate(&mut animation, "source", (0, 0, 0), (5, 4, 3));
    allocate(&mut animation, "Main", (1, 0, 0), (8, 1, 9));

    begin_region(&mut animation)?;
    let source_region = source.get_region("stall").ok_or("missing stall region")?;
    let bounds = source_region.get_bounding_box();
    for x in bounds.min.0..=bounds.max.0 {
        for y in bounds.min.1..=bounds.max.1 {
            for z in bounds.min.2..=bounds.max.2 {
                if let Some(block) = source_region.get_block(x, y, z) {
                    set(&mut animation, "source", x, y, z, &block.to_string())?;
                }
            }
        }
    }
    end_region(&mut animation)?;

    // Existing destination markers make replacement and exclusion visible.
    begin_region(&mut animation)?;
    set(&mut animation, "Main", 8, 0, 0, "minecraft:diamond_block")?;
    set(&mut animation, "Main", 8, 0, 6, "minecraft:emerald_block")?;
    set(&mut animation, "Main", 1, 0, 8, "minecraft:lapis_block")?;
    end_region(&mut animation)?;

    animation.stamp_region(&source, "stall", (8, 0, 0), &[], 1_250.0)?;
    animation.stamp_region(&source, "stall", (8, 0, 6), &[], 1_250.0)?;

    let mut variant = source.clone();
    variant.rotate_region_y("stall", 90)?;
    variant.flip_region_x("stall")?;
    animation.stamp_region(
        &variant,
        "stall",
        (1, 0, 8),
        &["minecraft:gold_block".into()],
        1_550.0,
    )?;
    Ok(animation)
}

fn build_axes() -> Result<BuildAnimation, String> {
    let mut animation = BuildAnimation::new("transform-conventions");
    animation.set_default_effect(assembly_effect());
    animation.set_step_ms(330.0);
    allocate(&mut animation, "rotate_x", (-8, 0, 0), (3, 2, 1));
    allocate(&mut animation, "rotate_y", (0, 0, 0), (3, 2, 1));
    allocate(&mut animation, "rotate_z", (8, 0, 0), (3, 2, 1));

    for (region, ox, material, directional) in [
        (
            "rotate_x",
            -8,
            "minecraft:copper_block",
            "minecraft:oak_stairs[facing=south,half=bottom,shape=straight]",
        ),
        (
            "rotate_y",
            0,
            "minecraft:gold_block",
            "minecraft:oak_stairs[facing=east,half=bottom,shape=straight]",
        ),
        (
            "rotate_z",
            8,
            "minecraft:diamond_block",
            "minecraft:oak_stairs[facing=east,half=bottom,shape=straight]",
        ),
    ] {
        begin_region(&mut animation)?;
        set(&mut animation, region, ox, 0, 0, material)?;
        set(&mut animation, region, ox + 1, 0, 0, material)?;
        set(&mut animation, region, ox + 2, 0, 0, directional)?;
        set(&mut animation, region, ox, 1, 0, "minecraft:sea_lantern")?;
        end_region(&mut animation)?;
    }

    animation.rotate_region_x("rotate_x", 90, 1_450.0)?;
    animation.rotate_region_y("rotate_y", 90, 1_450.0)?;
    animation.rotate_region_z("rotate_z", 90, 1_450.0)?;
    Ok(animation)
}

fn add_panel(
    animation: &mut BuildAnimation,
    source: &Region,
    name: &str,
    target: (i32, i32, i32),
) -> Result<(), String> {
    let bounds = source.get_bounding_box();
    allocate(animation, name, target, bounds.get_dimensions());
    begin_region(animation)?;
    for x in bounds.min.0..=bounds.max.0 {
        for y in bounds.min.1..=bounds.max.1 {
            for z in bounds.min.2..=bounds.max.2 {
                if let Some(block) = source.get_block(x, y, z) {
                    set(
                        animation,
                        name,
                        target.0 + x - bounds.min.0,
                        target.1 + y - bounds.min.1,
                        target.2 + z - bounds.min.2,
                        &block.to_string(),
                    )?;
                }
            }
        }
    }
    end_region(animation)
}

fn build_overlap() -> Result<BuildAnimation, String> {
    let mut source = UniversalSchematic::new("overlap-precedence-source".into());
    source.add_region(Region::new("Main".into(), (2, 1, 2), (3, 1, 3)));
    source.add_region(Region::new("alpha".into(), (1, 0, 1), (5, 2, 5)));
    source.add_region(Region::new("zeta".into(), (0, 0, 0), (7, 1, 7)));
    for x in 0..=6 {
        for z in 0..=6 {
            if x == 0 || x == 6 || z == 0 || z == 6 {
                source.try_set_block_in_region_str("zeta", x, 0, z, "minecraft:blue_concrete")?;
            }
        }
    }
    for x in 1..=5 {
        for z in 1..=5 {
            source.try_set_block_in_region_str("alpha", x, 0, z, "minecraft:orange_concrete")?;
        }
    }
    source.try_set_block_in_region_str("alpha", 3, 1, 3, "minecraft:redstone_block")?;
    for x in 2..=4 {
        for z in 2..=4 {
            let block = if x == 3 && z == 3 {
                "minecraft:air"
            } else {
                "minecraft:quartz_block"
            };
            source.try_set_block_in_region_str("Main", x, 1, z, block)?;
        }
    }

    let mut animation = BuildAnimation::new("overlap-precedence-panels");
    animation.set_default_effect(assembly_effect());
    animation.set_step_ms(950.0);
    add_panel(
        &mut animation,
        source.get_region("zeta").ok_or("missing zeta")?,
        "panel_zeta",
        (0, 0, 0),
    )?;
    add_panel(
        &mut animation,
        source.get_region("alpha").ok_or("missing alpha")?,
        "panel_alpha",
        (9, 0, 1),
    )?;
    add_panel(
        &mut animation,
        source.get_region("Main").ok_or("missing Main")?,
        "panel_main",
        (16, 0, 2),
    )?;
    let merged = source.get_merged_region();
    add_panel(&mut animation, &merged, "panel_result", (21, 0, 0))?;
    Ok(animation)
}

fn render_animation(
    animation: &BuildAnimation,
    pack: &ResourcePackSource,
    out: &Path,
    width: u32,
    height: u32,
    zoom: f32,
) -> Result<usize, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(out)?;
    let meshes = animation.mesh_outputs(pack, &MeshConfig::default())?;
    let frames = animation.frames(FPS, 950.0);
    let mut config = RenderConfig::isometric();
    config.width = width;
    config.height = height;
    config.zoom = zoom;
    config.sphere_fit = true;
    config.background = Some([0.018, 0.025, 0.042, 1.0]);
    config.grid = Some(GridConfig {
        fit_to_bounds: true,
        margin: 1,
        plane_y: -0.505,
        show_axes: false,
        line_rgba: [0.32, 0.42, 0.52, 0.28],
        ..GridConfig::default()
    });
    let prefix = out.join("f");
    let paths = render_animation_to_files(
        &meshes,
        &frames,
        &config,
        None,
        prefix.to_str().ok_or("non-UTF8 output path")?,
    )?;
    std::fs::write(
        out.join("receipts.json"),
        serde_json::to_string_pretty(&animation.operation_receipts())?,
    )?;
    std::fs::write(
        out.join("scene.schem"),
        animation.schematic().to_schematic()?,
    )?;
    Ok(paths.len())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let pack_path = args.next().ok_or("missing resource-pack ZIP")?;
    let out = args
        .next()
        .unwrap_or_else(|| "render_work/readme-regions".to_string());
    let out = Path::new(&out);
    std::fs::create_dir_all(out)?;
    let pack = ResourcePackSource::from_file(pack_path)?;

    let hero = build_gatehouse()?;
    let hero_frames = render_animation(&hero, &pack, &out.join("hero"), 760, 430, 1.18)?;

    let stamping = build_stamping()?;
    let stamping_frames =
        render_animation(&stamping, &pack, &out.join("stamping"), 760, 400, 1.18)?;

    let axes = build_axes()?;
    let axes_frames = render_animation(&axes, &pack, &out.join("axes"), 760, 330, 1.42)?;

    let overlap = build_overlap()?;
    let overlap_frames = render_animation(&overlap, &pack, &out.join("overlap"), 760, 360, 1.18)?;

    println!(
        "README regions media: hero={hero_frames} frames, stamping={stamping_frames}, axes={axes_frames}, overlap={overlap_frames} -> {}",
        out.display()
    );
    Ok(())
}
