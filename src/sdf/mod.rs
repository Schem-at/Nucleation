//! SDF (signed distance function) shape and terrain generation.
//!
//! A typed AST of distance-field primitives, boolean/smooth operators,
//! transforms, and seeded noise modifiers ([`SdfNode`]), plus a sampler that
//! rasterizes a tree into a [`crate::UniversalSchematic`] with declarative
//! [`MaterialRules`] (depth-based shells, absolute Y bands, noise gates,
//! palette-driven gradient fills, and surface scatter).
//!
//! Bindings expose owning, immutable `Sdf` graphs with typed constructors and
//! combinators. JSON import/export and the old JSON terrain sampler remain
//! available for persistence and compatibility.
//!
//! ```
//! use nucleation::sdf::{SdfNode, MaterialRules, sample_to_schematic};
//!
//! let island = SdfNode::Displace {
//!     child: Box::new(SdfNode::Ellipsoid { radii: [26.0, 16.0, 26.0] }),
//!     amplitude: 3.0,
//!     frequency: 0.08,
//!     seed: 42,
//!     octaves: 3,
//! };
//! let rules = MaterialRules::from_json(r#"{
//!   "fill": [
//!     {"when": {"depthBelowSurface": {"min": 0, "max": 0}}, "block": "minecraft:grass_block"},
//!     {"when": {"depthBelowSurface": {"min": 1, "max": 3}}, "block": "minecraft:dirt"},
//!     {"gradient": {"palette": "grayscale", "from": [60, 60, 65], "to": [140, 140, 140],
//!                   "axis": "y", "range": [34, 64]}}
//!   ]
//! }"#).unwrap();
//! let schematic = sample_to_schematic(&island, &rules, None, "island").unwrap();
//! assert!(schematic.total_blocks() > 0);
//! ```

mod node;
pub mod noise;
mod sampler;

pub use node::{Aabb, Axis, CellMode, SdfNode};
pub use sampler::{
    auto_bounds, sample_to_schematic, FillRule, GradientAxis, GradientFill, MaterialRules,
    NoiseCondition, PaletteSpec, RampMode, Range, SampleBounds, SurfaceRule, When,
};

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

/// Numerically estimate a normalized SDF gradient using finite representable
/// neighbors. Unlike fixed `point +/- epsilon`, this still moves at large
/// `f32` coordinates where the requested epsilon is smaller than one ULP.
pub(crate) fn numerical_normal(node: &SdfNode, point: [f32; 3], epsilon: f32) -> Option<[f64; 3]> {
    if !epsilon.is_finite() || epsilon <= 0.0 || point.iter().any(|v| !v.is_finite()) {
        return None;
    }

    let mut gradient = [0.0_f64; 3];
    for axis in 0..3 {
        let center = point[axis];
        let requested_plus = center + epsilon;
        let plus = if requested_plus.is_finite() && requested_plus > center {
            Some(requested_plus)
        } else {
            next_up(center)
        };
        let requested_minus = center - epsilon;
        let minus = if requested_minus.is_finite() && requested_minus < center {
            Some(requested_minus)
        } else {
            next_down(center)
        };

        let sample = |coordinate: f32| {
            let mut p = point;
            p[axis] = coordinate;
            let value = node.eval(p[0], p[1], p[2]);
            value.is_finite().then_some(f64::from(value))
        };

        gradient[axis] = match (minus, plus) {
            (Some(lo), Some(hi)) => (sample(hi)? - sample(lo)?) / (f64::from(hi) - f64::from(lo)),
            (Some(lo), None) => {
                (sample(center)? - sample(lo)?) / (f64::from(center) - f64::from(lo))
            }
            (None, Some(hi)) => {
                (sample(hi)? - sample(center)?) / (f64::from(hi) - f64::from(center))
            }
            (None, None) => return None,
        };
    }

    let length = gradient[0].hypot(gradient[1]).hypot(gradient[2]);
    if !length.is_finite() || length <= f64::EPSILON {
        return None;
    }
    Some([
        gradient[0] / length,
        gradient[1] / length,
        gradient[2] / length,
    ])
}

fn next_up(value: f32) -> Option<f32> {
    if value == f32::INFINITY {
        return None;
    }
    if value == -0.0 {
        return Some(f32::from_bits(1));
    }
    let bits = value.to_bits();
    let next = f32::from_bits(if value >= 0.0 { bits + 1 } else { bits - 1 });
    next.is_finite().then_some(next)
}

fn next_down(value: f32) -> Option<f32> {
    if value == f32::NEG_INFINITY {
        return None;
    }
    if value == 0.0 {
        return Some(f32::from_bits(0x8000_0001));
    }
    let bits = value.to_bits();
    let next = f32::from_bits(if value > 0.0 { bits - 1 } else { bits + 1 });
    next.is_finite().then_some(next)
}

#[cfg(test)]
mod tests;
