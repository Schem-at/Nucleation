use nucleation::building::{
    BlockPalette, BuildingTool, Cuboid, InterpolationSpace, LinearGradientBrush,
    MultiPointGradientBrush,
};
use nucleation::formats::manager::get_manager;
use nucleation::UniversalSchematic;
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

fn main() {
    // 1. Rainbow Gradient Floor (100x1x100)
    // Using Concrete for solid blocks
    {
        println!("Generating rainbow_floor.schem...");
        let mut schematic = UniversalSchematic::new("rainbow".to_string());
        let mut tool = BuildingTool::new(&mut schematic);

        let width = 100;
        let depth = 100;
        let shape = Cuboid::new((0, 0, 0), (width, 1, depth));

        // Rainbow stops
        let stops = vec![
            (0.0, (255, 0, 0)),    // Red
            (0.17, (255, 127, 0)), // Orange
            (0.33, (255, 255, 0)), // Yellow
            (0.5, (0, 255, 0)),    // Green
            (0.67, (0, 0, 255)),   // Blue
            (0.83, (75, 0, 130)),  // Indigo
            (1.0, (148, 0, 211)),  // Violet
        ];

        let palette = Arc::new(BlockPalette::new_concrete());

        let brush = MultiPointGradientBrush::new((0, 0, 0), (width, 0, 0), stops)
            .with_space(InterpolationSpace::Oklab) // Smoother perceptual transitions
            .with_palette(palette);

        tool.fill(&shape, &brush);
        save_schematic("rainbow_floor", &schematic);
    }

    // 2. Grayscale Gradient Floor
    {
        println!("Generating grayscale_floor.schem...");
        let mut schematic = UniversalSchematic::new("grayscale".to_string());
        let mut tool = BuildingTool::new(&mut schematic);

        let width = 100;
        let depth = 100;
        let shape = Cuboid::new((0, 0, 0), (width, 1, depth));

        // Use new grayscale-specific palette for better gradients without tints
        let palette = Arc::new(BlockPalette::new_grayscale());

        let brush = LinearGradientBrush::new(
            (0, 0, 0),
            (0, 0, 0), // Black
            (width, 0, 0),
            (255, 255, 255), // White
        )
        .with_space(InterpolationSpace::Oklab)
        .with_palette(palette);

        tool.fill(&shape, &brush);
        save_schematic("grayscale_floor", &schematic);
    }
}
