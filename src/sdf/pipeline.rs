//! Unified, brick-pruned, parallel voxelization pipeline.
//!
//! One engine powers:
//! - Bounded schematic creation (`VoxelPipeline::to_schematic`)
//! - In-place schematic editing (`VoxelPipeline::apply_to_schematic`)
//! - Infinite random-access chunk generation (`VoxelPipeline::generate_chunk`)
//!
//! All geometry, material fields, condition fields, fluid layers, and scatter
//! rules compile into ONE single CSE-deduplicated [`CompiledGraph`]. Empty
//! `8×8×8` bricks are pruned via conservative interval arithmetic, and on
//! native targets large volumes execute in parallel across column tiles with
//! 100% deterministic output identical to single-threaded WASM.

use super::compile::{CompiledField, CompiledGraph, TaggedSample};
use super::field::Field;
use super::interval::Interval3;
use super::material::{
    eval_condition, eval_material, face_mask, ChannelRegistry, FluidLayer, Material, ScatterRule,
    VoxelContext,
};
use super::noise::hash01_3;
use crate::building::FillMode;
use crate::{BlockState, UniversalSchematic};

#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

const BRICK_SIZE: i32 = 8;
#[cfg(not(target_arch = "wasm32"))]
const PARALLEL_TILE_THRESHOLD: usize = 64;

/// Inclusive integer sampling bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampleBounds {
    pub min: [i32; 3],
    pub max: [i32; 3],
}

impl SampleBounds {
    pub fn new(min: [i32; 3], max: [i32; 3]) -> Result<Self, String> {
        super::checked_sample_volume(min, max)?;
        Ok(Self { min, max })
    }
}

/// Derive inclusive integer sampling bounds from a field's conservative AABB.
pub fn auto_bounds(field: &Field) -> Result<SampleBounds, String> {
    let b = field.bounds().ok_or_else(|| {
        "field is unbounded (e.g. plane, noise, or uncounted repeat) — explicit bounds required"
            .to_string()
    })?;
    let min = [
        (b.min[0] - 1.0).floor() as i32,
        (b.min[1] - 1.0).floor() as i32,
        (b.min[2] - 1.0).floor() as i32,
    ];
    let max = [
        (b.max[0] + 1.0).ceil() as i32,
        (b.max[1] + 1.0).ceil() as i32,
        (b.max[2] + 1.0).ceil() as i32,
    ];
    SampleBounds::new(min, max)
}

/// Complete, immutable, reusable procedural voxelization recipe.
#[derive(Clone)]
pub struct VoxelPipeline {
    pub geometry: Field,
    pub material: Material,
    pub fluids: Vec<FluidLayer>,
    pub scatter: Vec<ScatterRule>,
    pub fill_mode: FillMode,
}

impl VoxelPipeline {
    pub fn new(geometry: Field, material: Material) -> Self {
        Self {
            geometry,
            material,
            fluids: Vec::new(),
            scatter: Vec::new(),
            fill_mode: FillMode::Replace,
        }
    }

    pub fn with_fluid(mut self, fluid: FluidLayer) -> Self {
        self.fluids.push(fluid);
        self
    }

    pub fn with_scatter(mut self, rule: ScatterRule) -> Self {
        self.scatter.push(rule);
        self
    }

    pub fn with_fill_mode(mut self, mode: FillMode) -> Self {
        self.fill_mode = mode;
        self
    }

    /// Compile the entire pipeline (geometry + all material/condition/fluid/scatter
    /// fields) into one CSE-deduplicated [`CompiledPipeline`].
    pub fn compile(&self) -> Result<CompiledPipeline, String> {
        self.geometry.validate().map_err(|e| e.to_string())?;
        for f in &self.fluids {
            f.field.validate().map_err(|e| e.to_string())?;
        }
        let mut reg = ChannelRegistry::default();
        reg.scan_material(&self.material);
        for rule in &self.scatter {
            reg.scan_condition(&rule.condition);
            reg.needs_exposed = true;
        }
        let mut fluid_root_indices = Vec::with_capacity(self.fluids.len());
        for fluid in &self.fluids {
            let aux_idx = reg.register_field(&fluid.field);
            fluid_root_indices.push(1 + aux_idx);
        }

        let mut all_roots = Vec::with_capacity(1 + reg.aux_fields.len());
        all_roots.push(self.geometry.clone());
        all_roots.extend(reg.aux_fields.iter().cloned());

        let graph = CompiledGraph::compile(&all_roots);
        if graph.has_tags() {
            reg.needs_tags = true;
        }
        let geometry_compiled = CompiledField::compile(&self.geometry);

        Ok(CompiledPipeline {
            graph,
            geometry_compiled,
            registry: reg,
            material: self.material.clone(),
            fluids: self.fluids.clone(),
            fluid_root_indices,
            scatter: self.scatter.clone(),
            fill_mode: self.fill_mode.clone(),
        })
    }

    /// Rasterize into a brand-new [`UniversalSchematic`].
    pub fn to_schematic(
        &self,
        bounds: Option<SampleBounds>,
        name: &str,
    ) -> Result<UniversalSchematic, String> {
        let bounds = match bounds {
            Some(b) => {
                super::checked_sample_volume(b.min, b.max)?;
                b
            }
            None => auto_bounds(&self.geometry)?,
        };
        let compiled = self.compile()?;
        let mut schematic = UniversalSchematic::new(name.to_string());
        compiled.apply_to_schematic(&mut schematic, bounds);
        Ok(schematic)
    }

    /// Rasterize onto an existing [`UniversalSchematic`] respecting `self.fill_mode`.
    pub fn apply_to_schematic(
        &self,
        schematic: &mut UniversalSchematic,
        bounds: Option<SampleBounds>,
    ) -> Result<(), String> {
        let bounds = match bounds {
            Some(b) => {
                super::checked_sample_volume(b.min, b.max)?;
                b
            }
            None => auto_bounds(&self.geometry)?,
        };
        let compiled = self.compile()?;
        compiled.apply_to_schematic(schematic, bounds);
        Ok(())
    }
}

/// Precompiled pipeline ready to rasterize many schematics or infinite chunks
/// with zero recompilation overhead.
#[derive(Clone)]
pub struct CompiledPipeline {
    graph: CompiledGraph,
    geometry_compiled: CompiledField,
    registry: ChannelRegistry,
    material: Material,
    fluids: Vec<FluidLayer>,
    fluid_root_indices: Vec<usize>,
    scatter: Vec<ScatterRule>,
    fill_mode: FillMode,
}

impl CompiledPipeline {
    pub fn graph_instruction_count(&self) -> usize {
        self.graph.instruction_count()
    }

    /// Rasterize a bounded 3D box into `schematic`, using brick-level interval
    /// pruning and (on native) parallel column-tile execution.
    pub fn apply_to_schematic(&self, schematic: &mut UniversalSchematic, bounds: SampleBounds) {
        let (x0, y0, z0) = (bounds.min[0], bounds.min[1], bounds.min[2]);
        let (x1, y1, z1) = (bounds.max[0], bounds.max[1], bounds.max[2]);
        schematic.ensure_bounds((x0, y0, z0), (x1, y1, z1));

        let mut tiles = Vec::new();
        let mut tx = x0;
        while tx <= x1 {
            let tx_end = (tx + BRICK_SIZE - 1).min(x1);
            let mut tz = z0;
            while tz <= z1 {
                let tz_end = (tz + BRICK_SIZE - 1).min(z1);
                tiles.push((tx, tx_end, tz, tz_end));
                tz = tz_end + 1;
            }
            tx = tx_end + 1;
        }

        let results: Vec<TileOutput> = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if tiles.len() >= PARALLEL_TILE_THRESHOLD {
                    tiles
                        .par_iter()
                        .map(|&(ax, bx, az, bz)| self.eval_column_tile(ax, bx, az, bz, y0, y1))
                        .collect()
                } else {
                    tiles
                        .iter()
                        .map(|&(ax, bx, az, bz)| self.eval_column_tile(ax, bx, az, bz, y0, y1))
                        .collect()
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                tiles
                    .iter()
                    .map(|&(ax, bx, az, bz)| self.eval_column_tile(ax, bx, az, bz, y0, y1))
                    .collect()
            }
        };

        // Pass 1: primary solid + fluid blocks
        for tile in &results {
            for (x, y, z, block) in &tile.blocks {
                if self.fill_mode.allows(schematic.get_block(*x, *y, *z)) {
                    schematic.set_block(*x, *y, *z, block);
                }
            }
        }
        // Pass 2: scatter decorations (only overwrite air/empty cells so solid geometry always wins)
        for tile in &results {
            for (x, y, z, block) in &tile.scatter_blocks {
                let existing = schematic.get_block(*x, *y, *z);
                let is_air = existing.is_none_or(|b| {
                    matches!(
                        b.get_name(),
                        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                    )
                });
                if is_air {
                    schematic.set_block(*x, *y, *z, block);
                }
            }
        }
    }

    /// Generate all non-air blocks for a 16×16 chunk column `(cx, cz)` over
    /// `min_y..=max_y`. Used directly by [`crate::world_generation::SdfChunkSource`].
    pub fn generate_chunk_blocks(
        &self,
        x0: i32,
        z0: i32,
        min_y: i32,
        max_y: i32,
    ) -> Vec<(i32, i32, i32, BlockState)> {
        let mut out = Vec::new();
        let mut scatter = Vec::new();
        for dx in [0, 8] {
            for dz in [0, 8] {
                let tile = self.eval_column_tile(
                    x0 + dx,
                    x0 + dx + 7,
                    z0 + dz,
                    z0 + dz + 7,
                    min_y,
                    max_y,
                );
                out.extend(tile.blocks);
                scatter.extend(tile.scatter_blocks);
            }
        }
        // Append scatter inside the chunk's horizontal and Y footprint; caller
        // merges air-only semantics cleanly when writing to sections.
        for (x, y, z, b) in scatter {
            if x >= x0 && x < x0 + 16 && z >= z0 && z < z0 + 16 && y >= min_y && y <= max_y {
                out.push((x, y, z, b));
            }
        }
        out
    }

    /// Evaluate one vertical column tile `[tx0..=tx1] × [y0..=y1] × [tz0..=tz1]`,
    /// pruning empty `8×8×8` sub-bricks via interval arithmetic!
    fn eval_column_tile(
        &self,
        tx0: i32,
        tx1: i32,
        tz0: i32,
        tz1: i32,
        y0: i32,
        y1: i32,
    ) -> TileOutput {
        let height = (y1 - y0 + 1) as usize;
        let tw = (tx1 - tx0 + 1) as usize;
        let td = (tz1 - tz0 + 1) as usize;
        let mut out = TileOutput::default();

        // Check vertical 8-block slabs with interval arithmetic to prune empty Y ranges.
        let num_slabs = (y1 - y0 + BRICK_SIZE) / BRICK_SIZE;
        let mut slab_has_solid = vec![false; num_slabs as usize];
        let mut slab_has_fluid = vec![false; num_slabs as usize];
        let mut any_solid = false;
        let mut any_fluid = false;

        for s in 0..num_slabs {
            let sy0 = y0 + s * BRICK_SIZE;
            let sy1 = (sy0 + BRICK_SIZE - 1).min(y1);
            let box3 = Interval3::from_box(
                [tx0 as f32 + 0.5, sy0 as f32 + 0.5, tz0 as f32 + 0.5],
                [tx1 as f32 + 0.5, sy1 as f32 + 0.5, tz1 as f32 + 0.5],
            );
            let geom_iv = self.graph.eval_interval(0, box3);
            if !geom_iv.strictly_positive() {
                slab_has_solid[s as usize] = true;
                any_solid = true;
            }
            for &fluid_root in &self.fluid_root_indices {
                let f_iv = self.graph.eval_interval(fluid_root, box3);
                if !f_iv.strictly_positive() {
                    slab_has_fluid[s as usize] = true;
                    any_fluid = true;
                    break;
                }
            }
        }

        if !any_solid && !any_fluid {
            return out;
        }

        let aux_len = self.registry.aux_fields.len();
        let num_roots = 1 + aux_len;
        let mut slab_f32 = vec![0.0f32; 8 * num_roots];
        let mut slab_tagged = [TaggedSample::plain(0.0); 8];
        let mut slab_aux = vec![0.0f32; 8 * aux_len];
        let mut depth_slab_f32 = [0.0f32; 8];

        // Per-column depth buffer (only used when depth_below_surface is requested)
        let mut col_solid = if self.registry.needs_depth {
            vec![false; height]
        } else {
            Vec::new()
        };
        let mut col_depth = if self.registry.needs_depth {
            vec![0i32; height]
        } else {
            Vec::new()
        };

        // Precompute exact column depth runs when needed
        for dx in 0..tw {
            let x = tx0 + dx as i32;
            let fx = x as f32 + 0.5;
            for dz in 0..td {
                let z = tz0 + dz as i32;
                let fz = z as f32 + 0.5;

                if self.registry.needs_depth {
                    for s in 0..num_slabs {
                        let sy0 = y0 + s * BRICK_SIZE;
                        let sy1 = (sy0 + BRICK_SIZE - 1).min(y1);
                        let slab_len = (sy1 - sy0 + 1) as usize;
                        if !slab_has_solid[s as usize] {
                            for k in 0..slab_len {
                                col_solid[(sy0 + k as i32 - y0) as usize] = false;
                            }
                        } else {
                            self.geometry_compiled.graph_eval_slab_f32(
                                fx,
                                sy0,
                                slab_len,
                                fz,
                                &mut depth_slab_f32[..slab_len],
                            );
                            for k in 0..slab_len {
                                col_solid[(sy0 + k as i32 - y0) as usize] =
                                    depth_slab_f32[k] <= 0.0;
                            }
                        }
                    }
                    // Walk top -> bottom to assign run depth
                    let mut depth = 0i32;
                    for yi in (0..height).rev() {
                        if col_solid[yi] {
                            col_depth[yi] = depth;
                            depth += 1;
                        } else {
                            col_depth[yi] = 0;
                            depth = 0;
                        }
                    }
                }

                for s in 0..num_slabs {
                    if !slab_has_solid[s as usize] && !slab_has_fluid[s as usize] {
                        continue;
                    }
                    let sy0 = y0 + s * BRICK_SIZE;
                    let sy1 = (sy0 + BRICK_SIZE - 1).min(y1);
                    let slab_len = (sy1 - sy0 + 1) as usize;

                    if self.registry.needs_tags {
                        self.graph.eval_slab_tagged(
                            fx,
                            sy0,
                            slab_len,
                            fz,
                            &mut slab_tagged[..slab_len],
                            &mut slab_aux,
                        );
                    } else {
                        self.graph
                            .eval_slab_f32(fx, sy0, slab_len, fz, &mut slab_f32);
                    }

                    for k in 0..slab_len {
                        let y = sy0 + k as i32;
                        let fy = y as f32 + 0.5;
                        let (sample, aux_slice): (TaggedSample, &[f32]) =
                            if self.registry.needs_tags {
                                let base = k * aux_len;
                                (slab_tagged[k], &slab_aux[base..base + aux_len])
                            } else {
                                let base = k * num_roots;
                                (
                                    TaggedSample::plain(slab_f32[base]),
                                    &slab_f32[base + 1..base + num_roots],
                                )
                            };

                        if sample.distance > 0.0 {
                            // Air voxel: check fluid layers in order
                            if slab_has_fluid[s as usize] {
                                for (i, fluid) in self.fluids.iter().enumerate() {
                                    let aux_idx = self.fluid_root_indices[i] - 1;
                                    if aux_slice[aux_idx] <= 0.0 {
                                        out.blocks.push((x, y, z, fluid.block.clone()));
                                        break;
                                    }
                                }
                            }
                            continue;
                        }

                        // Solid voxel! Compute lazy channels only as demanded.
                        let normal = if self.registry.needs_normal {
                            self.geometry_compiled
                                .normal(fx, fy, fz, 0.5)
                                .unwrap_or([0.0, 1.0, 0.0])
                        } else {
                            [0.0, 1.0, 0.0]
                        };

                        let curvature = if self.registry.needs_curvature {
                            self.geometry_compiled.curvature(fx, fy, fz, 1.0)
                        } else {
                            0.0
                        };

                        let exposed_faces = if self.registry.needs_exposed {
                            self.compute_exposed_faces(fx, fy, fz)
                        } else {
                            0
                        };

                        let depth_below_surface = if self.registry.needs_depth {
                            col_depth[(y - y0) as usize]
                        } else {
                            0
                        };

                        let ctx = VoxelContext {
                            x,
                            y,
                            z,
                            distance: sample.distance,
                            normal,
                            curvature,
                            depth_below_surface,
                            exposed_faces,
                            material_id: sample.material_id,
                            blend_material_id: sample.blend_material_id,
                            blend_weight: sample.blend_weight,
                            aux_fields: aux_slice,
                        };

                        let placed_block = eval_material(&self.material, &ctx, &self.registry);
                        if let Some(ref block) = placed_block {
                            out.blocks.push((x, y, z, block.clone()));
                        }

                        // Surface scatter rules on exposed faces of this solid voxel
                        if !self.scatter.is_empty() && exposed_faces != 0 {
                            for rule in &self.scatter {
                                if (exposed_faces & rule.face.mask()) == 0 || rule.density <= 0.0 {
                                    continue;
                                }
                                if let Some(ref on) = rule.on_block {
                                    match &placed_block {
                                        Some(b) if b.get_name().starts_with(on.as_str()) => {}
                                        _ => continue,
                                    }
                                }
                                if !eval_condition(&rule.condition, &ctx, &self.registry) {
                                    continue;
                                }
                                if hash01_3(x, y, z, rule.seed) < rule.density {
                                    let block = rule.pick_block(x, y, z).clone();
                                    let dir = rule.face.offset();
                                    let h = rule.pick_height(x, y, z);
                                    for step in 1..=h as i32 {
                                        let Some(sx) = x.checked_add(dir[0] * step) else {
                                            break;
                                        };
                                        let Some(sy) = y.checked_add(dir[1] * step) else {
                                            break;
                                        };
                                        let Some(sz) = z.checked_add(dir[2] * step) else {
                                            break;
                                        };
                                        // Stop multi-block scatter if it hits another solid surface
                                        if step > 1
                                            && self.geometry_compiled.eval(
                                                sx as f32 + 0.5,
                                                sy as f32 + 0.5,
                                                sz as f32 + 0.5,
                                            ) <= 0.0
                                        {
                                            break;
                                        }
                                        out.scatter_blocks.push((sx, sy, sz, block.clone()));
                                    }
                                    break; // first matching scatter rule wins per exposed voxel
                                }
                            }
                        }
                    }
                }
            }
        }

        out
    }

    #[inline]
    fn compute_exposed_faces(&self, fx: f32, fy: f32, fz: f32) -> u8 {
        let mut mask = 0u8;
        if self.geometry_compiled.eval(fx, fy + 1.0, fz) > 0.0 {
            mask |= face_mask::TOP;
        }
        if self.geometry_compiled.eval(fx, fy - 1.0, fz) > 0.0 {
            mask |= face_mask::BOTTOM;
        }
        if self.geometry_compiled.eval(fx, fy, fz - 1.0) > 0.0 {
            mask |= face_mask::NORTH;
        }
        if self.geometry_compiled.eval(fx, fy, fz + 1.0) > 0.0 {
            mask |= face_mask::SOUTH;
        }
        if self.geometry_compiled.eval(fx + 1.0, fy, fz) > 0.0 {
            mask |= face_mask::EAST;
        }
        if self.geometry_compiled.eval(fx - 1.0, fy, fz) > 0.0 {
            mask |= face_mask::WEST;
        }
        mask
    }
}

#[derive(Default)]
struct TileOutput {
    blocks: Vec<(i32, i32, i32, BlockState)>,
    scatter_blocks: Vec<(i32, i32, i32, BlockState)>,
}
