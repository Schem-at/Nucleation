use nucleation::blockpedia::ExtendedColorData;
use nucleation::building::{BlockPalette, Brush, FieldBrush, GradientStop};
use nucleation::sdf::{Field, FieldGraph};
use std::sync::Arc;

#[test]
fn normalized_value_noise_field_is_deterministic_bounded_and_serializable() {
    let field = Field::value_fbm3(0.125, 17, 4).unwrap();

    assert_eq!(field.output_range(), Some([-1.0, 1.0]));
    assert!((field.eval(1.25, -2.5, 0.75) - 0.120_397_57).abs() < 1.0e-6);
    assert_eq!(field.eval(1.25, -2.5, 0.75), field.eval(1.25, -2.5, 0.75));

    let json = field.to_json().unwrap();
    let reparsed = Field::from_json(&json).unwrap();
    assert_eq!(reparsed.output_range(), Some([-1.0, 1.0]));
    assert_eq!(
        reparsed.eval(1.25, -2.5, 0.75),
        field.eval(1.25, -2.5, 0.75)
    );
}

#[test]
fn surface_displace_consumes_field_and_uses_its_proven_range_for_bounds() {
    let field = Field::value_fbm3(0.125, 17, 4).unwrap();
    let surface = Field::sphere(10.0).unwrap().displace(&field, 2.5).unwrap();

    let point = [1.25_f32, -2.5, 0.75];
    let sphere_distance =
        (point[0] * point[0] + point[1] * point[1] + point[2] * point[2]).sqrt() - 10.0;
    let expected = sphere_distance + 2.5 * field.eval(point[0], point[1], point[2]);
    assert!((surface.eval(point[0], point[1], point[2]) - expected).abs() < 1.0e-6);

    let bounds = surface.bounds().unwrap();
    assert_eq!(bounds.min, [-12.5, -12.5, -12.5]);
    assert_eq!(bounds.max, [12.5, 12.5, 12.5]);
}

#[test]
fn field_range_is_preserved_at_extreme_finite_coordinates() {
    let field = Field::value_fbm3(0.125, 17, 4).unwrap();
    let value = field.eval(f32::MAX, -f32::MAX, f32::MAX);

    assert!(value.is_finite());
    assert!((-1.0..=1.0).contains(&value));
}

#[test]
fn zero_amplitude_field_displace_is_the_unchanged_surface() {
    let field = Field::value_fbm3(0.125, 17, 4).unwrap();
    let surface = Field::plane([1.0, 0.0, 0.0], 0.0)
        .unwrap()
        .displace(&field, 0.0)
        .unwrap();

    assert_eq!(surface.eval(f32::MAX, 0.0, 0.0), f32::MAX);
}

#[test]
fn field_brush_consumes_unified_field_directly() {
    let field = Field::value_fbm3(0.125, 17, 4).unwrap();
    let stops = vec![
        GradientStop {
            position: 0.0,
            color: ExtendedColorData::from_rgb(0, 0, 0),
        },
        GradientStop {
            position: 1.0,
            color: ExtendedColorData::from_rgb(255, 255, 255),
        },
    ];
    let palette = Arc::new(BlockPalette::from_block_ids([
        "minecraft:black_concrete",
        "minecraft:white_concrete",
    ]));
    let brush = FieldBrush::new(field, stops, -1.0, 1.0)
        .unwrap()
        .with_palette(palette);

    assert_eq!(
        brush
            .get_block(0, -7, -9, (0.0, 1.0, 0.0))
            .unwrap()
            .get_name(),
        "minecraft:black_concrete"
    );
    assert_eq!(
        brush
            .get_block(-33, -1, -17, (0.0, 1.0, 0.0))
            .unwrap()
            .get_name(),
        "minecraft:white_concrete"
    );
}

#[test]
fn field_brush_rejects_invalid_gradient_domains() {
    let field = Field::value_fbm3(0.125, 17, 4).unwrap();
    let stop = |position| GradientStop {
        position,
        color: ExtendedColorData::from_rgb(0, 0, 0),
    };

    assert!(FieldBrush::new(field.clone(), vec![stop(0.0), stop(1.0)], 1.0, -1.0).is_err());
    assert!(FieldBrush::new(field, vec![stop(1.0), stop(0.0)], -1.0, 1.0).is_err());
}

#[test]
fn multi_root_graph_round_trip_preserves_one_shared_field_node() {
    let field = Field::value_fbm3(0.125, 17, 4).unwrap();
    let graph = FieldGraph::from_roots([
        ("surface".to_owned(), field.clone()),
        ("material".to_owned(), field),
    ])
    .unwrap();

    let json = graph.to_json().unwrap();
    let document: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(document["version"], 2);
    assert_eq!(document["nodes"].as_array().unwrap().len(), 1);
    assert_eq!(document["roots"]["surface"], document["roots"]["material"]);

    let reparsed = FieldGraph::from_json(&json).unwrap();
    let surface = reparsed.root("surface").unwrap();
    let material = reparsed.root("material").unwrap();
    assert!(surface.shares_storage_with(&material));
    assert_eq!(
        surface.eval(4.5, -2.0, 8.25),
        material.eval(4.5, -2.0, 8.25)
    );
}

#[test]
fn field_inputs_and_graph_references_fail_closed() {
    assert!(Field::value_fbm3(0.0, 17, 4).is_err());
    assert!(Field::value_fbm3(f32::NAN, 17, 4).is_err());
    assert!(Field::value_fbm3(0.125, 17, 0).is_err());
    assert!(Field::value_fbm3(0.125, 17, 9).is_err());
    assert!(Field::value_fbm3(f32::MAX, 17, 8).is_err());

    let field = Field::value_fbm3(0.125, 17, 4).unwrap();
    assert!(Field::sphere(1.0).unwrap().displace(&field, -1.0).is_err());
    assert!(Field::sphere(1.0)
        .unwrap()
        .displace(&field, f32::NAN)
        .is_err());

    let wrong_version = r#"{"version":1,"nodes":[{"type":"sphere","radius":1}],"roots":{"surface":0}}"#;
    assert!(FieldGraph::from_json(wrong_version).is_err());
    let dangling_root = r#"{"version":2,"nodes":[{"type":"sphere","radius":1}],"roots":{"surface":1}}"#;
    assert!(FieldGraph::from_json(dangling_root).is_err());
}

#[cfg(feature = "bridge")]
#[test]
fn bridge_field_is_owned_by_surface_and_material_consumers() {
    use nucleation::bridge::building::ffi::{
        Brush as BridgeBrush, BuildingTool as BridgeBuildingTool, InterpolationSpace,
    };
    use nucleation::bridge::schematic::ffi::Schematic as BridgeSchematic;
    use nucleation::bridge::sdf::ffi::Field as BridgeField;

    let field = BridgeField::value_fbm3(0.125, 17, 4).unwrap();
    assert!((field.eval_at(1.25, -2.5, 0.75) - 0.120_397_57).abs() < 1.0e-6);

    let sphere = BridgeField::sphere(10.0).unwrap();
    let surface = sphere.displace(&field, 2.5).unwrap();
    let brush = BridgeBrush::field(
        &field,
        &[0.0, 1.0],
        &[0, 0, 0, 255, 255, 255],
        -1.0,
        1.0,
        InterpolationSpace::Oklab,
    )
    .unwrap();
    drop(field);

    assert!(surface.eval_at(10.0, 0.0, 0.0).is_finite());
    let shape = surface.to_shape().unwrap();
    let mut schematic = BridgeSchematic::create(b"field ownership");
    BridgeBuildingTool::fill(&mut schematic, &shape, &brush);
    assert!(schematic.block_count() > 0);
}

#[cfg(feature = "bridge")]
#[test]
fn bridge_field_reports_its_range_as_a_single_checked_value() {
    use nucleation::bridge::sdf::ffi::Field as BridgeField;

    let field = BridgeField::value_fbm3(0.125, 17, 4).unwrap();
    let range = field
        .output_range()
        .expect("value noise has a proven range");

    assert_eq!(range.min, -1.0);
    assert_eq!(range.max, 1.0);
    assert!(range.min.is_finite() && range.max.is_finite());
    assert!(range.min <= range.max);
}
