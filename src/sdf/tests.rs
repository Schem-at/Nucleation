use super::*;
use crate::blockpedia::ExtendedColorData;
use crate::building::{BlockPalette, GradientStop, InterpolationSpace};
use crate::BlockState;
use std::sync::Arc;

#[test]
fn primitives_and_bounds_are_exact() {
    let sphere = Field::sphere(5.0).unwrap();
    assert!((sphere.eval(0.0, 0.0, 0.0) + 5.0).abs() < 1e-5);
    assert!((sphere.eval(5.0, 0.0, 0.0)).abs() < 1e-5);
    assert!((sphere.eval(8.0, 0.0, 0.0) - 3.0).abs() < 1e-5);
    assert_eq!(
        sphere.bounds().unwrap(),
        Aabb {
            min: [-5.0, -5.0, -5.0],
            max: [5.0, 5.0, 5.0]
        }
    );

    let b = Field::box_shape([3.0, 4.0, 5.0], 0.5).unwrap();
    assert!(b.eval(0.0, 0.0, 0.0) < 0.0);
    assert!(b.eval(4.0, 0.0, 0.0) > 0.0);

    let pyr = Field::square_pyramid(4.0, 6.0).unwrap();
    assert!(pyr.eval(0.0, 0.0, 0.0) < 0.0);
    assert!(pyr.eval(0.0, 4.0, 0.0) > 0.0);

    let hm = Field::heightmap(10.0, Field::perlin_fbm2(0.05, 7, 3).unwrap().mul_const(4.0).unwrap()).unwrap();
    assert!(hm.eval(0.0, 0.0, 0.0) < 0.0);
    assert!(hm.eval(0.0, 30.0, 0.0) > 0.0);
}

#[test]
fn csg_chamfer_steps_and_polar_repeat_work() {
    let a = Field::sphere(5.0).unwrap();
    let b = Field::box_shape([4.0, 4.0, 4.0], 0.0)
        .unwrap()
        .translate([5.0, 0.0, 0.0])
        .unwrap();

    let smooth = a.smooth_union(&b, 2.0).unwrap();
    let hard = a.union(&b);
    assert!(smooth.eval(2.5, 0.0, 0.0) <= hard.eval(2.5, 0.0, 0.0));

    let chamfer = a.chamfer_union(&b, 2.0).unwrap();
    assert!(chamfer.eval(2.5, 0.0, 0.0) <= hard.eval(2.5, 0.0, 0.0));

    let stepped = a.steps_union(&b, 2.5, 4.0).unwrap();
    assert!(stepped.eval(2.5, 0.0, 0.0).is_finite());

    // Polar repeat of a sphere at +X creates 6 radial copies
    let petal = Field::sphere(1.5).unwrap().translate([6.0, 0.0, 0.0]).unwrap();
    let ring = petal.repeat_polar(6).unwrap();
    assert!(ring.eval(6.0, 0.0, 0.0) < 0.0);
    let ang = std::f32::consts::TAU / 6.0;
    assert!(ring.eval(6.0 * ang.cos(), 0.0, 6.0 * ang.sin()) < 0.0);
}

#[test]
fn infix_expression_supports_gyroid_mandelbulb_and_bound_fields() {
    // Gyroid surface
    let gyroid = Field::expr(
        "sin(x * 0.3) * cos(y * 0.3) + sin(y * 0.3) * cos(z * 0.3) + sin(z * 0.3) * cos(x * 0.3)",
    )
    .unwrap();
    assert!(gyroid.eval(0.0, 0.0, 0.0).abs() < 1e-6);

    // Bounded Mandelbulb loop!
    let bulb_src = r#"
        let z = p;
        let dr = 1.0;
        let r = 0.0;
        for i in 0..8 {
            r = length(z);
            if r > 2.0 { break; }
            let theta = acos(clamp(z.y / r, -1.0, 1.0)) * 8.0;
            let phi = atan2(z.z, z.x) * 8.0;
            let zr = pow(r, 8.0);
            dr = pow(r, 7.0) * 8.0 * dr + 1.0;
            z = vec3(sin(theta) * cos(phi), cos(theta), sin(theta) * sin(phi)) * zr + p;
        }
        0.5 * ln(r) * r / dr
    "#;
    let bulb = Field::expr_with(
        bulb_src,
        &[],
        Some(Aabb::new([-1.5; 3], [1.5; 3]).unwrap()),
        DistanceKind::Estimate,
    )
    .unwrap();
    assert!(bulb.eval(0.0, 0.0, 0.0) <= 0.0 || bulb.eval(0.1, 0.1, 0.1) <= 0.0);
    assert!(bulb.eval(1.4, 1.4, 1.4) > 0.0);

    // Combining bound external Field objects inside an expression + calling them at warped coords!
    let base = Field::sphere(6.0).unwrap();
    let noise = Field::perlin_fbm3(0.1, 42, 3).unwrap();
    let combined = Field::expr_with(
        "s(p - vec3(2.0, 0.0, 0.0)) + 1.5 * n",
        &[("s", base.clone()), ("n", noise.clone())],
        None,
        DistanceKind::Estimate,
    )
    .unwrap();
    let expected = base.eval(1.0, 0.0, 0.0) + 1.5 * noise.eval(3.0, 0.0, 0.0);
    assert!((combined.eval(3.0, 0.0, 0.0) - expected).abs() < 1e-5);
}

#[test]
fn cse_deduplicates_shared_subexpressions_across_graph() {
    let shared_noise = Field::perlin_fbm3(0.08, 99, 4).unwrap();
    let displaced = Field::sphere(8.0)
        .unwrap()
        .displace(&shared_noise, 2.0)
        .unwrap();

    // Compile [displaced, shared_noise] into one graph: shared_noise must only
    // appear once in the compiled instruction stream!
    let graph_both = CompiledGraph::compile(&[displaced.clone(), shared_noise.clone()]);
    let graph_single = CompiledGraph::compile(&[displaced]);
    assert_eq!(
        graph_both.instruction_count(),
        graph_single.instruction_count(),
        "CSE should share the exact same noise instruction between geometry and auxiliary root"
    );
}

#[test]
fn interval_pruning_matches_brute_force_bit_for_bit() {
    let a = Field::sphere(6.0).unwrap().translate([-3.0, 1.0, 0.0]).unwrap();
    let b = Field::box_shape([4.0, 4.0, 4.0], 0.5)
        .unwrap()
        .rotate([15.0, 30.0, 0.0])
        .unwrap()
        .translate([3.0, -1.0, 2.0])
        .unwrap();
    let noise = Field::perlin_fbm3(0.1, 13, 3).unwrap();
    let field = a
        .smooth_union(&b, 2.0)
        .unwrap()
        .subtract(&Field::capped_cylinder(2.0, 10.0).unwrap())
        .displace(&noise, 1.2)
        .unwrap();

    let compiled = field.compile();
    let bounds = auto_bounds(&field).unwrap();

    // Brute-force set of solid voxels
    let mut brute = Vec::new();
    for x in bounds.min[0]..=bounds.max[0] {
        for y in bounds.min[1]..=bounds.max[1] {
            for z in bounds.min[2]..=bounds.max[2] {
                if compiled.eval(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5) <= 0.0 {
                    brute.push((x, y, z));
                }
            }
        }
    }

    // Pipeline with 8x8x8 interval brick pruning + parallel tiles
    let pipe = VoxelPipeline::new(field, Material::solid("minecraft:stone"));
    let schem = pipe.to_schematic(Some(bounds), "pruned").unwrap();
    assert_eq!(
        schem.total_blocks() as usize,
        brute.len(),
        "brick-pruned voxel count must match brute force exactly"
    );
    for (x, y, z) in brute {
        assert!(
            schem.get_block(x, y, z).is_some(),
            "missing solid voxel ({x}, {y}, {z})"
        );
    }
}

#[test]
fn dual_autodiff_normals_match_exact_sphere_and_smooth_union() {
    let s = Field::sphere(10.0).unwrap().translate([3.0, -2.0, 5.0]).unwrap();
    let n = s.normal(13.0, -2.0, 5.0, 0.5).unwrap();
    assert!((n[0] - 1.0).abs() < 1e-5 && n[1].abs() < 1e-5 && n[2].abs() < 1e-5);

    // Custom differentiable expression normal via forward-mode Dual3
    let expr = Field::expr("length(p - vec3(1.0, 2.0, 3.0)) - 5.0").unwrap();
    let en = expr.normal(1.0, 7.0, 3.0, 0.5).unwrap();
    assert!(en[0].abs() < 1e-5 && (en[1] - 1.0).abs() < 1e-5 && en[2].abs() < 1e-5);
}

#[test]
fn multi_material_tags_blend_across_smooth_union_seam() {
    let stone_sphere = Field::sphere(6.0)
        .unwrap()
        .translate([-4.0, 0.0, 0.0])
        .unwrap()
        .tag(1)
        .unwrap();
    let copper_sphere = Field::sphere(6.0)
        .unwrap()
        .translate([4.0, 0.0, 0.0])
        .unwrap()
        .tag(2)
        .unwrap();
    let merged = stone_sphere.smooth_union(&copper_sphere, 4.0).unwrap();

    let compiled = merged.compile();
    assert_eq!(compiled.eval_tagged(-6.0, 0.0, 0.0).material_id, 1);
    assert_eq!(compiled.eval_tagged(6.0, 0.0, 0.0).material_id, 2);
    let seam = compiled.eval_tagged(0.0, 0.0, 0.0);
    assert!(seam.blend_weight > 0.4, "expected strong seam blend weight, got {}", seam.blend_weight);

    let mat = Material::match_tag(
        vec![
            (1, Material::solid("minecraft:stone")),
            (2, Material::solid("minecraft:copper_block")),
        ],
        Material::solid("minecraft:dirt"),
        true,
        42,
    );
    let schem = VoxelPipeline::new(merged, mat)
        .to_schematic(None, "tagged")
        .unwrap();
    assert_eq!(
        schem.get_block(-7, 0, 0).unwrap().get_name(),
        "minecraft:stone"
    );
    assert_eq!(
        schem.get_block(7, 0, 0).unwrap().get_name(),
        "minecraft:copper_block"
    );
}

#[test]
fn layered_materials_slope_depth_fluid_and_scatter_pipeline() {
    // Terrain hill with flat top + steep cliff, sea level fluid, grass cap + flowers
    let hill = Field::ellipsoid([14.0, 10.0, 14.0])
        .unwrap()
        .translate([0.0, 0.0, 0.0])
        .unwrap();
    let moisture = Field::perlin_fbm3(0.08, 7, 3).unwrap();

    let grass_cap = MaterialLayer {
        condition: Condition::depth_below_surface(Some(0), Some(0))
            .and(Condition::slope_range(None, Some(0.35)))
            .and(Condition::y_range(Some(2.0), None)),
        material: Material::solid("minecraft:grass_block"),
    };
    let dirt_sub = MaterialLayer {
        condition: Condition::depth_below_surface(Some(1), Some(2))
            .and(Condition::slope_range(None, Some(0.5))),
        material: Material::solid("minecraft:dirt"),
    };
    let rock_ramp = Material::channel_ramp(
        Channel::Field(moisture),
        -1.0,
        1.0,
        vec![
            BlockState::new("minecraft:deepslate"),
            BlockState::new("minecraft:stone"),
            BlockState::new("minecraft:andesite"),
        ],
        true,
    )
    .unwrap();

    let mat = Material::layered(vec![grass_cap, dirt_sub], Some(rock_ramp));
    let scatter = ScatterRule::new(
        ScatterFace::Top,
        0.5,
        vec![(BlockState::new("minecraft:poppy"), 1.0)],
        123,
    )
    .unwrap()
    .on_block("minecraft:grass_block");

    let pipe = VoxelPipeline::new(hill, mat)
        .with_fluid(FluidLayer::level(1.0, "minecraft:water").unwrap())
        .with_scatter(scatter);

    let bounds = SampleBounds::new([-16, -4, -16], [16, 12, 16]).unwrap();
    let schem = pipe.to_schematic(Some(bounds), "island").unwrap();

    let mut has_grass = false;
    let mut has_water = false;
    let mut has_poppy = false;
    for (_, b) in schem.iter_blocks() {
        match b.get_name() {
            "minecraft:grass_block" => has_grass = true,
            "minecraft:water" => has_water = true,
            "minecraft:poppy" => has_poppy = true,
            _ => {}
        }
    }
    assert!(has_grass, "expected grass cap on flat hill top");
    assert!(has_water, "expected water fluid around hill below y=1");
    assert!(has_poppy, "expected poppy scatter on top of grass blocks");
}

#[test]
fn graph_json_roundtrip_preserves_shared_identity_and_expressions() {
    let shared = Field::perlin_fbm3(0.05, 11, 4).unwrap();
    let surf = Field::sphere(10.0)
        .unwrap()
        .displace(&shared, 2.0)
        .unwrap()
        .tag(3)
        .unwrap();
    let expr = Field::expr_with("s + 0.5 * n", &[("s", surf.clone()), ("n", shared.clone())], None, DistanceKind::Estimate).unwrap();

    let graph = FieldGraph::from_roots([
        ("surface".into(), surf.clone()),
        ("noise".into(), shared.clone()),
        ("expr".into(), expr.clone()),
    ])
    .unwrap();

    let json = graph.to_json().unwrap();
    let parsed = FieldGraph::from_json(&json).unwrap();
    let p_surf = parsed.root("surface").unwrap();
    let p_noise = parsed.root("noise").unwrap();
    let p_expr = parsed.root("expr").unwrap();

    assert!((p_surf.eval(3.5, -2.0, 1.25) - surf.eval(3.5, -2.0, 1.25)).abs() < 1e-6);
    assert!((p_noise.eval(3.5, -2.0, 1.25) - shared.eval(3.5, -2.0, 1.25)).abs() < 1e-6);
    assert!((p_expr.eval(3.5, -2.0, 1.25) - expr.eval(3.5, -2.0, 1.25)).abs() < 1e-6);

    // ChannelGradient smoke check
    let palette = Arc::new(BlockPalette::new_concrete());
    let grad = Material::channel_gradient(
        Channel::Slope,
        vec![
            GradientStop {
                position: 0.0,
                color: ExtendedColorData::from_rgb(0, 0, 0),
            },
            GradientStop {
                position: 1.0,
                color: ExtendedColorData::from_rgb(255, 255, 255),
            },
        ],
        0.0,
        1.0,
        palette,
        InterpolationSpace::Oklab,
    )
    .unwrap();
    let s = VoxelPipeline::new(Field::sphere(4.0).unwrap(), grad)
        .to_schematic(None, "grad")
        .unwrap();
    assert!(s.total_blocks() > 0);
}

#[test]
fn expr_stdlib_domain_and_chamfer_builtins_work_with_dual3() {
    let twisted = Field::expr("box(rot_y(p, y * 0.15), vec3(3.0, 6.0, 3.0))").unwrap();
    assert!(twisted.eval(0.0, 0.0, 0.0) < 0.0);
    let n = twisted.normal(3.0, 0.0, 0.0, 0.05).unwrap();
    assert!((n[0] - 1.0).abs() < 0.05, "expected +X normal, got {n:?}");

    let polar = Field::expr("cylinder(repeat_polar(p, 6.0) - vec3(6.0, 0.0, 0.0), 1.5, 4.0)").unwrap();
    assert!(polar.eval(6.0, 0.0, 0.0) < 0.0);
    // 60 degrees rotated around Y should also be inside a repeated pillar
    let ang = std::f32::consts::TAU / 6.0;
    assert!(polar.eval(6.0 * ang.cos(), 0.0, 6.0 * ang.sin()) < 0.0);

    let rep = Field::expr("sphere(repeat(p, vec3(10.0, 0.0, 10.0)), 2.0)").unwrap();
    assert!(rep.eval(0.0, 0.0, 0.0) < 0.0);
    assert!(rep.eval(10.0, 0.0, -20.0) < 0.0);

    let ch = Field::expr("chamfer_min(sphere(p + vec3(2.0, 0.0, 0.0), 3.0), box(p - vec3(2.0, 0.0, 0.0), vec3(2.5, 2.5, 2.5)), 1.0)").unwrap();
    assert!(ch.eval(0.0, 0.0, 0.0) < 0.0);
    assert!(ch.normal(0.0, 2.6, 0.0, 0.05).is_some());
}

#[test]
fn schematic_to_sdf_grid_blends_and_roundtrips_json() {
    let base_schem = VoxelPipeline::new(
        Field::box_shape([3.0, 3.0, 3.0], 0.0).unwrap(),
        Material::solid("minecraft:stone"),
    )
    .to_schematic(None, "cube")
    .unwrap();
    let grid_field = Field::from_schematic(&base_schem, 4).unwrap();
    assert!(grid_field.eval(0.5, 0.5, 0.5) < 0.0, "center must be inside");
    assert!(grid_field.eval(8.0, 0.5, 0.5) > 2.0, "exterior must have positive distance");
    let n = grid_field.normal(3.2, 0.5, 0.5, 0.1).unwrap();
    assert!(n[0] > 0.7, "expected outward +X normal on cube face, got {n:?}");

    // Compose with smooth_union + roundtrip JSON
    let blended = grid_field
        .smooth_union(&Field::sphere(3.0).unwrap().translate([4.0, 0.0, 0.0]).unwrap(), 1.5)
        .unwrap();
    let json = blended.to_json().unwrap();
    let reloaded = Field::from_json(&json).unwrap();
    assert!((blended.eval(2.5, 0.5, 0.5) - reloaded.eval(2.5, 0.5, 0.5)).abs() < 1e-5);
}
