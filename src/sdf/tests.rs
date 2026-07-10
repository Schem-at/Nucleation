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
