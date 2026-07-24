use super::*;

fn sphere(r: f32) -> SdfNode {
    SdfNode::from_json(&format!(r#"{{"type":"sphere","radius":{r}}}"#)).unwrap()
}

#[test]
fn sphere_distances_are_exact() {
    let s = sphere(5.0);
    assert!((s.eval(0.0, 0.0, 0.0) - (-5.0)).abs() < 1e-6);
    assert!((s.eval(5.0, 0.0, 0.0) - 0.0).abs() < 1e-6);
    assert!((s.eval(8.0, 0.0, 0.0) - 3.0).abs() < 1e-6);
    assert!((s.eval(0.0, -7.0, 0.0) - 2.0).abs() < 1e-6);
}

#[test]
fn box_distance_and_rounding() {
    let b = SdfNode::from_json(r#"{"type":"box","halfExtents":[2,3,4]}"#).unwrap();
    assert!((b.eval(0.0, 0.0, 0.0) - (-2.0)).abs() < 1e-6);
    assert!((b.eval(4.0, 0.0, 0.0) - 2.0).abs() < 1e-6);
    // Corner distance
    let d = b.eval(3.0, 4.0, 5.0);
    assert!((d - (3f32).sqrt()).abs() < 1e-5);
    // Rounded box keeps the same overall extents
    let rb = SdfNode::from_json(r#"{"type":"box","halfExtents":[2,3,4],"rounding":1}"#).unwrap();
    assert!((rb.eval(4.0, 0.0, 0.0) - 2.0).abs() < 1e-6);
}

#[test]
fn smooth_union_blends() {
    let json = r#"{
        "type":"smoothUnion","k":2.0,
        "a":{"type":"sphere","radius":3},
        "b":{"type":"translate","offset":[6,0,0],"child":{"type":"sphere","radius":3}}
    }"#;
    let n = SdfNode::from_json(json).unwrap();
    // Midpoint (3,0,0): plain union distance would be 0; smooth union pulls it inside.
    assert!(n.eval(3.0, 0.0, 0.0) < 0.0);
    // Far away it converges to the plain distance
    assert!((n.eval(-13.0, 0.0, 0.0) - 10.0).abs() < 1e-3);
}

#[test]
fn json_round_trip_preserves_tree() {
    let json = r#"{
        "type":"smoothUnion","k":4.0,
        "a":{"type":"superPrism","halfExtents":[32,2,32],"exponent":6},
        "b":{"type":"displace","amplitude":3.0,"frequency":0.08,"seed":42,"octaves":3,
             "child":{"type":"translate","offset":[0,-14,0],
                      "child":{"type":"ellipsoid","radii":[26,16,26]}}}
    }"#;
    let n = SdfNode::from_json(json).unwrap();
    let re = SdfNode::from_json(&n.to_json().unwrap()).unwrap();
    // Same evaluation everywhere we probe
    for &(x, y, z) in &[
        (0.0, 0.0, 0.0),
        (10.0, -5.0, 3.0),
        (-31.0, 1.9, 12.0),
        (40.0, -20.0, -40.0),
    ] {
        assert_eq!(n.eval(x, y, z).to_bits(), re.eval(x, y, z).to_bits());
    }
}

#[test]
fn transforms_behave() {
    let t = SdfNode::from_json(
        r#"{"type":"translate","offset":[10,0,0],"child":{"type":"sphere","radius":2}}"#,
    )
    .unwrap();
    assert!(t.eval(10.0, 0.0, 0.0) < 0.0);
    assert!(t.eval(0.0, 0.0, 0.0) > 0.0);

    let s =
        SdfNode::from_json(r#"{"type":"scale","factor":2.0,"child":{"type":"sphere","radius":2}}"#)
            .unwrap();
    assert!((s.eval(4.0, 0.0, 0.0) - 0.0).abs() < 1e-5);

    let r = SdfNode::from_json(
        r#"{"type":"rotate","angles":[0,90,0],"child":{"type":"box","halfExtents":[4,1,1]}}"#,
    )
    .unwrap();
    // A long-X box rotated 90° about Y becomes long-Z
    assert!(r.eval(0.0, 0.0, 3.5) < 0.0);
    assert!(r.eval(3.5, 0.0, 0.0) > 0.0);
}

#[test]
fn unbounded_trees_require_explicit_bounds() {
    let p = SdfNode::from_json(r#"{"type":"plane","normal":[0,1,0]}"#).unwrap();
    assert!(p.bounds().is_none());
    let err = sample_to_schematic(&p, &MaterialRules::default(), None, "t");
    assert!(err.is_err());
}

#[test]
fn checked_sampling_bounds_enforce_exact_cap_without_overflow() {
    assert_eq!(
        checked_sample_volume([0, 0, 0], [255, 255, 255]).unwrap(),
        MAX_SDF_SAMPLE_VOLUME
    );
    assert!(checked_sample_volume([0, 0, 0], [256, 255, 255]).is_err());
    assert!(checked_sample_volume([i32::MIN, 0, 0], [i32::MAX, 0, 0]).is_err());
    assert!(checked_sample_volume([1, 0, 0], [0, 0, 0]).is_err());
}

#[test]
fn surface_decoration_at_max_y_is_skipped_without_overflow() {
    let node = SdfNode::Plane {
        normal: [0.0, -1.0, 0.0],
        offset: -f32::MAX,
    };
    let rules = MaterialRules::from_json(
        r#"{
            "fill": [{"block": "minecraft:stone"}],
            "surface": [{"density": 1.0, "blocks": ["minecraft:short_grass"]}]
        }"#,
    )
    .unwrap();
    let bounds = SampleBounds {
        min: [0, i32::MAX, 0],
        max: [0, i32::MAX, 0],
    };
    let schematic = sample_to_schematic(&node, &rules, Some(bounds), "max-y").unwrap();
    assert_eq!(schematic.total_blocks(), 1);
    assert_eq!(
        schematic.get_block(0, i32::MAX, 0).unwrap().get_name(),
        "minecraft:stone"
    );
}

fn island_tree() -> SdfNode {
    SdfNode::from_json(
        r#"{
        "type":"smoothUnion","k":4.0,
        "a":{"type":"translate","offset":[0,61,0],
             "child":{"type":"superPrism","halfExtents":[24,2.5,24],"exponent":6}},
        "b":{"type":"displace","amplitude":3.0,"frequency":0.07,"seed":42,
             "child":{"type":"translate","offset":[0,48,0],
                      "child":{"type":"ellipsoid","radii":[20,14,20]}}}
    }"#,
    )
    .unwrap()
}

fn island_rules() -> MaterialRules {
    MaterialRules::from_json(
        r#"{
        "fill": [
            {"when": {"depthBelowSurface": {"min": 0, "max": 0}}, "block": "minecraft:grass_block"},
            {"when": {"depthBelowSurface": {"min": 1, "max": 3}}, "block": "minecraft:dirt"},
            {"when": {"yRange": {"max": 40}}, "block": "minecraft:deepslate"},
            {"block": "minecraft:stone"}
        ],
        "surface": [
            {"density": 0.15, "blocks": ["minecraft:short_grass", "minecraft:fern"], "seed": 31, "on": "minecraft:grass_block"}
        ]
    }"#,
    )
    .unwrap()
}

#[test]
fn floating_island_samples_correctly() {
    let schematic = sample_to_schematic(&island_tree(), &island_rules(), None, "island").unwrap();
    assert!(
        schematic.total_blocks() > 1000,
        "island should have real volume"
    );

    // Plateau top is flat: superPrism top face at y = 61 + 2.5 → topmost solid
    // block is y=63 across the plateau interior.
    for &(x, z) in &[(0, 0), (10, -10), (-15, 15), (20, 20)] {
        let mut top = None;
        for y in (0..90).rev() {
            if schematic
                .get_block(x, y, z)
                .is_some_and(|b| b.name != "minecraft:air")
            {
                top = Some(y);
                break;
            }
        }
        assert_eq!(top, Some(63), "plateau top at ({x},{z})");
        let name = schematic.get_block(x, 63, z).unwrap().name.clone();
        assert_eq!(name, "minecraft:grass_block");
        let below = schematic.get_block(x, 62, z).unwrap().name.clone();
        assert_eq!(below, "minecraft:dirt");
    }

    // Belly: center column should reach well below the plateau underside
    let mut bottom = None;
    for y in 0..90 {
        if schematic
            .get_block(0, y, 0)
            .is_some_and(|b| b.name != "minecraft:air")
        {
            bottom = Some(y);
            break;
        }
    }
    let bottom = bottom.expect("center column has blocks");
    assert!(bottom < 45, "belly should taper deep, bottom was {bottom}");
    // Core is stone/deepslate
    let mid = schematic.get_block(0, bottom + 5, 0).unwrap().name.clone();
    assert!(
        mid == "minecraft:stone" || mid == "minecraft:deepslate",
        "core was {mid}"
    );
}

#[test]
fn sampling_is_deterministic() {
    let a = sample_to_schematic(&island_tree(), &island_rules(), None, "a").unwrap();
    let b = sample_to_schematic(&island_tree(), &island_rules(), None, "b").unwrap();
    assert_eq!(a.total_blocks(), b.total_blocks());
    let bb = a.get_bounding_box();
    for x in bb.min.0..=bb.max.0 {
        for y in bb.min.1..=bb.max.1 {
            for z in bb.min.2..=bb.max.2 {
                let na = a.get_block(x, y, z).map(|s| s.name.clone());
                let nb = b.get_block(x, y, z).map(|s| s.name.clone());
                assert_eq!(na, nb, "mismatch at ({x},{y},{z})");
            }
        }
    }
}

fn column_extremes(
    s: &crate::UniversalSchematic,
    x: i32,
    z: i32,
) -> (Option<(i32, String)>, Option<(i32, String)>) {
    let mut bottom = None;
    let mut top = None;
    for y in -64..64 {
        if let Some(b) = s.get_block(x, y, z) {
            if b.name != "minecraft:air" {
                if bottom.is_none() {
                    bottom = Some((y, b.name.to_string()));
                }
                top = Some((y, b.name.to_string()));
            }
        }
    }
    (bottom, top)
}

#[test]
fn y_gradient_fill_varies_over_height() {
    let rules = MaterialRules::from_json(
        r#"{
        "fill": [
            {"gradient": {"palette": "wool", "from": [0, 0, 0], "to": [255, 255, 255],
                          "axis": "y", "range": [-10, 10]}}
        ]
    }"#,
    )
    .unwrap();
    let schematic = sample_to_schematic(&sphere(10.0), &rules, None, "grad").unwrap();

    let (bottom, top) = column_extremes(&schematic, 0, 0);
    let (by, bottom) = bottom.expect("column has blocks");
    let (ty, top) = top.expect("column has blocks");
    assert!(ty > by, "column should span height");
    assert!(bottom.contains("wool"), "bottom was {bottom}");
    assert!(top.contains("wool"), "top was {top}");
    assert_ne!(bottom, top, "gradient should differ bottom vs top");
    // t = 0 at the bottom of the range → exactly the `from` color.
    assert_eq!(bottom, "minecraft:black_wool");
}

#[test]
fn lightness_ramp_indexes_sorted_palette() {
    let node = SdfNode::from_json(r#"{"type":"box","halfExtents":[1,10,1]}"#).unwrap();
    let rules = MaterialRules::from_json(
        r#"{
        "fill": [
            {"gradient": {"palette": "wool", "ramp": "lightness",
                          "axis": "y", "range": [-10, 9]}}
        ]
    }"#,
    )
    .unwrap();
    let schematic = sample_to_schematic(&node, &rules, None, "ramp").unwrap();
    let (bottom, top) = column_extremes(&schematic, 0, 0);
    let (_, bottom) = bottom.unwrap();
    let (_, top) = top.unwrap();
    // Dark → light across the full range: endpoints hit the ramp extremes.
    assert_eq!(bottom, "minecraft:black_wool");
    assert_eq!(top, "minecraft:white_wool");
}

#[test]
fn depth_gradient_and_explicit_ids_palette() {
    let rules = MaterialRules::from_json(
        r#"{
        "fill": [
            {"gradient": {"palette": {"ids": ["minecraft:white_concrete", "minecraft:black_concrete"]},
                          "from": [255, 255, 255], "to": [0, 0, 0],
                          "axis": "depth", "range": [0, 6]}}
        ]
    }"#,
    )
    .unwrap();
    let schematic = sample_to_schematic(&sphere(8.0), &rules, None, "depth").unwrap();
    // Center column: surface block (depth 0) is white, deep interior black.
    let (_, top) = column_extremes(&schematic, 0, 0);
    let (ty, top) = top.unwrap();
    assert_eq!(top, "minecraft:white_concrete");
    let deep = schematic.get_block(0, ty - 7, 0).unwrap();
    assert_eq!(deep.name, "minecraft:black_concrete");
}

#[test]
fn gradient_sampling_is_deterministic() {
    let rules = MaterialRules::from_json(
        r#"{
        "fill": [
            {"gradient": {"palette": "concrete", "from": [200, 40, 40], "to": [40, 40, 200],
                          "axis": "y", "range": [-10, 10]}}
        ]
    }"#,
    )
    .unwrap();
    let a = sample_to_schematic(&sphere(9.0), &rules, None, "a").unwrap();
    let b = sample_to_schematic(&sphere(9.0), &rules, None, "b").unwrap();
    let bb = a.get_bounding_box();
    for x in bb.min.0..=bb.max.0 {
        for y in bb.min.1..=bb.max.1 {
            for z in bb.min.2..=bb.max.2 {
                let na = a.get_block(x, y, z).map(|s| s.name.clone());
                let nb = b.get_block(x, y, z).map(|s| s.name.clone());
                assert_eq!(na, nb, "mismatch at ({x},{y},{z})");
            }
        }
    }
}

#[test]
fn invalid_gradient_rules_error() {
    // Unknown palette name
    let rules = MaterialRules::from_json(
        r#"{"fill": [{"gradient": {"palette": "chrome", "from": [0,0,0], "to": [1,1,1],
                                   "axis": "y", "range": [0, 4]}}]}"#,
    )
    .unwrap();
    assert!(sample_to_schematic(&sphere(3.0), &rules, None, "t").is_err());

    // Neither block nor gradient
    let rules =
        MaterialRules::from_json(r#"{"fill": [{"when": {"yRange": {"max": 4}}}]}"#).unwrap();
    assert!(sample_to_schematic(&sphere(3.0), &rules, None, "t").is_err());

    // Both block and gradient
    let rules = MaterialRules::from_json(
        r#"{"fill": [{"block": "minecraft:stone",
                      "gradient": {"palette": "wool", "ramp": "lightness", "range": [0, 4]}}]}"#,
    )
    .unwrap();
    assert!(sample_to_schematic(&sphere(3.0), &rules, None, "t").is_err());

    // Missing from/to without ramp
    let rules = MaterialRules::from_json(
        r#"{"fill": [{"gradient": {"palette": "wool", "range": [0, 4]}}]}"#,
    )
    .unwrap();
    assert!(sample_to_schematic(&sphere(3.0), &rules, None, "t").is_err());
}

#[test]
fn old_style_rules_still_parse_and_sample() {
    // The pre-gradient JSON shape (fixed `block` strings) is untouched.
    let schematic = sample_to_schematic(&island_tree(), &island_rules(), None, "compat").unwrap();
    assert!(schematic.total_blocks() > 1000);
    // Round-trip through serialization keeps the same shape.
    let json = serde_json::to_string(&island_rules()).unwrap();
    assert!(!json.contains("gradient"));
    let reparsed = MaterialRules::from_json(&json).unwrap();
    assert_eq!(reparsed.fill.len(), island_rules().fill.len());
}

#[test]
fn noise_is_deterministic_and_bounded() {
    for i in 0..500 {
        let v = noise::fbm3(
            i as f32 * 0.37,
            i as f32 * 0.11,
            -i as f32 * 0.23,
            1234,
            0.1,
            4,
        );
        assert!((-1.0..=1.0).contains(&v));
        let v2 = noise::fbm3(
            i as f32 * 0.37,
            i as f32 * 0.11,
            -i as f32 * 0.23,
            1234,
            0.1,
            4,
        );
        assert_eq!(v.to_bits(), v2.to_bits());
    }
}

#[test]
fn cells_value_is_unit_range_and_unbounded() {
    let v =
        SdfNode::from_json(r#"{"type":"cells","frequency":0.1,"seed":3,"mode":"value"}"#).unwrap();
    for i in 0..60 {
        let f = i as f32;
        let s = v.eval(f * 1.7, f * 0.3, f * 2.1 - 5.0);
        assert!((0.0..1.0).contains(&s), "cell value in [0,1): {s}");
    }
    assert!(v.bounds().is_none(), "cells is unbounded on its own");
}

#[test]
fn box_frame_is_hollow_shell_of_box_edges() {
    let f = SdfNode::BoxFrame {
        half_extents: [2.0, 2.0, 2.0],
        thickness: 0.25,
    };
    // Center of a face (not near an edge) is outside the frame (hollow).
    assert!(f.eval(0.0, 0.0, 2.0) > 0.0);
    // Just inside from an edge (within `thickness` of two faces), the frame is solid.
    assert!(f.eval(1.9, 1.9, 0.0) < 0.0);
    // Exactly on the outer ridge is the beam's own surface.
    assert!(f.eval(2.0, 2.0, 0.0).abs() < 1e-5);
    // Far outside is outside.
    assert!(f.eval(10.0, 0.0, 0.0) > 0.0);
    // Deep interior (away from all edges/faces) is outside the hollow frame.
    assert!(f.eval(0.0, 0.0, 0.0) > 0.0);
}

#[test]
fn capped_torus_matches_full_torus_at_180_degrees() {
    let full = SdfNode::Torus {
        major_radius: 5.0,
        minor_radius: 1.0,
    };
    let capped = SdfNode::CappedTorus {
        major_radius: 5.0,
        minor_radius: 1.0,
        cap_angle: 180.0,
    };
    for &(x, y, z) in &[
        (5.0, 0.0, 0.0),
        (0.0, 0.0, -5.0),
        (-5.0, 0.0, 0.0),
        (3.0, 1.0, -4.0),
        (0.0, 0.0, 0.0),
    ] {
        assert!(
            (full.eval(x, y, z) - capped.eval(x, y, z)).abs() < 1e-4,
            "mismatch at ({x},{y},{z}): full={}, capped={}",
            full.eval(x, y, z),
            capped.eval(x, y, z)
        );
    }
}

#[test]
fn capped_torus_cuts_the_ring_by_aperture_angle() {
    let capped = SdfNode::CappedTorus {
        major_radius: 5.0,
        minor_radius: 1.0,
        cap_angle: 90.0,
    };
    // Near x-axis at the tube center: still present (inside the open arc).
    assert!((capped.eval(5.0, 0.0, 0.0) - (-1.0)).abs() < 1e-4);
    // Directly behind (z negative, x=0): this part of the ring is capped away.
    assert!(capped.eval(0.0, 0.0, -5.0) > 5.0);
}

#[test]
fn link_matches_torus_at_zero_length_and_stretches_along_z() {
    let full = SdfNode::Torus {
        major_radius: 3.0,
        minor_radius: 0.75,
    };
    let link = SdfNode::Link {
        major_radius: 3.0,
        minor_radius: 0.75,
        half_length: 0.0,
    };
    for &(x, y, z) in &[(3.0, 0.0, 0.0), (0.0, 0.0, -3.0), (1.0, 0.5, 2.0)] {
        assert!((full.eval(x, y, z) - link.eval(x, y, z)).abs() < 1e-4);
    }

    let stretched = SdfNode::Link {
        major_radius: 3.0,
        minor_radius: 0.75,
        half_length: 4.0,
    };
    // Tube center of the far elongated cap: deep inside (~ -minor_radius).
    assert!((stretched.eval(0.0, 0.0, 7.0) - (-0.75)).abs() < 1e-4);
    // A plain torus would be far outside here; the link is not.
    assert!(full.eval(0.0, 0.0, 7.0) > 0.0);
}

#[test]
fn infinite_cylinder_is_exact_and_unbounded_along_y() {
    let c = SdfNode::InfiniteCylinder { radius: 2.0 };
    assert!((c.eval(0.0, 0.0, 0.0) - (-2.0)).abs() < 1e-6);
    assert!((c.eval(2.0, 0.0, 0.0) - 0.0).abs() < 1e-6);
    assert!((c.eval(5.0, 0.0, 0.0) - 3.0).abs() < 1e-6);
    // Distance is independent of Y, however far out.
    assert!((c.eval(5.0, 1.0e9, 0.0) - 3.0).abs() < 1e-3);
    assert!(c.bounds().is_none(), "infinite cylinder is unbounded");
}

#[test]
fn round_cone_matches_capsule_when_radii_equal() {
    let capsule = SdfNode::Capsule {
        a: [0.0, 0.0, 0.0],
        b: [0.0, 10.0, 0.0],
        radius: 2.0,
    };
    let round_cone = SdfNode::RoundCone {
        a: [0.0, 0.0, 0.0],
        b: [0.0, 10.0, 0.0],
        r1: 2.0,
        r2: 2.0,
    };
    for &(x, y, z) in &[
        (0.0, -5.0, 0.0),
        (0.0, 0.0, 0.0),
        (5.0, 5.0, 0.0),
        (0.0, 5.0, 1.5),
        (0.0, 10.0, 0.0),
        (0.0, 15.0, 0.0),
        (3.0, 3.0, 4.0),
    ] {
        let a = capsule.eval(x, y, z);
        let b = round_cone.eval(x, y, z);
        assert!(
            (a - b).abs() < 1e-4,
            "mismatch at ({x},{y},{z}): capsule={a}, round_cone={b}"
        );
    }
}

#[test]
fn round_cone_centers_are_exactly_inside_by_their_radius() {
    let rc = SdfNode::RoundCone {
        a: [0.0, 0.0, 0.0],
        b: [0.0, 10.0, 0.0],
        r1: 3.0,
        r2: 1.0,
    };
    // The center of each end sphere is always fully enclosed by that sphere,
    // and the round-cone surface can never cut into the sphere, so the
    // nearest-surface distance from a center is exactly its own radius.
    assert!((rc.eval(0.0, 0.0, 0.0) - (-3.0)).abs() < 1e-4);
    assert!((rc.eval(0.0, 10.0, 0.0) - (-1.0)).abs() < 1e-4);
}

#[test]
fn round_cone_caps_extend_correctly_beyond_endpoints() {
    let rc = SdfNode::RoundCone {
        a: [0.0, 0.0, 0.0],
        b: [0.0, 10.0, 0.0],
        r1: 3.0,
        r2: 1.0,
    };
    // Straight out along the axis beyond either endpoint, the nearest surface
    // is the spherical cap directly on the axis: distance is exactly the
    // extra length past `center +/- radius`.
    assert!((rc.eval(0.0, -8.0, 0.0) - 5.0).abs() < 1e-4); // 3 (radius) + 5 = 8 below origin
    assert!((rc.eval(0.0, 16.0, 0.0) - 5.0).abs() < 1e-4); // 1 (radius) + 5 = 6 past y=10
}

#[test]
fn round_cone_is_rotationally_symmetric_about_its_axis() {
    let rc = SdfNode::RoundCone {
        a: [0.0, 0.0, 0.0],
        b: [0.0, 10.0, 0.0],
        r1: 3.0,
        r2: 1.0,
    };
    let d0 = rc.eval(3.0, 5.0, 0.0);
    let d1 = rc.eval(0.0, 5.0, 3.0);
    let d2 = rc.eval(2.121_320_3, 5.0, 2.121_320_3);
    assert!((d0 - d1).abs() < 1e-4);
    assert!((d0 - d2).abs() < 1e-4);
}

#[test]
fn solid_angle_apex_is_on_the_surface() {
    let wedge = SdfNode::SolidAngle {
        radius: 10.0,
        angle: 30.0,
    };
    assert!(wedge.eval(0.0, 0.0, 0.0).abs() < 1e-4);
}

#[test]
fn solid_angle_axis_interior_matches_lateral_cone_wall() {
    let wedge = SdfNode::SolidAngle {
        radius: 10.0,
        angle: 30.0,
    };
    // On-axis, well inside the sphere: for a narrow wedge the slanted cone
    // wall is closer than the spherical cap. Distance = -y*sin(angle).
    assert!((wedge.eval(0.0, 5.0, 0.0) - (-2.5)).abs() < 1e-3);
}

#[test]
fn solid_angle_beyond_sphere_on_axis_matches_cap_distance() {
    let wedge = SdfNode::SolidAngle {
        radius: 10.0,
        angle: 30.0,
    };
    // Straight up the axis past the sphere: nearest surface is the cap, so
    // distance is simply y - radius.
    assert!((wedge.eval(0.0, 15.0, 0.0) - 5.0).abs() < 1e-3);
}

#[test]
fn solid_angle_behind_apex_matches_distance_to_vertex() {
    let wedge = SdfNode::SolidAngle {
        radius: 10.0,
        angle: 30.0,
    };
    // Directly behind the apex (outside the wedge entirely, even though
    // within the sphere's radius): nearest boundary is the apex itself.
    assert!((wedge.eval(0.0, -5.0, 0.0) - 5.0).abs() < 1e-3);
}

#[test]
fn solid_angle_at_90_degrees_flat_cap_is_the_equatorial_plane() {
    let hemisphere = SdfNode::SolidAngle {
        radius: 10.0,
        angle: 90.0,
    };
    // At a right angle the cone's lateral surface degenerates to the flat
    // y=0 plane, so a point on that plane inside the sphere sits exactly on
    // the boundary...
    assert!(hemisphere.eval(3.0, 0.0, 0.0).abs() < 1e-3);
    // ...and a point just below it is just outside the hemisphere.
    assert!((hemisphere.eval(3.0, -1.0, 0.0) - 1.0).abs() < 1e-3);
}

#[test]
fn cut_sphere_keeps_the_cap_above_the_cut_plane() {
    let dome = SdfNode::CutSphere {
        radius: 5.0,
        height: 2.0,
    };
    // North pole: always on the sphere surface, regardless of the cut.
    assert!(dome.eval(0.0, 5.0, 0.0).abs() < 1e-4);
    // Center of the flat cut disk: on the flat boundary face.
    assert!(dome.eval(0.0, 2.0, 0.0).abs() < 1e-4);
    // Sphere center: excluded (below the cut), 2 units short of the disk.
    assert!((dome.eval(0.0, 0.0, 0.0) - 2.0).abs() < 1e-4);
    // Just below the disk, still under its footprint: straight-up distance.
    assert!((dome.eval(0.0, 1.0, 0.0) - 1.0).abs() < 1e-4);
}

#[test]
fn cut_sphere_is_exact_at_the_rim_corner() {
    let dome = SdfNode::CutSphere {
        radius: 5.0,
        height: 2.0,
    };
    // Independently solved via calculus: on the cut plane but far out
    // radially (past the rim at w = sqrt(25-4) ~= 4.583), the nearest
    // boundary point is the rim vertex itself, distance 10 - w ~= 5.417.
    // A naive CSG-intersection (max of the two half-space SDFs) would give
    // ~5.198 here, underestimating at the corner -- exactness requires
    // explicitly finding the vertex.
    let w = (25.0f32 - 4.0).sqrt();
    assert!((dome.eval(10.0, 2.0, 0.0) - (10.0 - w)).abs() < 1e-3);
}

#[test]
fn cut_sphere_far_below_the_cap_matches_the_flat_segment() {
    let dome = SdfNode::CutSphere {
        radius: 5.0,
        height: 2.0,
    };
    // Independently solved: comparing distance to the flat segment's
    // nearest point (0,2) [=12] against the arc's nearest reachable point,
    // the rim vertex [=sqrt(21+144)~=12.845], the flat segment wins.
    assert!((dome.eval(0.0, -10.0, 0.0) - 12.0).abs() < 1e-3);
}

#[test]
fn cut_hollow_sphere_is_a_thin_open_shell_of_the_cap() {
    let bowl = SdfNode::CutHollowSphere {
        radius: 5.0,
        height: 2.0,
        thickness: 0.3,
    };
    // On the generating arc itself (e.g. the pole, or the rim), the shell
    // core sits at -thickness, same convention as Torus's minor_radius.
    assert!((bowl.eval(0.0, 5.0, 0.0) - (-0.3)).abs() < 1e-3);
    let w = (25.0f32 - 4.0).sqrt();
    assert!((bowl.eval(w, 2.0, 0.0) - (-0.3)).abs() < 1e-3);
    // Well outside the shell (past the excluded flat "floor" region, at the
    // equator): positive, roughly the planar gap to the rim minus thickness.
    let expected = ((5.0f32 - w).powi(2) + 4.0).sqrt() - 0.3;
    assert!((bowl.eval(5.0, 0.0, 0.0) - expected).abs() < 1e-3);
}

#[test]
fn infinite_cone_apex_is_on_surface_and_unbounded() {
    let cone = SdfNode::InfiniteCone { angle: 30.0 };
    assert!(cone.eval(0.0, 0.0, 0.0).abs() < 1e-4);
    assert!(cone.bounds().is_none(), "infinite cone is unbounded");
}

#[test]
fn infinite_cone_lateral_wall_and_behind_apex() {
    let cone = SdfNode::InfiniteCone { angle: 30.0 };
    // On-axis interior: nearest boundary is the slanted wall, distance = -y*sin(angle).
    let sin30 = 30f32.to_radians().sin();
    assert!((cone.eval(0.0, 5.0, 0.0) - (-5.0 * sin30)).abs() < 1e-3);
    // Directly behind the apex (negative Y): outside the single nappe.
    assert!(cone.eval(0.0, -5.0, 0.0) > 0.0);
    // Extends forever along +Y without ever closing off (unlike SolidAngle's
    // sphere cap): still deep inside arbitrarily far up the axis.
    assert!(cone.eval(0.0, 1.0e6, 0.0) < 0.0);
}

#[test]
fn infinite_cone_is_exact_at_a_hand_solved_lateral_point() {
    let angle_deg = 20.0f32;
    let cone = SdfNode::InfiniteCone { angle: angle_deg };
    // A point exactly on the cone wall, displaced further out along the
    // wall's own normal, is at a known exact distance from the surface.
    let a = angle_deg.to_radians();
    let (sin_a, cos_a) = a.sin_cos();
    let y0 = 4.0f32;
    let x0 = y0 * sin_a / cos_a; // on-wall point at height y0
    let normal_offset = 1.5f32;
    let (nx, ny) = (cos_a, -sin_a); // outward wall normal in the XY half-plane
    let px = x0 + nx * normal_offset;
    let py = y0 + ny * normal_offset;
    assert!((cone.eval(px, py, 0.0) - normal_offset).abs() < 1e-3);
}

#[test]
fn square_pyramid_apex_and_base_are_on_surface() {
    let pyr = SdfNode::SquarePyramid {
        half_base: 2.0,
        height: 4.0,
    };
    // Apex at y = height/2.
    assert!(pyr.eval(0.0, 2.0, 0.0).abs() < 1e-3);
    // Base center at y = -height/2.
    assert!(pyr.eval(0.0, -2.0, 0.0).abs() < 1e-3);
    // Base corner.
    assert!(pyr.eval(2.0, -2.0, 2.0).abs() < 1e-3);
}

#[test]
fn square_pyramid_interior_is_negative_and_exterior_positive() {
    let pyr = SdfNode::SquarePyramid {
        half_base: 2.0,
        height: 4.0,
    };
    assert!(
        pyr.eval(0.0, -1.9, 0.0) < 0.0,
        "just above base, near center"
    );
    assert!(
        pyr.eval(0.0, -2.0, 10.0) > 0.0,
        "far outside on the base plane"
    );
    assert!(pyr.eval(0.0, 10.0, 0.0) > 0.0, "far above the apex");
}

#[test]
fn square_pyramid_bounds_are_tight_and_finite() {
    let pyr = SdfNode::SquarePyramid {
        half_base: 3.0,
        height: 6.0,
    };
    let b = pyr.bounds().unwrap();
    assert!((b.min[0] + 3.0).abs() < 1e-6);
    assert!((b.max[0] - 3.0).abs() < 1e-6);
    assert!((b.min[1] + 3.0).abs() < 1e-6);
    assert!((b.max[1] - 3.0).abs() < 1e-6);
    assert!((b.min[2] + 3.0).abs() < 1e-6);
    assert!((b.max[2] - 3.0).abs() < 1e-6);
}

#[test]
fn xor_is_solid_in_exactly_one_child_not_both_or_neither() {
    let xor = SdfNode::Xor {
        a: Box::new(SdfNode::Sphere { radius: 3.0 }),
        b: Box::new(SdfNode::Translate {
            child: Box::new(SdfNode::Sphere { radius: 3.0 }),
            offset: [4.0, 0.0, 0.0],
        }),
    };
    // Deep inside only the left sphere.
    assert!(xor.eval(-2.0, 0.0, 0.0) < 0.0);
    // Deep inside only the right sphere.
    assert!(xor.eval(6.0, 0.0, 0.0) < 0.0);
    // Inside both spheres (overlap region near x=2): excluded by XOR.
    assert!(xor.eval(2.0, 0.0, 0.0) > 0.0);
    // Outside both.
    assert!(xor.eval(20.0, 0.0, 0.0) > 0.0);
}

#[test]
fn xor_bounds_union_only_when_both_children_bounded() {
    let bounded = SdfNode::Xor {
        a: Box::new(SdfNode::Sphere { radius: 3.0 }),
        b: Box::new(SdfNode::Translate {
            child: Box::new(SdfNode::Sphere { radius: 2.0 }),
            offset: [10.0, 0.0, 0.0],
        }),
    };
    let b = bounded.bounds().unwrap();
    assert!((b.max[0] - 12.0).abs() < 1e-5);
    assert!((b.min[0] + 3.0).abs() < 1e-5);

    let unbounded = SdfNode::Xor {
        a: Box::new(SdfNode::Sphere { radius: 3.0 }),
        b: Box::new(SdfNode::Plane {
            normal: [0.0, 1.0, 0.0],
            offset: 0.0,
        }),
    };
    assert!(unbounded.bounds().is_none());
}

#[test]
fn elongate_grows_a_sphere_into_a_capsule_shaped_footprint() {
    let elongated = SdfNode::Elongate {
        child: Box::new(SdfNode::Sphere { radius: 1.0 }),
        half_lengths: [3.0, 0.0, 0.0],
    };
    // Along the elongation axis, the surface sits `half_length` further out.
    assert!((elongated.eval(4.0, 0.0, 0.0) - 0.0).abs() < 1e-4);
    assert!(elongated.eval(0.0, 0.0, 0.0) < 0.0);
    // Perpendicular cross-section still matches the plain sphere radius.
    assert!((elongated.eval(0.0, 1.0, 0.0) - 0.0).abs() < 1e-4);
}

#[test]
fn elongate_bounds_grow_componentwise() {
    let elongated = SdfNode::Elongate {
        child: Box::new(SdfNode::Sphere { radius: 1.0 }),
        half_lengths: [3.0, 0.5, 0.0],
    };
    let b = elongated.bounds().unwrap();
    assert!((b.max[0] - 4.0).abs() < 1e-5);
    assert!((b.max[1] - 1.5).abs() < 1e-5);
    assert!((b.max[2] - 1.0).abs() < 1e-5);
}

#[test]
fn twist_preserves_y_range_and_rotates_off_axis_points() {
    let box_node = SdfNode::Box {
        half_extents: [1.0, 5.0, 1.0],
        rounding: 0.0,
    };
    let twisted = SdfNode::Twist {
        child: Box::new(box_node.clone()),
        amount: std::f32::consts::FRAC_PI_2,
    };
    // On the Y axis, twisting has no effect (rotation fixes the origin of XZ).
    assert!((twisted.eval(0.0, 0.0, 0.0) - box_node.eval(0.0, 0.0, 0.0)).abs() < 1e-4);
    // At y=1, a 90-degree-per-unit twist rotates the sample point 90 degrees;
    // evaluating the untwisted box at the rotated coordinates matches.
    let d_twisted = twisted.eval(1.0, 1.0, 0.0);
    let d_expected = box_node.eval(0.0, 1.0, 1.0);
    assert!((d_twisted - d_expected).abs() < 1e-4);
}

#[test]
fn twist_bounds_preserve_y_and_grow_radially() {
    let child = SdfNode::Box {
        half_extents: [1.0, 5.0, 1.0],
        rounding: 0.0,
    };
    let twisted = SdfNode::Twist {
        child: Box::new(child),
        amount: 0.5,
    };
    let b = twisted.bounds().unwrap();
    assert!((b.min[1] + 5.0).abs() < 1e-5);
    assert!((b.max[1] - 5.0).abs() < 1e-5);
    let expected_r = (2.0f32).sqrt();
    assert!((b.max[0] - expected_r).abs() < 1e-4);
    assert!((b.max[2] - expected_r).abs() < 1e-4);
}

#[test]
fn bend_preserves_z_range_and_rotates_off_axis_points() {
    let box_node = SdfNode::Box {
        half_extents: [5.0, 1.0, 1.0],
        rounding: 0.0,
    };
    let bent = SdfNode::Bend {
        child: Box::new(box_node.clone()),
        amount: std::f32::consts::FRAC_PI_2,
    };
    assert!((bent.eval(0.0, 0.0, 0.0) - box_node.eval(0.0, 0.0, 0.0)).abs() < 1e-4);
    // A 90-degree-per-unit bend rotates the sample point at x=1 by 90 degrees.
    let d_bent = bent.eval(1.0, 0.0, 0.0);
    let d_expected = box_node.eval(0.0, 1.0, 0.0);
    assert!((d_bent - d_expected).abs() < 1e-4);
}

#[test]
fn bend_bounds_preserve_z_and_grow_radially() {
    let child = SdfNode::Box {
        half_extents: [5.0, 1.0, 1.0],
        rounding: 0.0,
    };
    let bent = SdfNode::Bend {
        child: Box::new(child),
        amount: 0.5,
    };
    let b = bent.bounds().unwrap();
    assert!((b.min[2] + 1.0).abs() < 1e-5);
    assert!((b.max[2] - 1.0).abs() < 1e-5);
    let expected_r = (26.0f32).sqrt();
    assert!((b.max[0] - expected_r).abs() < 1e-4);
    assert!((b.max[1] - expected_r).abs() < 1e-4);
}

#[test]
fn new_iq_primitives_round_trip_through_json() {
    let cases = [
        r#"{"type":"roundCone","a":[0,0,0],"b":[0,10,0],"r1":3.0,"r2":1.0}"#,
        r#"{"type":"solidAngle","radius":5.0,"angle":45.0}"#,
        r#"{"type":"cutSphere","radius":5.0,"height":2.0}"#,
        r#"{"type":"cutHollowSphere","radius":5.0,"height":2.0,"thickness":0.3}"#,
    ];
    for json in cases {
        let n = SdfNode::from_json(json).unwrap_or_else(|e| panic!("{json}: {e}"));
        let re = SdfNode::from_json(&n.to_json().unwrap()).unwrap();
        for &(x, y, z) in &[(0.0, 0.0, 0.0), (2.0, 3.0, -1.0), (10.0, -4.0, 6.0)] {
            assert_eq!(
                n.eval(x, y, z).to_bits(),
                re.eval(x, y, z).to_bits(),
                "{json} mismatch at ({x},{y},{z})"
            );
        }
    }
}

#[test]
fn cells_distance_modes_are_nonnegative() {
    for mode in ["f1", "f2", "f2MinusF1"] {
        let json = format!(r#"{{"type":"cells","frequency":0.12,"seed":9,"mode":"{mode}"}}"#);
        let n = SdfNode::from_json(&json).unwrap();
        for i in 0..40 {
            assert!(
                n.eval(i as f32 * 0.9, 2.0, i as f32 * -1.3) >= -1e-4,
                "{mode} nonneg"
            );
        }
    }
}

// ── HexPrism bounds ─────────────────────────────────────────────────────────
//
// The public `radius` is the hexagon's apothem (inradius) toward the flat
// edges, not its circumradius. The cross-section lives in XZ (height along
// Y): flat edges face Z (extent stays `radius`), but the hexagon's corners
// stick out along X to the circumradius `2 * radius / sqrt(3)`.

#[test]
fn hex_prism_corner_sits_on_the_surface_at_the_circumradius() {
    let radius = 3.0_f32;
    let half_height = 2.0_f32;
    let node = SdfNode::HexPrism {
        radius,
        half_height,
    };
    let circumradius = 2.0 * radius / 3f32.sqrt();

    // The analytic extremum (hex corner, z=0/y=0) must sit on the surface.
    let at_corner = node.eval(circumradius, 0.0, 0.0);
    assert!(
        at_corner.abs() < 1e-3,
        "expected the hex corner at x={circumradius} to be on the surface, got {at_corner}"
    );
    // A point just short of the corner is still inside; just past it is outside.
    assert!(node.eval(circumradius - 0.05, 0.0, 0.0) < 0.0);
    assert!(node.eval(circumradius + 0.05, 0.0, 0.0) > 0.0);

    // The old (wrong) bound used the apothem `radius` itself for X, which
    // sits well inside the true corner and would have clipped the surface.
    assert!(radius < circumradius);
    assert!(
        node.eval(radius, 0.0, 0.0) < 0.0,
        "apothem-only X bound would clip the corner, which is still solid there"
    );
}

#[test]
fn hex_prism_bounds_cover_the_true_circumradius_without_clipping() {
    let radius = 3.0_f32;
    let half_height = 2.0_f32;
    let node = SdfNode::HexPrism {
        radius,
        half_height,
    };
    let bounds = node.bounds().expect("hex prism is bounded");
    let circumradius = 2.0 * radius / 3f32.sqrt();

    // X (through the corners) must reach the circumradius, not just the apothem.
    assert!(
        bounds.max[0] >= circumradius - 1e-4,
        "bounds.max[0]={} must cover the circumradius={circumradius}",
        bounds.max[0]
    );
    assert!(bounds.min[0] <= -circumradius + 1e-4);

    // Z (through the flats) keeps the apothem extent unchanged.
    assert!((bounds.max[2] - radius).abs() < 1e-4);
    assert!((bounds.min[2] + radius).abs() < 1e-4);

    // Y (extrusion axis) is untouched — orientation is preserved exactly.
    assert!((bounds.max[1] - half_height).abs() < 1e-6);
    assert!((bounds.min[1] + half_height).abs() < 1e-6);

    // A point just outside the reported bounds must be strictly outside the shape.
    assert!(node.eval(bounds.max[0] + 0.05, 0.0, 0.0) > 0.0);
    assert!(node.eval(0.0, 0.0, bounds.max[2] + 0.05) > 0.0);
    assert!(node.eval(0.0, bounds.max[1] + 0.05, 0.0) > 0.0);
}

// ── SdfNode::validate ────────────────────────────────────────────────────────

fn assert_json_invalid(json: &str) {
    assert!(
        SdfNode::from_json(json).is_err(),
        "expected invalid SDF JSON to be rejected: {json}"
    );
}

fn assert_json_valid(json: &str) {
    assert!(
        SdfNode::from_json(json).is_ok(),
        "expected valid SDF JSON to be accepted: {json}"
    );
}

#[test]
fn validate_rejects_non_finite_and_accepts_valid_primitives() {
    assert_json_valid(r#"{"type":"sphere","radius":5}"#);
    assert_json_invalid(r#"{"type":"sphere","radius":-5}"#);
    assert_json_invalid(r#"{"type":"sphere","radius":0}"#);
    // A JSON literal with an extreme exponent overflows f32 to infinity
    // without failing to *parse* as JSON — validate() must still reject it.
    assert_json_invalid(r#"{"type":"sphere","radius":1e400}"#);

    // Direct construction (bypassing JSON) also goes through validate().
    assert!(SdfNode::Sphere { radius: f32::NAN }.validate().is_err());
    assert!(SdfNode::Sphere {
        radius: f32::INFINITY
    }
    .validate()
    .is_err());
    assert!(SdfNode::Sphere { radius: 5.0 }.validate().is_ok());
}

#[test]
fn validate_rejects_invalid_signs_and_ranges() {
    // Box: rounding cannot exceed the smallest half-extent.
    assert_json_valid(r#"{"type":"box","halfExtents":[2,3,4],"rounding":2}"#);
    assert_json_invalid(r#"{"type":"box","halfExtents":[2,3,4],"rounding":5}"#);
    assert_json_invalid(r#"{"type":"box","halfExtents":[-2,3,4]}"#);

    // BoxFrame: same thickness-vs-extent constraint.
    assert_json_valid(r#"{"type":"boxFrame","halfExtents":[2,2,2],"thickness":0.25}"#);
    assert_json_invalid(r#"{"type":"boxFrame","halfExtents":[2,2,2],"thickness":2.5}"#);

    // CappedCone: radii cannot both be zero (degenerate point).
    assert_json_valid(r#"{"type":"cappedCone","halfHeight":2,"r1":1,"r2":0}"#);
    assert_json_invalid(r#"{"type":"cappedCone","halfHeight":2,"r1":0,"r2":0}"#);
    assert_json_invalid(r#"{"type":"cappedCone","halfHeight":-2,"r1":1,"r2":1}"#);

    // Plane: degenerate (zero-length) normal is rejected.
    assert_json_valid(r#"{"type":"plane","normal":[0,1,0]}"#);
    assert_json_invalid(r#"{"type":"plane","normal":[0,0,0]}"#);

    // RoundCone: coincident endpoints collapse the axis (division by zero).
    assert_json_valid(r#"{"type":"roundCone","a":[0,0,0],"b":[0,10,0],"r1":3,"r2":1}"#);
    assert_json_invalid(r#"{"type":"roundCone","a":[1,1,1],"b":[1,1,1],"r1":3,"r2":1}"#);

    // Later exact IQ primitives retain the same constraints through JSON.
    assert_json_valid(r#"{"type":"solidAngle","radius":5,"angle":60}"#);
    assert_json_invalid(r#"{"type":"solidAngle","radius":5,"angle":0}"#);
    assert_json_invalid(r#"{"type":"solidAngle","radius":5,"angle":180}"#);
    assert_json_valid(r#"{"type":"infiniteCone","angle":45}"#);
    assert_json_invalid(r#"{"type":"infiniteCone","angle":0}"#);
    assert_json_invalid(r#"{"type":"infiniteCone","angle":90}"#);
    assert_json_valid(r#"{"type":"squarePyramid","halfBase":2,"height":4}"#);
    assert_json_invalid(r#"{"type":"squarePyramid","halfBase":0,"height":4}"#);
    assert_json_valid(r#"{"type":"cutSphere","radius":5,"height":0}"#);
    assert_json_invalid(r#"{"type":"cutSphere","radius":5,"height":6}"#);
    assert_json_invalid(r#"{"type":"cutSphere","radius":5,"height":-6}"#);
    assert_json_valid(r#"{"type":"cutHollowSphere","radius":5,"height":0,"thickness":0.25}"#);
    assert_json_invalid(r#"{"type":"cutHollowSphere","radius":5,"height":0,"thickness":0}"#);
}

#[test]
fn validate_rejects_invalid_enum_specific_constraints() {
    // CappedTorus: cap_angle must be in (0, 180].
    assert_json_valid(r#"{"type":"cappedTorus","majorRadius":5,"minorRadius":1,"capAngle":90}"#);
    assert_json_invalid(r#"{"type":"cappedTorus","majorRadius":5,"minorRadius":1,"capAngle":0}"#);
    assert_json_invalid(r#"{"type":"cappedTorus","majorRadius":5,"minorRadius":1,"capAngle":181}"#);

    // Displace: octaves must be in 1..=8.
    let displace = |octaves: i32| {
        format!(
            r#"{{"type":"displace","amplitude":1,"frequency":0.1,"seed":1,"octaves":{octaves},
                 "child":{{"type":"sphere","radius":1}}}}"#
        )
    };
    assert_json_valid(&displace(3));
    assert_json_invalid(&displace(0));
    assert_json_invalid(&displace(9));
}

#[test]
fn validate_rejects_malformed_transforms() {
    // Scale factor must be a positive finite number.
    assert_json_valid(r#"{"type":"scale","factor":2.0,"child":{"type":"sphere","radius":2}}"#);
    assert_json_invalid(r#"{"type":"scale","factor":0,"child":{"type":"sphere","radius":2}}"#);
    assert_json_invalid(r#"{"type":"scale","factor":-1,"child":{"type":"sphere","radius":2}}"#);

    // Translate/Rotate reject non-finite offsets/angles.
    assert_json_invalid(
        r#"{"type":"translate","offset":[1e400,0,0],"child":{"type":"sphere","radius":2}}"#,
    );
    assert_json_invalid(
        r#"{"type":"rotate","angles":[1e400,0,0],"child":{"type":"sphere","radius":2}}"#,
    );

    // Repeat: spacing must be non-negative and not all zero.
    assert_json_valid(
        r#"{"type":"repeat","spacing":[4,0,4],"child":{"type":"sphere","radius":1}}"#,
    );
    assert_json_invalid(
        r#"{"type":"repeat","spacing":[-4,0,4],"child":{"type":"sphere","radius":1}}"#,
    );
    assert_json_invalid(
        r#"{"type":"repeat","spacing":[0,0,0],"child":{"type":"sphere","radius":1}}"#,
    );

    // Domain operators validate their own parameters and recurse into children.
    assert_json_valid(
        r#"{"type":"elongate","halfLengths":[2,0,1],"child":{"type":"sphere","radius":1}}"#,
    );
    assert_json_invalid(
        r#"{"type":"elongate","halfLengths":[0,0,0],"child":{"type":"sphere","radius":1}}"#,
    );
    assert_json_invalid(r#"{"type":"twist","amount":1e400,"child":{"type":"sphere","radius":1}}"#);
    assert_json_invalid(r#"{"type":"bend","amount":0.1,"child":{"type":"sphere","radius":-1}}"#);
    assert_json_invalid(
        r#"{"type":"xor","a":{"type":"sphere","radius":1},"b":{"type":"sphere","radius":0}}"#,
    );
}

#[test]
fn validate_rejects_invalid_field_program_payloads_at_any_depth() {
    // output_slot 0 with no declared slots: InvalidOutputSlot.
    let bad_program = r#"{"version":1,"slots":[],"instructions":[],"outputSlot":0,
        "bounds":{"min":[-1,-1,-1],"max":[1,1,1]}}"#;

    // Directly as the root node...
    assert_json_invalid(&format!(r#"{{"type":"program","program":{bad_program}}}"#));

    // ...and nested several levels deep inside operators/transforms.
    assert_json_invalid(&format!(
        r#"{{"type":"union","children":[
            {{"type":"sphere","radius":1}},
            {{"type":"translate","offset":[1,0,0],"child":
                {{"type":"round","radius":0.1,"child":
                    {{"type":"program","program":{bad_program}}}
                }}
            }}
        ]}}"#
    ));

    // A validly-formed program still parses.
    let good_program = r#"{"version":1,"slots":["scalar"],"instructions":[
        {"instr":"pushPos"},{"instr":"unary","op":"length"},
        {"instr":"pushConst","value":1.0},{"instr":"binary","op":"sub"},
        {"instr":"storeLocal","slot":0}
    ],"outputSlot":0,"bounds":{"min":[-1,-1,-1],"max":[1,1,1]}}"#;
    assert_json_valid(&format!(r#"{{"type":"program","program":{good_program}}}"#));
}

#[test]
fn validate_rejects_trees_past_the_depth_limit() {
    let mut deep = String::from(r#"{"type":"sphere","radius":1}"#);
    for _ in 0..200 {
        deep = format!(r#"{{"type":"round","radius":0.1,"child":{deep}}}"#);
    }
    assert_json_invalid(&deep);

    let mut shallow = String::from(r#"{"type":"sphere","radius":1}"#);
    for _ in 0..10 {
        shallow = format!(r#"{{"type":"round","radius":0.1,"child":{shallow}}}"#);
    }
    assert_json_valid(&shallow);
}

#[test]
fn validate_rejects_trees_past_the_node_count_limit() {
    let children: Vec<&str> = std::iter::repeat(r#"{"type":"sphere","radius":1}"#)
        .take(8000)
        .collect();
    let wide = format!(r#"{{"type":"union","children":[{}]}}"#, children.join(","));
    assert_json_invalid(&wide);

    let modest: Vec<&str> = std::iter::repeat(r#"{"type":"sphere","radius":1}"#)
        .take(50)
        .collect();
    let ok = format!(r#"{{"type":"union","children":[{}]}}"#, modest.join(","));
    assert_json_valid(&ok);
}

#[test]
fn validate_still_accepts_existing_realistic_trees() {
    assert!(island_tree().validate().is_ok());
    let json = r#"{
        "type":"smoothUnion","k":4.0,
        "a":{"type":"superPrism","halfExtents":[32,2,32],"exponent":6},
        "b":{"type":"displace","amplitude":3.0,"frequency":0.08,"seed":42,"octaves":3,
             "child":{"type":"translate","offset":[0,-14,0],
                      "child":{"type":"ellipsoid","radii":[26,16,26]}}}
    }"#;
    assert_json_valid(json);
}
