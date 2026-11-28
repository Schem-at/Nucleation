//! `nucleation sdf` — evaluate an SDF expression or `FieldGraph` JSON into a
//! schematic file (or pipe directly to stdout with `-o - --to schem`).
//!
//! Examples:
//! ```text
//! nucleation sdf --expr "sphere(p, 10)" --block minecraft:stone_bricks -o orb.schem
//! nucleation sdf --expr "max(abs(sin(x*0.3)*cos(y*0.3)+sin(y*0.3)*cos(z*0.3)+sin(z*0.3)*cos(x*0.3))-0.28, sphere(p, 14))" \
//!     --bounds -16,-16,-16,16,16,16 --palette minecraft:obsidian,minecraft:amethyst_block,minecraft:sea_lantern -o - --to schem \
//!     | nucleation mesh - -o gyroid.glb
//! ```

use std::path::PathBuf;
use std::sync::Arc;

use nucleation::building::{BlockPalette, GradientStop, InterpolationSpace};
use nucleation::sdf::{Aabb, Channel, DistanceKind, Field, Material, SampleBounds, VoxelPipeline};
use nucleation::BlockState;

use crate::usage_and_exit;

fn parse_bounds(spec: &str) -> Result<([i32; 3], [i32; 3]), String> {
    let nums: Vec<i32> = spec
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<i32>()
                .map_err(|e| format!("invalid integer in --bounds '{spec}': {e}"))
        })
        .collect::<Result<_, _>>()?;
    if nums.len() != 6 {
        return Err(format!(
            "--bounds expects 6 comma-separated ints minx,miny,minz,maxx,maxy,maxz (got {})",
            nums.len()
        ));
    }
    Ok(([nums[0], nums[1], nums[2]], [nums[3], nums[4], nums[5]]))
}

pub(crate) fn sdf_main(args: impl Iterator<Item = String>) {
    let mut expr: Option<String> = None;
    let mut expr_file: Option<PathBuf> = None;
    let mut graph_file: Option<PathBuf> = None;
    let mut bounds_spec: Option<String> = None;
    let mut block = "minecraft:stone_bricks".to_string();
    let mut palette_spec: Option<String> = None;
    let mut output: Option<PathBuf> = None;
    let mut to: Option<String> = None;
    let mut name = "sdf_build".to_string();

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--expr" | "-e" => expr = Some(args.next().unwrap_or_else(|| usage_and_exit())),
            "--expr-file" => {
                expr_file = Some(PathBuf::from(
                    args.next().unwrap_or_else(|| usage_and_exit()),
                ))
            }
            "--graph" | "-g" => {
                graph_file = Some(PathBuf::from(
                    args.next().unwrap_or_else(|| usage_and_exit()),
                ))
            }
            "--bounds" | "-b" => {
                bounds_spec = Some(args.next().unwrap_or_else(|| usage_and_exit()))
            }
            "--block" => block = args.next().unwrap_or_else(|| usage_and_exit()),
            "--palette" => palette_spec = Some(args.next().unwrap_or_else(|| usage_and_exit())),
            "--name" => name = args.next().unwrap_or_else(|| usage_and_exit()),
            "-o" | "--out" => {
                output = Some(PathBuf::from(
                    args.next().unwrap_or_else(|| usage_and_exit()),
                ))
            }
            "--to" => to = Some(args.next().unwrap_or_else(|| usage_and_exit())),
            other if other.starts_with("--") => usage_and_exit(),
            other => {
                if expr.is_none() && graph_file.is_none() && expr_file.is_none() {
                    expr = Some(other.to_string());
                } else {
                    usage_and_exit();
                }
            }
        }
    }

    let output = output.unwrap_or_else(|| PathBuf::from("sdf.schem"));
    let piping_out = super::io::is_stdio(&output);
    let format_override = to
        .map(|f| if f == "schem" { "schematic".to_string() } else { f })
        .or_else(|| piping_out.then(|| "schematic".to_string()));

    let result = (|| -> Result<(usize, usize), String> {
        let parsed_bounds = match &bounds_spec {
            Some(s) => Some(parse_bounds(s)?),
            None => None,
        };
        let aabb = parsed_bounds
            .map(|(lo, hi)| {
                Aabb::new(
                    [lo[0] as f32, lo[1] as f32, lo[2] as f32],
                    [hi[0] as f32, hi[1] as f32, hi[2] as f32],
                )
            })
            .transpose()
            .map_err(|e| format!("invalid bounds: {e}"))?;

        let field = if let Some(path) = &graph_file {
            let text = std::fs::read_to_string(path)
                .map_err(|e| format!("reading {}: {e}", path.display()))?;
            Field::from_json(&text).map_err(|e| format!("parsing field graph: {e}"))?
        } else {
            let source = if let Some(path) = &expr_file {
                std::fs::read_to_string(path)
                    .map_err(|e| format!("reading {}: {e}", path.display()))?
            } else if let Some(src) = &expr {
                src.clone()
            } else {
                return Err(
                    "pass --expr <formula>, --expr-file <path>, or --graph <field.json>"
                        .to_string(),
                );
            };
            let default_aabb = aabb
                .or_else(|| Aabb::new([-16.0, -16.0, -16.0], [16.0, 16.0, 16.0]).ok());
            Field::expr_with(&source, &[], default_aabb, DistanceKind::Implicit)
                .map_err(|e| format!("compiling expression: {e}"))?
        };

        let sample_bounds = parsed_bounds
            .map(|(lo, hi)| SampleBounds::new(lo, hi))
            .transpose()
            .map_err(|e| format!("invalid sample bounds: {e}"))?;

        let material = if let Some(pal_str) = &palette_spec {
            let ids: Vec<String> = pal_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if ids.is_empty() {
                return Err("--palette requires at least one block id".to_string());
            }
            if ids.len() == 1 {
                Material::solid(&ids[0])
            } else {
                let (lo_y, hi_y) = sample_bounds
                    .map(|b| (b.min[1] as f32, b.max[1] as f32))
                    .unwrap_or((-16.0, 16.0));
                let blocks: Vec<BlockState> = ids.iter().map(|s| BlockState::new(s)).collect();
                Material::channel_ramp(Channel::WorldY, lo_y, hi_y, blocks, true)
                    .map_err(|e| format!("building palette ramp: {e}"))?
            }
        } else if block == "oklab" {
            let pal = Arc::new(BlockPalette::new_concrete().dithered());
            let (lo_y, hi_y) = sample_bounds
                .map(|b| (b.min[1] as f32, b.max[1] as f32))
                .unwrap_or((-16.0, 16.0));
            Material::channel_gradient(
                Channel::WorldY,
                vec![
                    GradientStop {
                        position: 0.0,
                        color: nucleation::blockpedia::ExtendedColorData::from_rgb(36, 48, 148),
                    },
                    GradientStop {
                        position: 0.5,
                        color: nucleation::blockpedia::ExtendedColorData::from_rgb(56, 178, 164),
                    },
                    GradientStop {
                        position: 1.0,
                        color: nucleation::blockpedia::ExtendedColorData::from_rgb(244, 208, 64),
                    },
                ],
                lo_y,
                hi_y,
                pal,
                InterpolationSpace::Oklab,
            )
            .map_err(|e| format!("building oklab material: {e}"))?
        } else {
            Material::solid(&block)
        };

        let schem = VoxelPipeline::new(field, material)
            .to_schematic(sample_bounds, &name)
            .map_err(|e| format!("voxelizing SDF: {e}"))?;
        let block_count = schem.total_blocks().max(0) as usize;

        let manager = nucleation::formats::manager::get_manager();
        let manager = manager.lock().map_err(|e| format!("format manager: {e}"))?;
        let written = match &format_override {
            Some(fmt) => manager
                .write(fmt, &schem, None)
                .map_err(|e| format!("writing as {fmt}: {e:?}"))?,
            None => manager
                .write_auto(&output.display().to_string(), &schem, None)
                .map_err(|e| format!("writing {}: {e:?}", output.display()))?,
        };
        let bytes_len = written.len();
        super::io::write_output(&output, &written)?;
        Ok((block_count, bytes_len))
    })();

    match result {
        Ok((blocks, bytes)) => super::io::status(
            &format!(
                "sdf -> {} ({blocks} blocks, {bytes} bytes)",
                super::io::display_out(&output)
            ),
            piping_out,
        ),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    }
}
