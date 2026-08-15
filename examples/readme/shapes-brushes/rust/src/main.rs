//! Executable Rust source for docs/features/shapes-and-brushes.md.

use std::error::Error;
use std::fs;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn Error>> {
    // --8<-- [start:build]
    use nucleation::building::{
        BlockPalette, BrushEnum, BuildingTool, Cuboid, CurveGradientBrush, FillMode, Hollow,
        InterpolationSpace, ShadedBrush, ShapeEnum, SolidBrush, Sphere, Torus, Union,
    };
    use nucleation::{BlockState, UniversalSchematic};

    let mut garden = UniversalSchematic::new("orbital_garden".into());
    let plinth = ShapeEnum::Cuboid(Cuboid::new((-20, 0, -16), (20, 2, 16)));
    let stone = BrushEnum::Solid(SolidBrush::new(BlockState::new("minecraft:stone_bricks")));

    let weathering = ShapeEnum::Sphere(Sphere::new((-10, 2, 0), 8.0));
    let moss = BrushEnum::Solid(SolidBrush::new(BlockState::new(
        "minecraft:mossy_stone_bricks",
    )));

    let orbit = ShapeEnum::Torus(Torus::new((0.0, 14.0, 0.0), 12.0, 3.0, (0.0, 1.0, 0.0)));
    let rainbow = BrushEnum::CurveGradient(
        CurveGradientBrush::new(vec![
            (0.0, (255, 48, 48)),
            (0.25, (255, 190, 32)),
            (0.5, (64, 190, 255)),
            (0.75, (174, 72, 255)),
            (1.0, (255, 48, 48)),
        ])
        .with_space(InterpolationSpace::Oklab)
        .with_palette(Arc::new(BlockPalette::new_wool())),
    );

    let joined = ShapeEnum::Union(Union::new(
        ShapeEnum::Sphere(Sphere::new((-4, 14, 0), 6.0)),
        ShapeEnum::Sphere(Sphere::new((4, 14, 0), 6.0)),
    ));
    let shell = ShapeEnum::Hollow(Hollow::new(joined, 1));
    let clay = BrushEnum::Shaded(
        ShadedBrush::new((224, 130, 84), (-1.0, 0.7, -0.3))
            .with_palette(Arc::new(BlockPalette::new_terracotta())),
    );

    let mut tool = BuildingTool::new(&mut garden);
    tool.fill_enum(&plinth, &stone);
    tool.fill_enum_masked(
        &weathering,
        &moss,
        &FillMode::ReplaceOnly(vec!["minecraft:stone_bricks".into()]),
    );
    tool.fill_enum(&orbit, &rainbow);
    tool.fill_enum(&shell, &clay);
    // --8<-- [end:build]

    // --8<-- [start:inspect]
    println!("{}", garden.total_blocks());
    println!("{:?}", garden.get_tight_dimensions());
    println!("{}", garden.get_block(-20, 0, -16).unwrap());

    // --8<-- [end:inspect]

    assert_eq!(garden.total_blocks(), 6_627);
    assert_eq!(garden.get_tight_dimensions(), (41, 21, 33));
    assert_eq!(
        garden.get_block(-20, 0, -16).unwrap().to_string(),
        "minecraft:stone_bricks"
    );

    let output =
        std::env::var("SHAPES_BRUSHES_OUT").unwrap_or_else(|_| "orbital-garden.schem".into());
    fs::write(&output, garden.to_schematic()?)?;
    println!("Shapes and brushes Rust example: OK ({output})");
    Ok(())
}
