//! `nucleation worldgen` — stream an infinite SDF `VoxelPipeline` world directly
//! into playable Java Anvil `.mca` region files via `SdfChunkSource` + `WorldSink`.
//!
//! Examples:
//! ```text
//! nucleation worldgen --preset archipelago --chunks -2,-2,2,2 -o ./my-world
//! nucleation worldgen --preset karst --chunks -1,-1,1,1 --y-range -16,32 -o ./caves-world
//! nucleation worldgen --expr "y - sin(x*0.08)*8.0 - cos(z*0.08)*8.0" --chunks 0,0,1,1 -o ./sine-world
//! ```

use std::path::PathBuf;
use std::sync::Arc;

use nucleation::formats::world_stream::WorldSink;
use nucleation::sdf::{
    face_mask, Channel, Condition, DistanceKind, Field, FluidLayer, Material, MaterialLayer,
    ScatterFace, ScatterRule, VoxelPipeline,
};
use nucleation::world_generation::{
    ChunkBounds, ChunkSource, GeneratedChunkStream, SdfChunkSource, SourceProvenance,
};
use nucleation::BlockState;

use crate::usage_and_exit;

fn parse_chunk_range(spec: &str) -> Result<(i32, i32, i32, i32), String> {
    let nums: Vec<i32> = spec
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<i32>()
                .map_err(|e| format!("invalid chunk coord in --chunks '{spec}': {e}"))
        })
        .collect::<Result<_, _>>()?;
    if nums.len() != 4 {
        return Err(format!(
            "--chunks expects min_cx,min_cz,max_cx,max_cz (got {})",
            nums.len()
        ));
    }
    if nums[0] > nums[2] || nums[1] > nums[3] {
        return Err("--chunks min must be <= max".to_string());
    }
    Ok((nums[0], nums[1], nums[2], nums[3]))
}

fn parse_y_range(spec: &str) -> Result<(i32, i32), String> {
    let nums: Vec<i32> = spec
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<i32>()
                .map_err(|e| format!("invalid integer in --y-range '{spec}': {e}"))
        })
        .collect::<Result<_, _>>()?;
    if nums.len() != 2 || nums[0] >= nums[1] {
        return Err("--y-range expects min_y,max_y with min_y < max_y".to_string());
    }
    Ok((nums[0], nums[1]))
}

fn build_preset_pipeline(preset: &str) -> Result<VoxelPipeline, String> {
    let rock_ramp = || -> Result<Material, String> {
        Material::channel_ramp(
            Channel::WorldY,
            -16.0,
            24.0,
            vec![
                BlockState::new("minecraft:deepslate"),
                BlockState::new("minecraft:tuff"),
                BlockState::new("minecraft:stone"),
                BlockState::new("minecraft:andesite"),
            ],
            true,
        )
        .map_err(|e| format!("rock ramp: {e}"))
    };

    match preset {
        "archipelago" => {
            let hills = Field::perlin_fbm2(0.025, 42, 4)
                .and_then(|f| f.mul_const(14.0))
                .map_err(|e| e.to_string())?;
            let peaks = Field::ridged_fbm3(0.018, 99, 3)
                .and_then(|f| f.mul_const(10.0))
                .map_err(|e| e.to_string())?;
            let terrain = Field::heightmap(2.0, hills.add(&peaks)).map_err(|e| e.to_string())?;
            let mat = Material::layered(
                vec![
                    MaterialLayer {
                        condition: Condition::exposed_face(face_mask::TOP, true)
                            .and(Condition::y_range(Some(18.0), None))
                            .and(Condition::slope_range(Some(0.0), Some(0.70))),
                        material: Material::solid("minecraft:snow_block"),
                    },
                    MaterialLayer {
                        condition: Condition::exposed_face(face_mask::TOP, true)
                            .and(Condition::y_range(Some(4.0), Some(19.0)))
                            .and(Condition::slope_range(Some(0.0), Some(0.68))),
                        material: Material::solid("minecraft:grass_block"),
                    },
                    MaterialLayer {
                        condition: Condition::depth_below_surface(Some(1), Some(3))
                            .and(Condition::y_range(Some(3.0), Some(18.0))),
                        material: Material::solid("minecraft:dirt"),
                    },
                ],
                Some(rock_ramp()?),
            );
            let mut pipe = VoxelPipeline::new(terrain, mat).with_fluid(
                FluidLayer::level(5.0, "minecraft:water").map_err(|e| e.to_string())?,
            );
            let flora = ScatterRule::new(
                ScatterFace::Top,
                0.22,
                vec![
                    (BlockState::new("minecraft:short_grass"), 0.72),
                    (BlockState::new("minecraft:poppy"), 0.14),
                    (BlockState::new("minecraft:dandelion"), 0.14),
                ],
                17,
            )
            .map_err(|e| e.to_string())?
            .on_block("minecraft:grass_block");
            pipe = pipe.with_scatter(flora);
            Ok(pipe)
        }
        "karst" => {
            let hills = Field::perlin_fbm2(0.03, 17, 3)
                .and_then(|f| f.mul_const(8.0))
                .map_err(|e| e.to_string())?;
            let surface = Field::heightmap(10.0, hills).map_err(|e| e.to_string())?;
            let caves = Field::expr(
                "let q = p * 0.16; abs(sin(q.x)*cos(q.y) + sin(q.y)*cos(q.z) + sin(q.z)*cos(q.x)) - 0.34",
            )
            .map_err(|e| e.to_string())?;
            let carved = surface.subtract(&caves);
            let mat = Material::layered(
                vec![
                    MaterialLayer {
                        condition: Condition::exposed_face(face_mask::TOP, true)
                            .and(Condition::y_range(Some(4.0), None))
                            .and(Condition::slope_range(Some(0.0), Some(0.62))),
                        material: Material::solid("minecraft:grass_block"),
                    },
                    MaterialLayer {
                        condition: Condition::exposed_face(face_mask::BOTTOM, true)
                            .and(Condition::chance(0.06, 71).map_err(|e| e.to_string())?),
                        material: Material::solid("minecraft:glowstone"),
                    },
                ],
                Some(rock_ramp()?),
            );
            Ok(VoxelPipeline::new(carved, mat).with_fluid(
                FluidLayer::level(-4.0, "minecraft:water").map_err(|e| e.to_string())?,
            ))
        }
        other => Err(format!(
            "unknown worldgen preset '{other}' (expected 'archipelago' or 'karst')"
        )),
    }
}

pub(crate) fn worldgen_main(args: impl Iterator<Item = String>) {
    let mut preset = "archipelago".to_string();
    let mut expr: Option<String> = None;
    let mut chunks_spec = "-1,-1,1,1".to_string();
    let mut y_spec = "-16,32".to_string();
    let mut out_dir: Option<PathBuf> = None;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--preset" => preset = args.next().unwrap_or_else(|| usage_and_exit()),
            "--expr" | "-e" => expr = Some(args.next().unwrap_or_else(|| usage_and_exit())),
            "--chunks" => chunks_spec = args.next().unwrap_or_else(|| usage_and_exit()),
            "--y-range" => y_spec = args.next().unwrap_or_else(|| usage_and_exit()),
            "-o" | "--out" => {
                out_dir = Some(PathBuf::from(
                    args.next().unwrap_or_else(|| usage_and_exit()),
                ))
            }
            _ => usage_and_exit(),
        }
    }

    let Some(out_dir) = out_dir else {
        eprintln!("worldgen requires -o <world_dir>");
        std::process::exit(2);
    };

    let result = (|| -> Result<usize, String> {
        let (min_cx, min_cz, max_cx, max_cz) = parse_chunk_range(&chunks_spec)?;
        let (min_y, max_y) = parse_y_range(&y_spec)?;

        let pipeline = if let Some(source) = &expr {
            let field = Field::expr_with(source, &[], None, DistanceKind::Implicit)
                .map_err(|e| format!("compiling expression: {e}"))?;
            let mat = Material::layered(
                vec![
                    MaterialLayer {
                        condition: Condition::exposed_face(face_mask::TOP, true)
                            .and(Condition::slope_range(Some(0.0), Some(0.65))),
                        material: Material::solid("minecraft:grass_block"),
                    },
                    MaterialLayer {
                        condition: Condition::depth_below_surface(Some(1), Some(2)),
                        material: Material::solid("minecraft:dirt"),
                    },
                ],
                Some(Material::solid("minecraft:stone")),
            );
            VoxelPipeline::new(field, mat)
        } else {
            build_preset_pipeline(&preset)?
        };

        let provenance = SourceProvenance::new(format!("cli:worldgen:{preset}"), "v1")
            .map_err(|e| format!("provenance: {e}"))?;
        let source = SdfChunkSource::from_pipeline(pipeline, min_y, max_y, provenance)
            .map_err(|e| format!("creating SdfChunkSource: {e}"))?;
        let bounds = ChunkBounds::new(min_cx, min_cz, max_cx, max_cz)
            .map_err(|e| format!("chunk bounds: {e}"))?;
        let stream = GeneratedChunkStream::new(Arc::new(source) as Arc<dyn ChunkSource>, bounds);

        let mut sink = WorldSink::create(&out_dir, None)
            .map_err(|e| format!("creating Anvil WorldSink at {}: {e}", out_dir.display()))?;

        let mut chunks_written = 0usize;
        for item in stream {
            let chunk = item.map_err(|e| format!("generating chunk: {e}"))?;
            let view = chunk.into_chunk();
            sink.write_chunk(&view)
                .map_err(|e| format!("writing chunk: {e}"))?;
            chunks_written += 1;
        }
        sink.finish().map_err(|e| format!("flushing world: {e}"))?;
        Ok(chunks_written)
    })();

    match result {
        Ok(chunks) => {
            eprintln!("worldgen -> {} ({chunks} chunks)", out_dir.display());
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    }
}
