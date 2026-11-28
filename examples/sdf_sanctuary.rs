//! "The Sunken Gyroid Sanctuary" — showcase of the reworked Nucleation SDF pipeline.
//!
//! Demonstrates in ~180 lines of pure declarative Rust (zero manual voxel loops):
//! - Unified `Field` DAG combining primitives, Perlin/ridged FBM, Worley cells,
//!   polar repeat, stepped & chamfer CSG, and sandboxed infix expressions
//! - Multi-material CSG tagging (`Field::tag`) with smooth seam dithering
//! - Composable layered `Material` shaders reading `DepthBelowSurface`, `Slope`,
//!   `Curvature`, `Distance`, `WorldY`, and CSE-shared `Field` channels
//! - Localized `FluidLayer` (a caldera lagoon bounded by an SDF, not a global plane)
//! - Multi-face `ScatterRule`s (meadow flora on top + multi-block hanging vines underneath)
//! - Brick-pruned, parallel rasterization via `VoxelPipeline`
//!
//! Run (generates schematic):
//!   cargo run --release --example sdf_sanctuary
//!
//! Run with GPU render preview:
//!   cargo run --release --example sdf_sanctuary --features rendering -- render_work/pack.zip render_work/sdf_sanctuary.png

use nucleation::sdf::{
    face_mask, Channel, Condition, DistanceKind, Field, FluidLayer, Material, MaterialLayer,
    SampleBounds, ScatterFace, ScatterRule, VoxelPipeline,
};
use nucleation::BlockState;
use std::collections::BTreeMap;
use std::time::Instant;

const TAG_TERRAIN: u16 = 1;
const TAG_TEMPLE: u16 = 2;
const TAG_CRYSTAL: u16 = 3;

fn build_sanctuary_pipeline() -> Result<(VoxelPipeline, SampleBounds), Box<dyn std::error::Error>> {
    // ── Shared reusable scalar fields (evaluated ONCE per voxel via CSE!) ──
    let macro_noise = Field::perlin_fbm3(0.032, 42, 4)?;
    let ridge_noise = Field::ridged_fbm3(0.045, 137, 4)?;
    let strata_field = Field::expr_with(
        "sin(y * 0.35 + n * 2.4) * 0.5 + r * 0.6",
        &[("n", macro_noise.clone()), ("r", ridge_noise.clone())],
        None,
        DistanceKind::Implicit,
    )?;
    let patina_noise = Field::perlin_fbm3(0.09, 701, 3)?;

    // ── 1. Floating volcanic caldera island (TAG_TERRAIN) ──────────────────
    // Inverted tapered underside + broad plateau rim
    let under_spire = Field::round_cone([0.0, -30.0, 0.0], [0.0, 1.0, 0.0], 5.0, 36.0)?;
    let rim_disk = Field::ellipsoid([38.0, 11.0, 38.0])?.translate([0.0, 1.5, 0.0])?;
    let island_base = under_spire
        .smooth_union(&rim_disk, 8.0)?
        .displace(&macro_noise, 4.0)?
        .displace(&ridge_noise, 1.8)?;

    // Flat-ish rolling meadow top + central caldera lagoon basin
    let rolling_top = Field::heightmap(
        7.5,
        Field::perlin_fbm2(0.036, 19, 4)?.mul_const(4.5)?,
    )?;
    let caldera_bowl = Field::ellipsoid([15.5, 8.5, 15.5])?.translate([0.0, 8.5, 0.0])?;

    // Organic gyroid tunnels & arches carved ONLY through the deep rocky underside (y < -6)
    let gyroid_caves = Field::expr(
        "0.28 - abs(sin(x * 0.20) * cos(y * 0.20) + sin(y * 0.20) * cos(z * 0.20) + sin(z * 0.20) * cos(x * 0.20))",
    )?
    .intersect(&Field::plane([0.0, 1.0, 0.0], 7.0)?) // y <= -7 only
    .intersect(&Field::infinite_cylinder(30.0)?);

    let terrain = island_base
        .smooth_intersect(&rolling_top, 3.0)?
        .smooth_subtract(&caldera_bowl, 2.5)?
        .smooth_subtract(&gyroid_caves, 1.8)?
        .tag(TAG_TERRAIN)?;

    // ── 2. Ancient Hexagonal Sanctuary Temple (TAG_TEMPLE) ─────────────────
    // Stepped podium rising naturally out of the lagoon edge
    let podium = Field::hex_prism(22.0, 3.0)?
        .translate([0.0, 4.0, 0.0])?
        .subtract(&Field::capped_cylinder(15.0, 6.0)?.translate([0.0, 5.0, 0.0])?);

    // 6 radially repeated twisted columns + chamfered capitals + broken arch ring
    let single_pillar = Field::capped_cylinder(1.85, 9.5)?
        .twist(0.14)?
        .chamfer_union(
            &Field::box_shape([2.6, 1.1, 2.6], 0.25)?.translate([0.0, 9.4, 0.0])?,
            0.8,
        )?
        .chamfer_union(
            &Field::box_shape([2.8, 1.2, 2.8], 0.25)?.translate([0.0, -9.2, 0.0])?,
            0.8,
        )?
        .translate([18.5, 14.5, 0.0])?;
    let colonnade = single_pillar.repeat_polar(6)?;

    // Crown ring connecting the pillars + floating tilted astrolabe above the caldera
    let crown_ring = Field::torus(18.5, 1.6)?.translate([0.0, 24.5, 0.0])?;
    let astrolabe_outer = Field::torus(9.5, 1.05)?
        .rotate([28.0, 15.0, -18.0])?
        .translate([0.0, 22.0, 0.0])?;
    let astrolabe_inner = Field::box_frame([4.5, 4.5, 4.5], 0.85)?
        .rotate([35.0, 45.0, 20.0])?
        .translate([0.0, 22.0, 0.0])?;

    let temple = podium
        .chamfer_union(&colonnade, 1.2)?
        .smooth_union(&crown_ring, 1.2)?
        .union(&astrolabe_outer)
        .union(&astrolabe_inner)
        .tag(TAG_TEMPLE)?;

    // ── 3. Glowing Amethyst Crystal Spires & Core (TAG_CRYSTAL) ────────────
    let crystal_shard = Field::round_cone([0.0, 0.0, 0.0], [0.0, 13.0, 0.0], 2.4, 0.45)?
        .rotate([0.0, 0.0, -24.0])?
        .translate([12.0, 2.0, 0.0])?;
    let crystal_ring = crystal_shard.repeat_polar(5)?.rotate([0.0, 18.0, 0.0])?;
    let floating_core = Field::octahedron(3.4)?.translate([0.0, 22.0, 0.0])?;
    let crystals = crystal_ring.union(&floating_core).tag(TAG_CRYSTAL)?;

    // ── Combine all 3 tagged domains with stepped + smooth CSG! ────────────
    let geometry = terrain
        .steps_union(&temple, 2.5, 3.0)?
        .smooth_union(&crystals, 2.2)?;

    // ── Composable Layered Materials per CSG Tag ───────────────────────────

    // Terrain shader: grass cap -> shoreline -> dirt -> weathered ridges -> strata rock -> basalt underside
    let terrain_material = Material::layered(
        vec![
            // Lush meadow top on gentle slopes above the lagoon waterline
            MaterialLayer {
                condition: Condition::depth_below_surface(Some(0), Some(0))
                    .and(Condition::slope_range(None, Some(0.38)))
                    .and(Condition::y_range(Some(3.0), None)),
                material: Material::weighted(
                    vec![
                        (BlockState::new("minecraft:grass_block"), 0.72),
                        (BlockState::new("minecraft:moss_block"), 0.28),
                    ],
                    101,
                )?,
            },
            // Shoreline gravel/sand/mud right around the lagoon water level
            MaterialLayer {
                condition: Condition::exposed_face(face_mask::ANY, false)
                    .and(Condition::y_range(Some(-1.0), Some(3.0)))
                    .and(Condition::slope_range(None, Some(0.65))),
                material: Material::weighted(
                    vec![
                        (BlockState::new("minecraft:gravel"), 0.45),
                        (BlockState::new("minecraft:mud"), 0.30),
                        (BlockState::new("minecraft:coarse_dirt"), 0.25),
                    ],
                    202,
                )?,
            },
            // Subsurface soil band beneath flat grass
            MaterialLayer {
                condition: Condition::depth_below_surface(Some(1), Some(3))
                    .and(Condition::slope_range(None, Some(0.55)))
                    .and(Condition::y_range(Some(2.0), None)),
                material: Material::solid("minecraft:dirt"),
            },
            // Underside ceiling / overhangs -> dark volcanic basalt
            MaterialLayer {
                condition: Condition::slope_range(Some(1.35), None),
                material: Material::weighted(
                    vec![
                        (BlockState::new("minecraft:basalt"), 0.55),
                        (BlockState::new("minecraft:cobbled_deepslate"), 0.45),
                    ],
                    303,
                )?,
            },
            // Sharp exposed ridges (positive curvature) -> weathered light tuff & calcite
            MaterialLayer {
                condition: Condition::curvature_range(Some(0.32), None)
                    .and(Condition::exposed_face(face_mask::ANY, false)),
                material: Material::channel_ramp(
                    Channel::Field(ridge_noise.clone()),
                    0.2,
                    0.9,
                    vec![
                        BlockState::new("minecraft:tuff"),
                        BlockState::new("minecraft:andesite"),
                        BlockState::new("minecraft:calcite"),
                    ],
                    true,
                )?,
            },
        ],
        // Default bedrock & cliff face: CSE-shared dithered geological strata!
        Some(Material::channel_ramp(
            Channel::Field(strata_field),
            -0.5,
            1.0,
            vec![
                BlockState::new("minecraft:blackstone"),
                BlockState::new("minecraft:deepslate"),
                BlockState::new("minecraft:tuff"),
                BlockState::new("minecraft:stone"),
                BlockState::new("minecraft:andesite"),
            ],
            true,
        )?),
    );

    // Temple shader: weathered oxidized copper patina + polished deepslate/quartz masonry + sea lantern accents
    let temple_material = Material::layered(
        vec![
            // Glowing inlays on the inner floating astrolabe frame
            MaterialLayer {
                condition: Condition::y_range(Some(19.0), Some(25.0))
                    .and(Condition::curvature_range(Some(0.85), None))
                    .and(Condition::chance(0.45, 777)?),
                material: Material::solid("minecraft:sea_lantern"),
            },
            // Oxidized copper patina patches creeping over pillars & rings
            MaterialLayer {
                condition: Condition::field_range(patina_noise.clone(), Some(0.15), None),
                material: Material::channel_ramp(
                    Channel::Field(patina_noise),
                    0.15,
                    0.85,
                    vec![
                        BlockState::new("minecraft:copper_block"),
                        BlockState::new("minecraft:exposed_copper"),
                        BlockState::new("minecraft:weathered_copper"),
                        BlockState::new("minecraft:oxidized_copper"),
                    ],
                    true,
                )?,
            },
            // Flat temple deck & stair treads
            MaterialLayer {
                condition: Condition::slope_range(None, Some(0.22)),
                material: Material::solid("minecraft:quartz_bricks"),
            },
        ],
        Some(Material::channel_ramp(
            Channel::WorldY,
            3.0,
            27.0,
            vec![
                BlockState::new("minecraft:polished_deepslate"),
                BlockState::new("minecraft:deepslate_bricks"),
                BlockState::new("minecraft:prismarine_bricks"),
                BlockState::new("minecraft:smooth_quartz"),
            ],
            true,
        )?),
    );

    // Crystal shader: radiant core -> amethyst -> translucent outer facets driven by exact SDF distance!
    let crystal_material = Material::layered(
        vec![MaterialLayer {
            condition: Condition::distance_range(None, Some(-1.4)),
            material: Material::solid("minecraft:sea_lantern"),
        }],
        Some(Material::channel_ramp(
            Channel::Distance,
            -1.4,
            0.0,
            vec![
                BlockState::new("minecraft:amethyst_block"),
                BlockState::new("minecraft:budding_amethyst"),
                BlockState::new("minecraft:purple_stained_glass"),
            ],
            true,
        )?),
    );

    // Dispatch by CSG tag with deterministic smooth seam blending!
    let material = Material::match_tag(
        vec![
            (TAG_TERRAIN, terrain_material),
            (TAG_TEMPLE, temple_material),
            (TAG_CRYSTAL, crystal_material),
        ],
        Material::solid("minecraft:stone"),
        true,
        909,
    );

    // ── Localized Fluid Layer: Lagoon filling ONLY inside the caldera! ─────
    let lagoon_fluid = Field::capped_cylinder(14.5, 3.0)?.translate([0.0, 2.0, 0.0])?;

    // ── Multi-face Surface Scatter ─────────────────────────────────────────
    let meadow_flora = ScatterRule::new(
        ScatterFace::Top,
        0.18,
        vec![
            (BlockState::new("minecraft:short_grass"), 0.55),
            (BlockState::new("minecraft:poppy"), 0.15),
            (BlockState::new("minecraft:blue_orchid"), 0.15),
            (BlockState::new("minecraft:azalea"), 0.15),
        ],
        404,
    )?
    .on_block("minecraft:grass_block")
    .with_condition(Condition::slope_range(None, Some(0.3)));

    let hanging_vines = ScatterRule::new(
        ScatterFace::Bottom,
        0.09,
        vec![
            (
                BlockState::new("minecraft:cave_vines_plant").with_property("berries", "true"),
                0.40,
            ),
            (BlockState::new("minecraft:cave_vines_plant"), 0.60),
        ],
        505,
    )?
    .with_condition(Condition::y_range(None, Some(-4.0)))
    .with_height_range(2, 7)?;

    let pipeline = VoxelPipeline::new(geometry, material)
        .with_fluid(FluidLayer::new(lagoon_fluid, "minecraft:water"))
        .with_scatter(meadow_flora)
        .with_scatter(hanging_vines);

    let bounds = SampleBounds::new([-42, -36, -42], [42, 34, 42])?;
    Ok((pipeline, bounds))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let t0 = Instant::now();
    let (pipeline, bounds) = build_sanctuary_pipeline()?;
    let compiled = pipeline.compile()?;
    let compile_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let t1 = Instant::now();
    let schem = pipeline.to_schematic(Some(bounds), "sunken_gyroid_sanctuary")?;
    let voxelize_ms = t1.elapsed().as_secs_f64() * 1000.0;

    let total_voxels = (bounds.max[0] - bounds.min[0] + 1) as u64
        * (bounds.max[1] - bounds.min[1] + 1) as u64
        * (bounds.max[2] - bounds.min[2] + 1) as u64;

    // Count top non-air block types for a quick summary
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for (_, b) in schem.iter_blocks() {
        let name = b.get_name();
        if name != "minecraft:air" {
            *counts.entry(name.to_string()).or_default() += 1;
        }
    }
    let mut top: Vec<_> = counts.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1));

    let out_schem = "render_work/sdf_sanctuary.litematic";
    std::fs::create_dir_all("render_work")?;
    let bytes = nucleation::litematic::to_litematic(&schem)?;
    std::fs::write(out_schem, &bytes)?;

    let json = pipeline.geometry.to_json()?;
    println!("=== The Sunken Gyroid Sanctuary ===");
    println!(
        "Volume:        {}x{}x{} ({} voxel centers)",
        bounds.max[0] - bounds.min[0] + 1,
        bounds.max[1] - bounds.min[1] + 1,
        bounds.max[2] - bounds.min[2] + 1,
        total_voxels
    );
    println!(
        "Compiled DAG:  {} CSE instructions ({:.2} ms)",
        compiled.graph_instruction_count(),
        compile_ms
    );
    println!(
        "Voxelized:     {} blocks ({} unique states) in {:.1} ms",
        schem.total_blocks(),
        top.len(),
        voxelize_ms
    );
    println!("DAG JSON v2:   {} bytes", json.len());
    println!("Saved:         {out_schem}");
    println!("\nTop materials:");
    for (name, count) in top.iter().take(12) {
        println!("  {count:>6}  {name}");
    }

    #[cfg(feature = "rendering")]
    {
        let mut args = std::env::args().skip(1);
        if let Some(pack_path) = args.next() {
            let png_path = args
                .next()
                .unwrap_or_else(|| "render_work/sdf_sanctuary.png".to_string());
            let pack = nucleation::meshing::ResourcePackSource::from_file(&pack_path)?;
            let mut cfg = nucleation::rendering::RenderConfig::isometric();
            cfg.width = 960;
            cfg.height = 720;
            cfg.yaw = 35.0;
            cfg.pitch = 26.0;
            cfg.background = Some([0.06, 0.08, 0.13, 1.0]);
            schem.render_to_file(&pack, &png_path, &cfg)?;
            println!("\nRendered preview: {png_path}");
        }
    }

    Ok(())
}
