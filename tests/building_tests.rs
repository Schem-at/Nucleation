use nucleation::building::{
    BilinearGradientBrush, BlockPalette, Brush, BuildingTool, ColorBrush, Cuboid,
    InterpolationSpace, LinearGradientBrush, MultiPointGradientBrush, PointGradientBrush,
    ShadedBrush, Sphere,
};
use nucleation::UniversalSchematic;
use std::sync::Arc;

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
fn test_gradient_fill_wool() {
    let mut schematic = UniversalSchematic::new("gradient_wool".to_string());
    let mut tool = BuildingTool::new(&mut schematic);

    let cuboid = Cuboid::new((0, 0, 0), (10, 0, 0));
    let wool_palette = Arc::new(BlockPalette::new_wool());

    // Gradient from Red to Blue using ONLY wool
    let brush = LinearGradientBrush::new((0, 0, 0), (255, 0, 0), (10, 0, 0), (0, 0, 255))
        .with_palette(wool_palette);

    tool.fill(&cuboid, &brush);

    let start = schematic.get_block(0, 0, 0).unwrap();
    let end = schematic.get_block(10, 0, 0).unwrap();

    println!("Wool Start: {}", start.name);
    println!("Wool End: {}", end.name);

    assert!(start.name.contains("red_wool"));
    assert!(end.name.contains("blue_wool"));
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
            (0.0, (255, 0, 0)), // Red at start
            (0.5, (0, 255, 0)), // Green at middle
            (1.0, (0, 0, 255)), // Blue at end
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
    assert!(
        mid.name.contains("lime") || mid.name.contains("green") || mid.name.contains("emerald")
    );
    assert!(end.name.contains("blue"));
}

#[test]
fn test_bilinear_gradient() {
    let mut schematic = UniversalSchematic::new("bilinear".to_string());
    let mut tool = BuildingTool::new(&mut schematic);

    let cuboid = Cuboid::new((0, 0, 0), (10, 10, 0));

    // Quad colors:
    // (0,0) Red, (10,0) Blue
    // (0,10) Green, (10,10) Yellow
    let brush = BilinearGradientBrush::new(
        (0, 0, 0),
        (10, 0, 0),
        (0, 10, 0),
        (255, 0, 0),   // c00 Red
        (0, 0, 255),   // c10 Blue
        (0, 255, 0),   // c01 Green
        (255, 255, 0), // c11 Yellow
    );

    tool.fill(&cuboid, &brush);

    let c00 = schematic.get_block(0, 0, 0).unwrap();
    let c10 = schematic.get_block(10, 0, 0).unwrap();
    let c01 = schematic.get_block(0, 10, 0).unwrap();
    let c11 = schematic.get_block(10, 10, 0).unwrap();
    let center = schematic.get_block(5, 5, 0).unwrap();

    println!("C00: {}", c00.name);
    println!("C10: {}", c10.name);
    println!("C01: {}", c01.name);
    println!("C11: {}", c11.name);
    println!("Center: {}", center.name);

    assert!(c00.name.contains("red"));
    assert!(c10.name.contains("blue"));
    assert!(
        c01.name.contains("green") || c01.name.contains("lime") || c01.name.contains("emerald")
    ); // Lime is often closer to pure green than green_wool
    assert!(c11.name.contains("yellow") || c11.name.contains("gold"));
    // Center should be a mix (greyish or brownish depending on interpolation space)
}

#[test]
fn test_point_gradient_brush() {
    let mut schematic = UniversalSchematic::new("point_gradient".to_string());
    let mut tool = BuildingTool::new(&mut schematic);

    let cuboid = Cuboid::new((0, 0, 0), (10, 10, 10));

    // 3D Points:
    // (0,0,0) Red
    // (10,10,10) Blue
    // (5,5,5) Green (Center)
    // (10,0,0) Yellow
    let points = vec![
        ((0, 0, 0), (255, 0, 0)),
        ((10, 10, 10), (0, 0, 255)),
        ((5, 5, 5), (0, 255, 0)),
        ((10, 0, 0), (255, 255, 0)),
    ];

    let brush = PointGradientBrush::new(points).with_falloff(2.5);

    tool.fill(&cuboid, &brush);

    let origin = schematic.get_block(0, 0, 0).unwrap();
    let center = schematic.get_block(5, 5, 5).unwrap();
    let far = schematic.get_block(10, 10, 10).unwrap();
    let corner = schematic.get_block(10, 0, 0).unwrap();

    // Test exact points
    println!("Origin: {}", origin.name);
    println!("Center: {}", center.name);
    println!("Far: {}", far.name);
    println!("Corner: {}", corner.name);

    assert!(origin.name.contains("red"));
    assert!(
        center.name.contains("green")
            || center.name.contains("lime")
            || center.name.contains("emerald")
    );
    assert!(far.name.contains("blue"));
    assert!(corner.name.contains("yellow") || corner.name.contains("gold"));

    // Test interpolated point (between red and yellow)
    let mid_edge = schematic.get_block(5, 0, 0).unwrap();
    println!("Mid Edge (5,0,0): {}", mid_edge.name);
    // Should be orange-ish
    assert!(
        mid_edge.name.contains("orange")
            || mid_edge.name.contains("terracotta")
            || mid_edge.name.contains("acacia")
            || mid_edge.name.contains("honeycomb")
    );
}

#[test]
fn test_concrete_palette() {
    let mut schematic = UniversalSchematic::new("concrete".to_string());
    let mut tool = BuildingTool::new(&mut schematic);

    let sphere = Sphere::new((0, 0, 0), 5.0);
    let concrete_palette = Arc::new(BlockPalette::new_concrete());

    // Use a color that might map to something else in default palette (e.g. glass or wool)
    // Bright Red (255, 0, 0)
    let brush = ColorBrush::with_palette(255, 0, 0, concrete_palette);

    tool.fill(&sphere, &brush);

    let center = schematic.get_block(0, 0, 0).unwrap();
    println!("Concrete Center: {}", center.name);
    assert!(center.name.contains("concrete"));
    assert!(!center.name.contains("powder"));
    assert!(center.name.contains("red"));
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
fn test_custom_filter_palette() {
    let mut schematic = UniversalSchematic::new("custom".to_string());
    let mut tool = BuildingTool::new(&mut schematic);

    let sphere = Sphere::new((0, 0, 0), 5.0);

    // Create a palette that only allows "grass_block" or "moss_block"
    let nature_palette = Arc::new(BlockPalette::new_filtered(|f| {
        f.id == "minecraft:grass_block" || f.id == "minecraft:moss_block"
    }));

    // Green color
    let brush = ColorBrush::with_palette(0, 255, 0, nature_palette);

    tool.fill(&sphere, &brush);

    let center = schematic.get_block(0, 0, 0).unwrap();
    println!("Nature Center: {}", center.name);
    assert!(center.name == "minecraft:grass_block" || center.name == "minecraft:moss_block");
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
