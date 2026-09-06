//! Axis-based import and baked directional light. The surface path rasterizes
//! triangles sparsely; it never builds the dense triangle grid/interior field.
use super::material::sample_triangle;
use super::shape::{closest_point_on_triangle, cross, sub};
use super::SurfaceSample;
use super::{MeshModel, MeshShape};
use crate::blockpedia::ExtendedColorData;
use crate::building::{BlockPalette, Shape};
use crate::{BlockState, UniversalSchematic};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

const MAX_VOLUME: u64 = 128 * 1024 * 1024;
const MAX_FILLED_VOLUME: u64 = 16 * 1024 * 1024;
const MAX_SURFACE_BLOCKS: usize = 8 * 1024 * 1024;
const MAX_CANDIDATES: u64 = 200_000_000;

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct VoxelizeOptions {
    pub target_size: f32,
    /// longest, x (width), y (height), or z (depth).
    pub axis: String,
    pub hollow: bool,
    pub lighting: Option<VoxelLight>,
    /// Keep untextured, unlit OBJ imports in the caller's chosen material.
    pub untextured_block: Option<String>,
}
impl Default for VoxelizeOptions {
    fn default() -> Self {
        Self {
            target_size: 64.0,
            axis: "longest".into(),
            hollow: true,
            lighting: None,
            untextured_block: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoxelLight {
    /// Vector from the surface toward the light, in model/world axes.
    pub direction: [f32; 3],
    /// 0 preserves colours; 1 has no ambient fill. No cast shadows.
    pub strength: f32,
}
impl VoxelLight {
    fn validate(&self) -> Result<(), String> {
        let length: f64 = self.direction.iter().map(|v| (*v as f64).powi(2)).sum();
        if !self.direction.iter().all(|v| v.is_finite())
            || length < 1e-12
            || !self.strength.is_finite()
            || !(0.0..=1.0).contains(&self.strength)
        {
            return Err("Choose a finite light direction and a strength from 0 to 1.".into());
        }
        Ok(())
    }
    fn brightness(&self, normal: [f32; 3]) -> f32 {
        let n = normal.map(|v| v as f64);
        let l = self.direction.map(|v| v as f64);
        let nl = n.iter().map(|v| v * v).sum::<f64>().sqrt();
        let ll = l.iter().map(|v| v * v).sum::<f64>().sqrt();
        let lambert = if nl > 1e-12 {
            ((n[0] * l[0] + n[1] * l[1] + n[2] * l[2]) / (nl * ll)).max(0.0)
        } else {
            0.0
        };
        (1.0 - self.strength as f64 * (1.0 - lambert)) as f32
    }
}

#[derive(Debug, Serialize)]
pub struct VoxelizePlan {
    /// Width, height, depth in blocks; model proportions are preserved.
    pub dimensions: [u32; 3],
    pub volume: u64,
    #[serde(skip)]
    scale: f32,
    #[serde(skip)]
    min: [f32; 3],
}

impl MeshModel {
    /// Inspect dimensions and reject unsupported allocations before indexing.
    pub fn voxelize_plan(&self, options: &VoxelizeOptions) -> Result<VoxelizePlan, String> {
        if !options.target_size.is_finite() || !(1.0..=8192.0).contains(&options.target_size) {
            return Err("Target size must be between 1 and 8192 blocks.".into());
        }
        if let Some(light) = &options.lighting {
            light.validate()?;
        }
        if self
            .triangles
            .iter()
            .flat_map(|t| t.positions.iter().flatten())
            .any(|v| !v.is_finite())
        {
            return Err("The model contains non-finite vertex coordinates.".into());
        }
        let (min, max) = self.aabb().ok_or("The model has no triangles.")?;
        let extents = sub(max, min);
        let side = match options.axis.as_str() {
            "longest" => extents.into_iter().fold(0.0_f32, f32::max),
            "x" => extents[0],
            "y" => extents[1],
            "z" => extents[2],
            _ => return Err("Choose longest side, width, height, or depth.".into()),
        };
        if side <= 1e-12 {
            return Err("The model is flat on that axis. Choose another size axis.".into());
        }
        let scale = options.target_size / side;
        let scaled = extents.map(|v| snap(v * scale));
        if scaled.iter().any(|v| !v.is_finite() || *v > 8192.0) {
            return Err("A calculated dimension exceeds 8192 blocks. Reduce the target size or choose another axis.".into());
        }
        let dimensions = scaled.map(|v| v.ceil().max(1.0) as u32);
        let volume = dimensions.iter().map(|v| *v as u64).product();
        let limit = if options.hollow {
            MAX_VOLUME
        } else {
            MAX_FILLED_VOLUME
        };
        if volume > limit {
            return Err(format!(
                "This produces {} × {} × {} blocks. {}",
                dimensions[0],
                dimensions[1],
                dimensions[2],
                if options.hollow {
                    "Reduce the target size: the output exceeds the 128-million-cell working limit."
                } else {
                    "Enable Hollow or reduce the size: filled models have a 16-million-cell working limit."
                }
            ));
        }
        Ok(VoxelizePlan {
            dimensions,
            volume,
            scale,
            min,
        })
    }

    /// Uniform fit anchored at (0,0,0), with no origin padding. Texture/light
    /// colours are palette-matched before writing the schematic.
    pub fn voxelize_with_options(
        &self,
        options: &VoxelizeOptions,
        palette: &BlockPalette,
        name: &str,
    ) -> Result<UniversalSchematic, String> {
        let plan = self.voxelize_plan(options)?;
        let mut model = self.clone();
        for tri in &mut model.triangles {
            for p in &mut tri.positions {
                for a in 0..3 {
                    p[a] = snap((p[a] - plan.min[a]) * plan.scale);
                }
            }
        }
        let d = plan.dimensions;
        // Finish the guarded sparse pass before allocating the output volume.
        let hits = if options.hollow {
            Some(surface_hits(&model, d)?)
        } else {
            None
        };
        let mut out = UniversalSchematic::new(name.into());
        out.default_region = crate::region::Region::try_new(
            "Main".into(),
            (0, 0, 0),
            (d[0] as i32, d[1] as i32, d[2] as i32),
        )?;
        let palettes = [palette.for_material(false), palette.for_material(true)];
        let mut memo: FxHashMap<([u8; 3], bool), BlockState> = FxHashMap::default();
        let lighting = options
            .lighting
            .as_ref()
            .filter(|light| light.strength > 0.0);
        let untextured = options.untextured_block.as_ref().map(BlockState::new);
        let mut put = |x, y, z, sample: SurfaceSample, normal| -> Result<(), String> {
            if !sample.visible {
                return Ok(());
            }
            if !sample.textured && lighting.is_none() {
                if let Some(state) = &untextured {
                    out.set_block(x, y, z, state);
                    return Ok(());
                }
            }
            // Directional shading describes reflected light, not glass tint.
            let brightness = if sample.translucent {
                1.0
            } else {
                lighting.map_or(1.0, |l| l.brightness(normal))
            };
            let rgb = sample.rgb(brightness);
            let matching = &palettes[sample.translucent as usize];
            if matching.is_empty() {
                return Err(if sample.translucent {
                    "The model contains glass, but the palette has no glass blocks. Choose Solid + glass or add a glass block."
                } else {
                    "The model contains opaque surfaces, but the palette has no opaque blocks. Add a solid block."
                }.into());
            }
            let state = memo.entry((rgb, sample.translucent)).or_insert_with(|| {
                BlockState::new(
                    matching
                        .find_closest(&ExtendedColorData::from_rgb(rgb[0], rgb[1], rgb[2]))
                        .expect("nonempty material palette"),
                )
            });
            out.set_block(x, y, z, state);
            Ok(())
        };
        if let Some(hits) = hits {
            // Hash iteration order must not change palette ordering or exports.
            let mut hits: Vec<_> = hits.into_iter().collect();
            hits.sort_unstable_by_key(|(key, _)| *key);
            for (key, (_, ti)) in hits {
                let x = (key % d[0] as u64) as i32;
                let z = (key / d[0] as u64 % d[2] as u64) as i32;
                let y = (key / (d[0] as u64 * d[2] as u64)) as i32;
                let tri = &model.triangles[ti as usize];
                let p = [x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5];
                let normal = cross(
                    sub(tri.positions[1], tri.positions[0]),
                    sub(tri.positions[2], tri.positions[0]),
                );
                put(x, y, z, sample_triangle(&model.materials, tri, p), normal)?;
            }
        } else {
            let shape = MeshShape::new(model);
            let mut error = None;
            shape.for_each_point(|x, y, z| {
                if error.is_some() {
                    return;
                }
                let n = shape.normal_at(x, y, z);
                if let Err(e) = put(
                    x,
                    y,
                    z,
                    shape.surface_sample(x, y, z).unwrap_or_default(),
                    [n.0 as f32, n.1 as f32, n.2 as f32],
                ) {
                    error = Some(e);
                }
            });
            if let Some(e) = error {
                return Err(e);
            }
        }
        Ok(out)
    }
}

// Keep floating-point noise at exact voxel boundaries from adding a layer.
fn snap(v: f32) -> f32 {
    if (v - v.round()).abs() <= (4.0 * f32::EPSILON * v.abs().max(1.0)).max(1e-4) {
        v.round()
    } else {
        v
    }
}

/// Project onto each triangle's dominant plane. Only test a narrow band along
/// the remaining axis, then use exact point/triangle distance. This bounds work
/// by projected surface area rather than a sloping triangle's 3D bounding box.
fn surface_hits(model: &MeshModel, dims: [u32; 3]) -> Result<FxHashMap<u64, (f32, u32)>, String> {
    let mut hits: FxHashMap<u64, (f32, u32)> = FxHashMap::default();
    let mut candidates = 0_u64;
    for (ti, tri) in model.triangles.iter().enumerate() {
        let t = &tri.positions;
        let n = cross(sub(t[1], t[0]), sub(t[2], t[0]));
        let a = (0..3)
            .max_by(|&a, &b| n[a].abs().total_cmp(&n[b].abs()))
            .unwrap();
        if n[a].abs() < 1e-12 {
            continue;
        }
        let b = (a + 1) % 3;
        let c = (a + 2) % 3;
        let low = |axis: usize| {
            ((t.iter().map(|p| p[axis]).fold(f32::INFINITY, f32::min) - 1.5).ceil() as i32).max(0)
        };
        let high = |axis: usize| {
            ((t.iter().map(|p| p[axis]).fold(f32::NEG_INFINITY, f32::max) + 0.5).floor() as i32)
                .min(dims[axis] as i32 - 1)
        };
        candidates += (high(b) - low(b) + 1).max(0) as u64 * (high(c) - low(c) + 1).max(0) as u64;
        if candidates > MAX_CANDIDATES {
            return Err("Surface is too complex at this size. Reduce the target size.".into());
        }
        let radius = (n.iter().map(|v| v * v).sum::<f32>()).sqrt() / n[a].abs();
        for ib in low(b)..=high(b) {
            for ic in low(c)..=high(c) {
                let mut p = [0.0; 3];
                p[b] = ib as f32 + 0.5;
                p[c] = ic as f32 + 0.5;
                let plane = t[0][a] - (n[b] * (p[b] - t[0][b]) + n[c] * (p[c] - t[0][c])) / n[a];
                let lo = ((plane - radius - 0.5).ceil() as i32).max(low(a));
                let hi = ((plane + radius - 0.5).floor() as i32).min(high(a));
                for ia in lo..=hi {
                    candidates += 1;
                    if candidates > MAX_CANDIDATES {
                        return Err(
                            "Surface is too complex at this size. Reduce the target size.".into(),
                        );
                    }
                    p[a] = ia as f32 + 0.5;
                    let delta = sub(p, closest_point_on_triangle(p, t));
                    let dist = delta.iter().map(|v| v * v).sum::<f32>();
                    if dist > 1.0 {
                        continue;
                    }
                    // Reject holes before competing for a voxel, so a masked
                    // foreground cannot hide a visible surface behind it.
                    let masked = tri
                        .material
                        .and_then(|i| model.materials.get(i as usize))
                        .and_then(Option::as_ref)
                        .is_some_and(|m| m.alpha_mode != super::AlphaMode::Opaque);
                    if masked && !sample_triangle(&model.materials, tri, p).visible {
                        continue;
                    }
                    let mut xyz = [0; 3];
                    xyz[a] = ia;
                    xyz[b] = ib;
                    xyz[c] = ic;
                    let key = (xyz[1] as u64 * dims[2] as u64 + xyz[2] as u64) * dims[0] as u64
                        + xyz[0] as u64;
                    match hits.entry(key) {
                        std::collections::hash_map::Entry::Vacant(e) => {
                            e.insert((dist, ti as u32));
                        }
                        std::collections::hash_map::Entry::Occupied(mut e) => {
                            if dist < e.get().0 {
                                e.insert((dist, ti as u32));
                            }
                        }
                    }
                    if hits.len() > MAX_SURFACE_BLOCKS {
                        return Err(
                            "The surface exceeds 8 million blocks. Reduce the target size.".into(),
                        );
                    }
                }
            }
        }
    }
    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxelize::MeshTriangle;
    fn cube() -> MeshModel {
        MeshModel::from_obj_str("v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nv 0 0 1\nv 1 0 1\nv 1 1 1\nv 0 1 1\nf 1 3 2\nf 1 4 3\nf 5 6 7\nf 5 7 8\nf 1 5 8\nf 1 8 4\nf 2 3 7\nf 2 7 6\nf 1 2 6\nf 1 6 5\nf 4 8 7\nf 4 7 3").unwrap()
    }
    fn box_model(extents: [f32; 3]) -> MeshModel {
        let mut model = cube();
        for t in &mut model.triangles {
            for p in &mut t.positions {
                for a in 0..3 {
                    p[a] *= extents[a];
                }
            }
        }
        model
    }
    fn opts(size: f32, axis: &str) -> VoxelizeOptions {
        VoxelizeOptions {
            target_size: size,
            axis: axis.into(),
            untextured_block: Some("minecraft:stone".into()),
            ..Default::default()
        }
    }
    #[test]
    fn sizing_preserves_proportions_for_each_axis() {
        let model = box_model([2.0, 1.0, 3.0]);
        for (axis, dimensions) in [
            ("x", [24, 12, 36]),
            ("y", [48, 24, 72]),
            ("z", [16, 8, 24]),
            ("longest", [16, 8, 24]),
        ] {
            assert_eq!(
                model.voxelize_plan(&opts(24.0, axis)).unwrap().dimensions,
                dimensions
            );
        }
        assert_eq!(
            cube().voxelize_plan(&opts(19.0, "x")).unwrap().dimensions,
            [19, 19, 19]
        );
    }
    #[test]
    fn height_384_is_real_output_not_just_an_estimate() {
        let model = box_model([1.0, 16.0, 4.0]);
        let options = opts(384.0, "y");
        let out = model
            .voxelize_with_options(&options, &BlockPalette::new_wool(), "scan")
            .unwrap();
        assert_eq!(out.default_region.size, (24, 384, 96));
        assert!(out
            .get_block(12, 383, 48)
            .is_some_and(|b| b.name == "minecraft:stone"));
        assert!(out
            .get_block(12, 192, 48)
            .is_none_or(|b| b.name == "minecraft:air"));
    }
    #[test]
    fn sparse_shell_matches_dense_distance_test_including_sloped_triangles() {
        let mut model = box_model([12.0, 9.0, 15.0]);
        // A sloping open triangle tests all projection and edge-distance cases.
        model.triangles.push(MeshTriangle {
            positions: [[1.0, 2.0, 1.0], [11.0, 8.0, 12.0], [2.0, 7.0, 14.0]],
            uvs: None,
            emissive_uvs: None,
            transmission_uvs: None,
            colors: None,
            material: None,
        });
        let hits = surface_hits(&model, [12, 9, 15]).unwrap();
        let dense = MeshShape::new(model).with_surface_shell(1.0);
        for x in 0..12 {
            for y in 0..9 {
                for z in 0..15 {
                    assert_eq!(
                        hits.contains_key(&((y * 15 + z) * 12 + x)),
                        dense.contains(x as i32, y as i32, z as i32),
                        "{x},{y},{z}"
                    );
                }
            }
        }
    }
    #[test]
    fn light_direction_changes_the_exported_palette_blocks() {
        let model = cube();
        let mut options = opts(12.0, "longest");
        options.lighting = Some(VoxelLight {
            direction: [0.0, 1.0, 0.0],
            strength: 0.8,
        });
        let palette = BlockPalette::new_grayscale();
        let up = model
            .voxelize_with_options(&options, &palette, "up")
            .unwrap();
        options.lighting.as_mut().unwrap().direction = [0.0, -1.0, 0.0];
        let down = model
            .voxelize_with_options(&options, &palette, "down")
            .unwrap();
        let name =
            |out: &UniversalSchematic, x, y, z| out.get_block(x, y, z).unwrap().name.to_string();
        assert_ne!(name(&up, 6, 11, 6), name(&up, 6, 0, 6));
        assert_eq!(name(&up, 6, 11, 6), name(&down, 6, 0, 6));
        assert_eq!(name(&up, 6, 0, 6), name(&down, 6, 11, 6));
    }
    #[test]
    fn invalid_or_oversized_requests_fail_before_allocating() {
        let flat = box_model([1.0, 0.0, 1.0]);
        assert!(flat
            .voxelize_plan(&opts(384.0, "y"))
            .unwrap_err()
            .contains("flat"));
        assert!(cube()
            .voxelize_plan(&opts(2048.0, "longest"))
            .unwrap_err()
            .contains("2048 × 2048 × 2048"));
        let mut options = opts(384.0, "y");
        options.hollow = false;
        assert!(cube()
            .voxelize_plan(&options)
            .unwrap_err()
            .contains("Enable Hollow"));
        options = opts(12.0, "nope");
        assert!(cube().voxelize_plan(&options).is_err());
        options = opts(f32::NAN, "x");
        assert!(cube().voxelize_plan(&options).is_err());
        options = opts(12.0, "x");
        options.lighting = Some(VoxelLight {
            direction: [0.0; 3],
            strength: 0.5,
        });
        assert!(cube().voxelize_plan(&options).is_err());
    }
    #[test]
    fn hollow_skips_interior_while_filled_keeps_it() {
        let mut options = opts(8.0, "longest");
        let palette = BlockPalette::new_wool();
        let hollow = cube()
            .voxelize_with_options(&options, &palette, "hollow")
            .unwrap();
        options.hollow = false;
        let filled = cube()
            .voxelize_with_options(&options, &palette, "filled")
            .unwrap();
        assert_eq!(hollow.default_region.count_blocks(), 296);
        assert_eq!(filled.default_region.count_blocks(), 512);
    }
}

#[cfg(test)]
mod texture_tests {
    use super::*;
    #[test]
    fn transformed_glb_retains_texture_and_zero_strength_is_identity() {
        let model = MeshModel::from_glb_bytes(include_bytes!(
            "../../tests/fixtures/voxelize-transformed-scan.glb"
        ))
        .unwrap();
        let mut options = VoxelizeOptions {
            target_size: 48.0,
            axis: "y".into(),
            ..Default::default()
        };
        let palette = BlockPalette::new_solid();
        let plain = model
            .voxelize_with_options(&options, &palette, "plain")
            .unwrap();
        assert_eq!(plain.default_region.size, (12, 48, 3));
        options.lighting = Some(VoxelLight {
            direction: [0.0, 1.0, 0.0],
            strength: 0.0,
        });
        let zero = model
            .voxelize_with_options(&options, &palette, "zero")
            .unwrap();
        assert_eq!(plain.default_region.blocks, zero.default_region.blocks);
        options.lighting.as_mut().unwrap().strength = 0.8;
        let lit = model
            .voxelize_with_options(&options, &palette, "lit")
            .unwrap();
        assert_ne!(plain.get_block(6, 0, 1), lit.get_block(6, 0, 1));
        assert_eq!(plain.get_block(6, 47, 1), lit.get_block(6, 47, 1));
    }
}
