//! Executable Rust source for docs/features/sdf-and-fields.md.

use std::error::Error;
use std::fs;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn Error>> {
    // --8<-- [start:graph]
    use nucleation::field::Field3;
    use nucleation::sdf::SdfNode;

    let field = Field3::value_noise_fbm(0.13, 73, 3)?;

    let body = SdfNode::Ellipsoid {
        radii: [11.0, 7.0, 11.0],
    }
    .offset_by_field(field.clone(), 1.7)?;
    let shaft = SdfNode::CappedCylinder {
        radius: 3.2,
        half_height: 12.0,
    };
    let equator = SdfNode::Torus {
        major_radius: 9.2,
        minor_radius: 1.45,
    };
    let form = SdfNode::SmoothUnion {
        a: Box::new(SdfNode::Subtract {
            a: Box::new(body),
            b: Box::new(shaft),
        }),
        b: Box::new(equator),
        k: 0.7,
    };
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
    let brush = FieldBrush::from_field3(field.clone(), stops, -1.0, 1.0)?
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
    let restored = SdfNode::from_json(&form.to_json()?)?;
    assert_eq!(observatory.total_blocks(), 3_175);
    assert_eq!(observatory.get_tight_dimensions(), (22, 14, 24));
    assert_eq!(value_range, [-1.0, 1.0]);
    assert!(form.eval(0.0, 0.0, 0.0) > 0.0);
    assert!((restored.eval(5.0, 2.0, 1.0) - form.eval(5.0, 2.0, 1.0)).abs() < 1e-6);
    // --8<-- [end:inspect]

    let output = std::env::var("SDF_FIELDS_OUT")
        .unwrap_or_else(|_| "field-observatory.schem".into());
    fs::write(&output, observatory.to_schematic()?)?;
    println!("SDFs and fields Rust example: OK ({output})");
    Ok(())
}
