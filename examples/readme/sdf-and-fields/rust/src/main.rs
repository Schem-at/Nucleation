//! Executable Rust source for docs/features/sdf-and-fields.md.

use std::error::Error;
use std::fs;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn Error>> {
    // --8<-- [start:graph]
    use nucleation::sdf::Field;

    let field = Field::value_fbm3(0.13, 73, 3)?;

    let body = Field::ellipsoid([11.0, 7.0, 11.0])?.displace(&field, 1.7)?;
    let shaft = Field::capped_cylinder(3.2, 12.0)?;
    let equator = Field::torus(9.2, 1.45)?;
    let form = body.subtract(&shaft).smooth_union(&equator, 0.7)?;
    form.validate()?;
    // --8<-- [end:graph]

    // --8<-- [start:build]
    use nucleation::blockpedia::ExtendedColorData;
    use nucleation::building::{
        BlockPalette, BrushEnum, BuildingTool, FieldBrush, GradientStop, InterpolationSpace,
        SdfShape, ShapeEnum,
    };
    use nucleation::UniversalSchematic;

    let stops = vec![
        GradientStop { position: 0.0, color: ExtendedColorData::from_rgb(25, 38, 105) },
        GradientStop { position: 0.5, color: ExtendedColorData::from_rgb(42, 185, 165) },
        GradientStop { position: 1.0, color: ExtendedColorData::from_rgb(245, 185, 48) },
    ];
    let brush = FieldBrush::new(field.clone(), stops, -1.0, 1.0)?
        .with_space(InterpolationSpace::Oklab)
        .with_palette(Arc::new(BlockPalette::new_concrete().dithered()));

    let shape = SdfShape::new(form.clone()).expect("form has finite bounds");
    let mut observatory = UniversalSchematic::new("field_observatory".into());
    BuildingTool::new(&mut observatory).fill_enum(
        &ShapeEnum::Sdf(shape),
        &BrushEnum::Field(brush),
    );
    // --8<-- [end:build]

    // --8<-- [start:inspect]
    let value_range = field.output_range().expect("FBM has a proven range");
    let restored = Field::from_json(&form.to_json()?)?;
    assert_eq!(observatory.total_blocks(), 3_175);
    assert_eq!(observatory.get_tight_dimensions(), (22, 14, 24));
    assert_eq!(value_range, [-1.0, 1.0]);
    assert!(form.eval(0.0, 0.0, 0.0) > 0.0);
    assert!((restored.eval(5.0, 2.0, 1.0) - form.eval(5.0, 2.0, 1.0)).abs() < 1e-6);
    // --8<-- [end:inspect]

    let output = std::env::var("SDF_FIELDS_OUT")
        .unwrap_or_else(|_| "field-observatory.schem".into());
    fs::write(&output, observatory.to_schematic()?)?;
    let dims = observatory.get_tight_dimensions();
    println!(
        "SDFs and fields Rust example: OK ({output}, {} blocks, {}x{}x{})",
        observatory.total_blocks(),
        dims.0,
        dims.1,
        dims.2,
    );
    Ok(())
}
