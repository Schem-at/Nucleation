//! SDF (signed distance function) shape and terrain generation.
//!
//! A JSON AST of distance-field primitives, boolean/smooth operators,
//! transforms, and seeded noise modifiers ([`SdfNode`]), plus a sampler that
//! rasterizes a tree into a [`crate::UniversalSchematic`] with declarative
//! [`MaterialRules`] (depth-based shells, absolute Y bands, noise gates, and
//! surface scatter).
//!
//! Every binding (FFI, Python, WASM, JVM) exposes the same two entry points:
//! `from_sdf(sdf_json, rules_json[, bounds])` and `sdf_eval(sdf_json, x, y, z)`,
//! so shape definitions are portable across all consumers, and identical
//! inputs always yield identical schematics.
//!
//! ```
//! use nucleation::sdf::{SdfNode, MaterialRules, sample_to_schematic};
//!
//! let island: SdfNode = serde_json::from_str(r#"{
//!   "type": "smoothUnion", "k": 4.0,
//!   "a": {"type": "superPrism", "halfExtents": [32, 2, 32], "exponent": 6},
//!   "b": {"type": "displace", "amplitude": 3.0, "frequency": 0.08, "seed": 42,
//!          "child": {"type": "translate", "offset": [0, -14, 0],
//!                    "child": {"type": "ellipsoid", "radii": [26, 16, 26]}}}
//! }"#).unwrap();
//! let rules = MaterialRules::from_json(r#"{
//!   "fill": [
//!     {"when": {"depthBelowSurface": {"min": 0, "max": 0}}, "block": "minecraft:grass_block"},
//!     {"when": {"depthBelowSurface": {"min": 1, "max": 3}}, "block": "minecraft:dirt"},
//!     {"block": "minecraft:stone"}
//!   ]
//! }"#).unwrap();
//! let schematic = sample_to_schematic(&island, &rules, None, "island").unwrap();
//! assert!(schematic.total_blocks() > 0);
//! ```

mod node;
pub mod noise;
mod sampler;

pub use node::{Aabb, Axis, SdfNode};
pub use sampler::{
    auto_bounds, sample_to_schematic, FillRule, MaterialRules, NoiseCondition, Range, SampleBounds,
    SurfaceRule, When,
};

#[cfg(test)]
mod tests;
