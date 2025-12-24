use nucleation::building::{
    Brush, BuildingTool, ColorBrush, Cuboid, InterpolationSpace, LinearGradientBrush,
    MultiPointGradientBrush, ShadedBrush, Sphere,
};
use nucleation::UniversalSchematic;

#[test]
fn test_solid_fill() {
    let mut schematic = UniversalSchematic::new("test".to_string());
    let mut tool = BuildingTool::new(&mut schematic);

    // Create a sphere of red blocks
    let sphere = Sphere::new((0, 0, 0), 5.0);
    // Use red color (255, 0, 0)
    let brush = ColorBrush::new(255, 0, 0);

    tool.fill(&sphere, &brush);

    // Check center block
    let block = schematic.get_block(0, 0, 0);
    assert!(block.is_some());
    let name = &block.unwrap().name;
    // blockpedia should map red to something like red_concrete or red_wool
    println!("Center block: {}", name);
    assert!(name.contains("red"));
}

#[test]
fn test_gradient_fill() {
    let mut schematic = UniversalSchematic::new("gradient".to_string());
    let mut tool = BuildingTool::new(&mut schematic);

    let cuboid = Cuboid::new((0, 0, 0), (10, 0, 0));

    // Gradient from Red to Blue
    let brush = LinearGradientBrush::new((0, 0, 0), (255, 0, 0), (10, 0, 0), (0, 0, 255));

    tool.fill(&cuboid, &brush);

    let start = schematic.get_block(0, 0, 0).unwrap();
    let mid = schematic.get_block(5, 0, 0).unwrap();
    let end = schematic.get_block(10, 0, 0).unwrap();

    println!("Start: {}", start.name);
    println!("Mid: {}", mid.name);
    println!("End: {}", end.name);

    assert!(start.name.contains("red"));
    assert!(end.name.contains("blue"));
    // Mid should be purple-ish or something in between
}

#[test]
fn test_multi_point_gradient() {
    let mut schematic = UniversalSchematic::new("multi_gradient".to_string());
    let mut tool = BuildingTool::new(&mut schematic);

    let cuboid = Cuboid::new((0, 0, 0), (20, 0, 0));

    // Red -> Green -> Blue
    let brush = MultiPointGradientBrush::new(
        (0, 0, 0),
        (20, 0, 0),
        vec![
            (0.0, (255, 0, 0)),   // Red at start
            (0.5, (0, 255, 0)),   // Green at middle
            (1.0, (0, 0, 255)),   // Blue at end
        ],
    );

    tool.fill(&cuboid, &brush);

    let start = schematic.get_block(0, 0, 0).unwrap();
    let mid = schematic.get_block(10, 0, 0).unwrap();
    let end = schematic.get_block(20, 0, 0).unwrap();

    println!("Multi Start: {}", start.name);
    println!("Multi Mid: {}", mid.name);
    println!("Multi End: {}", end.name);

    assert!(start.name.contains("red"));
    assert!(mid.name.contains("lime") || mid.name.contains("green") || mid.name.contains("emerald"));
    assert!(end.name.contains("blue"));
}

#[test]
fn test_oklab_interpolation() {
    // Just verify it compiles and runs without panic for now
    let brush = LinearGradientBrush::new((0, 0, 0), (255, 0, 0), (10, 0, 0), (0, 0, 255))
        .with_space(InterpolationSpace::Oklab);

    let block = brush.get_block(5, 0, 0, (0.0, 1.0, 0.0));
    assert!(block.is_some());
}

#[test]
fn test_shaded_sphere() {
    let mut schematic = UniversalSchematic::new("shaded".to_string());
    let mut tool = BuildingTool::new(&mut schematic);

    let sphere = Sphere::new((0, 0, 0), 5.0);
    // Light coming from top (+Y)
    let brush = ShadedBrush::new((255, 255, 255), (0.0, 1.0, 0.0));

    tool.fill(&sphere, &brush);

    // Top block (0, 5, 0) should be bright white
    let top = schematic.get_block(0, 5, 0).unwrap();
    println!("Top block: {}", top.name);

    // Bottom block (0, -5, 0) should be darker (grey/black)
    let bottom = schematic.get_block(0, -5, 0).unwrap();
    println!("Bottom block: {}", bottom.name);

    assert_ne!(top.name, bottom.name);
}
