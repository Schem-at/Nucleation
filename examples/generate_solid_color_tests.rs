use nucleation::building::{
    BlockPalette, BuildingTool, Cuboid, InterpolationSpace, LinearGradientBrush,
    MultiPointGradientBrush, PointGradientBrush, Sphere,
};
use nucleation::formats::manager::get_manager;
use nucleation::{BlockState, UniversalSchematic};
use rand::Rng;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

fn save_schematic(name: &str, schematic: &UniversalSchematic) {
    let manager = get_manager();
    let locked = manager.lock().unwrap();
    let data = locked
        .write("schematic", schematic, None)
        .expect("Failed to serialize schematic");

    let filename = format!("{}.schem", name);
    let mut file = File::create(&filename).expect("Failed to create file");
    file.write_all(&data).expect("Failed to write data");
    println!("Saved {}", filename);
}

fn create_solid_palette() -> Arc<BlockPalette> {
    println!("Building solid block palette...");
    Arc::new(
        BlockPalette::builder()
            .full_blocks_only()
            .exclude_transparent()
            .exclude_tile_entities()
            .survival_obtainable_only()
            .exclude_keyword("shulker") // Shulkers are entities/tile entities but sometimes slip through as blocks depending on data
            .exclude_keyword("barrel")
            .exclude_keyword("leaves")
            .exclude_keyword("pumpkin")
            .exclude_keyword("melon")
            .exclude_keyword("tnt")
            .exclude_keyword("sponge")
            .build(),
    )
}

fn main() {
    let solid_palette = create_solid_palette();
    let width = 150;
    let depth = 150;

    // 1. Solid Grayscale Floor
    {
        println!("Generating solid_grayscale_floor.schem...");
        let mut schematic = UniversalSchematic::new("solid_grayscale".to_string());
        // Pre-expand bounds
        schematic.ensure_bounds((0, 0, 0), (width, 0, depth));

        let mut tool = BuildingTool::new(&mut schematic);
        let shape = Cuboid::new((0, 0, 0), (width, 0, depth));

        let brush = LinearGradientBrush::new(
            (0, 0, 0),
            (0, 0, 0), // Black
            (width, 0, 0),
            (255, 255, 255), // White
        )
        .with_space(InterpolationSpace::Oklab)
        .with_palette(solid_palette.clone());

        tool.fill(&shape, &brush);
        save_schematic("solid_grayscale_floor", &schematic);
    }

    // 2. Solid Rainbow Floor
    {
        println!("Generating solid_rainbow_floor.schem...");
        let mut schematic = UniversalSchematic::new("solid_rainbow".to_string());
        schematic.ensure_bounds((0, 0, 0), (width, 0, depth));

        let mut tool = BuildingTool::new(&mut schematic);
        let shape = Cuboid::new((0, 0, 0), (width, 0, depth));

        let stops = vec![
            (0.0, (255, 0, 0)),
            (0.17, (255, 127, 0)),
            (0.33, (255, 255, 0)),
            (0.5, (0, 255, 0)),
            (0.67, (0, 0, 255)),
            (0.83, (75, 0, 130)),
            (1.0, (148, 0, 211)),
        ];

        let brush = MultiPointGradientBrush::new((0, 0, 0), (width, 0, 0), stops)
            .with_space(InterpolationSpace::Oklab)
            .with_palette(solid_palette.clone());

        tool.fill(&shape, &brush);
        save_schematic("solid_rainbow_floor", &schematic);
    }

    // 3. Colored Splotches (Point Gradient)
    {
        println!("Generating solid_splotches.schem...");
        let mut schematic = UniversalSchematic::new("solid_splotches".to_string());
        schematic.ensure_bounds((0, 0, 0), (width, 0, depth));

        let mut tool = BuildingTool::new(&mut schematic);
        let shape = Cuboid::new((0, 0, 0), (width, 0, depth));

        let mut rng = rand::thread_rng();
        let mut points = Vec::new();

        // Add random colored points
        for _ in 0..20 {
            let x = rng.gen_range(0..width);
            let z = rng.gen_range(0..depth);
            let r = rng.gen_range(0..255);
            let g = rng.gen_range(0..255);
            let b = rng.gen_range(0..255);
            points.push(((x, 0, z), (r, g, b)));
        }

        let brush = PointGradientBrush::new(points)
            .with_decay(2.0) // Controls how fast colors blend
            .with_space(InterpolationSpace::Oklab)
            .with_palette(solid_palette.clone());

        tool.fill(&shape, &brush);
        save_schematic("solid_splotches", &schematic);
    }

    // 4. Big Sphere Gradient
    {
        println!("Generating solid_sphere_gradient.schem...");
        let mut schematic = UniversalSchematic::new("solid_sphere".to_string());
        let radius = 60.0;
        let center = (64, 64, 64);

        // Pre-expand bounds for sphere
        schematic.ensure_bounds(
            (
                center.0 - radius as i32 - 1,
                center.1 - radius as i32 - 1,
                center.2 - radius as i32 - 1,
            ),
            (
                center.0 + radius as i32 + 1,
                center.1 + radius as i32 + 1,
                center.2 + radius as i32 + 1,
            ),
        );

        let mut tool = BuildingTool::new(&mut schematic);
        let shape = Sphere::new(center, radius);

        // Vertical gradient from dark blue to light blue/cyan
        let brush = LinearGradientBrush::new(
            (center.0, center.1 - radius as i32, center.2),
            (0, 0, 50), // Dark Blue
            (center.0, center.1 + radius as i32, center.2),
            (0, 255, 255), // Cyan
        )
        .with_space(InterpolationSpace::Oklab)
        .with_palette(solid_palette.clone());

        tool.fill(&shape, &brush);
        save_schematic("solid_sphere_gradient", &schematic);
    }

    // 5. RGB vs Oklab Comparison
    {
        println!("Generating solid_rgb_vs_oklab.schem...");
        let mut schematic = UniversalSchematic::new("rgb_vs_oklab".to_string());
        let width = 100;
        let depth = 50;

        // Left side: RGB
        let shape_rgb = Cuboid::new((0, 0, 0), (width, 0, depth));
        let brush_rgb = LinearGradientBrush::new(
            (0, 0, 0),
            (255, 0, 0), // Red
            (width, 0, 0),
            (0, 255, 0), // Green
        )
        .with_space(InterpolationSpace::Rgb)
        .with_palette(solid_palette.clone());

        // Right side: Oklab (offset by depth + 10)
        let shape_oklab = Cuboid::new((0, 0, depth + 10), (width, 0, depth * 2 + 10));
        let brush_oklab = LinearGradientBrush::new(
            (0, 0, depth + 10),
            (255, 0, 0), // Red
            (width, 0, depth + 10),
            (0, 255, 0), // Green
        )
        .with_space(InterpolationSpace::Oklab)
        .with_palette(solid_palette.clone());

        let mut tool = BuildingTool::new(&mut schematic);
        tool.fill(&shape_rgb, &brush_rgb);
        tool.fill(&shape_oklab, &brush_oklab);

        // Add text labels using blocks (simple writing manually)
        // Actually, just let the user see the difference. RGB often has a dark muddy middle between Red/Green.
        // Oklab should pass through yellow/orange naturally.

        save_schematic("solid_rgb_vs_oklab", &schematic);
    }
}
