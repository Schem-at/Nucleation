//! Unified immutable spatial field / SDF DAG (`Field`) and deduplicated
//! multi-root graph (`FieldGraph`).
//!
//! Replaces the old fragmented `SdfNode`, `Field3`, and `FieldProgram` types
//! with a single composable IR where SDF primitives, CSG (hard, smooth,
//! chamfer, stepped), transforms, domain warps, procedural noise (value,
//! Perlin, FBM, ridged, Worley), arbitrary scalar math, material tags, and
//! inline infix expressions all compose freely and share storage via `Arc`.

use super::expr::ExprProgram;
use super::noise::{CellMode, MAX_SAFE_LATTICE_COORD};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::sync::Arc;

pub const FIELD_GRAPH_VERSION: u32 = 2;
pub const MAX_FIELD_JSON_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_TREE_DEPTH: u32 = 64;
pub const MAX_UNIQUE_NODES: usize = 8_192;
pub const MAX_WALK_NODES: u32 = 16_384;
pub const MAX_REPEAT_POINTS: usize = 4_096;
pub const MAX_GRAPH_ROOTS: usize = 256;
pub const MAX_ROOT_NAME_BYTES: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldError {
    InputTooLarge,
    InvalidArgument(String),
    Parse(String),
    BudgetExceeded(String),
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldError::InputTooLarge => f.write_str("field JSON exceeds the size limit"),
            FieldError::InvalidArgument(msg) => write!(f, "invalid field argument: {msg}"),
            FieldError::Parse(msg) => write!(f, "field parse error: {msg}"),
            FieldError::BudgetExceeded(msg) => write!(f, "field budget exceeded: {msg}"),
        }
    }
}

impl std::error::Error for FieldError {}

impl From<FieldError> for String {
    fn from(e: FieldError) -> Self {
        e.to_string()
    }
}

/// Axis selector for mirror operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    X,
    Y,
    Z,
}

/// Distance semantics of a field.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DistanceKind {
    Exact,
    LowerBound,
    #[default]
    Estimate,
    Implicit,
}

/// Axis-aligned bounding box in continuous space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Aabb {
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Result<Self, FieldError> {
        finite_all(&min)?;
        finite_all(&max)?;
        for a in 0..3 {
            if min[a] > max[a] {
                return Err(FieldError::InvalidArgument("AABB min > max".into()));
            }
        }
        Ok(Self { min, max })
    }

    #[inline]
    pub fn grow(self, amount: f32) -> Aabb {
        let a = amount.abs();
        Aabb {
            min: [self.min[0] - a, self.min[1] - a, self.min[2] - a],
            max: [self.max[0] + a, self.max[1] + a, self.max[2] + a],
        }
    }

    #[inline]
    pub fn union(self, other: Aabb) -> Aabb {
        Aabb {
            min: [
                self.min[0].min(other.min[0]),
                self.min[1].min(other.min[1]),
                self.min[2].min(other.min[2]),
            ],
            max: [
                self.max[0].max(other.max[0]),
                self.max[1].max(other.max[1]),
                self.max[2].max(other.max[2]),
            ],
        }
    }

    #[inline]
    pub fn intersection(self, other: Aabb) -> Aabb {
        Aabb {
            min: [
                self.min[0].max(other.min[0]),
                self.min[1].max(other.min[1]),
                self.min[2].max(other.min[2]),
            ],
            max: [
                self.max[0].min(other.max[0]),
                self.max[1].min(other.max[1]),
                self.max[2].min(other.max[2]),
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UnaryMathOp {
    Neg,
    Abs,
    Sqrt,
    Floor,
    Ceil,
    Fract,
    Sin,
    Cos,
    Exp,
    Ln,
    Sign,
    Saturate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BinaryMathOp {
    Add,
    Sub,
    Mul,
    Div,
    Min,
    Max,
    Pow,
    Mod,
    Atan2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoiseBasis {
    Value,
    Perlin,
}

/// Internal DAG node backing [`Field`].
#[derive(Debug, Clone)]
pub(crate) enum FieldNode {
    // ── Coordinates & Constants ────────────────────────────────────────────
    Const(f32),
    Coord(Axis),

    // ── SDF Primitives ─────────────────────────────────────────────────────
    Sphere {
        radius: f32,
    },
    Box {
        half_extents: [f32; 3],
        rounding: f32,
    },
    Ellipsoid {
        radii: [f32; 3],
    },
    Torus {
        major_radius: f32,
        minor_radius: f32,
    },
    CappedTorus {
        major_radius: f32,
        minor_radius: f32,
        cap_angle: f32,
    },
    Link {
        major_radius: f32,
        minor_radius: f32,
        half_length: f32,
    },
    Capsule {
        a: [f32; 3],
        b: [f32; 3],
        radius: f32,
    },
    RoundCone {
        a: [f32; 3],
        b: [f32; 3],
        r1: f32,
        r2: f32,
    },
    SolidAngle {
        radius: f32,
        angle: f32,
    },
    CutSphere {
        radius: f32,
        height: f32,
    },
    CutHollowSphere {
        radius: f32,
        height: f32,
        thickness: f32,
    },
    CappedCylinder {
        radius: f32,
        half_height: f32,
    },
    InfiniteCylinder {
        radius: f32,
    },
    CappedCone {
        half_height: f32,
        r1: f32,
        r2: f32,
    },
    Plane {
        normal: [f32; 3],
        offset: f32,
    },
    Octahedron {
        size: f32,
    },
    HexPrism {
        radius: f32,
        half_height: f32,
    },
    SuperPrism {
        half_extents: [f32; 3],
        exponent: f32,
    },
    BoxFrame {
        half_extents: [f32; 3],
        thickness: f32,
    },
    InfiniteCone {
        angle: f32,
    },
    SquarePyramid {
        half_base: f32,
        height: f32,
    },
    /// Terrain heightmap surface: `y - (base_y + height(x, 0, z))`.
    Heightmap {
        base_y: f32,
        height: Field,
    },

    // ── Procedural Noise & Cellular Fields ─────────────────────────────────
    Noise2 {
        basis: NoiseBasis,
        frequency: f32,
        seed: i32,
    },
    Noise3 {
        basis: NoiseBasis,
        frequency: f32,
        seed: i32,
    },
    Fbm2 {
        basis: NoiseBasis,
        frequency: f32,
        seed: i32,
        octaves: u32,
        lacunarity: f32,
        gain: f32,
        ridged: bool,
    },
    Fbm3 {
        basis: NoiseBasis,
        frequency: f32,
        seed: i32,
        octaves: u32,
        lacunarity: f32,
        gain: f32,
        ridged: bool,
    },
    Cells {
        frequency: f32,
        seed: i32,
        jitter: f32,
        mode: CellMode,
        threshold: f32,
    },

    // ── Scalar Math & Combinators ──────────────────────────────────────────
    Unary {
        op: UnaryMathOp,
        child: Field,
    },
    Binary {
        op: BinaryMathOp,
        a: Field,
        b: Field,
    },
    Clamp {
        child: Field,
        lo: f32,
        hi: f32,
    },
    Smoothstep {
        child: Field,
        edge0: f32,
        edge1: f32,
    },
    Step {
        child: Field,
        edge: f32,
    },
    Remap {
        child: Field,
        in_lo: f32,
        in_hi: f32,
        out_lo: f32,
        out_hi: f32,
        clamp: bool,
    },
    Lerp {
        a: Field,
        b: Field,
        t: Field,
    },
    Select {
        cond: Field,
        then_field: Field,
        else_field: Field,
    },

    // ── CSG & Surface Modifiers ────────────────────────────────────────────
    Union {
        children: Vec<Field>,
    },
    Intersect {
        children: Vec<Field>,
    },
    Subtract {
        a: Field,
        b: Field,
    },
    Xor {
        a: Field,
        b: Field,
    },
    SmoothUnion {
        a: Field,
        b: Field,
        k: f32,
    },
    SmoothSubtract {
        a: Field,
        b: Field,
        k: f32,
    },
    SmoothIntersect {
        a: Field,
        b: Field,
        k: f32,
    },
    ChamferUnion {
        a: Field,
        b: Field,
        r: f32,
    },
    ChamferSubtract {
        a: Field,
        b: Field,
        r: f32,
    },
    ChamferIntersect {
        a: Field,
        b: Field,
        r: f32,
    },
    StepsUnion {
        a: Field,
        b: Field,
        r: f32,
        steps: f32,
    },
    Round {
        child: Field,
        radius: f32,
    },
    Shell {
        child: Field,
        thickness: f32,
    },
    Elongate {
        child: Field,
        half_lengths: [f32; 3],
    },
    /// Offset a surface by any scalar field: `child(p) + amplitude * offset(p)`.
    Displace {
        child: Field,
        offset: Field,
        amplitude: f32,
    },

    // ── Domain Transforms & Space Warps ────────────────────────────────────
    Translate {
        child: Field,
        offset: [f32; 3],
    },
    Rotate {
        child: Field,
        angles: [f32; 3],
    },
    Scale {
        child: Field,
        factor: f32,
    },
    ScaleNonUniform {
        child: Field,
        factors: [f32; 3],
    },
    Mirror {
        child: Field,
        axis: Axis,
    },
    MirrorPlane {
        child: Field,
        normal: [f32; 3],
        offset: f32,
    },
    Repeat {
        child: Field,
        spacing: [f32; 3],
        count: Option<[u32; 3]>,
    },
    RepeatPolar {
        child: Field,
        count: u32,
    },
    RepeatPoints {
        child: Field,
        offsets: Vec<[f32; 3]>,
    },
    Twist {
        child: Field,
        amount: f32,
    },
    Bend {
        child: Field,
        amount: f32,
    },
    /// General domain warp driven by 3 arbitrary fields: `child(p + amp * [wx, wy, wz])`.
    DomainWarp {
        child: Field,
        warp_x: Field,
        warp_y: Field,
        warp_z: Field,
        amplitude: f32,
    },

    // ── Metadata, Tagging & Custom Expressions ─────────────────────────────
    /// Attach a material tag ID (`1..=65535`) to a volume branch. Propagates
    /// through CSG ops and blends across smooth CSG seams!
    Tag {
        child: Field,
        material_id: u16,
    },
    /// Attach explicit conservative finite bounds to an otherwise unbounded field.
    Bounded {
        child: Field,
        bounds: Aabb,
    },
    /// Sandboxed infix expression program with optional bound external `Field` inputs.
    Expr {
        program: Arc<ExprProgram>,
        inputs: Vec<Field>,
        bounds: Option<Aabb>,
        distance_kind: DistanceKind,
    },
}

/// Unified, immutable, reference-counted spatial field / SDF expression graph.
///
/// Cloning is an `Arc` refcount bump, so subexpressions can be freely shared
/// across geometry, displacement, masks, and materials — and the compiler
/// deduplicates shared nodes automatically via CSE.
#[derive(Debug, Clone)]
pub struct Field(pub(crate) Arc<FieldNode>);

impl Field {
    #[inline]
    fn wrap(node: FieldNode) -> Self {
        Self(Arc::new(node))
    }

    /// True when two handles share the exact same underlying DAG node.
    #[inline]
    pub fn shares_storage_with(&self, other: &Field) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    /// Compile this field into a flat, CSE-deduplicated evaluator with interval
    /// pruning and dual-number gradient support.
    pub fn compile(&self) -> super::compile::CompiledField {
        super::compile::CompiledField::compile(self)
    }

    /// Convenience point evaluation (compiles on the fly; prefer `compile()`
    /// when evaluating many points).
    pub fn eval(&self, x: f32, y: f32, z: f32) -> f32 {
        self.compile().eval(x, y, z)
    }

    /// Convenience normal estimation at `(x, y, z)`.
    pub fn normal(&self, x: f32, y: f32, z: f32, epsilon: f32) -> Option<[f32; 3]> {
        self.compile().normal(x, y, z, epsilon)
    }

    // ── Coordinates & Constants ────────────────────────────────────────────

    pub fn constant(value: f32) -> Result<Self, FieldError> {
        finite_all(&[value])?;
        Ok(Self::wrap(FieldNode::Const(value)))
    }

    pub fn x() -> Self {
        Self::wrap(FieldNode::Coord(Axis::X))
    }

    pub fn y() -> Self {
        Self::wrap(FieldNode::Coord(Axis::Y))
    }

    pub fn z() -> Self {
        Self::wrap(FieldNode::Coord(Axis::Z))
    }

    // ── Primitives ─────────────────────────────────────────────────────────

    pub fn sphere(radius: f32) -> Result<Self, FieldError> {
        positive_all(&[radius])?;
        Ok(Self::wrap(FieldNode::Sphere { radius }))
    }

    pub fn box_shape(half_extents: [f32; 3], rounding: f32) -> Result<Self, FieldError> {
        positive_all(&half_extents)?;
        non_negative_all(&[rounding])?;
        if rounding > half_extents[0].min(half_extents[1]).min(half_extents[2]) {
            return Err(FieldError::InvalidArgument(
                "box rounding cannot exceed smallest half-extent".into(),
            ));
        }
        Ok(Self::wrap(FieldNode::Box {
            half_extents,
            rounding,
        }))
    }

    pub fn ellipsoid(radii: [f32; 3]) -> Result<Self, FieldError> {
        positive_all(&radii)?;
        Ok(Self::wrap(FieldNode::Ellipsoid { radii }))
    }

    pub fn torus(major_radius: f32, minor_radius: f32) -> Result<Self, FieldError> {
        positive_all(&[major_radius, minor_radius])?;
        Ok(Self::wrap(FieldNode::Torus {
            major_radius,
            minor_radius,
        }))
    }

    pub fn capped_torus(
        major_radius: f32,
        minor_radius: f32,
        cap_angle: f32,
    ) -> Result<Self, FieldError> {
        positive_all(&[major_radius, minor_radius])?;
        finite_all(&[cap_angle])?;
        if cap_angle <= 0.0 || cap_angle > 180.0 {
            return Err(FieldError::InvalidArgument(
                "capped torus cap_angle must be in (0, 180]".into(),
            ));
        }
        Ok(Self::wrap(FieldNode::CappedTorus {
            major_radius,
            minor_radius,
            cap_angle,
        }))
    }

    pub fn link(major_radius: f32, minor_radius: f32, half_length: f32) -> Result<Self, FieldError> {
        positive_all(&[major_radius, minor_radius])?;
        non_negative_all(&[half_length])?;
        Ok(Self::wrap(FieldNode::Link {
            major_radius,
            minor_radius,
            half_length,
        }))
    }

    pub fn capsule(a: [f32; 3], b: [f32; 3], radius: f32) -> Result<Self, FieldError> {
        finite_all(&a)?;
        finite_all(&b)?;
        positive_all(&[radius])?;
        Ok(Self::wrap(FieldNode::Capsule { a, b, radius }))
    }

    pub fn round_cone(a: [f32; 3], b: [f32; 3], r1: f32, r2: f32) -> Result<Self, FieldError> {
        finite_all(&a)?;
        finite_all(&b)?;
        positive_all(&[r1, r2])?;
        if a == b {
            return Err(FieldError::InvalidArgument(
                "round cone endpoints must not coincide".into(),
            ));
        }
        Ok(Self::wrap(FieldNode::RoundCone { a, b, r1, r2 }))
    }

    pub fn solid_angle(radius: f32, angle: f32) -> Result<Self, FieldError> {
        positive_all(&[radius])?;
        finite_all(&[angle])?;
        if angle <= 0.0 || angle >= 180.0 {
            return Err(FieldError::InvalidArgument(
                "solid angle must be in (0, 180) degrees".into(),
            ));
        }
        Ok(Self::wrap(FieldNode::SolidAngle { radius, angle }))
    }

    pub fn cut_sphere(radius: f32, height: f32) -> Result<Self, FieldError> {
        positive_all(&[radius])?;
        finite_all(&[height])?;
        if height <= -radius || height >= radius {
            return Err(FieldError::InvalidArgument(
                "cut sphere height must be strictly in (-radius, radius)".into(),
            ));
        }
        Ok(Self::wrap(FieldNode::CutSphere { radius, height }))
    }

    pub fn cut_hollow_sphere(radius: f32, height: f32, thickness: f32) -> Result<Self, FieldError> {
        positive_all(&[radius, thickness])?;
        finite_all(&[height])?;
        if height <= -radius || height >= radius {
            return Err(FieldError::InvalidArgument(
                "cut hollow sphere height must be strictly in (-radius, radius)".into(),
            ));
        }
        Ok(Self::wrap(FieldNode::CutHollowSphere {
            radius,
            height,
            thickness,
        }))
    }

    pub fn capped_cylinder(radius: f32, half_height: f32) -> Result<Self, FieldError> {
        positive_all(&[radius, half_height])?;
        Ok(Self::wrap(FieldNode::CappedCylinder {
            radius,
            half_height,
        }))
    }

    pub fn infinite_cylinder(radius: f32) -> Result<Self, FieldError> {
        positive_all(&[radius])?;
        Ok(Self::wrap(FieldNode::InfiniteCylinder { radius }))
    }

    pub fn capped_cone(half_height: f32, r1: f32, r2: f32) -> Result<Self, FieldError> {
        positive_all(&[half_height])?;
        non_negative_all(&[r1, r2])?;
        if r1 == 0.0 && r2 == 0.0 {
            return Err(FieldError::InvalidArgument(
                "capped cone radii cannot both be zero".into(),
            ));
        }
        Ok(Self::wrap(FieldNode::CappedCone {
            half_height,
            r1,
            r2,
        }))
    }

    pub fn plane(normal: [f32; 3], offset: f32) -> Result<Self, FieldError> {
        finite_all(&normal)?;
        finite_all(&[offset])?;
        let n = normalize_f64(normal)?;
        Ok(Self::wrap(FieldNode::Plane { normal: n, offset }))
    }

    pub fn octahedron(size: f32) -> Result<Self, FieldError> {
        positive_all(&[size])?;
        Ok(Self::wrap(FieldNode::Octahedron { size }))
    }

    pub fn hex_prism(radius: f32, half_height: f32) -> Result<Self, FieldError> {
        positive_all(&[radius, half_height])?;
        Ok(Self::wrap(FieldNode::HexPrism {
            radius,
            half_height,
        }))
    }

    pub fn super_prism(half_extents: [f32; 3], exponent: f32) -> Result<Self, FieldError> {
        positive_all(&half_extents)?;
        positive_all(&[exponent])?;
        Ok(Self::wrap(FieldNode::SuperPrism {
            half_extents,
            exponent,
        }))
    }

    pub fn box_frame(half_extents: [f32; 3], thickness: f32) -> Result<Self, FieldError> {
        positive_all(&half_extents)?;
        non_negative_all(&[thickness])?;
        if thickness > half_extents[0].min(half_extents[1]).min(half_extents[2]) {
            return Err(FieldError::InvalidArgument(
                "box frame thickness cannot exceed smallest half-extent".into(),
            ));
        }
        Ok(Self::wrap(FieldNode::BoxFrame {
            half_extents,
            thickness,
        }))
    }

    pub fn infinite_cone(angle: f32) -> Result<Self, FieldError> {
        finite_all(&[angle])?;
        if angle <= 0.0 || angle >= 90.0 {
            return Err(FieldError::InvalidArgument(
                "infinite cone angle must be in (0, 90) degrees".into(),
            ));
        }
        Ok(Self::wrap(FieldNode::InfiniteCone { angle }))
    }

    pub fn square_pyramid(half_base: f32, height: f32) -> Result<Self, FieldError> {
        positive_all(&[half_base, height])?;
        if height < f32::MIN_POSITIVE {
            return Err(FieldError::InvalidArgument(
                "pyramid height too small".into(),
            ));
        }
        Ok(Self::wrap(FieldNode::SquarePyramid { half_base, height }))
    }

    /// Terrain heightmap surface: solid where `y <= base_y + height(x, 0, z)`.
    pub fn heightmap(base_y: f32, height: Field) -> Result<Self, FieldError> {
        finite_all(&[base_y])?;
        Ok(Self::wrap(FieldNode::Heightmap { base_y, height }))
    }

    // ── Noise & Cellular Fields ────────────────────────────────────────────

    pub fn value_noise2(frequency: f32, seed: i32) -> Result<Self, FieldError> {
        validate_frequency(frequency, 1, 2.0)?;
        Ok(Self::wrap(FieldNode::Noise2 {
            basis: NoiseBasis::Value,
            frequency,
            seed,
        }))
    }

    pub fn value_noise3(frequency: f32, seed: i32) -> Result<Self, FieldError> {
        validate_frequency(frequency, 1, 2.0)?;
        Ok(Self::wrap(FieldNode::Noise3 {
            basis: NoiseBasis::Value,
            frequency,
            seed,
        }))
    }

    pub fn perlin2(frequency: f32, seed: i32) -> Result<Self, FieldError> {
        validate_frequency(frequency, 1, 2.0)?;
        Ok(Self::wrap(FieldNode::Noise2 {
            basis: NoiseBasis::Perlin,
            frequency,
            seed,
        }))
    }

    pub fn perlin3(frequency: f32, seed: i32) -> Result<Self, FieldError> {
        validate_frequency(frequency, 1, 2.0)?;
        Ok(Self::wrap(FieldNode::Noise3 {
            basis: NoiseBasis::Perlin,
            frequency,
            seed,
        }))
    }

    pub fn fbm2(
        basis: NoiseBasis,
        frequency: f32,
        seed: i32,
        octaves: u32,
        lacunarity: f32,
        gain: f32,
        ridged: bool,
    ) -> Result<Self, FieldError> {
        validate_fbm(frequency, octaves, lacunarity, gain)?;
        Ok(Self::wrap(FieldNode::Fbm2 {
            basis,
            frequency,
            seed,
            octaves,
            lacunarity,
            gain,
            ridged,
        }))
    }

    pub fn fbm3(
        basis: NoiseBasis,
        frequency: f32,
        seed: i32,
        octaves: u32,
        lacunarity: f32,
        gain: f32,
        ridged: bool,
    ) -> Result<Self, FieldError> {
        validate_fbm(frequency, octaves, lacunarity, gain)?;
        Ok(Self::wrap(FieldNode::Fbm3 {
            basis,
            frequency,
            seed,
            octaves,
            lacunarity,
            gain,
            ridged,
        }))
    }

    /// Shorthand for deterministic 3D value-noise FBM in `[-1, 1]`.
    pub fn value_fbm3(frequency: f32, seed: i32, octaves: u32) -> Result<Self, FieldError> {
        Self::fbm3(NoiseBasis::Value, frequency, seed, octaves, 2.0, 0.5, false)
    }

    /// Shorthand for smooth 3D Perlin FBM in `[-1, 1]`.
    pub fn perlin_fbm3(frequency: f32, seed: i32, octaves: u32) -> Result<Self, FieldError> {
        Self::fbm3(NoiseBasis::Perlin, frequency, seed, octaves, 2.0, 0.5, false)
    }

    /// Shorthand for 3D ridged multifractal Perlin FBM in `[0, 1]`.
    pub fn ridged_fbm3(frequency: f32, seed: i32, octaves: u32) -> Result<Self, FieldError> {
        Self::fbm3(NoiseBasis::Perlin, frequency, seed, octaves, 2.0, 0.5, true)
    }

    /// Shorthand for 2D Perlin FBM over `(x, z)` in `[-1, 1]`.
    pub fn perlin_fbm2(frequency: f32, seed: i32, octaves: u32) -> Result<Self, FieldError> {
        Self::fbm2(NoiseBasis::Perlin, frequency, seed, octaves, 2.0, 0.5, false)
    }

    pub fn cells(
        frequency: f32,
        seed: i32,
        jitter: f32,
        mode: CellMode,
        threshold: f32,
    ) -> Result<Self, FieldError> {
        positive_all(&[frequency])?;
        non_negative_all(&[jitter])?;
        finite_all(&[threshold])?;
        Ok(Self::wrap(FieldNode::Cells {
            frequency,
            seed,
            jitter,
            mode,
            threshold,
        }))
    }

    // ── Scalar Math & Combinators ──────────────────────────────────────────

    pub fn unary(&self, op: UnaryMathOp) -> Self {
        Self::wrap(FieldNode::Unary {
            op,
            child: self.clone(),
        })
    }

    pub fn binary(&self, op: BinaryMathOp, other: &Field) -> Self {
        Self::wrap(FieldNode::Binary {
            op,
            a: self.clone(),
            b: other.clone(),
        })
    }

    pub fn neg(&self) -> Self {
        self.unary(UnaryMathOp::Neg)
    }
    pub fn abs(&self) -> Self {
        self.unary(UnaryMathOp::Abs)
    }
    pub fn sqrt(&self) -> Self {
        self.unary(UnaryMathOp::Sqrt)
    }
    pub fn sin(&self) -> Self {
        self.unary(UnaryMathOp::Sin)
    }
    pub fn cos(&self) -> Self {
        self.unary(UnaryMathOp::Cos)
    }
    pub fn saturate(&self) -> Self {
        self.unary(UnaryMathOp::Saturate)
    }

    pub fn add(&self, other: &Field) -> Self {
        self.binary(BinaryMathOp::Add, other)
    }
    pub fn sub(&self, other: &Field) -> Self {
        self.binary(BinaryMathOp::Sub, other)
    }
    pub fn mul(&self, other: &Field) -> Self {
        self.binary(BinaryMathOp::Mul, other)
    }
    pub fn div(&self, other: &Field) -> Self {
        self.binary(BinaryMathOp::Div, other)
    }
    pub fn min_with(&self, other: &Field) -> Self {
        self.binary(BinaryMathOp::Min, other)
    }
    pub fn max_with(&self, other: &Field) -> Self {
        self.binary(BinaryMathOp::Max, other)
    }

    pub fn add_const(&self, v: f32) -> Result<Self, FieldError> {
        Ok(self.add(&Field::constant(v)?))
    }

    pub fn mul_const(&self, v: f32) -> Result<Self, FieldError> {
        Ok(self.mul(&Field::constant(v)?))
    }

    pub fn clamp(&self, lo: f32, hi: f32) -> Result<Self, FieldError> {
        finite_all(&[lo, hi])?;
        if lo > hi {
            return Err(FieldError::InvalidArgument("clamp lo > hi".into()));
        }
        Ok(Self::wrap(FieldNode::Clamp {
            child: self.clone(),
            lo,
            hi,
        }))
    }

    pub fn smoothstep(&self, edge0: f32, edge1: f32) -> Result<Self, FieldError> {
        finite_all(&[edge0, edge1])?;
        if (edge1 - edge0).abs() <= f32::EPSILON {
            return Err(FieldError::InvalidArgument(
                "smoothstep edges must differ".into(),
            ));
        }
        Ok(Self::wrap(FieldNode::Smoothstep {
            child: self.clone(),
            edge0,
            edge1,
        }))
    }

    pub fn step(&self, edge: f32) -> Result<Self, FieldError> {
        finite_all(&[edge])?;
        Ok(Self::wrap(FieldNode::Step {
            child: self.clone(),
            edge,
        }))
    }

    pub fn remap(
        &self,
        in_lo: f32,
        in_hi: f32,
        out_lo: f32,
        out_hi: f32,
        clamp: bool,
    ) -> Result<Self, FieldError> {
        finite_all(&[in_lo, in_hi, out_lo, out_hi])?;
        if (in_hi - in_lo).abs() <= f32::EPSILON {
            return Err(FieldError::InvalidArgument(
                "remap input range cannot be zero".into(),
            ));
        }
        Ok(Self::wrap(FieldNode::Remap {
            child: self.clone(),
            in_lo,
            in_hi,
            out_lo,
            out_hi,
            clamp,
        }))
    }

    pub fn lerp(&self, other: &Field, t: &Field) -> Self {
        Self::wrap(FieldNode::Lerp {
            a: self.clone(),
            b: other.clone(),
            t: t.clone(),
        })
    }

    pub fn select(cond: &Field, then_field: &Field, else_field: &Field) -> Self {
        Self::wrap(FieldNode::Select {
            cond: cond.clone(),
            then_field: then_field.clone(),
            else_field: else_field.clone(),
        })
    }

    // ── CSG & Surface Operators ────────────────────────────────────────────

    pub fn union(&self, other: &Field) -> Self {
        Self::wrap(FieldNode::Union {
            children: vec![self.clone(), other.clone()],
        })
    }

    pub fn union_all(children: Vec<Field>) -> Result<Self, FieldError> {
        if children.is_empty() {
            return Err(FieldError::InvalidArgument(
                "union_all requires at least one child".into(),
            ));
        }
        Ok(Self::wrap(FieldNode::Union { children }))
    }

    pub fn intersect(&self, other: &Field) -> Self {
        Self::wrap(FieldNode::Intersect {
            children: vec![self.clone(), other.clone()],
        })
    }

    pub fn intersect_all(children: Vec<Field>) -> Result<Self, FieldError> {
        if children.is_empty() {
            return Err(FieldError::InvalidArgument(
                "intersect_all requires at least one child".into(),
            ));
        }
        Ok(Self::wrap(FieldNode::Intersect { children }))
    }

    pub fn subtract(&self, other: &Field) -> Self {
        Self::wrap(FieldNode::Subtract {
            a: self.clone(),
            b: other.clone(),
        })
    }

    pub fn xor(&self, other: &Field) -> Self {
        Self::wrap(FieldNode::Xor {
            a: self.clone(),
            b: other.clone(),
        })
    }

    pub fn smooth_union(&self, other: &Field, k: f32) -> Result<Self, FieldError> {
        positive_all(&[k])?;
        Ok(Self::wrap(FieldNode::SmoothUnion {
            a: self.clone(),
            b: other.clone(),
            k,
        }))
    }

    pub fn smooth_subtract(&self, other: &Field, k: f32) -> Result<Self, FieldError> {
        positive_all(&[k])?;
        Ok(Self::wrap(FieldNode::SmoothSubtract {
            a: self.clone(),
            b: other.clone(),
            k,
        }))
    }

    pub fn smooth_intersect(&self, other: &Field, k: f32) -> Result<Self, FieldError> {
        positive_all(&[k])?;
        Ok(Self::wrap(FieldNode::SmoothIntersect {
            a: self.clone(),
            b: other.clone(),
            k,
        }))
    }

    pub fn chamfer_union(&self, other: &Field, r: f32) -> Result<Self, FieldError> {
        positive_all(&[r])?;
        Ok(Self::wrap(FieldNode::ChamferUnion {
            a: self.clone(),
            b: other.clone(),
            r,
        }))
    }

    pub fn chamfer_subtract(&self, other: &Field, r: f32) -> Result<Self, FieldError> {
        positive_all(&[r])?;
        Ok(Self::wrap(FieldNode::ChamferSubtract {
            a: self.clone(),
            b: other.clone(),
            r,
        }))
    }

    pub fn chamfer_intersect(&self, other: &Field, r: f32) -> Result<Self, FieldError> {
        positive_all(&[r])?;
        Ok(Self::wrap(FieldNode::ChamferIntersect {
            a: self.clone(),
            b: other.clone(),
            r,
        }))
    }

    pub fn steps_union(&self, other: &Field, r: f32, steps: f32) -> Result<Self, FieldError> {
        positive_all(&[r, steps])?;
        if steps < 1.0 {
            return Err(FieldError::InvalidArgument("steps must be >= 1".into()));
        }
        Ok(Self::wrap(FieldNode::StepsUnion {
            a: self.clone(),
            b: other.clone(),
            r,
            steps,
        }))
    }

    pub fn round(&self, radius: f32) -> Result<Self, FieldError> {
        non_negative_all(&[radius])?;
        Ok(Self::wrap(FieldNode::Round {
            child: self.clone(),
            radius,
        }))
    }

    pub fn shell(&self, thickness: f32) -> Result<Self, FieldError> {
        positive_all(&[thickness])?;
        Ok(Self::wrap(FieldNode::Shell {
            child: self.clone(),
            thickness,
        }))
    }

    pub fn elongate(&self, half_lengths: [f32; 3]) -> Result<Self, FieldError> {
        non_negative_all(&half_lengths)?;
        if half_lengths[0] <= 0.0 && half_lengths[1] <= 0.0 && half_lengths[2] <= 0.0 {
            return Err(FieldError::InvalidArgument(
                "elongate requires at least one positive half-length".into(),
            ));
        }
        Ok(Self::wrap(FieldNode::Elongate {
            child: self.clone(),
            half_lengths,
        }))
    }

    /// Displace this surface by ANY scalar field: `self(p) + amplitude * offset(p)`.
    pub fn displace(&self, offset: &Field, amplitude: f32) -> Result<Self, FieldError> {
        non_negative_all(&[amplitude])?;
        if amplitude == 0.0 {
            return Ok(self.clone());
        }
        Ok(Self::wrap(FieldNode::Displace {
            child: self.clone(),
            offset: offset.clone(),
            amplitude,
        }))
    }

    /// Convenience: displace by 3D FBM noise.
    pub fn displace_noise(
        &self,
        amplitude: f32,
        frequency: f32,
        seed: i32,
        octaves: u32,
    ) -> Result<Self, FieldError> {
        let n = Field::value_fbm3(frequency, seed, octaves)?;
        self.displace(&n, amplitude)
    }

    // ── Domain Transforms & Warps ──────────────────────────────────────────

    pub fn translate(&self, offset: [f32; 3]) -> Result<Self, FieldError> {
        finite_all(&offset)?;
        Ok(Self::wrap(FieldNode::Translate {
            child: self.clone(),
            offset,
        }))
    }

    pub fn rotate(&self, angles_deg: [f32; 3]) -> Result<Self, FieldError> {
        finite_all(&angles_deg)?;
        Ok(Self::wrap(FieldNode::Rotate {
            child: self.clone(),
            angles: angles_deg,
        }))
    }

    pub fn scale(&self, factor: f32) -> Result<Self, FieldError> {
        positive_all(&[factor])?;
        Ok(Self::wrap(FieldNode::Scale {
            child: self.clone(),
            factor,
        }))
    }

    pub fn scale_xyz(&self, factors: [f32; 3]) -> Result<Self, FieldError> {
        positive_all(&factors)?;
        Ok(Self::wrap(FieldNode::ScaleNonUniform {
            child: self.clone(),
            factors,
        }))
    }

    pub fn mirror(&self, axis: Axis) -> Self {
        Self::wrap(FieldNode::Mirror {
            child: self.clone(),
            axis,
        })
    }

    pub fn mirror_plane(&self, normal: [f32; 3], offset: f32) -> Result<Self, FieldError> {
        finite_all(&normal)?;
        finite_all(&[offset])?;
        let n = normalize_f64(normal)?;
        Ok(Self::wrap(FieldNode::MirrorPlane {
            child: self.clone(),
            normal: n,
            offset,
        }))
    }

    pub fn repeat_infinite(&self, spacing: [f32; 3]) -> Result<Self, FieldError> {
        non_negative_all(&spacing)?;
        Ok(Self::wrap(FieldNode::Repeat {
            child: self.clone(),
            spacing,
            count: None,
        }))
    }

    pub fn repeat_counted(&self, spacing: [f32; 3], count: [u32; 3]) -> Result<Self, FieldError> {
        non_negative_all(&spacing)?;
        Ok(Self::wrap(FieldNode::Repeat {
            child: self.clone(),
            spacing,
            count: Some(count),
        }))
    }

    /// Radial/polar repetition around the Y axis (`count >= 2` sectors).
    pub fn repeat_polar(&self, count: u32) -> Result<Self, FieldError> {
        if count < 2 || count > 1024 {
            return Err(FieldError::InvalidArgument(
                "repeat_polar count must be in 2..=1024".into(),
            ));
        }
        Ok(Self::wrap(FieldNode::RepeatPolar {
            child: self.clone(),
            count,
        }))
    }

    pub fn repeat_points(&self, offsets: Vec<[f32; 3]>) -> Result<Self, FieldError> {
        if offsets.is_empty() || offsets.len() > MAX_REPEAT_POINTS {
            return Err(FieldError::InvalidArgument(format!(
                "repeat_points requires 1..={MAX_REPEAT_POINTS} points"
            )));
        }
        for p in &offsets {
            finite_all(p)?;
        }
        Ok(Self::wrap(FieldNode::RepeatPoints {
            child: self.clone(),
            offsets,
        }))
    }

    pub fn twist(&self, amount: f32) -> Result<Self, FieldError> {
        finite_all(&[amount])?;
        Ok(Self::wrap(FieldNode::Twist {
            child: self.clone(),
            amount,
        }))
    }

    pub fn bend(&self, amount: f32) -> Result<Self, FieldError> {
        finite_all(&[amount])?;
        Ok(Self::wrap(FieldNode::Bend {
            child: self.clone(),
            amount,
        }))
    }

    /// General domain warp using 3 arbitrary fields for X, Y, Z offsets.
    pub fn warp(
        &self,
        warp_x: &Field,
        warp_y: &Field,
        warp_z: &Field,
        amplitude: f32,
    ) -> Result<Self, FieldError> {
        non_negative_all(&[amplitude])?;
        if amplitude == 0.0 {
            return Ok(self.clone());
        }
        Ok(Self::wrap(FieldNode::DomainWarp {
            child: self.clone(),
            warp_x: warp_x.clone(),
            warp_y: warp_y.clone(),
            warp_z: warp_z.clone(),
            amplitude,
        }))
    }

    /// Convenience: domain warp using 3 decorrelated seeded noise fields.
    pub fn warp_noise(
        &self,
        amplitude: f32,
        frequency: f32,
        seed: i32,
        octaves: u32,
    ) -> Result<Self, FieldError> {
        let wx = Field::perlin_fbm3(frequency, seed, octaves)?;
        let wy = Field::perlin_fbm3(frequency, seed.wrapping_add(7919), octaves)?;
        let wz = Field::perlin_fbm3(frequency, seed.wrapping_add(104_729), octaves)?;
        self.warp(&wx, &wy, &wz, amplitude)
    }

    // ── Tagging, Bounds & Expressions ──────────────────────────────────────

    /// Tag this field branch with a material ID (`1..=65535`).
    pub fn tag(&self, material_id: u16) -> Result<Self, FieldError> {
        if material_id == 0 {
            return Err(FieldError::InvalidArgument(
                "material_id must be non-zero (0 is untagged)".into(),
            ));
        }
        Ok(Self::wrap(FieldNode::Tag {
            child: self.clone(),
            material_id,
        }))
    }

    /// Attach explicit finite sampling bounds to this field.
    pub fn bounded(&self, bounds: Aabb) -> Result<Self, FieldError> {
        let bounds = Aabb::new(bounds.min, bounds.max)?;
        Ok(Self::wrap(FieldNode::Bounded {
            child: self.clone(),
            bounds,
        }))
    }

    /// Parse a sandboxed infix expression with no external bindings.
    pub fn expr(source: &str) -> Result<Self, FieldError> {
        Self::expr_with(source, &[], None, DistanceKind::Estimate)
    }

    /// Parse a sandboxed infix expression referencing named external [`Field`]s,
    /// with optional explicit bounds and distance kind.
    pub fn expr_with(
        source: &str,
        bindings: &[(&str, Field)],
        bounds: Option<Aabb>,
        distance_kind: DistanceKind,
    ) -> Result<Self, FieldError> {
        if let Some(b) = bounds {
            Aabb::new(b.min, b.max)?;
        }
        let mut names = Vec::with_capacity(bindings.len());
        let mut inputs = Vec::with_capacity(bindings.len());
        for (name, field) in bindings {
            if name.is_empty() {
                return Err(FieldError::InvalidArgument(
                    "binding name cannot be empty".into(),
                ));
            }
            names.push((*name).to_string());
            inputs.push(field.clone());
        }
        let program = ExprProgram::compile(source, &names).map_err(FieldError::Parse)?;
        Ok(Self::wrap(FieldNode::Expr {
            program: Arc::new(program),
            inputs,
            bounds,
            distance_kind,
        }))
    }

    // ── Static Analysis: Bounds, Output Range, DistanceKind, Validation ────

    /// Analytically proven inclusive output range `[min, max]`, when known.
    pub fn output_range(&self) -> Option<[f32; 2]> {
        match &*self.0 {
            FieldNode::Const(v) => Some([*v, *v]),
            FieldNode::Noise2 {
                basis: NoiseBasis::Value,
                ..
            }
            | FieldNode::Noise3 {
                basis: NoiseBasis::Value,
                ..
            } => Some([0.0, 1.0]),
            FieldNode::Noise2 {
                basis: NoiseBasis::Perlin,
                ..
            }
            | FieldNode::Noise3 {
                basis: NoiseBasis::Perlin,
                ..
            } => Some([-1.0, 1.0]),
            FieldNode::Fbm2 { ridged, .. } | FieldNode::Fbm3 { ridged, .. } => {
                if *ridged {
                    Some([0.0, 1.0])
                } else {
                    Some([-1.0, 1.0])
                }
            }
            FieldNode::Cells {
                frequency,
                mode,
                threshold,
                ..
            } => {
                let inv_f = 1.0 / frequency.max(1e-6);
                let [lo, hi] = match mode {
                    CellMode::Value => [0.0, 1.0],
                    CellMode::F1 => [0.0, 3.0 * inv_f],
                    CellMode::F2 | CellMode::F2MinusF1 => [0.0, 3.5 * inv_f],
                };
                Some([lo - threshold, hi - threshold])
            }
            FieldNode::Unary { op, child } => match op {
                UnaryMathOp::Sin | UnaryMathOp::Cos => Some([-1.0, 1.0]),
                UnaryMathOp::Saturate => Some([0.0, 1.0]),
                UnaryMathOp::Sign => Some([-1.0, 1.0]),
                UnaryMathOp::Fract => Some([0.0, 1.0]),
                UnaryMathOp::Neg => child.output_range().map(|[lo, hi]| [-hi, -lo]),
                UnaryMathOp::Abs => child.output_range().map(|[lo, hi]| {
                    if lo >= 0.0 {
                        [lo, hi]
                    } else if hi <= 0.0 {
                        [-hi, -lo]
                    } else {
                        [0.0, hi.max(-lo)]
                    }
                }),
                UnaryMathOp::Sqrt => child
                    .output_range()
                    .filter(|[lo, _]| *lo >= 0.0)
                    .map(|[lo, hi]| [lo.sqrt(), hi.sqrt()]),
                UnaryMathOp::Floor => child.output_range().map(|[lo, hi]| [lo.floor(), hi.floor()]),
                UnaryMathOp::Ceil => child.output_range().map(|[lo, hi]| [lo.ceil(), hi.ceil()]),
                UnaryMathOp::Exp => child
                    .output_range()
                    .map(|[lo, hi]| [lo.exp(), hi.exp()])
                    .filter(|r| r[0].is_finite() && r[1].is_finite()),
                UnaryMathOp::Ln => child
                    .output_range()
                    .filter(|[lo, _]| *lo > 0.0)
                    .map(|[lo, hi]| [lo.ln(), hi.ln()]),
            },
            FieldNode::Binary { op, a, b } => {
                let [a0, a1] = a.output_range()?;
                let [b0, b1] = b.output_range()?;
                let res = match op {
                    BinaryMathOp::Add => [a0 + b0, a1 + b1],
                    BinaryMathOp::Sub => [a0 - b1, a1 - b0],
                    BinaryMathOp::Mul => {
                        let ps = [a0 * b0, a0 * b1, a1 * b0, a1 * b1];
                        [
                            ps[0].min(ps[1]).min(ps[2]).min(ps[3]),
                            ps[0].max(ps[1]).max(ps[2]).max(ps[3]),
                        ]
                    }
                    BinaryMathOp::Min => [a0.min(b0), a1.min(b1)],
                    BinaryMathOp::Max => [a0.max(b0), a1.max(b1)],
                    BinaryMathOp::Atan2 => [-std::f32::consts::PI, std::f32::consts::PI],
                    BinaryMathOp::Div | BinaryMathOp::Pow | BinaryMathOp::Mod => return None,
                };
                (res[0].is_finite() && res[1].is_finite()).then_some(res)
            }
            FieldNode::Clamp { lo, hi, .. } => Some([*lo, *hi]),
            FieldNode::Smoothstep { .. } | FieldNode::Step { .. } => Some([0.0, 1.0]),
            FieldNode::Remap {
                child,
                in_lo,
                in_hi,
                out_lo,
                out_hi,
                clamp,
            } => {
                if *clamp {
                    Some([out_lo.min(*out_hi), out_lo.max(*out_hi)])
                } else {
                    let [a, b] = child.output_range()?;
                    let f = |v: f32| out_lo + (v - in_lo) / (in_hi - in_lo) * (out_hi - out_lo);
                    let r0 = f(a);
                    let r1 = f(b);
                    (r0.is_finite() && r1.is_finite()).then_some([r0.min(r1), r0.max(r1)])
                }
            }
            FieldNode::Lerp { a, b, t } => {
                let [a0, a1] = a.output_range()?;
                let [b0, b1] = b.output_range()?;
                let [t0, t1] = t.output_range()?;
                if t0 >= 0.0 && t1 <= 1.0 {
                    Some([a0.min(b0), a1.max(b1)])
                } else {
                    None
                }
            }
            FieldNode::Select {
                then_field,
                else_field,
                ..
            } => {
                let [a0, a1] = then_field.output_range()?;
                let [b0, b1] = else_field.output_range()?;
                Some([a0.min(b0), a1.max(b1)])
            }
            FieldNode::Translate { child, .. }
            | FieldNode::Rotate { child, .. }
            | FieldNode::Mirror { child, .. }
            | FieldNode::MirrorPlane { child, .. }
            | FieldNode::Repeat { child, .. }
            | FieldNode::RepeatPolar { child, .. }
            | FieldNode::Twist { child, .. }
            | FieldNode::Bend { child, .. }
            | FieldNode::DomainWarp { child, .. }
            | FieldNode::Tag { child, .. }
            | FieldNode::Bounded { child, .. } => child.output_range(),
            _ => None,
        }
    }

    /// Conservative finite AABB of the negative (solid) region, or `None` if unbounded.
    pub fn bounds(&self) -> Option<Aabb> {
        fn sym(hx: f32, hy: f32, hz: f32) -> Option<Aabb> {
            Some(Aabb {
                min: [-hx, -hy, -hz],
                max: [hx, hy, hz],
            })
        }
        match &*self.0 {
            FieldNode::Sphere { radius } => sym(*radius, *radius, *radius),
            FieldNode::Box {
                half_extents: b, ..
            } => sym(b[0], b[1], b[2]),
            FieldNode::Ellipsoid { radii: r } => sym(r[0], r[1], r[2]),
            FieldNode::Torus {
                major_radius,
                minor_radius,
            }
            | FieldNode::CappedTorus {
                major_radius,
                minor_radius,
                ..
            } => sym(
                major_radius + minor_radius,
                *minor_radius,
                major_radius + minor_radius,
            ),
            FieldNode::Link {
                major_radius,
                minor_radius,
                half_length,
            } => sym(
                major_radius + minor_radius,
                *minor_radius,
                major_radius + minor_radius + half_length,
            ),
            FieldNode::Capsule { a, b, radius } => Some(Aabb {
                min: [
                    a[0].min(b[0]) - radius,
                    a[1].min(b[1]) - radius,
                    a[2].min(b[2]) - radius,
                ],
                max: [
                    a[0].max(b[0]) + radius,
                    a[1].max(b[1]) + radius,
                    a[2].max(b[2]) + radius,
                ],
            }),
            FieldNode::RoundCone { a, b, r1, r2 } => Some(Aabb {
                min: [
                    (a[0] - r1).min(b[0] - r2),
                    (a[1] - r1).min(b[1] - r2),
                    (a[2] - r1).min(b[2] - r2),
                ],
                max: [
                    (a[0] + r1).max(b[0] + r2),
                    (a[1] + r1).max(b[1] + r2),
                    (a[2] + r1).max(b[2] + r2),
                ],
            }),
            FieldNode::SolidAngle { radius, .. } => sym(*radius, *radius, *radius),
            FieldNode::CutSphere { radius: r, height } => {
                let w = (r * r - height * height).max(0.0).sqrt();
                let rad = if *height <= 0.0 { *r } else { w };
                Some(Aabb {
                    min: [-rad, *height, -rad],
                    max: [rad, *r, rad],
                })
            }
            FieldNode::CutHollowSphere {
                radius: r,
                height,
                thickness,
            } => {
                let w = (r * r - height * height).max(0.0).sqrt();
                let rad = if *height <= 0.0 { *r } else { w };
                Some(Aabb {
                    min: [-rad - thickness, height - thickness, -rad - thickness],
                    max: [rad + thickness, r + thickness, rad + thickness],
                })
            }
            FieldNode::CappedCylinder {
                radius,
                half_height,
            } => sym(*radius, *half_height, *radius),
            FieldNode::CappedCone {
                half_height,
                r1,
                r2,
            } => {
                let r = r1.max(*r2);
                sym(r, *half_height, r)
            }
            FieldNode::Octahedron { size } => sym(*size, *size, *size),
            FieldNode::HexPrism {
                radius,
                half_height,
            } => sym(*radius * 1.154_700_5, *half_height, *radius),
            FieldNode::SuperPrism {
                half_extents: b, ..
            }
            | FieldNode::BoxFrame {
                half_extents: b, ..
            } => sym(b[0], b[1], b[2]),
            FieldNode::SquarePyramid { half_base, height } => {
                sym(*half_base, height * 0.5, *half_base)
            }
            FieldNode::Union { children } => {
                let mut acc: Option<Aabb> = None;
                for c in children {
                    let cb = c.bounds()?;
                    acc = Some(match acc {
                        Some(a) => a.union(cb),
                        None => cb,
                    });
                }
                acc
            }
            FieldNode::Intersect { children } => {
                let mut acc: Option<Aabb> = None;
                for c in children {
                    if let Some(cb) = c.bounds() {
                        acc = Some(match acc {
                            Some(a) => a.intersection(cb),
                            None => cb,
                        });
                    }
                }
                acc
            }
            FieldNode::Subtract { a, .. } => a.bounds(),
            FieldNode::Xor { a, b } => Some(a.bounds()?.union(b.bounds()?)),
            FieldNode::SmoothUnion { a, b, k } => Some(a.bounds()?.union(b.bounds()?).grow(*k)),
            FieldNode::SmoothSubtract { a, k, .. } => a.bounds().map(|bb| bb.grow(*k)),
            FieldNode::SmoothIntersect { a, b, k } => match (a.bounds(), b.bounds()) {
                (Some(ab), Some(bb)) => Some(ab.intersection(bb).grow(*k)),
                (Some(ab), None) => Some(ab.grow(*k)),
                (None, Some(bb)) => Some(bb.grow(*k)),
                (None, None) => None,
            },
            FieldNode::ChamferUnion { a, b, r } | FieldNode::StepsUnion { a, b, r, .. } => {
                Some(a.bounds()?.union(b.bounds()?).grow(*r))
            }
            FieldNode::ChamferSubtract { a, r, .. } => a.bounds().map(|bb| bb.grow(*r)),
            FieldNode::ChamferIntersect { a, b, r } => match (a.bounds(), b.bounds()) {
                (Some(ab), Some(bb)) => Some(ab.intersection(bb).grow(*r)),
                (Some(ab), None) => Some(ab.grow(*r)),
                (None, Some(bb)) => Some(bb.grow(*r)),
                (None, None) => None,
            },
            FieldNode::Round { child, radius } => child.bounds().map(|b| b.grow(*radius)),
            FieldNode::Shell { child, thickness } => child.bounds().map(|b| b.grow(*thickness)),
            FieldNode::Elongate {
                child,
                half_lengths: h,
            } => child.bounds().map(|b| {
                let ex = b.min[0].abs().max(b.max[0].abs()) + h[0];
                let ey = b.min[1].abs().max(b.max[1].abs()) + h[1];
                let ez = b.min[2].abs().max(b.max[2].abs()) + h[2];
                Aabb {
                    min: [-ex, -ey, -ez],
                    max: [ex, ey, ez],
                }
            }),
            FieldNode::Displace {
                child,
                offset,
                amplitude,
            } => {
                let [lo, hi] = offset.output_range()?;
                let pad = amplitude.abs() * lo.abs().max(hi.abs());
                child.bounds().map(|b| b.grow(pad))
            }
            FieldNode::Translate { child, offset } => child.bounds().map(|b| Aabb {
                min: [
                    b.min[0] + offset[0],
                    b.min[1] + offset[1],
                    b.min[2] + offset[2],
                ],
                max: [
                    b.max[0] + offset[0],
                    b.max[1] + offset[1],
                    b.max[2] + offset[2],
                ],
            }),
            FieldNode::Rotate { child, angles } => {
                let b = child.bounds()?;
                let m = rot_matrix(*angles);
                let mut min = [f32::INFINITY; 3];
                let mut max = [f32::NEG_INFINITY; 3];
                for i in 0..8 {
                    let c = [
                        if i & 1 == 0 { b.min[0] } else { b.max[0] },
                        if i & 2 == 0 { b.min[1] } else { b.max[1] },
                        if i & 4 == 0 { b.min[2] } else { b.max[2] },
                    ];
                    let r = rotate_point(&m, c);
                    for a in 0..3 {
                        min[a] = min[a].min(r[a]);
                        max[a] = max[a].max(r[a]);
                    }
                }
                Some(Aabb { min, max })
            }
            FieldNode::Scale { child, factor } => {
                let f = factor.abs();
                child.bounds().map(|b| Aabb {
                    min: [b.min[0] * f, b.min[1] * f, b.min[2] * f],
                    max: [b.max[0] * f, b.max[1] * f, b.max[2] * f],
                })
            }
            FieldNode::ScaleNonUniform { child, factors } => child.bounds().map(|b| Aabb {
                min: [
                    b.min[0] * factors[0],
                    b.min[1] * factors[1],
                    b.min[2] * factors[2],
                ],
                max: [
                    b.max[0] * factors[0],
                    b.max[1] * factors[1],
                    b.max[2] * factors[2],
                ],
            }),
            FieldNode::Mirror { child, axis } => {
                let b = child.bounds()?;
                let idx = match axis {
                    Axis::X => 0,
                    Axis::Y => 1,
                    Axis::Z => 2,
                };
                let hi = b.max[idx].abs().max(b.min[idx].abs());
                let mut min = b.min;
                let mut max = b.max;
                min[idx] = -hi;
                max[idx] = hi;
                Some(Aabb { min, max })
            }
            FieldNode::MirrorPlane {
                child,
                normal,
                offset,
            } => {
                // Mirror across plane n·p + offset = 0: union original box and reflected corners.
                let b = child.bounds()?;
                let mut min = b.min;
                let mut max = b.max;
                for i in 0..8 {
                    let c = [
                        if i & 1 == 0 { b.min[0] } else { b.max[0] },
                        if i & 2 == 0 { b.min[1] } else { b.max[1] },
                        if i & 4 == 0 { b.min[2] } else { b.max[2] },
                    ];
                    let d = c[0] * normal[0] + c[1] * normal[1] + c[2] * normal[2] + offset;
                    let r = [
                        c[0] - 2.0 * d * normal[0],
                        c[1] - 2.0 * d * normal[1],
                        c[2] - 2.0 * d * normal[2],
                    ];
                    for a in 0..3 {
                        min[a] = min[a].min(r[a]);
                        max[a] = max[a].max(r[a]);
                    }
                }
                Some(Aabb { min, max })
            }
            FieldNode::Repeat {
                child,
                spacing,
                count,
            } => {
                let b = child.bounds()?;
                match count {
                    Some(n) => Some(Aabb {
                        min: [
                            b.min[0] - spacing[0] * n[0] as f32,
                            b.min[1] - spacing[1] * n[1] as f32,
                            b.min[2] - spacing[2] * n[2] as f32,
                        ],
                        max: [
                            b.max[0] + spacing[0] * n[0] as f32,
                            b.max[1] + spacing[1] * n[1] as f32,
                            b.max[2] + spacing[2] * n[2] as f32,
                        ],
                    }),
                    None => {
                        if spacing.iter().all(|&s| s <= 0.0) {
                            Some(b)
                        } else {
                            None
                        }
                    }
                }
            }
            FieldNode::RepeatPolar { child, .. } => {
                let b = child.bounds()?;
                let mut max_r = 0.0f32;
                for &xx in &[b.min[0], b.max[0]] {
                    for &zz in &[b.min[2], b.max[2]] {
                        max_r = max_r.max((xx * xx + zz * zz).sqrt());
                    }
                }
                Some(Aabb {
                    min: [-max_r, b.min[1], -max_r],
                    max: [max_r, b.max[1], max_r],
                })
            }
            FieldNode::RepeatPoints { child, offsets } => {
                let b = child.bounds()?;
                let mut min = [f32::INFINITY; 3];
                let mut max = [f32::NEG_INFINITY; 3];
                for o in offsets {
                    for a in 0..3 {
                        min[a] = min[a].min(b.min[a] + o[a]);
                        max[a] = max[a].max(b.max[a] + o[a]);
                    }
                }
                Some(Aabb { min, max })
            }
            FieldNode::Twist { child, .. } => {
                let b = child.bounds()?;
                let mut max_r = 0.0f32;
                for &xx in &[b.min[0], b.max[0]] {
                    for &zz in &[b.min[2], b.max[2]] {
                        max_r = max_r.max((xx * xx + zz * zz).sqrt());
                    }
                }
                Some(Aabb {
                    min: [-max_r, b.min[1], -max_r],
                    max: [max_r, b.max[1], max_r],
                })
            }
            FieldNode::Bend { child, .. } => {
                let b = child.bounds()?;
                let mut max_r = 0.0f32;
                for &xx in &[b.min[0], b.max[0]] {
                    for &yy in &[b.min[1], b.max[1]] {
                        max_r = max_r.max((xx * xx + yy * yy).sqrt());
                    }
                }
                Some(Aabb {
                    min: [-max_r, -max_r, b.min[2]],
                    max: [max_r, max_r, b.max[2]],
                })
            }
            FieldNode::DomainWarp {
                child,
                warp_x,
                warp_y,
                warp_z,
                amplitude,
            } => {
                let rx = warp_x.output_range()?;
                let ry = warp_y.output_range()?;
                let rz = warp_z.output_range()?;
                let px = amplitude.abs() * rx[0].abs().max(rx[1].abs());
                let py = amplitude.abs() * ry[0].abs().max(ry[1].abs());
                let pz = amplitude.abs() * rz[0].abs().max(rz[1].abs());
                child.bounds().map(|b| Aabb {
                    min: [b.min[0] - px, b.min[1] - py, b.min[2] - pz],
                    max: [b.max[0] + px, b.max[1] + py, b.max[2] + pz],
                })
            }
            FieldNode::Tag { child, .. } => child.bounds(),
            FieldNode::Bounded { bounds, .. } => Some(*bounds),
            FieldNode::Expr { bounds, .. } => *bounds,
            _ => None,
        }
    }

    /// Distance semantics of this field.
    pub fn distance_kind(&self) -> DistanceKind {
        match &*self.0 {
            FieldNode::Sphere { .. }
            | FieldNode::Box { .. }
            | FieldNode::Torus { .. }
            | FieldNode::CappedTorus { .. }
            | FieldNode::Link { .. }
            | FieldNode::Capsule { .. }
            | FieldNode::RoundCone { .. }
            | FieldNode::SolidAngle { .. }
            | FieldNode::CutSphere { .. }
            | FieldNode::CutHollowSphere { .. }
            | FieldNode::CappedCylinder { .. }
            | FieldNode::InfiniteCylinder { .. }
            | FieldNode::CappedCone { .. }
            | FieldNode::Plane { .. }
            | FieldNode::Octahedron { .. }
            | FieldNode::HexPrism { .. }
            | FieldNode::BoxFrame { .. }
            | FieldNode::InfiniteCone { .. }
            | FieldNode::SquarePyramid { .. } => DistanceKind::Exact,
            FieldNode::Ellipsoid { .. }
            | FieldNode::SuperPrism { .. }
            | FieldNode::Intersect { .. }
            | FieldNode::Subtract { .. }
            | FieldNode::Xor { .. } => DistanceKind::LowerBound,
            FieldNode::Translate { child, .. }
            | FieldNode::Rotate { child, .. }
            | FieldNode::Scale { child, .. }
            | FieldNode::Round { child, .. }
            | FieldNode::Shell { child, .. }
            | FieldNode::Tag { child, .. }
            | FieldNode::Bounded { child, .. } => child.distance_kind(),
            FieldNode::Expr { distance_kind, .. } => *distance_kind,
            FieldNode::Const(_)
            | FieldNode::Coord(_)
            | FieldNode::Noise2 { .. }
            | FieldNode::Noise3 { .. }
            | FieldNode::Fbm2 { .. }
            | FieldNode::Fbm3 { .. } => DistanceKind::Implicit,
            _ => DistanceKind::Estimate,
        }
    }

    /// Validate structural budgets (depth, unique node count, expanded walk).
    pub fn validate(&self) -> Result<(), FieldError> {
        let mut seen = HashMap::<usize, ()>::new();
        let mut walk_budget = MAX_WALK_NODES;
        self.validate_at(0, &mut seen, &mut walk_budget)
    }

    fn validate_at(
        &self,
        depth: u32,
        seen: &mut HashMap<usize, ()>,
        walk_budget: &mut u32,
    ) -> Result<(), FieldError> {
        if depth > MAX_TREE_DEPTH {
            return Err(FieldError::BudgetExceeded(format!(
                "field graph exceeds max depth {MAX_TREE_DEPTH}"
            )));
        }
        if *walk_budget == 0 {
            return Err(FieldError::BudgetExceeded(format!(
                "field graph exceeds walk limit {MAX_WALK_NODES}"
            )));
        }
        *walk_budget -= 1;
        let ptr = Arc::as_ptr(&self.0) as usize;
        if seen.insert(ptr, ()).is_some() {
            // Already validated this shared DAG node.
            return Ok(());
        }
        if seen.len() > MAX_UNIQUE_NODES {
            return Err(FieldError::BudgetExceeded(format!(
                "field graph exceeds max unique nodes {MAX_UNIQUE_NODES}"
            )));
        }
        self.for_each_child(|child| child.validate_at(depth + 1, seen, walk_budget))
    }

    pub(crate) fn for_each_child<F>(&self, mut f: F) -> Result<(), FieldError>
    where
        F: FnMut(&Field) -> Result<(), FieldError>,
    {
        match &*self.0 {
            FieldNode::Heightmap { height, .. }
            | FieldNode::Unary { child: height, .. }
            | FieldNode::Clamp { child: height, .. }
            | FieldNode::Smoothstep { child: height, .. }
            | FieldNode::Step { child: height, .. }
            | FieldNode::Remap { child: height, .. }
            | FieldNode::Round { child: height, .. }
            | FieldNode::Shell { child: height, .. }
            | FieldNode::Elongate { child: height, .. }
            | FieldNode::Translate { child: height, .. }
            | FieldNode::Rotate { child: height, .. }
            | FieldNode::Scale { child: height, .. }
            | FieldNode::ScaleNonUniform { child: height, .. }
            | FieldNode::Mirror { child: height, .. }
            | FieldNode::MirrorPlane { child: height, .. }
            | FieldNode::Repeat { child: height, .. }
            | FieldNode::RepeatPolar { child: height, .. }
            | FieldNode::RepeatPoints { child: height, .. }
            | FieldNode::Twist { child: height, .. }
            | FieldNode::Bend { child: height, .. }
            | FieldNode::Tag { child: height, .. }
            | FieldNode::Bounded { child: height, .. } => f(height),

            FieldNode::Binary { a, b, .. }
            | FieldNode::Subtract { a, b }
            | FieldNode::Xor { a, b }
            | FieldNode::SmoothUnion { a, b, .. }
            | FieldNode::SmoothSubtract { a, b, .. }
            | FieldNode::SmoothIntersect { a, b, .. }
            | FieldNode::ChamferUnion { a, b, .. }
            | FieldNode::ChamferSubtract { a, b, .. }
            | FieldNode::ChamferIntersect { a, b, .. }
            | FieldNode::StepsUnion { a, b, .. }
            | FieldNode::Displace {
                child: a,
                offset: b,
                ..
            } => {
                f(a)?;
                f(b)
            }

            FieldNode::Lerp { a, b, t } => {
                f(a)?;
                f(b)?;
                f(t)
            }
            FieldNode::Select {
                cond,
                then_field,
                else_field,
            } => {
                f(cond)?;
                f(then_field)?;
                f(else_field)
            }
            FieldNode::DomainWarp {
                child,
                warp_x,
                warp_y,
                warp_z,
                ..
            } => {
                f(child)?;
                f(warp_x)?;
                f(warp_y)?;
                f(warp_z)
            }
            FieldNode::Union { children } | FieldNode::Intersect { children } => {
                for c in children {
                    f(c)?;
                }
                Ok(())
            }
            FieldNode::Expr { inputs, .. } => {
                for inp in inputs {
                    f(inp)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    // ── Lossless Deduplicated DAG JSON Serialization ───────────────────────

    pub fn to_json(&self) -> Result<String, FieldError> {
        let graph = FieldGraph::from_roots([("main".to_string(), self.clone())])?;
        graph.to_json()
    }

    pub fn from_json(json: &str) -> Result<Self, FieldError> {
        let graph = FieldGraph::from_json(json)?;
        graph
            .root("main")
            .ok_or_else(|| FieldError::Parse("field JSON missing 'main' root".into()))
    }
}

// ── Deduplicated Multi-Root DAG Serialization (v2) ───────────────────────────

/// Versioned multi-root DAG document preserving shared `Arc` identity.
#[derive(Debug, Clone)]
pub struct FieldGraph {
    roots: BTreeMap<String, Field>,
}

impl FieldGraph {
    pub fn from_roots<I>(roots: I) -> Result<Self, FieldError>
    where
        I: IntoIterator<Item = (String, Field)>,
    {
        let mut map = BTreeMap::new();
        for (name, field) in roots {
            if name.is_empty() || name.len() > MAX_ROOT_NAME_BYTES || map.len() >= MAX_GRAPH_ROOTS {
                return Err(FieldError::InvalidArgument(
                    "invalid graph root name or count".into(),
                ));
            }
            field.validate()?;
            if map.insert(name, field).is_some() {
                return Err(FieldError::InvalidArgument("duplicate graph root".into()));
            }
        }
        if map.is_empty() {
            return Err(FieldError::InvalidArgument(
                "graph must have at least one root".into(),
            ));
        }
        Ok(Self { roots: map })
    }

    pub fn root(&self, name: &str) -> Option<Field> {
        self.roots.get(name).cloned()
    }

    pub fn root_names(&self) -> impl Iterator<Item = &str> {
        self.roots.keys().map(String::as_str)
    }

    pub fn to_json(&self) -> Result<String, FieldError> {
        let mut ptr_to_id = HashMap::<usize, usize>::new();
        let mut nodes = Vec::<WireNode>::new();
        let mut roots = BTreeMap::<String, usize>::new();

        for (name, field) in &self.roots {
            let id = encode_field(field, &mut ptr_to_id, &mut nodes)?;
            roots.insert(name.clone(), id);
        }

        let doc = FieldGraphDoc {
            version: FIELD_GRAPH_VERSION,
            nodes,
            roots,
        };
        serde_json::to_string(&doc).map_err(|e| FieldError::Parse(e.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self, FieldError> {
        if json.len() > MAX_FIELD_JSON_BYTES {
            return Err(FieldError::InputTooLarge);
        }
        let doc: FieldGraphDoc =
            serde_json::from_str(json).map_err(|e| FieldError::Parse(e.to_string()))?;
        if doc.version != FIELD_GRAPH_VERSION {
            return Err(FieldError::Parse(format!(
                "unsupported field graph version {} (expected {FIELD_GRAPH_VERSION})",
                doc.version
            )));
        }
        if doc.nodes.is_empty()
            || doc.nodes.len() > MAX_UNIQUE_NODES
            || doc.roots.is_empty()
            || doc.roots.len() > MAX_GRAPH_ROOTS
        {
            return Err(FieldError::BudgetExceeded(
                "invalid node or root count in field graph".into(),
            ));
        }

        // Topological decode: every WireNode may only reference strictly smaller indices.
        let mut decoded = Vec::<Field>::with_capacity(doc.nodes.len());
        for (idx, wn) in doc.nodes.into_iter().enumerate() {
            let field = decode_node(wn, idx, &decoded)?;
            decoded.push(field);
        }

        let mut roots = BTreeMap::new();
        for (name, id) in doc.roots {
            if name.is_empty() || name.len() > MAX_ROOT_NAME_BYTES {
                return Err(FieldError::InvalidArgument("invalid root name".into()));
            }
            let field = decoded
                .get(id)
                .ok_or_else(|| FieldError::Parse(format!("dangling root reference {id}")))?
                .clone();
            field.validate()?;
            roots.insert(name, field);
        }
        Ok(Self { roots })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FieldGraphDoc {
    version: u32,
    nodes: Vec<WireNode>,
    roots: BTreeMap<String, usize>,
}

#[derive(Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum WireNode {
    Const { value: f32 },
    Coord { axis: Axis },
    Sphere { radius: f32 },
    Box { half_extents: [f32; 3], rounding: f32 },
    Ellipsoid { radii: [f32; 3] },
    Torus { major_radius: f32, minor_radius: f32 },
    CappedTorus { major_radius: f32, minor_radius: f32, cap_angle: f32 },
    Link { major_radius: f32, minor_radius: f32, half_length: f32 },
    Capsule { a: [f32; 3], b: [f32; 3], radius: f32 },
    RoundCone { a: [f32; 3], b: [f32; 3], r1: f32, r2: f32 },
    SolidAngle { radius: f32, angle: f32 },
    CutSphere { radius: f32, height: f32 },
    CutHollowSphere { radius: f32, height: f32, thickness: f32 },
    CappedCylinder { radius: f32, half_height: f32 },
    InfiniteCylinder { radius: f32 },
    CappedCone { half_height: f32, r1: f32, r2: f32 },
    Plane { normal: [f32; 3], offset: f32 },
    Octahedron { size: f32 },
    HexPrism { radius: f32, half_height: f32 },
    SuperPrism { half_extents: [f32; 3], exponent: f32 },
    BoxFrame { half_extents: [f32; 3], thickness: f32 },
    InfiniteCone { angle: f32 },
    SquarePyramid { half_base: f32, height: f32 },
    Heightmap { base_y: f32, height: usize },
    Noise2 { basis: NoiseBasis, frequency: f32, seed: i32 },
    Noise3 { basis: NoiseBasis, frequency: f32, seed: i32 },
    Fbm2 {
        basis: NoiseBasis,
        frequency: f32,
        seed: i32,
        octaves: u32,
        lacunarity: f32,
        gain: f32,
        ridged: bool,
    },
    Fbm3 {
        basis: NoiseBasis,
        frequency: f32,
        seed: i32,
        octaves: u32,
        lacunarity: f32,
        gain: f32,
        ridged: bool,
    },
    Cells { frequency: f32, seed: i32, jitter: f32, mode: CellMode, threshold: f32 },
    Unary { op: UnaryMathOp, child: usize },
    Binary { op: BinaryMathOp, a: usize, b: usize },
    Clamp { child: usize, lo: f32, hi: f32 },
    Smoothstep { child: usize, edge0: f32, edge1: f32 },
    Step { child: usize, edge: f32 },
    Remap { child: usize, in_lo: f32, in_hi: f32, out_lo: f32, out_hi: f32, clamp: bool },
    Lerp { a: usize, b: usize, t: usize },
    Select { cond: usize, then_field: usize, else_field: usize },
    Union { children: Vec<usize> },
    Intersect { children: Vec<usize> },
    Subtract { a: usize, b: usize },
    Xor { a: usize, b: usize },
    SmoothUnion { a: usize, b: usize, k: f32 },
    SmoothSubtract { a: usize, b: usize, k: f32 },
    SmoothIntersect { a: usize, b: usize, k: f32 },
    ChamferUnion { a: usize, b: usize, r: f32 },
    ChamferSubtract { a: usize, b: usize, r: f32 },
    ChamferIntersect { a: usize, b: usize, r: f32 },
    StepsUnion { a: usize, b: usize, r: f32, steps: f32 },
    Round { child: usize, radius: f32 },
    Shell { child: usize, thickness: f32 },
    Elongate { child: usize, half_lengths: [f32; 3] },
    Displace { child: usize, offset: usize, amplitude: f32 },
    Translate { child: usize, offset: [f32; 3] },
    Rotate { child: usize, angles: [f32; 3] },
    Scale { child: usize, factor: f32 },
    ScaleNonUniform { child: usize, factors: [f32; 3] },
    Mirror { child: usize, axis: Axis },
    MirrorPlane { child: usize, normal: [f32; 3], offset: f32 },
    Repeat { child: usize, spacing: [f32; 3], count: Option<[u32; 3]> },
    RepeatPolar { child: usize, count: u32 },
    RepeatPoints { child: usize, offsets: Vec<[f32; 3]> },
    Twist { child: usize, amount: f32 },
    Bend { child: usize, amount: f32 },
    DomainWarp { child: usize, warp_x: usize, warp_y: usize, warp_z: usize, amplitude: f32 },
    Tag { child: usize, material_id: u16 },
    Bounded { child: usize, bounds: Aabb },
    Expr {
        source: String,
        input_names: Vec<String>,
        inputs: Vec<usize>,
        bounds: Option<Aabb>,
        distance_kind: DistanceKind,
    },
}

fn encode_field(
    field: &Field,
    ptr_to_id: &mut HashMap<usize, usize>,
    nodes: &mut Vec<WireNode>,
) -> Result<usize, FieldError> {
    let ptr = Arc::as_ptr(&field.0) as usize;
    if let Some(&id) = ptr_to_id.get(&ptr) {
        return Ok(id);
    }
    let mut rec = |f: &Field| encode_field(f, ptr_to_id, nodes);
    let wn = match &*field.0 {
        FieldNode::Const(value) => WireNode::Const { value: *value },
        FieldNode::Coord(axis) => WireNode::Coord { axis: *axis },
        FieldNode::Sphere { radius } => WireNode::Sphere { radius: *radius },
        FieldNode::Box { half_extents, rounding } => WireNode::Box {
            half_extents: *half_extents,
            rounding: *rounding,
        },
        FieldNode::Ellipsoid { radii } => WireNode::Ellipsoid { radii: *radii },
        FieldNode::Torus { major_radius, minor_radius } => WireNode::Torus {
            major_radius: *major_radius,
            minor_radius: *minor_radius,
        },
        FieldNode::CappedTorus { major_radius, minor_radius, cap_angle } => WireNode::CappedTorus {
            major_radius: *major_radius,
            minor_radius: *minor_radius,
            cap_angle: *cap_angle,
        },
        FieldNode::Link { major_radius, minor_radius, half_length } => WireNode::Link {
            major_radius: *major_radius,
            minor_radius: *minor_radius,
            half_length: *half_length,
        },
        FieldNode::Capsule { a, b, radius } => WireNode::Capsule { a: *a, b: *b, radius: *radius },
        FieldNode::RoundCone { a, b, r1, r2 } => WireNode::RoundCone { a: *a, b: *b, r1: *r1, r2: *r2 },
        FieldNode::SolidAngle { radius, angle } => WireNode::SolidAngle { radius: *radius, angle: *angle },
        FieldNode::CutSphere { radius, height } => WireNode::CutSphere { radius: *radius, height: *height },
        FieldNode::CutHollowSphere { radius, height, thickness } => WireNode::CutHollowSphere {
            radius: *radius,
            height: *height,
            thickness: *thickness,
        },
        FieldNode::CappedCylinder { radius, half_height } => WireNode::CappedCylinder {
            radius: *radius,
            half_height: *half_height,
        },
        FieldNode::InfiniteCylinder { radius } => WireNode::InfiniteCylinder { radius: *radius },
        FieldNode::CappedCone { half_height, r1, r2 } => WireNode::CappedCone {
            half_height: *half_height,
            r1: *r1,
            r2: *r2,
        },
        FieldNode::Plane { normal, offset } => WireNode::Plane { normal: *normal, offset: *offset },
        FieldNode::Octahedron { size } => WireNode::Octahedron { size: *size },
        FieldNode::HexPrism { radius, half_height } => WireNode::HexPrism {
            radius: *radius,
            half_height: *half_height,
        },
        FieldNode::SuperPrism { half_extents, exponent } => WireNode::SuperPrism {
            half_extents: *half_extents,
            exponent: *exponent,
        },
        FieldNode::BoxFrame { half_extents, thickness } => WireNode::BoxFrame {
            half_extents: *half_extents,
            thickness: *thickness,
        },
        FieldNode::InfiniteCone { angle } => WireNode::InfiniteCone { angle: *angle },
        FieldNode::SquarePyramid { half_base, height } => WireNode::SquarePyramid {
            half_base: *half_base,
            height: *height,
        },
        FieldNode::Heightmap { base_y, height } => WireNode::Heightmap {
            base_y: *base_y,
            height: rec(height)?,
        },
        FieldNode::Noise2 { basis, frequency, seed } => WireNode::Noise2 {
            basis: *basis,
            frequency: *frequency,
            seed: *seed,
        },
        FieldNode::Noise3 { basis, frequency, seed } => WireNode::Noise3 {
            basis: *basis,
            frequency: *frequency,
            seed: *seed,
        },
        FieldNode::Fbm2 { basis, frequency, seed, octaves, lacunarity, gain, ridged } => WireNode::Fbm2 {
            basis: *basis,
            frequency: *frequency,
            seed: *seed,
            octaves: *octaves,
            lacunarity: *lacunarity,
            gain: *gain,
            ridged: *ridged,
        },
        FieldNode::Fbm3 { basis, frequency, seed, octaves, lacunarity, gain, ridged } => WireNode::Fbm3 {
            basis: *basis,
            frequency: *frequency,
            seed: *seed,
            octaves: *octaves,
            lacunarity: *lacunarity,
            gain: *gain,
            ridged: *ridged,
        },
        FieldNode::Cells { frequency, seed, jitter, mode, threshold } => WireNode::Cells {
            frequency: *frequency,
            seed: *seed,
            jitter: *jitter,
            mode: *mode,
            threshold: *threshold,
        },
        FieldNode::Unary { op, child } => WireNode::Unary { op: *op, child: rec(child)? },
        FieldNode::Binary { op, a, b } => WireNode::Binary { op: *op, a: rec(a)?, b: rec(b)? },
        FieldNode::Clamp { child, lo, hi } => WireNode::Clamp { child: rec(child)?, lo: *lo, hi: *hi },
        FieldNode::Smoothstep { child, edge0, edge1 } => WireNode::Smoothstep {
            child: rec(child)?,
            edge0: *edge0,
            edge1: *edge1,
        },
        FieldNode::Step { child, edge } => WireNode::Step { child: rec(child)?, edge: *edge },
        FieldNode::Remap { child, in_lo, in_hi, out_lo, out_hi, clamp } => WireNode::Remap {
            child: rec(child)?,
            in_lo: *in_lo,
            in_hi: *in_hi,
            out_lo: *out_lo,
            out_hi: *out_hi,
            clamp: *clamp,
        },
        FieldNode::Lerp { a, b, t } => WireNode::Lerp { a: rec(a)?, b: rec(b)?, t: rec(t)? },
        FieldNode::Select { cond, then_field, else_field } => WireNode::Select {
            cond: rec(cond)?,
            then_field: rec(then_field)?,
            else_field: rec(else_field)?,
        },
        FieldNode::Union { children } => {
            let mut ids = Vec::with_capacity(children.len());
            for c in children {
                ids.push(rec(c)?);
            }
            WireNode::Union { children: ids }
        }
        FieldNode::Intersect { children } => {
            let mut ids = Vec::with_capacity(children.len());
            for c in children {
                ids.push(rec(c)?);
            }
            WireNode::Intersect { children: ids }
        }
        FieldNode::Subtract { a, b } => WireNode::Subtract { a: rec(a)?, b: rec(b)? },
        FieldNode::Xor { a, b } => WireNode::Xor { a: rec(a)?, b: rec(b)? },
        FieldNode::SmoothUnion { a, b, k } => WireNode::SmoothUnion { a: rec(a)?, b: rec(b)?, k: *k },
        FieldNode::SmoothSubtract { a, b, k } => WireNode::SmoothSubtract { a: rec(a)?, b: rec(b)?, k: *k },
        FieldNode::SmoothIntersect { a, b, k } => WireNode::SmoothIntersect { a: rec(a)?, b: rec(b)?, k: *k },
        FieldNode::ChamferUnion { a, b, r } => WireNode::ChamferUnion { a: rec(a)?, b: rec(b)?, r: *r },
        FieldNode::ChamferSubtract { a, b, r } => WireNode::ChamferSubtract { a: rec(a)?, b: rec(b)?, r: *r },
        FieldNode::ChamferIntersect { a, b, r } => WireNode::ChamferIntersect { a: rec(a)?, b: rec(b)?, r: *r },
        FieldNode::StepsUnion { a, b, r, steps } => WireNode::StepsUnion {
            a: rec(a)?,
            b: rec(b)?,
            r: *r,
            steps: *steps,
        },
        FieldNode::Round { child, radius } => WireNode::Round { child: rec(child)?, radius: *radius },
        FieldNode::Shell { child, thickness } => WireNode::Shell { child: rec(child)?, thickness: *thickness },
        FieldNode::Elongate { child, half_lengths } => WireNode::Elongate {
            child: rec(child)?,
            half_lengths: *half_lengths,
        },
        FieldNode::Displace { child, offset, amplitude } => WireNode::Displace {
            child: rec(child)?,
            offset: rec(offset)?,
            amplitude: *amplitude,
        },
        FieldNode::Translate { child, offset } => WireNode::Translate { child: rec(child)?, offset: *offset },
        FieldNode::Rotate { child, angles } => WireNode::Rotate { child: rec(child)?, angles: *angles },
        FieldNode::Scale { child, factor } => WireNode::Scale { child: rec(child)?, factor: *factor },
        FieldNode::ScaleNonUniform { child, factors } => WireNode::ScaleNonUniform {
            child: rec(child)?,
            factors: *factors,
        },
        FieldNode::Mirror { child, axis } => WireNode::Mirror { child: rec(child)?, axis: *axis },
        FieldNode::MirrorPlane { child, normal, offset } => WireNode::MirrorPlane {
            child: rec(child)?,
            normal: *normal,
            offset: *offset,
        },
        FieldNode::Repeat { child, spacing, count } => WireNode::Repeat {
            child: rec(child)?,
            spacing: *spacing,
            count: *count,
        },
        FieldNode::RepeatPolar { child, count } => WireNode::RepeatPolar { child: rec(child)?, count: *count },
        FieldNode::RepeatPoints { child, offsets } => WireNode::RepeatPoints {
            child: rec(child)?,
            offsets: offsets.clone(),
        },
        FieldNode::Twist { child, amount } => WireNode::Twist { child: rec(child)?, amount: *amount },
        FieldNode::Bend { child, amount } => WireNode::Bend { child: rec(child)?, amount: *amount },
        FieldNode::DomainWarp { child, warp_x, warp_y, warp_z, amplitude } => WireNode::DomainWarp {
            child: rec(child)?,
            warp_x: rec(warp_x)?,
            warp_y: rec(warp_y)?,
            warp_z: rec(warp_z)?,
            amplitude: *amplitude,
        },
        FieldNode::Tag { child, material_id } => WireNode::Tag {
            child: rec(child)?,
            material_id: *material_id,
        },
        FieldNode::Bounded { child, bounds } => WireNode::Bounded {
            child: rec(child)?,
            bounds: *bounds,
        },
        FieldNode::Expr { program, inputs, bounds, distance_kind } => {
            let mut ids = Vec::with_capacity(inputs.len());
            for inp in inputs {
                ids.push(rec(inp)?);
            }
            WireNode::Expr {
                source: program.source.clone(),
                input_names: program.input_names.clone(),
                inputs: ids,
                bounds: *bounds,
                distance_kind: *distance_kind,
            }
        }
    };
    let id = nodes.len();
    nodes.push(wn);
    ptr_to_id.insert(ptr, id);
    Ok(id)
}

fn decode_node(wn: WireNode, idx: usize, decoded: &[Field]) -> Result<Field, FieldError> {
    let get = |id: usize| -> Result<Field, FieldError> {
        if id >= idx {
            return Err(FieldError::Parse(format!(
                "non-topological or out-of-range node reference {id} at node {idx}"
            )));
        }
        Ok(decoded[id].clone())
    };
    match wn {
        WireNode::Const { value } => Field::constant(value),
        WireNode::Coord { axis } => Ok(Field::wrap(FieldNode::Coord(axis))),
        WireNode::Sphere { radius } => Field::sphere(radius),
        WireNode::Box { half_extents, rounding } => Field::box_shape(half_extents, rounding),
        WireNode::Ellipsoid { radii } => Field::ellipsoid(radii),
        WireNode::Torus { major_radius, minor_radius } => Field::torus(major_radius, minor_radius),
        WireNode::CappedTorus { major_radius, minor_radius, cap_angle } => {
            Field::capped_torus(major_radius, minor_radius, cap_angle)
        }
        WireNode::Link { major_radius, minor_radius, half_length } => {
            Field::link(major_radius, minor_radius, half_length)
        }
        WireNode::Capsule { a, b, radius } => Field::capsule(a, b, radius),
        WireNode::RoundCone { a, b, r1, r2 } => Field::round_cone(a, b, r1, r2),
        WireNode::SolidAngle { radius, angle } => Field::solid_angle(radius, angle),
        WireNode::CutSphere { radius, height } => Field::cut_sphere(radius, height),
        WireNode::CutHollowSphere { radius, height, thickness } => {
            Field::cut_hollow_sphere(radius, height, thickness)
        }
        WireNode::CappedCylinder { radius, half_height } => Field::capped_cylinder(radius, half_height),
        WireNode::InfiniteCylinder { radius } => Field::infinite_cylinder(radius),
        WireNode::CappedCone { half_height, r1, r2 } => Field::capped_cone(half_height, r1, r2),
        WireNode::Plane { normal, offset } => Field::plane(normal, offset),
        WireNode::Octahedron { size } => Field::octahedron(size),
        WireNode::HexPrism { radius, half_height } => Field::hex_prism(radius, half_height),
        WireNode::SuperPrism { half_extents, exponent } => Field::super_prism(half_extents, exponent),
        WireNode::BoxFrame { half_extents, thickness } => Field::box_frame(half_extents, thickness),
        WireNode::InfiniteCone { angle } => Field::infinite_cone(angle),
        WireNode::SquarePyramid { half_base, height } => Field::square_pyramid(half_base, height),
        WireNode::Heightmap { base_y, height } => Field::heightmap(base_y, get(height)?),
        WireNode::Noise2 { basis, frequency, seed } => match basis {
            NoiseBasis::Value => Field::value_noise2(frequency, seed),
            NoiseBasis::Perlin => Field::perlin2(frequency, seed),
        },
        WireNode::Noise3 { basis, frequency, seed } => match basis {
            NoiseBasis::Value => Field::value_noise3(frequency, seed),
            NoiseBasis::Perlin => Field::perlin3(frequency, seed),
        },
        WireNode::Fbm2 { basis, frequency, seed, octaves, lacunarity, gain, ridged } => {
            Field::fbm2(basis, frequency, seed, octaves, lacunarity, gain, ridged)
        }
        WireNode::Fbm3 { basis, frequency, seed, octaves, lacunarity, gain, ridged } => {
            Field::fbm3(basis, frequency, seed, octaves, lacunarity, gain, ridged)
        }
        WireNode::Cells { frequency, seed, jitter, mode, threshold } => {
            Field::cells(frequency, seed, jitter, mode, threshold)
        }
        WireNode::Unary { op, child } => Ok(get(child)?.unary(op)),
        WireNode::Binary { op, a, b } => Ok(get(a)?.binary(op, &get(b)?)),
        WireNode::Clamp { child, lo, hi } => get(child)?.clamp(lo, hi),
        WireNode::Smoothstep { child, edge0, edge1 } => get(child)?.smoothstep(edge0, edge1),
        WireNode::Step { child, edge } => get(child)?.step(edge),
        WireNode::Remap { child, in_lo, in_hi, out_lo, out_hi, clamp } => {
            get(child)?.remap(in_lo, in_hi, out_lo, out_hi, clamp)
        }
        WireNode::Lerp { a, b, t } => Ok(get(a)?.lerp(&get(b)?, &get(t)?)),
        WireNode::Select { cond, then_field, else_field } => {
            Ok(Field::select(&get(cond)?, &get(then_field)?, &get(else_field)?))
        }
        WireNode::Union { children } => {
            let mut fs = Vec::with_capacity(children.len());
            for c in children {
                fs.push(get(c)?);
            }
            Field::union_all(fs)
        }
        WireNode::Intersect { children } => {
            let mut fs = Vec::with_capacity(children.len());
            for c in children {
                fs.push(get(c)?);
            }
            Field::intersect_all(fs)
        }
        WireNode::Subtract { a, b } => Ok(get(a)?.subtract(&get(b)?)),
        WireNode::Xor { a, b } => Ok(get(a)?.xor(&get(b)?)),
        WireNode::SmoothUnion { a, b, k } => get(a)?.smooth_union(&get(b)?, k),
        WireNode::SmoothSubtract { a, b, k } => get(a)?.smooth_subtract(&get(b)?, k),
        WireNode::SmoothIntersect { a, b, k } => get(a)?.smooth_intersect(&get(b)?, k),
        WireNode::ChamferUnion { a, b, r } => get(a)?.chamfer_union(&get(b)?, r),
        WireNode::ChamferSubtract { a, b, r } => get(a)?.chamfer_subtract(&get(b)?, r),
        WireNode::ChamferIntersect { a, b, r } => get(a)?.chamfer_intersect(&get(b)?, r),
        WireNode::StepsUnion { a, b, r, steps } => get(a)?.steps_union(&get(b)?, r, steps),
        WireNode::Round { child, radius } => get(child)?.round(radius),
        WireNode::Shell { child, thickness } => get(child)?.shell(thickness),
        WireNode::Elongate { child, half_lengths } => get(child)?.elongate(half_lengths),
        WireNode::Displace { child, offset, amplitude } => get(child)?.displace(&get(offset)?, amplitude),
        WireNode::Translate { child, offset } => get(child)?.translate(offset),
        WireNode::Rotate { child, angles } => get(child)?.rotate(angles),
        WireNode::Scale { child, factor } => get(child)?.scale(factor),
        WireNode::ScaleNonUniform { child, factors } => get(child)?.scale_xyz(factors),
        WireNode::Mirror { child, axis } => Ok(get(child)?.mirror(axis)),
        WireNode::MirrorPlane { child, normal, offset } => get(child)?.mirror_plane(normal, offset),
        WireNode::Repeat { child, spacing, count } => match count {
            Some(c) => get(child)?.repeat_counted(spacing, c),
            None => get(child)?.repeat_infinite(spacing),
        },
        WireNode::RepeatPolar { child, count } => get(child)?.repeat_polar(count),
        WireNode::RepeatPoints { child, offsets } => get(child)?.repeat_points(offsets),
        WireNode::Twist { child, amount } => get(child)?.twist(amount),
        WireNode::Bend { child, amount } => get(child)?.bend(amount),
        WireNode::DomainWarp { child, warp_x, warp_y, warp_z, amplitude } => {
            get(child)?.warp(&get(warp_x)?, &get(warp_y)?, &get(warp_z)?, amplitude)
        }
        WireNode::Tag { child, material_id } => get(child)?.tag(material_id),
        WireNode::Bounded { child, bounds } => get(child)?.bounded(bounds),
        WireNode::Expr { source, input_names, inputs, bounds, distance_kind } => {
            if input_names.len() != inputs.len() {
                return Err(FieldError::Parse(
                    "expr input_names and inputs length mismatch".into(),
                ));
            }
            let mut resolved = Vec::with_capacity(inputs.len());
            for (name, id) in input_names.iter().zip(inputs.into_iter()) {
                resolved.push((name.as_str(), get(id)?));
            }
            Field::expr_with(&source, &resolved, bounds, distance_kind)
        }
    }
}

// ── Shared Math & Validation Helpers ─────────────────────────────────────────

pub(crate) fn finite_all(values: &[f32]) -> Result<(), FieldError> {
    if values.iter().all(|v| v.is_finite()) {
        Ok(())
    } else {
        Err(FieldError::InvalidArgument("value must be finite".into()))
    }
}

pub(crate) fn positive_all(values: &[f32]) -> Result<(), FieldError> {
    finite_all(values)?;
    if values.iter().all(|v| *v > 0.0) {
        Ok(())
    } else {
        Err(FieldError::InvalidArgument("value must be positive".into()))
    }
}

pub(crate) fn non_negative_all(values: &[f32]) -> Result<(), FieldError> {
    finite_all(values)?;
    if values.iter().all(|v| *v >= 0.0) {
        Ok(())
    } else {
        Err(FieldError::InvalidArgument(
            "value must be non-negative".into(),
        ))
    }
}

fn validate_frequency(frequency: f32, octaves: u32, lacunarity: f32) -> Result<(), FieldError> {
    positive_all(&[frequency, lacunarity])?;
    if !(1..=8).contains(&octaves) {
        return Err(FieldError::InvalidArgument(
            "octaves must be in 1..=8".into(),
        ));
    }
    let max_freq = (frequency as f64) * (lacunarity as f64).powi((octaves - 1) as i32);
    if !max_freq.is_finite() || max_freq > MAX_SAFE_LATTICE_COORD {
        return Err(FieldError::InvalidArgument(
            "noise frequency exceeds safe lattice range".into(),
        ));
    }
    Ok(())
}

fn validate_fbm(frequency: f32, octaves: u32, lacunarity: f32, gain: f32) -> Result<(), FieldError> {
    validate_frequency(frequency, octaves, lacunarity)?;
    positive_all(&[gain])?;
    if gain > 1.0 {
        return Err(FieldError::InvalidArgument("FBM gain must be <= 1.0".into()));
    }
    Ok(())
}

pub(crate) fn normalize_f64(v: [f32; 3]) -> Result<[f32; 3], FieldError> {
    let len = ((v[0] as f64).powi(2) + (v[1] as f64).powi(2) + (v[2] as f64).powi(2)).sqrt();
    if !len.is_finite() || len <= f64::from(f32::EPSILON) {
        return Err(FieldError::InvalidArgument(
            "normal vector must be non-zero".into(),
        ));
    }
    let out = [
        (v[0] as f64 / len) as f32,
        (v[1] as f64 / len) as f32,
        (v[2] as f64 / len) as f32,
    ];
    finite_all(&out)?;
    Ok(out)
}

/// Column-major 3×3 Euler rotation matrix (`R = Rz * Ry * Rx`, degrees).
pub(crate) fn rot_matrix(deg: [f32; 3]) -> [[f32; 3]; 3] {
    let (sx, cx) = deg[0].to_radians().sin_cos();
    let (sy, cy) = deg[1].to_radians().sin_cos();
    let (sz, cz) = deg[2].to_radians().sin_cos();
    [
        [cz * cy, cz * sy * sx - sz * cx, cz * sy * cx + sz * sx],
        [sz * cy, sz * sy * sx + cz * cx, sz * sy * cx - cz * sx],
        [-sy, cy * sx, cy * cx],
    ]
}

#[inline]
pub(crate) fn rotate_point(m: &[[f32; 3]; 3], p: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2],
        m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2],
        m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2],
    ]
}

#[inline]
pub(crate) fn inv_rotate(m: &[[f32; 3]; 3], p: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * p[0] + m[1][0] * p[1] + m[2][0] * p[2],
        m[0][1] * p[0] + m[1][1] * p[1] + m[2][1] * p[2],
        m[0][2] * p[0] + m[1][2] * p[1] + m[2][2] * p[2],
    ]
}
