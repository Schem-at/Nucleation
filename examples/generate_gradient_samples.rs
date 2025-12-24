use nucleation::building::{
    BilinearGradientBrush, BuildingTool, Cuboid, InterpolationSpace, PointGradientBrush,
    ShadedBrush, Sphere,
};
use nucleation::formats::manager::get_manager;
use nucleation::UniversalSchematic;
use std::fs::File;
use std::io::Write;

fn save_schematic(name: &str, schematic: &UniversalSchematic) {
    let manager = get_manager();
    let locked = manager.lock().unwrap();
    // Use .schem format (Sponge Schematic)
    let data = locked
        .write("schematic", schematic, None)
        .expect("Failed to serialize schematic");

    let filename = format!("{}.schem", name);
    let mut file = File::create(&filename).expect("Failed to create file");
    file.write_all(&data).expect("Failed to write data");
    println!("Saved {}", filename);
}

fn main() {
    println!("Generating samples...");

    // 1. Point Cloud (Nebula) - Demonstrates PointGradientBrush
    {
        let mut schematic = UniversalSchematic::new("nebula".to_string());
        let mut tool = BuildingTool::new(&mut schematic);

        // Fill a 30x30x30 cube
        let shape = Cuboid::new((0, 0, 0), (30, 30, 30));

        let points = vec![
            ((0, 0, 0), (255, 0, 0)),        // Red corner
            ((30, 30, 30), (0, 0, 255)),     // Blue corner
            ((15, 15, 15), (255, 255, 255)), // White center core
            ((30, 0, 0), (255, 255, 0)),     // Yellow
            ((0, 30, 0), (0, 255, 0)),       // Green
            ((0, 0, 30), (255, 0, 255)),     // Magenta
        ];

        let brush = PointGradientBrush::new(points)
            .with_falloff(2.5) // Higher falloff = tighter pockets of color
            .with_space(InterpolationSpace::Oklab);

        println!("Generating sample_nebula.schem...");
        tool.fill(&shape, &brush);
        save_schematic("sample_nebula", &schematic);
    }

    // 2. Bilinear Floor (Quad) - Demonstrates BilinearGradientBrush
    {
        let mut schematic = UniversalSchematic::new("floor".to_string());
        let mut tool = BuildingTool::new(&mut schematic);

        let shape = Cuboid::new((0, 0, 0), (40, 1, 40));

        // Quad Colors
        let brush = BilinearGradientBrush::new(
            (0, 0, 0),
            (40, 0, 0),
            (0, 0, 40),
            (255, 0, 0),   // Origin: Red
            (0, 0, 255),   // U-end: Blue
            (0, 255, 0),   // V-end: Green
            (255, 255, 0), // Opposite: Yellow
        )
        .with_space(InterpolationSpace::Oklab);

        println!("Generating sample_floor.schem...");
        tool.fill(&shape, &brush);
        save_schematic("sample_floor", &schematic);
    }

    // 3. Shaded Sphere - Demonstrates ShadedBrush & Normals
    {
        let mut schematic = UniversalSchematic::new("sphere".to_string());
        let mut tool = BuildingTool::new(&mut schematic);

        let shape = Sphere::new((15, 15, 15), 5.0);

        // White sphere lit from top-right-front
        let brush = ShadedBrush::new((255, 255, 255), (1.0, 1.0, 1.0));

        println!("Generating sample_sphere.schem...");
        tool.fill(&shape, &brush);
        save_schematic("sample_sphere", &schematic);
    }

    println!("Done! Check the current directory for .schem files.");
}
