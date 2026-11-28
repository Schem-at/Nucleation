//! Power-8 Mandelbulb SDF + Orbit-Trap Coloring showcase.
//!
//! Built entirely with the reworked Nucleation SDF pipeline:
//! - Sandboxed infix expression with bounded `for` loop (`Field::expr_with`)
//! - Second expression computing a true 3D Mandelbulb orbit-trap scalar field
//! - Layered `Material` shader combining orbit-trap palette ramp, radial
//!   gradient, `Curvature` edge highlights, and glowing interior fissures
//! - Zero manual voxel loops — compiled & rasterized in parallel via `VoxelPipeline`
//!
//! Run (generates schematic):
//!   cargo run --release --example sdf_mandelbulb
//!
//! Run with GPU render preview:
//!   cargo run --release --example sdf_mandelbulb --features rendering -- render_work/pack.zip render_work/sdf_mandelbulb.png

use nucleation::blockpedia::ExtendedColorData;
use nucleation::building::{BlockPalette, GradientStop, InterpolationSpace};
use nucleation::sdf::{
    Aabb, Channel, Condition, DistanceKind, Field, Material, MaterialLayer, VoxelPipeline,
};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

const SCALE: f32 = 34.0;

/// Classic power-8 Mandelbulb distance estimator written directly in Nucleation's
/// infix expression language (compiled to a statically bounded bytecode VM).
const MANDELBULB_SDF: &str = r#"
    if length(p) > 1.35 {
        length(p) - 1.25
    } else {
        let z = p;
        let dr = 1.0;
        let r = 0.0;
        for i in 0..10 {
            r = length(z);
            if r > 2.0 { break; }
            let r_safe = max(r, 0.0001);
            let theta = acos(clamp(z.y / r_safe, -1.0, 1.0)) * 8.0;
            let phi = atan2(z.z, z.x) * 8.0;
            let zr = pow(r_safe, 8.0);
            dr = pow(r_safe, 7.0) * 8.0 * dr + 1.0;
            z = vec3(
                sin(theta) * cos(phi),
                cos(theta),
                sin(theta) * sin(phi)
            ) * zr + p;
        }
        let de = if r <= 0.0001 { -0.5 } else { 0.5 * ln(r) * r / dr };
        if r < 2.0 { min(de, -0.001) } else { de - 0.014 }
    }
"#;

/// Orbit-trap scalar field over the same Mandelbulb iteration: tracks the closest
/// approach of the orbit `z` to the origin and coordinate planes, giving deep
/// crevices vs exposed fractal lobes distinct color bands.
const MANDELBULB_ORBIT_TRAP: &str = r#"
    let z = p;
    let trap = 10.0;
    for i in 0..8 {
        let r = length(z);
        if r > 2.0 { break; }
        trap = min(trap, r);
        let r_safe = max(r, 0.0001);
        let theta = acos(clamp(z.y / r_safe, -1.0, 1.0)) * 8.0;
        let phi = atan2(z.z, z.x) * 8.0;
        let zr = pow(r_safe, 8.0);
        z = vec3(
            sin(theta) * cos(phi),
            cos(theta),
            sin(theta) * sin(phi)
        ) * zr + p;
    }
    clamp(trap, 0.0, 1.2)
"#;

fn build_mandelbulb_pipeline() -> Result<VoxelPipeline, Box<dyn std::error::Error>> {
    let unit_bounds = Aabb::new([-1.28, -1.28, -1.28], [1.28, 1.28, 1.28])?;

    // Scale both the SDF and the orbit-trap field into voxel space
    let bulb = Field::expr_with(
        MANDELBULB_SDF,
        &[],
        Some(unit_bounds),
        DistanceKind::Estimate,
    )?
    .scale(SCALE)?;

    let orbit_trap = Field::expr(MANDELBULB_ORBIT_TRAP)?.scale(SCALE)?;
    // Note: `scale(S)` multiplies both domain by `1/S` and output distance by `S`,
    // so divide by `SCALE` to keep the orbit-trap scalar in `[0.0, 1.2]`.
    let trap_normalized = orbit_trap.mul_const(1.0 / SCALE)?;

    // Build a curated palette of rich opaque blocks (dithered in Oklab space!)
    let palette = Arc::new(
        BlockPalette::builder()
            .full_blocks_only()
            .exclude_transparent()
            .exclude_falling()
            .exclude_tile_entities()
            .exclude_light_sources()
            .build()
            .dithered(),
    );

    // Iridescent cosmic ramp: deep obsidian core -> royal purple -> magenta -> teal -> warm gold lobes
    let orbit_gradient = Material::channel_gradient(
        Channel::Field(trap_normalized.clone()),
        vec![
            GradientStop {
                position: 0.00,
                color: ExtendedColorData::from_rgb(20, 14, 34), // deep void crevices
            },
            GradientStop {
                position: 0.28,
                color: ExtendedColorData::from_rgb(76, 36, 130), // royal indigo/amethyst
            },
            GradientStop {
                position: 0.55,
                color: ExtendedColorData::from_rgb(168, 52, 165), // vivid magenta
            },
            GradientStop {
                position: 0.78,
                color: ExtendedColorData::from_rgb(56, 178, 186), // luminous cyan-teal
            },
            GradientStop {
                position: 1.00,
                color: ExtendedColorData::from_rgb(245, 206, 98), // sunlit gold tips
            },
        ],
        0.15,
        1.05,
        palette,
        InterpolationSpace::Oklab,
    )?;

    // Layer glowing accents in deep high-trap pockets + crisp amethyst highlights on sharp ridges
    let material = Material::layered(
        vec![
            // Glowing sea-lantern / shroomlight nodes tucked into tight fractal tips
            MaterialLayer {
                condition: Condition::field_range(trap_normalized.clone(), Some(0.96), None)
                    .and(Condition::curvature_range(Some(0.35), None))
                    .and(Condition::chance(0.35, 42)?),
                material: Material::solid("minecraft:ochre_froglight"),
            },
            // Amethyst crystalline shimmer on high-curvature ridges
            MaterialLayer {
                condition: Condition::curvature_range(Some(0.45), None)
                    .and(Condition::field_range(trap_normalized, Some(0.4), Some(0.85))),
                material: Material::solid("minecraft:amethyst_block"),
            },
        ],
        Some(orbit_gradient),
    );

    Ok(VoxelPipeline::new(bulb, material))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let t0 = Instant::now();
    let pipeline = build_mandelbulb_pipeline()?;
    let compiled = pipeline.compile()?;
    let compile_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let t1 = Instant::now();
    // Auto-inferred bounds directly from `unit_bounds.scale(SCALE)`!
    let schem = pipeline.to_schematic(None, "mandelbulb_power8")?;
    let voxelize_ms = t1.elapsed().as_secs_f64() * 1000.0;

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for (_, b) in schem.iter_blocks() {
        let name = b.get_name();
        if name != "minecraft:air" {
            *counts.entry(name.to_string()).or_default() += 1;
        }
    }
    let mut top: Vec<_> = counts.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1));

    let out_schem = "render_work/sdf_mandelbulb.litematic";
    std::fs::create_dir_all("render_work")?;
    std::fs::write(out_schem, nucleation::litematic::to_litematic(&schem)?)?;

    let json = pipeline.geometry.to_json()?;
    println!("=== Power-8 Mandelbulb SDF ===");
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
    println!("\nTop blocks:");
    for (name, count) in top.iter().take(10) {
        println!("  {count:>6}  {name}");
    }

    #[cfg(feature = "rendering")]
    {
        let mut args = std::env::args().skip(1);
        if let Some(pack_path) = args.next() {
            let png_path = args
                .next()
                .unwrap_or_else(|| "render_work/sdf_mandelbulb.png".to_string());
            let pack = nucleation::meshing::ResourcePackSource::from_file(&pack_path)?;
            let mut cfg = nucleation::rendering::RenderConfig::isometric();
            cfg.width = 960;
            cfg.height = 960;
            cfg.yaw = 38.0;
            cfg.pitch = 24.0;
            cfg.background = Some([0.04, 0.03, 0.07, 1.0]);
            schem.render_to_file(&pack, &png_path, &cfg)?;
            println!("\nRendered preview: {png_path}");
        }
    }

    Ok(())
}
