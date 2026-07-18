use nucleation::building::{
    BezierCurve, BilinearGradientBrush, BlockPalette, Brush, BrushEnum, BuildingTool, ColorBrush,
    Cone, Cuboid, CurveGradientBrush, Cylinder, Difference, Disk, Ellipsoid, Hollow,
    InterpolationSpace, Intersection, Line, LinearGradientBrush, MultiPointGradientBrush, Plane,
    PointGradientBrush, Pyramid, ShadedBrush, Shape, ShapeEnum, SolidBrush, Sphere, Torus,
    Triangle, Union,
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

// ============================================================================
// New shape tests
// ============================================================================

#[test]
fn test_ellipsoid_shape() {
    let e = Ellipsoid::new((0, 0, 0), (5.0, 3.0, 4.0));
    assert!(e.contains(0, 0, 0)); // center
    assert!(e.contains(4, 0, 0)); // within x radius
    assert!(!e.contains(6, 0, 0)); // outside x radius
    assert!(e.contains(0, 2, 0)); // within y radius
    assert!(!e.contains(0, 4, 0)); // outside y radius

    let points = e.points();
    assert!(!points.is_empty());

    let (min_x, min_y, min_z, max_x, max_y, max_z) = e.bounds();
    assert!(min_x <= -5 && max_x >= 5);
    assert!(min_y <= -3 && max_y >= 3);
    assert!(min_z <= -4 && max_z >= 4);
}

#[test]
fn test_cylinder_shape() {
    // Y-axis cylinder at origin
    let c = Cylinder::new((0.0, 0.0, 0.0), (0.0, 1.0, 0.0), 3.0, 10.0);
    assert!(c.contains(0, 5, 0)); // on axis
    assert!(c.contains(2, 5, 0)); // within radius
    assert!(!c.contains(4, 5, 0)); // outside radius
    assert!(!c.contains(0, -1, 0)); // below base
    assert!(!c.contains(0, 11, 0)); // above top

    let points = c.points();
    assert!(!points.is_empty());
}

#[test]
fn test_cylinder_between() {
    let c = Cylinder::between((0.0, 0.0, 0.0), (10.0, 0.0, 0.0), 2.0);
    assert!(c.contains(5, 0, 0)); // midpoint on axis
    assert!(c.contains(5, 1, 0)); // within radius
    assert!(!c.contains(5, 3, 0)); // outside radius
}

#[test]
fn test_cone_shape() {
    // Apex at origin, axis +Y, base_radius=5, height=10
    let cone = Cone::new((0.0, 0.0, 0.0), (0.0, 1.0, 0.0), 5.0, 10.0);
    assert!(!cone.contains(1, 0, 0)); // at apex, radius should be 0
    assert!(cone.contains(0, 5, 0)); // on axis midpoint
    assert!(cone.contains(2, 5, 0)); // within radius at midpoint (r=2.5)
    assert!(!cone.contains(4, 5, 0)); // outside radius at midpoint
    assert!(cone.contains(4, 9, 0)); // near base, should be within radius
}

#[test]
fn test_torus_shape() {
    // Torus at origin, Y-axis, major=10, minor=3
    let t = Torus::new((0.0, 0.0, 0.0), 10.0, 3.0, (0.0, 1.0, 0.0));
    assert!(t.contains(10, 0, 0)); // on the ring
    assert!(t.contains(12, 0, 0)); // within minor radius
    assert!(!t.contains(0, 0, 0)); // center hole
    assert!(!t.contains(14, 0, 0)); // outside
    assert!(t.contains(0, 0, 10)); // 90 degrees around

    let points = t.points();
    assert!(!points.is_empty());
}

#[test]
fn test_pyramid_shape() {
    // Base at origin, Y-axis up, half_size 5x5, height 10
    let p = Pyramid::new((0.0, 0.0, 0.0), (5.0, 5.0), 10.0, (0.0, 1.0, 0.0));
    assert!(p.contains(0, 0, 0)); // base center
    assert!(p.contains(4, 0, 4)); // near edge of base
    assert!(!p.contains(6, 0, 0)); // outside base
    assert!(p.contains(0, 5, 0)); // midway up, center
    assert!(!p.contains(4, 5, 0)); // midway up, outside (half_size * 0.5 = 2.5)
    assert!(p.contains(0, 9, 0)); // near top

    let points = p.points();
    assert!(!points.is_empty());
}

#[test]
fn test_disk_shape() {
    // Horizontal disk at y=5, radius=5, normal +Y
    let d = Disk::new((0.0, 5.0, 0.0), 5.0, (0.0, 1.0, 0.0), 1.0);
    assert!(d.contains(0, 5, 0)); // center
    assert!(d.contains(4, 5, 0)); // within radius
    assert!(!d.contains(6, 5, 0)); // outside radius
    assert!(!d.contains(0, 7, 0)); // outside thickness
}

#[test]
fn test_plane_shape() {
    // Horizontal plane at origin, U=+X, V=+Z, extent 5 in each direction
    let p = Plane::new(
        (0.0, 0.0, 0.0),
        (1.0, 0.0, 0.0),
        (0.0, 0.0, 1.0),
        5.0,
        5.0,
        1.0,
    );
    assert!(p.contains(0, 0, 0)); // center
    assert!(p.contains(3, 0, 3)); // within extents
    assert!(!p.contains(6, 0, 0)); // outside u extent
    assert!(!p.contains(0, 2, 0)); // outside thickness
}

#[test]
fn test_triangle_shape() {
    let t = Triangle::new((0.0, 0.0, 0.0), (10.0, 0.0, 0.0), (5.0, 10.0, 0.0), 1.0);
    assert!(t.contains(5, 3, 0)); // inside
    assert!(t.contains(1, 0, 0)); // on edge
    assert!(!t.contains(0, 10, 0)); // outside

    let points = t.points();
    assert!(!points.is_empty());
}

#[test]
fn test_line_shape() {
    let l = Line::new((0.0, 0.0, 0.0), (10.0, 0.0, 0.0), 2.0);
    assert!(l.contains(5, 0, 0)); // on line
    assert!(!l.contains(5, 2, 0)); // outside thickness
    assert!(!l.contains(-2, 0, 0)); // before start

    let points = l.points();
    assert!(!points.is_empty());
}

#[test]
fn test_line_thin() {
    // Thin line uses Bresenham
    let l = Line::new((0.0, 0.0, 0.0), (10.0, 0.0, 0.0), 0.0);
    let points = l.points();
    assert_eq!(points.len(), 11); // 0 to 10 inclusive
    assert!(points.contains(&(0, 0, 0)));
    assert!(points.contains(&(10, 0, 0)));
}

#[test]
fn test_bezier_curve() {
    let bez = BezierCurve::new(
        vec![(0.0, 0.0, 0.0), (5.0, 10.0, 0.0), (10.0, 0.0, 0.0)],
        2.0,
        32,
    );
    assert!(bez.contains(0, 0, 0)); // start
    assert!(bez.contains(10, 0, 0)); // end

    let points = bez.points();
    assert!(!points.is_empty());
}

#[test]
fn test_hollow_shape() {
    let sphere = ShapeEnum::Sphere(Sphere::new((0, 0, 0), 10.0));
    let hollow = Hollow::new(sphere, 1);
    assert!(!hollow.contains(0, 0, 0)); // center should be empty
    assert!(hollow.contains(10, 0, 0)); // edge should be filled
    assert!(hollow.contains(0, 10, 0)); // edge
    assert!(hollow.contains(0, 0, 10)); // edge

    let points = hollow.points();
    // Hollow should have fewer points than the solid
    let solid_points = Sphere::new((0, 0, 0), 10.0).points();
    assert!(points.len() < solid_points.len());
}

#[test]
fn test_union_shape() {
    let a = ShapeEnum::Sphere(Sphere::new((0, 0, 0), 3.0));
    let b = ShapeEnum::Sphere(Sphere::new((5, 0, 0), 3.0));
    let u = Union::new(a, b);
    assert!(u.contains(0, 0, 0)); // in A
    assert!(u.contains(5, 0, 0)); // in B
    assert!(u.contains(3, 0, 0)); // overlap region
    assert!(!u.contains(10, 0, 0)); // outside both
}

#[test]
fn test_intersection_shape() {
    let a = ShapeEnum::Sphere(Sphere::new((0, 0, 0), 5.0));
    let b = ShapeEnum::Sphere(Sphere::new((3, 0, 0), 5.0));
    let i = Intersection::new(a, b);
    assert!(i.contains(2, 0, 0)); // in both
    assert!(!i.contains(-4, 0, 0)); // only in A
    assert!(!i.contains(7, 0, 0)); // only in B
}

#[test]
fn test_difference_shape() {
    let a = ShapeEnum::Sphere(Sphere::new((0, 0, 0), 5.0));
    let b = ShapeEnum::Sphere(Sphere::new((3, 0, 0), 5.0));
    let d = Difference::new(a, b);
    assert!(d.contains(-4, 0, 0)); // in A but not B
    assert!(!d.contains(2, 0, 0)); // in both (subtracted)
    assert!(!d.contains(7, 0, 0)); // only in B
}

#[test]
fn test_parametric_line() {
    use nucleation::building::ParametricShape;
    let l = Line::new((0.0, 0.0, 0.0), (10.0, 0.0, 0.0), 2.0);
    assert!((l.parameter_at(0, 0, 0) - 0.0).abs() < 0.01);
    assert!((l.parameter_at(5, 0, 0) - 0.5).abs() < 0.01);
    assert!((l.parameter_at(10, 0, 0) - 1.0).abs() < 0.01);
}

#[test]
fn test_parametric_cylinder() {
    use nucleation::building::ParametricShape;
    let c = Cylinder::new((0.0, 0.0, 0.0), (0.0, 1.0, 0.0), 3.0, 10.0);
    assert!((c.parameter_at(0, 0, 0) - 0.0).abs() < 0.01);
    assert!((c.parameter_at(0, 5, 0) - 0.5).abs() < 0.01);
    assert!((c.parameter_at(0, 10, 0) - 1.0).abs() < 0.01);
}

#[test]
fn test_curve_gradient_brush() {
    let brush = CurveGradientBrush::new(vec![
        (0.0, (255, 0, 0)),
        (0.5, (0, 255, 0)),
        (1.0, (0, 0, 255)),
    ]);
    // At t=0 should be red-ish
    let b0 = brush.get_block_parametric(0, 0, 0, (0.0, 1.0, 0.0), Some(0.0));
    assert!(b0.is_some());
    let name0 = b0.unwrap().name;
    println!("CurveGrad t=0: {}", name0);
    assert!(name0.contains("red"));

    // At t=1 should be blue-ish
    let b1 = brush.get_block_parametric(0, 0, 0, (0.0, 1.0, 0.0), Some(1.0));
    assert!(b1.is_some());
    let name1 = b1.unwrap().name;
    println!("CurveGrad t=1: {}", name1);
    assert!(name1.contains("blue"));
}

#[test]
fn test_rstack() {
    let mut schematic = UniversalSchematic::new("rstack".to_string());
    let shape = ShapeEnum::Cuboid(Cuboid::new((0, 0, 0), (2, 2, 2)));
    let brush = BrushEnum::Solid(SolidBrush::new(nucleation::BlockState::new(
        "minecraft:stone".to_string(),
    )));

    let mut tool = BuildingTool::new(&mut schematic);
    tool.rstack(&shape, &brush, 3, (5, 0, 0));

    // Check first copy
    assert!(schematic.get_block(1, 1, 1).is_some());
    // Check second copy at offset (5,0,0)
    assert!(schematic.get_block(6, 1, 1).is_some());
    // Check third copy at offset (10,0,0)
    assert!(schematic.get_block(11, 1, 1).is_some());
    // Gap between copies should be empty
    assert!(
        schematic.get_block(3, 1, 1).is_none()
            || schematic.get_block(3, 1, 1).unwrap().name == "minecraft:air"
    );
}

#[test]
fn test_fill_enum_with_parametric() {
    let mut schematic = UniversalSchematic::new("fill_enum".to_string());
    let shape = ShapeEnum::Line(Line::new((0.0, 0.0, 0.0), (20.0, 0.0, 0.0), 2.0));
    let brush = BrushEnum::CurveGradient(CurveGradientBrush::new(vec![
        (0.0, (255, 0, 0)),
        (1.0, (0, 0, 255)),
    ]));

    let mut tool = BuildingTool::new(&mut schematic);
    tool.fill_enum(&shape, &brush);

    let start = schematic.get_block(0, 0, 0);
    let end = schematic.get_block(20, 0, 0);
    assert!(start.is_some());
    assert!(end.is_some());
    // Start should be red, end should be blue
    println!("Parametric Start: {}", start.unwrap().name);
    println!("Parametric End: {}", end.unwrap().name);
}

#[test]
fn palette_from_block_ids_and_introspection() {
    use nucleation::building::BlockPalette;

    let palette = BlockPalette::from_block_ids(
        ["minecraft:stone", "minecraft:oak_planks", "minecraft:definitely_not_a_block"]
            .into_iter(),
    );
    // Unknown ids are skipped; both real blocks have colors in blockpedia.
    assert_eq!(palette.len(), 2);
    let ids: Vec<&str> = palette.block_ids().collect();
    assert!(ids.contains(&"minecraft:stone"));
    assert!(ids.contains(&"minecraft:oak_planks"));
}

#[test]
fn set_palette_constrains_fill_to_palette_blocks() {
    use nucleation::building::{BlockPalette, ColorBrush, Cuboid};

    let wool = Arc::new(BlockPalette::new_wool());
    assert!(!wool.is_empty(), "wool palette must not be empty");

    let mut schematic = UniversalSchematic::new("palette_fill".to_string());
    let shape = ShapeEnum::Cuboid(Cuboid::new((0, 0, 0), (3, 3, 3)));
    let mut brush = BrushEnum::Color(ColorBrush::new(200, 30, 30));
    brush.set_palette(wool.clone());

    let mut tool = BuildingTool::new(&mut schematic);
    tool.fill_enum(&shape, &brush);

    for x in 0..=3 {
        for y in 0..=3 {
            for z in 0..=3 {
                let block = schematic.get_block(x, y, z).expect("block placed");
                assert!(
                    block.name.contains("wool"),
                    "expected wool at ({x},{y},{z}), got {}",
                    block.name
                );
            }
        }
    }
}

#[test]
fn set_palette_is_noop_for_solid_brush() {
    use nucleation::building::{BlockPalette, Cuboid, SolidBrush};
    use nucleation::BlockState;

    let mut schematic = UniversalSchematic::new("solid_palette".to_string());
    let shape = ShapeEnum::Cuboid(Cuboid::new((0, 0, 0), (1, 1, 1)));
    let mut brush = BrushEnum::Solid(SolidBrush::new(BlockState::new(
        "minecraft:glass".to_string(),
    )));
    brush.set_palette(Arc::new(BlockPalette::new_wool()));

    let mut tool = BuildingTool::new(&mut schematic);
    tool.fill_enum(&shape, &brush);
    assert_eq!(
        schematic.get_block(0, 0, 0).unwrap().name.as_str(),
        "minecraft:glass"
    );
}

#[test]
fn default_palette_is_broad_and_current() {
    use nucleation::building::BlockPalette;

    let all = BlockPalette::new_all();
    // blockpedia 0.2's texture-derived colors cover ~1100 blocks; anything
    // below this means the color data regressed to the old sparse cache.
    assert!(all.len() > 800, "default palette too small: {}", all.len());
    // 1.21.x content must be present and colored.
    let ids: Vec<&str> = all.block_ids().collect();
    assert!(ids.contains(&"minecraft:crafter"), "1.21 blocks missing");
    // Technical blocks must never be palette candidates.
    assert!(!ids.contains(&"minecraft:nether_portal"));
    assert!(!ids.contains(&"minecraft:water"));
}

#[test]
fn palette_ramps_and_gradient_ids() {
    use nucleation::building::BlockPalette;

    // Lightness ordering turns wool into a ramp: darkest first, lightest last.
    let wool = BlockPalette::new_wool().sorted_by_lightness();
    let ids: Vec<&str> = wool.block_ids().collect();
    assert!(ids.len() >= 16, "wool palette too small: {}", ids.len());
    assert_eq!(ids.first().copied(), Some("minecraft:black_wool"));
    assert_eq!(ids.last().copied(), Some("minecraft:white_wool"));

    // Gradient sampling: exactly N entries, endpoints snap to the extremes.
    let ramp = BlockPalette::new_wool().gradient_ids((0, 0, 0), (255, 255, 255), 16);
    assert_eq!(ramp.len(), 16);
    assert_eq!(ramp.first().map(String::as_str), Some("minecraft:black_wool"));
    assert_eq!(ramp.last().map(String::as_str), Some("minecraft:white_wool"));

    // Wood preset is the planks family.
    let wood = BlockPalette::new_wood();
    assert!(wood.len() >= 8, "wood palette too small: {}", wood.len());
    assert!(wood.block_ids().all(|id| id.ends_with("_planks") || id == "minecraft:bamboo_mosaic"));

    // Empty palette yields an empty gradient, not a panic.
    assert!(BlockPalette::from_block_ids(std::iter::empty())
        .gradient_ids((0, 0, 0), (255, 255, 255), 8)
        .is_empty());
}

// ============================================================================
// Masked fills
// ============================================================================

#[test]
fn test_fill_only_air_preserves_existing() {
    use nucleation::building::FillMode;
    use nucleation::BlockState;

    let mut schematic = UniversalSchematic::new("mask_air".to_string());
    let mut tool = BuildingTool::new(&mut schematic);

    // Stone core...
    let stone_box = ShapeEnum::Cuboid(Cuboid::new((0, 0, 0), (2, 2, 2)));
    let stone = BrushEnum::Solid(SolidBrush::new(BlockState::new(
        "minecraft:stone".to_string(),
    )));
    tool.fill_enum(&stone_box, &stone);

    // ...then a larger overlapping glass fill that must not touch it.
    let glass_box = ShapeEnum::Cuboid(Cuboid::new((0, 0, 0), (4, 4, 4)));
    let glass = BrushEnum::Solid(SolidBrush::new(BlockState::new(
        "minecraft:glass".to_string(),
    )));
    tool.fill_enum_masked(&glass_box, &glass, &FillMode::KeepExisting);

    // Stone untouched everywhere it was placed.
    for &(x, y, z) in &[(0, 0, 0), (1, 1, 1), (2, 2, 2), (0, 2, 1)] {
        assert_eq!(
            schematic.get_block(x, y, z).unwrap().name,
            "minecraft:stone",
            "stone overwritten at ({x},{y},{z})"
        );
    }
    // New cells got glass.
    for &(x, y, z) in &[(3, 0, 0), (4, 4, 4), (0, 0, 3), (3, 3, 3)] {
        assert_eq!(
            schematic.get_block(x, y, z).unwrap().name,
            "minecraft:glass",
            "expected glass at ({x},{y},{z})"
        );
    }
}

#[test]
fn test_fill_replacing_swaps_only_targets() {
    use nucleation::building::FillMode;
    use nucleation::BlockState;

    let mut schematic = UniversalSchematic::new("mask_replace".to_string());
    let mut tool = BuildingTool::new(&mut schematic);

    // Half stone, half dirt.
    let stone_box = ShapeEnum::Cuboid(Cuboid::new((0, 0, 0), (2, 0, 0)));
    let dirt_box = ShapeEnum::Cuboid(Cuboid::new((3, 0, 0), (5, 0, 0)));
    let stone = BrushEnum::Solid(SolidBrush::new(BlockState::new(
        "minecraft:stone".to_string(),
    )));
    let dirt = BrushEnum::Solid(SolidBrush::new(BlockState::new(
        "minecraft:dirt".to_string(),
    )));
    tool.fill_enum(&stone_box, &stone);
    tool.fill_enum(&dirt_box, &dirt);

    // Replace only stone with bricks across the whole strip (plus one air cell).
    let strip = ShapeEnum::Cuboid(Cuboid::new((0, 0, 0), (6, 0, 0)));
    let bricks = BrushEnum::Solid(SolidBrush::new(BlockState::new(
        "minecraft:bricks".to_string(),
    )));
    tool.fill_enum_masked(
        &strip,
        &bricks,
        &FillMode::ReplaceOnly(vec!["minecraft:stone".to_string()]),
    );

    for x in 0..=2 {
        assert_eq!(
            schematic.get_block(x, 0, 0).unwrap().name,
            "minecraft:bricks",
            "stone at x={x} should be bricks"
        );
    }
    for x in 3..=5 {
        assert_eq!(
            schematic.get_block(x, 0, 0).unwrap().name,
            "minecraft:dirt",
            "dirt at x={x} must be untouched"
        );
    }
    // Air cell stays air (never written).
    assert!(schematic
        .get_block(6, 0, 0)
        .is_none_or(|b| b.name.contains("air")));
}

#[test]
fn test_fill_mode_replace_matches_fill_enum() {
    use nucleation::building::FillMode;
    use nucleation::BlockState;

    let mut a = UniversalSchematic::new("a".to_string());
    let mut b = UniversalSchematic::new("b".to_string());
    let shape = ShapeEnum::Sphere(Sphere::new((0, 0, 0), 4.0));
    let brush = BrushEnum::Solid(SolidBrush::new(BlockState::new(
        "minecraft:stone".to_string(),
    )));

    BuildingTool::new(&mut a).fill_enum(&shape, &brush);
    BuildingTool::new(&mut b).fill_enum_masked(&shape, &brush, &FillMode::Replace);

    assert_eq!(a.total_blocks(), b.total_blocks());
}

#[test]
fn torus_parameter_covers_the_full_ring() {
    use nucleation::building::ParametricShape;

    // Regression: parameter_at used raw world components of the radial
    // projection, so a y-up torus collapsed t to {0.5, 1.0} (two-color
    // gradient halves). The angle must be measured in an in-plane basis.
    let torus = Torus::new((0.0, 0.0, 0.0), 16.0, 6.0, (0.0, 1.0, 0.0));
    let mut deciles = std::collections::HashSet::new();
    for i in 0..64 {
        let a = 2.0 * std::f64::consts::PI * (i as f64) / 64.0;
        let (x, z) = ((16.0 * a.cos()).round() as i32, (16.0 * a.sin()).round() as i32);
        let t = torus.parameter_at(x, 0, z);
        assert!((0.0..=1.0).contains(&t));
        deciles.insert((t * 10.0).floor() as i32);
    }
    assert!(deciles.len() >= 9, "t only hit deciles {:?}", deciles);

    // A tilted-axis torus must cover the ring too.
    let tilted = Torus::new((0.0, 0.0, 0.0), 16.0, 6.0, (1.0, 0.0, 0.0));
    let mut deciles = std::collections::HashSet::new();
    for i in 0..64 {
        let a = 2.0 * std::f64::consts::PI * (i as f64) / 64.0;
        let (y, z) = ((16.0 * a.cos()).round() as i32, (16.0 * a.sin()).round() as i32);
        deciles.insert((tilted.parameter_at(0, y, z) * 10.0).floor() as i32);
    }
    assert!(deciles.len() >= 9, "tilted t only hit deciles {:?}", deciles);
}

#[test]
fn ramp_ids_picks_distinct_monotonic_blocks() {
    use nucleation::building::BlockPalette;

    // 28 distinct blocks, pure white -> pure black, from the full palette.
    let all = BlockPalette::new_all();
    let ramp = all.ramp_ids((255, 255, 255), (0, 0, 0), 28).expect("ramp");
    assert_eq!(ramp.len(), 28);
    let distinct: std::collections::HashSet<_> = ramp.iter().collect();
    assert_eq!(distinct.len(), 28, "blocks must not repeat");

    // Endpoints anchor near the extremes and lightness decreases monotonically
    // (the matching is monotonic along the white->black line = pure -L axis).
    let l = |id: &str| {
        let f = nucleation::blockpedia::get_block(id).unwrap();
        f.extras.color.as_ref().unwrap().to_extended().oklab[0]
    };
    assert!(l(&ramp[0]) > 0.9, "start too dark: {} ({})", ramp[0], l(&ramp[0]));
    assert!(l(&ramp[27]) < 0.25, "end too light: {} ({})", ramp[27], l(&ramp[27]));
    for w in ramp.windows(2) {
        assert!(
            l(&w[0]) >= l(&w[1]) - 1e-4,
            "lightness not monotonic: {} -> {}",
            w[0],
            w[1]
        );
    }

    // Too many steps for a small palette -> None, not a panic.
    assert!(BlockPalette::new_wool().ramp_ids((255, 255, 255), (0, 0, 0), 17).is_none());
    // Degenerate line -> None.
    assert!(all.ramp_ids((10, 10, 10), (10, 10, 10), 5).is_none());
}

#[test]
fn sdf_shapes_plug_into_the_building_system() {
    use nucleation::building::{BlockPalette, ColorBrush, SdfShape};
    use nucleation::sdf::SdfNode;

    // A smooth-union blob as a fillable Shape.
    let node = SdfNode::from_json(
        r#"{"type": "smoothUnion", "k": 4.0,
            "a": {"type": "sphere", "radius": 6},
            "b": {"type": "translate", "offset": [7, 0, 0],
                   "child": {"type": "sphere", "radius": 5}}}"#,
    )
    .unwrap();
    let shape = SdfShape::new(node).expect("bounded tree");

    let mut s = UniversalSchematic::new("blob".to_string());
    let mut brush = BrushEnum::Color(ColorBrush::new(200, 60, 60));
    brush.set_palette(Arc::new(BlockPalette::new_wool()));
    let mut tool = BuildingTool::new(&mut s);
    tool.fill_enum(&ShapeEnum::Sdf(shape.clone()), &brush);

    // The blend region between the spheres must be solid (smooth union).
    assert!(s.get_block(3, 0, 0).is_some_and(|b| b.name.contains("wool")));
    // Membership matches the sampler convention (center eval <= 0).
    assert!(shape.contains(0, 0, 0));
    assert!(!shape.contains(0, 7, 0));

    // Gradient normals are sane: on top of the first sphere, mostly +y.
    let (nx, ny, nz) = shape.normal_at(0, 5, 0);
    assert!(ny > 0.8, "top normal should point up, got ({nx},{ny},{nz})");

    // Unbounded trees refuse auto-bounds.
    let plane = SdfNode::from_json(r#"{"type": "plane", "normal": [0, 1, 0]}"#).unwrap();
    assert!(SdfShape::new(plane).is_none());
}

#[test]
fn palette_builder_color_logic() {
    use nucleation::building::BlockPalette;

    // Pure color-value logic: near-neutral, mid-lightness only.
    let grays = BlockPalette::builder()
        .chroma_below(0.022)
        .lightness_between(0.35, 0.75)
        .build();
    assert!(grays.len() > 5, "too few mid-grays: {}", grays.len());
    for id in grays.block_ids() {
        let f = nucleation::blockpedia::get_block(id).unwrap();
        let ok = f.extras.color.as_ref().unwrap().to_extended().oklab;
        assert!((ok[1] * ok[1] + ok[2] * ok[2]).sqrt() <= 0.022, "{id} too chromatic");
        assert!((0.35..=0.75).contains(&ok[0]), "{id} lightness {} out of band", ok[0]);
    }

    // color_near: everything close to lime.
    let limes = BlockPalette::builder().color_near(120, 200, 60, 0.12).build();
    assert!(limes.len() >= 2);
    assert!(limes.block_ids().any(|i| i.contains("lime")), "lime family expected");
}
