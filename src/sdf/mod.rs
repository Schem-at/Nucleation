//! Unified SDF, scalar field, expression, and voxelization pipeline.
//!
//! Everything in 3D space is an immutable, reference-counted [`Field`] DAG node:
//! SDF primitives, CSG (hard, smooth, chamfer, stepped), domain transforms and
//! warps, procedural noise (value, Perlin, FBM, ridged, Worley), arbitrary
//! scalar math, multi-material CSG tags, and sandboxed infix expressions all
//! compose freely and compile into a single CSE-deduplicated flat evaluator
//! ([`CompiledField`] / [`CompiledGraph`]) with conservative interval brick
//! pruning and forward-mode dual-number gradients.
//!
//! [`VoxelPipeline`] unifies schematic generation, in-place building fills,
//! and infinite chunk generation behind composable [`Material`] layers,
//! [`Condition`] predicates, [`FluidLayer`]s, and [`ScatterRule`]s.

pub mod compile;
pub mod expr;
pub mod field;
pub mod grid;
pub mod interval;
pub mod material;
pub mod noise;
pub mod pipeline;

pub use compile::{CompiledField, CompiledGraph, TaggedSample};
pub use expr::{Dual3, ExprProgram, ExprType};
pub use field::{
    Aabb, Axis, BinaryMathOp, DistanceKind, Field, FieldError, FieldGraph, NoiseBasis,
    UnaryMathOp, FIELD_GRAPH_VERSION,
};
pub use grid::VoxelDistanceGrid;
pub use interval::{Interval, Interval3};
pub use material::{
    face_mask, Channel, Condition, FluidLayer, Material, MaterialLayer, ScatterFace, ScatterRule,
    VoxelContext,
};
pub use noise::CellMode;
pub use pipeline::{auto_bounds, CompiledPipeline, SampleBounds, VoxelPipeline};

/// Maximum number of voxel centers evaluated by any bounded SDF operation.
pub const MAX_SDF_SAMPLE_VOLUME: u64 = 16_777_216;

/// Validate inclusive integer sampling bounds without signed overflow.
pub fn checked_sample_volume(min: [i32; 3], max: [i32; 3]) -> Result<u64, String> {
    let mut volume = 1_u64;
    for axis in 0..3 {
        let span = i64::from(max[axis])
            .checked_sub(i64::from(min[axis]))
            .and_then(|value| value.checked_add(1))
            .and_then(|value| u64::try_from(value).ok())
            .filter(|span| *span > 0)
            .ok_or_else(|| format!("Invalid SDF sampling bounds on axis {axis}"))?;
        volume = volume
            .checked_mul(span)
            .ok_or_else(|| "SDF sampling volume overflow".to_string())?;
    }
    if volume > MAX_SDF_SAMPLE_VOLUME {
        return Err(format!(
            "SDF sampling volume {volume} exceeds limit {MAX_SDF_SAMPLE_VOLUME}"
        ));
    }
    Ok(volume)
}

#[cfg(test)]
mod tests;
