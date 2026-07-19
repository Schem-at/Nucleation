#![cfg(feature = "voxelize")]

use nucleation::building::{BlockPalette, Brush, Shape, SpotlightBrush};
use nucleation::voxelize::{voxelize_textured, MeshModel, MeshShape};
use std::sync::Arc;

/// Closed unit cube, 12 triangles, outward winding, no UVs.
const CUBE_OBJ: &str = "
v 0 0 0
v 1 0 0
v 1 1 0
v 0 1 0
v 0 0 1
v 1 0 1
v 1 1 1
v 0 1 1
# -z face
f 1 3 2
f 1 4 3
# +z face
f 5 6 7
f 5 7 8
# -x face
f 1 5 8
f 1 8 4
# +x face
f 2 3 7
f 2 7 6
# -y face
f 1 2 6
f 1 6 5
# +y face
f 4 8 7
f 4 7 3
";

fn fitted_cube(size: f32) -> MeshShape {
    let mut model = MeshModel::from_obj_str(CUBE_OBJ).expect("cube OBJ parses");
    model.fit(size);
    MeshShape::new(model)
}

#[test]
fn obj_cube_voxelizes_solid() {
    let shape = fitted_cube(10.0);

    // Fitted: x/z centered on 0, base at y = 0, largest dim 10.
    let (x0, y0, z0, x1, y1, z1) = shape.bounds();
    assert_eq!((x0, y0, z0), (-5, 0, -5));
    assert_eq!((x1, y1, z1), (4, 9, 4));

    // Center is solid, points outside the cube are not.
    assert!(shape.contains(0, 5, 0), "center must be inside");
    assert!(!shape.contains(-6, 5, 0));
    assert!(!shape.contains(6, 5, 0));
    assert!(!shape.contains(0, 10, 0));
    assert!(!shape.contains(0, -1, 0));
    assert!(!shape.contains(-6, -1, -6));
    assert!(!shape.contains(5, 10, 5));

    // ~1000 solid voxels (10^3), small tolerance for boundary rounding.
    let count = shape.points().len() as i64;
    assert!(
        (count - 1000).abs() <= 50,
        "cube voxel count {count} not within 1000 +/- 50"
    );

    // Normal near the top face points up.
    let (nx, ny, nz) = shape.normal_at(0, 9, 0);
    assert!(
        ny > 0.9,
        "+y face normal should be (0,1,0)-ish, got ({nx},{ny},{nz})"
    );
    // Normal near the bottom face points down.
    let (_, ny_bottom, _) = shape.normal_at(0, 0, 0);
    assert!(ny_bottom < -0.9, "-y face normal should point down, got {ny_bottom}");
}

/// Surface-only voxelization keeps a skin and skips the parity interior fill:
/// a closed cube comes out hollow. This is the mode open ribbons that fold
/// back on themselves (a road with dips and self-overlaps) need, so parity
/// does not fill the enclosed volume.
#[test]
fn surface_shell_skips_the_interior_fill() {
    let mut model = MeshModel::from_obj_str(CUBE_OBJ).expect("cube OBJ parses");
    model.fit(16.0);
    let skin = MeshShape::new(model).with_surface_shell(1.0);

    // Deep interior is empty — a solid/parity cube would fill it.
    assert!(!skin.contains(0, 8, 0), "surface-only cube must be hollow at the center");
    // The faces themselves are present.
    assert!(skin.contains(0, 0, 0), "bottom face voxel present");
    assert!(skin.contains(0, 15, 0), "top face voxel present");

    // Far fewer voxels than the solid parity fill.
    let solid = fitted_cube(16.0).points().len();
    let shell = skin.points().len();
    assert!(
        shell < solid / 2,
        "surface skin {shell} should be well under solid fill {solid}"
    );
}

/// Octahedron: |x| + |y| + |z| <= r, closed and convex — parity must agree
/// with the analytic solid almost everywhere, with no stray voxels.
#[test]
fn obj_octahedron_parity_is_robust() {
    let obj = "
v 5 0 0
v -5 0 0
v 0 5 0
v 0 -5 0
v 0 0 5
v 0 0 -5
f 1 3 5
f 3 2 5
f 2 4 5
f 4 1 5
f 3 1 6
f 2 3 6
f 4 2 6
f 1 4 6
";
    let mut model = MeshModel::from_obj_str(obj).expect("octahedron parses");
    model.fit(10.0);
    let shape = MeshShape::new(model);

    let (x0, y0, z0, x1, y1, z1) = shape.bounds();
    let mut mismatches = 0usize;
    let mut analytic = 0usize;
    let mut solid = 0usize;
    // Fitted octahedron center: x/z at 0, y at 5.
    for x in x0..=x1 {
        for y in y0..=y1 {
            for z in z0..=z1 {
                let (cx, cy, cz) = (x as f64 + 0.5, y as f64 + 0.5, z as f64 + 0.5);
                let inside = cx.abs() + (cy - 5.0).abs() + cz.abs() <= 5.0;
                let voxel = shape.contains(x, y, z);
                analytic += inside as usize;
                solid += voxel as usize;
                // Ignore centers within half a voxel of the surface — those
                // may legitimately land either way.
                let margin = (cx.abs() + (cy - 5.0).abs() + cz.abs() - 5.0).abs();
                if voxel != inside && margin > 0.87 {
                    mismatches += 1;
                }
            }
        }
    }
    assert!(analytic > 100, "analytic octahedron unexpectedly small: {analytic}");
    assert_eq!(
        mismatches, 0,
        "parity voxels disagree with the analytic octahedron away from the surface"
    );
    let ratio = solid as f64 / analytic as f64;
    assert!(
        (0.75..=1.35).contains(&ratio),
        "solid count {solid} implausible vs analytic {analytic}"
    );

    // No stray voxels outside the fitted bounds' diamond corners.
    for (x, y, z) in shape.points() {
        let (cx, cy, cz) = (x as f64 + 0.5, y as f64 + 0.5, z as f64 + 0.5);
        assert!(
            cx.abs() + (cy - 5.0).abs() + cz.abs() <= 5.0 + 0.87,
            "stray voxel at ({x},{y},{z})"
        );
    }
}

#[test]
fn glb_boxtextured_voxelizes_with_texture_colors() {
    let bytes = std::fs::read("tests/samples/BoxTextured.glb").expect("committed sample");
    let mut model = MeshModel::from_glb_bytes(&bytes).expect("BoxTextured loads");
    assert_eq!(model.triangles.len(), 12, "BoxTextured is a 12-triangle cube");
    assert!(
        model.materials.iter().flatten().next().is_some(),
        "BoxTextured must decode its embedded texture"
    );

    model.fit(8.0);
    let shape = MeshShape::new(model);
    let palette = BlockPalette::new_all();
    let schematic = voxelize_textured(&shape, &palette, "boxtextured");

    let counts = schematic.count_block_types();
    let non_air: Vec<_> = counts
        .iter()
        .filter(|(b, _)| !b.name.contains("air"))
        .collect();
    let total: usize = non_air.iter().map(|(_, c)| **c).sum();
    assert!(total >= 300, "8^3 textured box too sparse: {total} blocks");
    assert!(
        non_air.len() > 1,
        "textured voxelization should produce more than one distinct block, got {:?}",
        non_air
    );
}

#[test]
fn spotlight_brush_lights_and_shades() {
    let palette = Arc::new(BlockPalette::new_grayscale());
    // Light straight above the origin, pointing down, 30-degree cone.
    let brush = SpotlightBrush::new((0.0, 20.0, 0.0), (0.0, -1.0, 0.0), 30.0, (255, 255, 255))
        .with_palette(palette);

    let lightness = |block: &nucleation::BlockState| -> f32 {
        let facts = nucleation::blockpedia::get_block(&block.name).expect("palette block known");
        facts.extras.color.as_ref().unwrap().to_extended().oklab[0]
    };

    // Facing the light, on-axis: bright.
    let lit = brush.get_block(0, 0, 0, (0.0, 1.0, 0.0)).unwrap();
    // Facing away from the light: ambient floor only.
    let back = brush.get_block(0, 0, 0, (0.0, -1.0, 0.0)).unwrap();
    // Far off-axis (outside the 30-degree cone) but still facing the light.
    let outside = brush.get_block(40, 0, 0, (0.0, 1.0, 0.0)).unwrap();

    let (l_lit, l_back, l_out) = (lightness(&lit), lightness(&back), lightness(&outside));
    assert!(l_lit > 0.8, "lit face should be bright, got {l_lit} ({})", lit.name);
    assert!(l_back < 0.45, "back face should be dark, got {l_back} ({})", back.name);
    assert!(l_out < 0.45, "outside-cone face should be dark, got {l_out} ({})", outside.name);
    assert!(l_lit > l_back + 0.3);
    assert!(l_lit > l_out + 0.3);
}

#[test]
fn obj_negative_indices_and_fit() {
    // Same cube via negative (relative) indices for the last face group.
    let obj = "
v 0 0 0
v 2 0 0
v 2 2 0
v 0 2 0
f -4 -2 -3
f -4 -1 -2
";
    let model = MeshModel::from_obj_str(obj).expect("negative indices parse");
    assert_eq!(model.triangles.len(), 2);

    // Degenerate input errors instead of panicking.
    assert!(MeshModel::from_obj_str("v 0 0 0\n").is_err());
    assert!(MeshModel::from_glb_bytes(b"not a glb").is_err());
}
