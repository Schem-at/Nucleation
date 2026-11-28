//! Flat CSE-deduplicated DAG compiler and evaluator.
//!
//! Compiles one or more [`Field`] roots into a single linear instruction array
//! (`CompiledGraph`) with coordinate-space tracking so shared subexpressions
//! (e.g. a noise field driving both surface displacement and material coloring)
//! are evaluated **only once** per voxel.
//!
//! Supports four evaluation modes over the exact same compiled representation:
//! 1. Fast primal `f32` (`eval`)
//! 2. Multi-material tagged evaluation (`eval_tagged`) propagating `material_id`
//!    and smooth-CSG seam blend weights
//! 3. Conservative 3D box interval evaluation (`eval_interval`) for octree/brick pruning
//! 4. Forward-mode dual-number gradient (`eval_dual`) + ULP-safe numerical fallback (`normal`)

use super::expr::{Dual3, ExprProgram, ScalarNum};
use super::field::{
    inv_rotate, rot_matrix, Aabb, Axis, BinaryMathOp, Field, FieldNode, NoiseBasis, UnaryMathOp,
};
use super::interval::{Interval, Interval3};
use super::noise::{fbm2_params, fbm3_params, perlin2, perlin3, value_noise2, value_noise3, worley3, CellMode};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

/// Output of evaluating a compiled SDF with material-tag propagation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TaggedSample {
    pub distance: f32,
    pub material_id: u16,
    pub blend_material_id: u16,
    /// Blend weight toward `blend_material_id` in `[0.0, 0.5]`.
    pub blend_weight: f32,
}

impl TaggedSample {
    #[inline]
    pub fn plain(distance: f32) -> Self {
        Self {
            distance,
            material_id: 0,
            blend_material_id: 0,
            blend_weight: 0.0,
        }
    }
}

/// Coordinate transform producing a new `Vec3` register `p_dst` from `p_src`.
#[derive(Debug, Clone)]
enum CoordOp {
    Translate { src: u32, offset: [f32; 3] },
    InvRotate { src: u32, mat: [[f32; 3]; 3] },
    InvScale { src: u32, inv_factor: f32 },
    InvScaleNonUniform { src: u32, inv_factors: [f32; 3] },
    Mirror { src: u32, axis: Axis },
    MirrorPlane { src: u32, normal: [f32; 3], offset: f32 },
    Repeat { src: u32, spacing: [f32; 3], count: Option<[u32; 3]> },
    RepeatPolar { src: u32, sector: f32 },
    Elongate { src: u32, half_lengths: [f32; 3] },
    Twist { src: u32, amount: f32 },
    Bend { src: u32, amount: f32 },
    ProjectXZ { src: u32 },
    Warp { src: u32, wx: u32, wy: u32, wz: u32, amplitude: f32 },
}

/// One flat instruction in the compiled DAG.
#[derive(Debug, Clone)]
enum CompiledInstr {
    /// Writes coordinate register `coord_dst`.
    Coord { dst: u32, op: CoordOp },
    /// Writes scalar/tagged register `dst`.
    Eval { dst: u32, coord: u32, op: EvalOp },
}

#[derive(Debug, Clone)]
enum EvalOp {
    Const(f32),
    CoordAxis(Axis),
    Sphere { radius: f32 },
    Box { half_extents: [f32; 3], rounding: f32 },
    Ellipsoid { radii: [f32; 3] },
    Torus { major_radius: f32, minor_radius: f32 },
    CappedTorus { major_radius: f32, minor_radius: f32, sin_a: f32, cos_a: f32 },
    Link { major_radius: f32, minor_radius: f32, half_length: f32 },
    Capsule { a: [f32; 3], b: [f32; 3], radius: f32 },
    RoundCone { a: [f32; 3], b: [f32; 3], r1: f32, r2: f32 },
    SolidAngle { radius: f32, sin_a: f32, cos_a: f32 },
    CutSphere { radius: f32, height: f32, rim_w: f32 },
    CutHollowSphere { radius: f32, height: f32, rim_w: f32, thickness: f32 },
    CappedCylinder { radius: f32, half_height: f32 },
    InfiniteCylinder { radius: f32 },
    CappedCone { half_height: f32, r1: f32, r2: f32 },
    Plane { normal: [f32; 3], offset: f32 },
    Octahedron { size: f32 },
    HexPrism { radius: f32, half_height: f32 },
    SuperPrism { half_extents: [f32; 3], exponent: f32 },
    BoxFrame { half_extents: [f32; 3], thickness: f32 },
    InfiniteCone { sin_a: f32, cos_a: f32 },
    SquarePyramid { half_base: f32, height: f32 },
    Heightmap { base_y: f32, height_reg: u32 },
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
    Unary { op: UnaryMathOp, child: u32 },
    Binary { op: BinaryMathOp, a: u32, b: u32 },
    Clamp { child: u32, lo: f32, hi: f32 },
    Smoothstep { child: u32, edge0: f32, edge1: f32 },
    Step { child: u32, edge: f32 },
    Remap { child: u32, in_lo: f32, in_hi: f32, out_lo: f32, out_hi: f32, clamp: bool },
    Lerp { a: u32, b: u32, t: u32 },
    Select { cond: u32, then_r: u32, else_r: u32 },
    Union { children: Vec<u32> },
    Intersect { children: Vec<u32> },
    Subtract { a: u32, b: u32 },
    Xor { a: u32, b: u32 },
    SmoothUnion { a: u32, b: u32, k: f32 },
    SmoothSubtract { a: u32, b: u32, k: f32 },
    SmoothIntersect { a: u32, b: u32, k: f32 },
    ChamferUnion { a: u32, b: u32, r: f32 },
    ChamferSubtract { a: u32, b: u32, r: f32 },
    ChamferIntersect { a: u32, b: u32, r: f32 },
    StepsUnion { a: u32, b: u32, r: f32, steps: f32 },
    Round { child: u32, radius: f32 },
    Shell { child: u32, thickness: f32 },
    ElongateFinish { child: u32, half_lengths: [f32; 3] },
    ScaleDist { child: u32, factor: f32 },
    Displace { child: u32, offset: u32, amplitude: f32, offset_range: Option<[f32; 2]> },
    RepeatPoints { child: CompiledField, offsets: Vec<[f32; 3]> },
    Tag { child: u32, material_id: u16 },
    Expr { program: Arc<ExprProgram>, inputs: Vec<CompiledField> },
}

/// A compiled multi-root DAG: root `0..N` all share intermediate registers via CSE.
#[derive(Debug, Clone)]
pub struct CompiledGraph {
    instrs: Vec<CompiledInstr>,
    num_coords: usize,
    num_vals: usize,
    roots: Vec<u32>,
    has_tags: bool,
}

/// A compiled single-root field ready for fast evaluation, brick interval pruning,
/// and analytic/numerical normal estimation.
#[derive(Debug, Clone)]
pub struct CompiledField {
    graph: CompiledGraph,
    bounds: Option<Aabb>,
}

struct GraphCompiler {
    instrs: Vec<CompiledInstr>,
    num_coords: u32,
    num_vals: u32,
    /// CSE map: `(coord_space, node_ptr)` -> value register
    memo: HashMap<(u32, usize), u32>,
    has_tags: bool,
}

impl GraphCompiler {
    fn new() -> Self {
        Self {
            instrs: Vec::new(),
            num_coords: 1, // register 0 is always world position (x, y, z)
            num_vals: 0,
            memo: HashMap::new(),
            has_tags: false,
        }
    }

    fn alloc_coord(&mut self, op: CoordOp) -> u32 {
        let dst = self.num_coords;
        self.num_coords += 1;
        self.instrs.push(CompiledInstr::Coord { dst, op });
        dst
    }

    fn alloc_eval(&mut self, coord: u32, op: EvalOp) -> u32 {
        let dst = self.num_vals;
        self.num_vals += 1;
        self.instrs.push(CompiledInstr::Eval { dst, coord, op });
        dst
    }

    fn compile_node(&mut self, field: &Field, coord: u32) -> u32 {
        let ptr = Arc::as_ptr(&field.0) as usize;
        if let Some(&reg) = self.memo.get(&(coord, ptr)) {
            return reg;
        }
        let reg = match &*field.0 {
            FieldNode::Const(v) => self.alloc_eval(coord, EvalOp::Const(*v)),
            FieldNode::Coord(axis) => self.alloc_eval(coord, EvalOp::CoordAxis(*axis)),
            FieldNode::Sphere { radius } => self.alloc_eval(coord, EvalOp::Sphere { radius: *radius }),
            FieldNode::Box { half_extents, rounding } => self.alloc_eval(
                coord,
                EvalOp::Box {
                    half_extents: *half_extents,
                    rounding: *rounding,
                },
            ),
            FieldNode::Ellipsoid { radii } => {
                self.alloc_eval(coord, EvalOp::Ellipsoid { radii: *radii })
            }
            FieldNode::Torus { major_radius, minor_radius } => self.alloc_eval(
                coord,
                EvalOp::Torus {
                    major_radius: *major_radius,
                    minor_radius: *minor_radius,
                },
            ),
            FieldNode::CappedTorus { major_radius, minor_radius, cap_angle } => {
                let (sin_a, cos_a) = cap_angle.to_radians().sin_cos();
                self.alloc_eval(
                    coord,
                    EvalOp::CappedTorus {
                        major_radius: *major_radius,
                        minor_radius: *minor_radius,
                        sin_a,
                        cos_a,
                    },
                )
            }
            FieldNode::Link { major_radius, minor_radius, half_length } => self.alloc_eval(
                coord,
                EvalOp::Link {
                    major_radius: *major_radius,
                    minor_radius: *minor_radius,
                    half_length: *half_length,
                },
            ),
            FieldNode::Capsule { a, b, radius } => self.alloc_eval(
                coord,
                EvalOp::Capsule {
                    a: *a,
                    b: *b,
                    radius: *radius,
                },
            ),
            FieldNode::RoundCone { a, b, r1, r2 } => self.alloc_eval(
                coord,
                EvalOp::RoundCone {
                    a: *a,
                    b: *b,
                    r1: *r1,
                    r2: *r2,
                },
            ),
            FieldNode::SolidAngle { radius, angle } => {
                let (sin_a, cos_a) = angle.to_radians().sin_cos();
                self.alloc_eval(
                    coord,
                    EvalOp::SolidAngle {
                        radius: *radius,
                        sin_a,
                        cos_a,
                    },
                )
            }
            FieldNode::CutSphere { radius, height } => {
                let rim_w = (radius * radius - height * height).max(0.0).sqrt();
                self.alloc_eval(
                    coord,
                    EvalOp::CutSphere {
                        radius: *radius,
                        height: *height,
                        rim_w,
                    },
                )
            }
            FieldNode::CutHollowSphere { radius, height, thickness } => {
                let rim_w = (radius * radius - height * height).max(0.0).sqrt();
                self.alloc_eval(
                    coord,
                    EvalOp::CutHollowSphere {
                        radius: *radius,
                        height: *height,
                        rim_w,
                        thickness: *thickness,
                    },
                )
            }
            FieldNode::CappedCylinder { radius, half_height } => self.alloc_eval(
                coord,
                EvalOp::CappedCylinder {
                    radius: *radius,
                    half_height: *half_height,
                },
            ),
            FieldNode::InfiniteCylinder { radius } => {
                self.alloc_eval(coord, EvalOp::InfiniteCylinder { radius: *radius })
            }
            FieldNode::CappedCone { half_height, r1, r2 } => self.alloc_eval(
                coord,
                EvalOp::CappedCone {
                    half_height: *half_height,
                    r1: *r1,
                    r2: *r2,
                },
            ),
            FieldNode::Plane { normal, offset } => self.alloc_eval(
                coord,
                EvalOp::Plane {
                    normal: *normal,
                    offset: *offset,
                },
            ),
            FieldNode::Octahedron { size } => {
                self.alloc_eval(coord, EvalOp::Octahedron { size: *size })
            }
            FieldNode::HexPrism { radius, half_height } => self.alloc_eval(
                coord,
                EvalOp::HexPrism {
                    radius: *radius,
                    half_height: *half_height,
                },
            ),
            FieldNode::SuperPrism { half_extents, exponent } => self.alloc_eval(
                coord,
                EvalOp::SuperPrism {
                    half_extents: *half_extents,
                    exponent: *exponent,
                },
            ),
            FieldNode::BoxFrame { half_extents, thickness } => self.alloc_eval(
                coord,
                EvalOp::BoxFrame {
                    half_extents: *half_extents,
                    thickness: *thickness,
                },
            ),
            FieldNode::InfiniteCone { angle } => {
                let (sin_a, cos_a) = angle.to_radians().sin_cos();
                self.alloc_eval(coord, EvalOp::InfiniteCone { sin_a, cos_a })
            }
            FieldNode::SquarePyramid { half_base, height } => self.alloc_eval(
                coord,
                EvalOp::SquarePyramid {
                    half_base: *half_base,
                    height: *height,
                },
            ),
            FieldNode::Heightmap { base_y, height } => {
                let xz_coord = self.alloc_coord(CoordOp::ProjectXZ { src: coord });
                let h_reg = self.compile_node(height, xz_coord);
                self.alloc_eval(
                    coord,
                    EvalOp::Heightmap {
                        base_y: *base_y,
                        height_reg: h_reg,
                    },
                )
            }
            FieldNode::Noise2 { basis, frequency, seed } => self.alloc_eval(
                coord,
                EvalOp::Noise2 {
                    basis: *basis,
                    frequency: *frequency,
                    seed: *seed,
                },
            ),
            FieldNode::Noise3 { basis, frequency, seed } => self.alloc_eval(
                coord,
                EvalOp::Noise3 {
                    basis: *basis,
                    frequency: *frequency,
                    seed: *seed,
                },
            ),
            FieldNode::Fbm2 {
                basis,
                frequency,
                seed,
                octaves,
                lacunarity,
                gain,
                ridged,
            } => self.alloc_eval(
                coord,
                EvalOp::Fbm2 {
                    basis: *basis,
                    frequency: *frequency,
                    seed: *seed,
                    octaves: *octaves,
                    lacunarity: *lacunarity,
                    gain: *gain,
                    ridged: *ridged,
                },
            ),
            FieldNode::Fbm3 {
                basis,
                frequency,
                seed,
                octaves,
                lacunarity,
                gain,
                ridged,
            } => self.alloc_eval(
                coord,
                EvalOp::Fbm3 {
                    basis: *basis,
                    frequency: *frequency,
                    seed: *seed,
                    octaves: *octaves,
                    lacunarity: *lacunarity,
                    gain: *gain,
                    ridged: *ridged,
                },
            ),
            FieldNode::Cells {
                frequency,
                seed,
                jitter,
                mode,
                threshold,
            } => self.alloc_eval(
                coord,
                EvalOp::Cells {
                    frequency: *frequency,
                    seed: *seed,
                    jitter: *jitter,
                    mode: *mode,
                    threshold: *threshold,
                },
            ),
            FieldNode::Unary { op, child } => {
                let c = self.compile_node(child, coord);
                self.alloc_eval(coord, EvalOp::Unary { op: *op, child: c })
            }
            FieldNode::Binary { op, a, b } => {
                let ra = self.compile_node(a, coord);
                let rb = self.compile_node(b, coord);
                self.alloc_eval(coord, EvalOp::Binary { op: *op, a: ra, b: rb })
            }
            FieldNode::Clamp { child, lo, hi } => {
                let c = self.compile_node(child, coord);
                self.alloc_eval(
                    coord,
                    EvalOp::Clamp {
                        child: c,
                        lo: *lo,
                        hi: *hi,
                    },
                )
            }
            FieldNode::Smoothstep { child, edge0, edge1 } => {
                let c = self.compile_node(child, coord);
                self.alloc_eval(
                    coord,
                    EvalOp::Smoothstep {
                        child: c,
                        edge0: *edge0,
                        edge1: *edge1,
                    },
                )
            }
            FieldNode::Step { child, edge } => {
                let c = self.compile_node(child, coord);
                self.alloc_eval(
                    coord,
                    EvalOp::Step {
                        child: c,
                        edge: *edge,
                    },
                )
            }
            FieldNode::Remap {
                child,
                in_lo,
                in_hi,
                out_lo,
                out_hi,
                clamp,
            } => {
                let c = self.compile_node(child, coord);
                self.alloc_eval(
                    coord,
                    EvalOp::Remap {
                        child: c,
                        in_lo: *in_lo,
                        in_hi: *in_hi,
                        out_lo: *out_lo,
                        out_hi: *out_hi,
                        clamp: *clamp,
                    },
                )
            }
            FieldNode::Lerp { a, b, t } => {
                let ra = self.compile_node(a, coord);
                let rb = self.compile_node(b, coord);
                let rt = self.compile_node(t, coord);
                self.alloc_eval(coord, EvalOp::Lerp { a: ra, b: rb, t: rt })
            }
            FieldNode::Select {
                cond,
                then_field,
                else_field,
            } => {
                let rc = self.compile_node(cond, coord);
                let rt = self.compile_node(then_field, coord);
                let re = self.compile_node(else_field, coord);
                self.alloc_eval(
                    coord,
                    EvalOp::Select {
                        cond: rc,
                        then_r: rt,
                        else_r: re,
                    },
                )
            }
            FieldNode::Union { children } => {
                let cs = children.iter().map(|c| self.compile_node(c, coord)).collect();
                self.alloc_eval(coord, EvalOp::Union { children: cs })
            }
            FieldNode::Intersect { children } => {
                let cs = children.iter().map(|c| self.compile_node(c, coord)).collect();
                self.alloc_eval(coord, EvalOp::Intersect { children: cs })
            }
            FieldNode::Subtract { a, b } => {
                let ra = self.compile_node(a, coord);
                let rb = self.compile_node(b, coord);
                self.alloc_eval(coord, EvalOp::Subtract { a: ra, b: rb })
            }
            FieldNode::Xor { a, b } => {
                let ra = self.compile_node(a, coord);
                let rb = self.compile_node(b, coord);
                self.alloc_eval(coord, EvalOp::Xor { a: ra, b: rb })
            }
            FieldNode::SmoothUnion { a, b, k } => {
                let ra = self.compile_node(a, coord);
                let rb = self.compile_node(b, coord);
                self.alloc_eval(coord, EvalOp::SmoothUnion { a: ra, b: rb, k: *k })
            }
            FieldNode::SmoothSubtract { a, b, k } => {
                let ra = self.compile_node(a, coord);
                let rb = self.compile_node(b, coord);
                self.alloc_eval(coord, EvalOp::SmoothSubtract { a: ra, b: rb, k: *k })
            }
            FieldNode::SmoothIntersect { a, b, k } => {
                let ra = self.compile_node(a, coord);
                let rb = self.compile_node(b, coord);
                self.alloc_eval(coord, EvalOp::SmoothIntersect { a: ra, b: rb, k: *k })
            }
            FieldNode::ChamferUnion { a, b, r } => {
                let ra = self.compile_node(a, coord);
                let rb = self.compile_node(b, coord);
                self.alloc_eval(coord, EvalOp::ChamferUnion { a: ra, b: rb, r: *r })
            }
            FieldNode::ChamferSubtract { a, b, r } => {
                let ra = self.compile_node(a, coord);
                let rb = self.compile_node(b, coord);
                self.alloc_eval(coord, EvalOp::ChamferSubtract { a: ra, b: rb, r: *r })
            }
            FieldNode::ChamferIntersect { a, b, r } => {
                let ra = self.compile_node(a, coord);
                let rb = self.compile_node(b, coord);
                self.alloc_eval(coord, EvalOp::ChamferIntersect { a: ra, b: rb, r: *r })
            }
            FieldNode::StepsUnion { a, b, r, steps } => {
                let ra = self.compile_node(a, coord);
                let rb = self.compile_node(b, coord);
                self.alloc_eval(
                    coord,
                    EvalOp::StepsUnion {
                        a: ra,
                        b: rb,
                        r: *r,
                        steps: *steps,
                    },
                )
            }
            FieldNode::Round { child, radius } => {
                let c = self.compile_node(child, coord);
                self.alloc_eval(
                    coord,
                    EvalOp::Round {
                        child: c,
                        radius: *radius,
                    },
                )
            }
            FieldNode::Shell { child, thickness } => {
                let c = self.compile_node(child, coord);
                self.alloc_eval(
                    coord,
                    EvalOp::Shell {
                        child: c,
                        thickness: *thickness,
                    },
                )
            }
            FieldNode::Elongate { child, half_lengths } => {
                let next_c = self.alloc_coord(CoordOp::Elongate {
                    src: coord,
                    half_lengths: *half_lengths,
                });
                let rc = self.compile_node(child, next_c);
                self.alloc_eval(
                    coord,
                    EvalOp::ElongateFinish {
                        child: rc,
                        half_lengths: *half_lengths,
                    },
                )
            }
            FieldNode::Displace {
                child,
                offset,
                amplitude,
            } => {
                let rc = self.compile_node(child, coord);
                let ro = self.compile_node(offset, coord);
                self.alloc_eval(
                    coord,
                    EvalOp::Displace {
                        child: rc,
                        offset: ro,
                        amplitude: *amplitude,
                        offset_range: offset.output_range(),
                    },
                )
            }
            FieldNode::Translate { child, offset } => {
                let next_c = self.alloc_coord(CoordOp::Translate {
                    src: coord,
                    offset: *offset,
                });
                self.compile_node(child, next_c)
            }
            FieldNode::Rotate { child, angles } => {
                let next_c = self.alloc_coord(CoordOp::InvRotate {
                    src: coord,
                    mat: rot_matrix(*angles),
                });
                self.compile_node(child, next_c)
            }
            FieldNode::Scale { child, factor } => {
                let next_c = self.alloc_coord(CoordOp::InvScale {
                    src: coord,
                    inv_factor: 1.0 / *factor,
                });
                let rc = self.compile_node(child, next_c);
                self.alloc_eval(
                    coord,
                    EvalOp::ScaleDist {
                        child: rc,
                        factor: factor.abs(),
                    },
                )
            }
            FieldNode::ScaleNonUniform { child, factors } => {
                let next_c = self.alloc_coord(CoordOp::InvScaleNonUniform {
                    src: coord,
                    inv_factors: [1.0 / factors[0], 1.0 / factors[1], 1.0 / factors[2]],
                });
                let rc = self.compile_node(child, next_c);
                let min_s = factors[0].min(factors[1]).min(factors[2]).abs();
                self.alloc_eval(
                    coord,
                    EvalOp::ScaleDist {
                        child: rc,
                        factor: min_s,
                    },
                )
            }
            FieldNode::Mirror { child, axis } => {
                let next_c = self.alloc_coord(CoordOp::Mirror {
                    src: coord,
                    axis: *axis,
                });
                self.compile_node(child, next_c)
            }
            FieldNode::MirrorPlane {
                child,
                normal,
                offset,
            } => {
                let next_c = self.alloc_coord(CoordOp::MirrorPlane {
                    src: coord,
                    normal: *normal,
                    offset: *offset,
                });
                self.compile_node(child, next_c)
            }
            FieldNode::Repeat {
                child,
                spacing,
                count,
            } => {
                let next_c = self.alloc_coord(CoordOp::Repeat {
                    src: coord,
                    spacing: *spacing,
                    count: *count,
                });
                self.compile_node(child, next_c)
            }
            FieldNode::RepeatPolar { child, count } => {
                let next_c = self.alloc_coord(CoordOp::RepeatPolar {
                    src: coord,
                    sector: std::f32::consts::TAU / (*count as f32),
                });
                self.compile_node(child, next_c)
            }
            FieldNode::RepeatPoints { child, offsets } => {
                let compiled_child = CompiledField::compile(child);
                if compiled_child.graph.has_tags {
                    self.has_tags = true;
                }
                self.alloc_eval(
                    coord,
                    EvalOp::RepeatPoints {
                        child: compiled_child,
                        offsets: offsets.clone(),
                    },
                )
            }
            FieldNode::Twist { child, amount } => {
                let next_c = self.alloc_coord(CoordOp::Twist {
                    src: coord,
                    amount: *amount,
                });
                self.compile_node(child, next_c)
            }
            FieldNode::Bend { child, amount } => {
                let next_c = self.alloc_coord(CoordOp::Bend {
                    src: coord,
                    amount: *amount,
                });
                self.compile_node(child, next_c)
            }
            FieldNode::DomainWarp {
                child,
                warp_x,
                warp_y,
                warp_z,
                amplitude,
            } => {
                let rx = self.compile_node(warp_x, coord);
                let ry = self.compile_node(warp_y, coord);
                let rz = self.compile_node(warp_z, coord);
                let next_c = self.alloc_coord(CoordOp::Warp {
                    src: coord,
                    wx: rx,
                    wy: ry,
                    wz: rz,
                    amplitude: *amplitude,
                });
                self.compile_node(child, next_c)
            }
            FieldNode::Tag { child, material_id } => {
                self.has_tags = true;
                let rc = self.compile_node(child, coord);
                self.alloc_eval(
                    coord,
                    EvalOp::Tag {
                        child: rc,
                        material_id: *material_id,
                    },
                )
            }
            FieldNode::Bounded { child, .. } => self.compile_node(child, coord),
            FieldNode::Expr { program, inputs, .. } => {
                let compiled_inputs = inputs.iter().map(CompiledField::compile).collect();
                self.alloc_eval(
                    coord,
                    EvalOp::Expr {
                        program: Arc::clone(program),
                        inputs: compiled_inputs,
                    },
                )
            }
        };
        self.memo.insert((coord, ptr), reg);
        reg
    }
}

impl CompiledGraph {
    /// Compile multiple [`Field`] roots into one CSE-deduplicated flat graph.
    pub fn compile(roots: &[Field]) -> Self {
        let mut compiler = GraphCompiler::new();
        let root_regs = roots.iter().map(|f| compiler.compile_node(f, 0)).collect();
        CompiledGraph {
            instrs: compiler.instrs,
            num_coords: compiler.num_coords as usize,
            num_vals: compiler.num_vals as usize,
            roots: root_regs,
            has_tags: compiler.has_tags,
        }
    }

    pub fn instruction_count(&self) -> usize {
        self.instrs.len()
    }

    pub fn has_tags(&self) -> bool {
        self.has_tags
    }

    /// Evaluate all roots at `(x, y, z)` in one pass, writing each root's scalar
    /// value into `out` (`out.len() == self.roots.len()`).
    pub fn eval_all_f32(&self, x: f32, y: f32, z: f32, out: &mut [f32]) {
        GRAPH_F32_SCRATCH.with(|cell| {
            let mut eval_with = |st: &mut GraphScratch<f32>| {
                st.prepare(self.num_coords, self.num_vals);
                st.coords[0] = [x, y, z];
                self.run_f32(&mut st.coords, &mut st.vals);
                for (slot, &r) in out.iter_mut().zip(&self.roots) {
                    *slot = st.vals[r as usize];
                }
            };
            if let Ok(mut guard) = cell.try_borrow_mut() {
                eval_with(&mut guard);
            } else {
                let mut local = GraphScratch {
                    coords: Vec::new(),
                    vals: Vec::new(),
                };
                eval_with(&mut local);
            }
        });
    }

    /// Evaluate root 0 with full material-tag propagation, plus all auxiliary
    /// roots `1..` in one single CSE pass!
    pub fn eval_primary_tagged_and_aux(
        &self,
        x: f32,
        y: f32,
        z: f32,
        aux_out: &mut [f32],
    ) -> TaggedSample {
        GRAPH_TAG_SCRATCH.with(|cell| {
            let mut eval_with = |st: &mut GraphScratch<TaggedSample>| -> TaggedSample {
                st.prepare(self.num_coords, self.num_vals);
                st.coords[0] = [x, y, z];
                self.run_tagged(&mut st.coords, &mut st.vals);
                let primary = st.vals[self.roots[0] as usize];
                for (slot, &r) in aux_out.iter_mut().zip(self.roots.iter().skip(1)) {
                    *slot = st.vals[r as usize].distance;
                }
                primary
            };
            if let Ok(mut guard) = cell.try_borrow_mut() {
                eval_with(&mut guard)
            } else {
                let mut local = GraphScratch {
                    coords: Vec::new(),
                    vals: Vec::new(),
                };
                eval_with(&mut local)
            }
        })
    }

    /// Evaluate root `root_idx` over a conservative 3D interval box.
    pub fn eval_interval(&self, root_idx: usize, box3: Interval3) -> Interval {
        let mut coords = vec![Interval3::new(Interval::ENTIRE, Interval::ENTIRE, Interval::ENTIRE); self.num_coords];
        let mut vals = vec![Interval::ENTIRE; self.num_vals];
        coords[0] = box3;
        for instr in &self.instrs {
            match instr {
                CompiledInstr::Coord { dst, op } => {
                    coords[*dst as usize] = eval_coord_interval(op, &coords, &vals);
                }
                CompiledInstr::Eval { dst, coord, op } => {
                    vals[*dst as usize] = eval_op_interval(op, coords[*coord as usize], &vals);
                }
            }
        }
        vals[self.roots[root_idx] as usize]
    }

    /// Evaluate root `root_idx` with forward-mode dual numbers for an exact analytic gradient.
    /// Returns `None` if any op is non-differentiable or produces non-finite derivatives.
    pub fn eval_dual(&self, root_idx: usize, x: f32, y: f32, z: f32) -> Option<(f32, [f32; 3])> {
        let mut coords = vec![[Dual3::constant(0.0); 3]; self.num_coords];
        let mut vals = vec![Dual3::constant(0.0); self.num_vals];
        coords[0] = [Dual3::var(x, 0), Dual3::var(y, 1), Dual3::var(z, 2)];
        for instr in &self.instrs {
            match instr {
                CompiledInstr::Coord { dst, op } => {
                    coords[*dst as usize] = eval_coord_dual(op, &coords, &vals)?;
                }
                CompiledInstr::Eval { dst, coord, op } => {
                    vals[*dst as usize] = eval_op_dual(op, coords[*coord as usize], &vals)?;
                }
            }
        }
        let d = vals[self.roots[root_idx] as usize];
        if d.v.is_finite() && d.d.iter().all(|c| c.is_finite()) {
            Some((d.v, d.d))
        } else {
            None
        }
    }

    fn run_f32(&self, coords: &mut [[f32; 3]], vals: &mut [f32]) {
        for instr in &self.instrs {
            match instr {
                CompiledInstr::Coord { dst, op } => {
                    coords[*dst as usize] = eval_coord_f32(op, coords, vals);
                }
                CompiledInstr::Eval { dst, coord, op } => {
                    vals[*dst as usize] = eval_op_f32(op, coords[*coord as usize], vals);
                }
            }
        }
    }

    fn run_tagged(&self, coords: &mut [[f32; 3]], vals: &mut [TaggedSample]) {
        for instr in &self.instrs {
            match instr {
                CompiledInstr::Coord { dst, op } => {
                    coords[*dst as usize] = eval_coord_tagged(op, coords, vals);
                }
                CompiledInstr::Eval { dst, coord, op } => {
                    vals[*dst as usize] = eval_op_tagged(op, coords[*coord as usize], vals);
                }
            }
        }
    }
}

impl CompiledField {
    pub fn compile(field: &Field) -> Self {
        Self {
            graph: CompiledGraph::compile(std::slice::from_ref(field)),
            bounds: field.bounds(),
        }
    }

    pub fn bounds(&self) -> Option<Aabb> {
        self.bounds
    }

    pub fn instruction_count(&self) -> usize {
        self.graph.instruction_count()
    }

    #[inline]
    pub fn eval(&self, x: f32, y: f32, z: f32) -> f32 {
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return f32::INFINITY;
        }
        let mut out = [0.0f32; 1];
        self.graph.eval_all_f32(x, y, z, &mut out);
        out[0]
    }

    #[inline]
    pub fn eval_tagged(&self, x: f32, y: f32, z: f32) -> TaggedSample {
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return TaggedSample::plain(f32::INFINITY);
        }
        self.graph.eval_primary_tagged_and_aux(x, y, z, &mut [])
    }

    #[inline]
    pub fn eval_interval(&self, box3: Interval3) -> Interval {
        self.graph.eval_interval(0, box3)
    }

    /// Estimate normalized outward surface normal. Uses forward-mode dual-number
    /// autodiff when exact and non-singular, falling back to ULP-safe central
    /// differences otherwise.
    pub fn normal(&self, x: f32, y: f32, z: f32, epsilon: f32) -> Option<[f32; 3]> {
        if !epsilon.is_finite() || epsilon <= 0.0 || !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return None;
        }
        if let Some((_, g)) = self.graph.eval_dual(0, x, y, z) {
            let len = (f64::from(g[0]).powi(2) + f64::from(g[1]).powi(2) + f64::from(g[2]).powi(2)).sqrt();
            if len.is_finite() && len > 1e-6 {
                return Some([
                    (f64::from(g[0]) / len) as f32,
                    (f64::from(g[1]) / len) as f32,
                    (f64::from(g[2]) / len) as f32,
                ]);
            }
        }
        numerical_normal_fn(|px, py, pz| self.eval(px, py, pz), [x, y, z], epsilon)
            .map(|n| [n[0] as f32, n[1] as f32, n[2] as f32])
    }

    /// Mean curvature proxy via 6-neighbor discrete Laplacian `(sum(neighbors) - 6*center) / eps^2`.
    /// Positive on sharp convex peaks/ridges, negative in concave valleys/crevices.
    pub fn curvature(&self, x: f32, y: f32, z: f32, eps: f32) -> f32 {
        let c = self.eval(x, y, z);
        let sum = self.eval(x + eps, y, z)
            + self.eval(x - eps, y, z)
            + self.eval(x, y + eps, z)
            + self.eval(x, y - eps, z)
            + self.eval(x, y, z + eps)
            + self.eval(x, y, z - eps);
        let lap = (sum - 6.0 * c) / (eps * eps);
        if lap.is_finite() {
            lap
        } else {
            0.0
        }
    }
}

/// ULP-safe numerical normal estimation for any scalar evaluator.
pub(crate) fn numerical_normal_fn(
    eval: impl Fn(f32, f32, f32) -> f32,
    point: [f32; 3],
    epsilon: f32,
) -> Option<[f64; 3]> {
    if !epsilon.is_finite() || epsilon <= 0.0 || point.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let mut g = [0.0f64; 3];
    for axis in 0..3 {
        let c = point[axis];
        let req_p = c + epsilon;
        let plus = if req_p.is_finite() && req_p > c {
            Some(req_p)
        } else {
            next_up(c)
        };
        let req_m = c - epsilon;
        let minus = if req_m.is_finite() && req_m < c {
            Some(req_m)
        } else {
            next_down(c)
        };
        let sample = |coord: f32| {
            let mut p = point;
            p[axis] = coord;
            let v = eval(p[0], p[1], p[2]);
            v.is_finite().then_some(f64::from(v))
        };
        g[axis] = match (minus, plus) {
            (Some(lo), Some(hi)) => (sample(hi)? - sample(lo)?) / (f64::from(hi) - f64::from(lo)),
            (Some(lo), None) => (sample(c)? - sample(lo)?) / (f64::from(c) - f64::from(lo)),
            (None, Some(hi)) => (sample(hi)? - sample(c)?) / (f64::from(hi) - f64::from(c)),
            (None, None) => return None,
        };
    }
    let len = g[0].hypot(g[1]).hypot(g[2]);
    if !len.is_finite() || len <= f64::EPSILON {
        return None;
    }
    Some([g[0] / len, g[1] / len, g[2] / len])
}

fn next_up(v: f32) -> Option<f32> {
    if v == f32::INFINITY {
        return None;
    }
    if v == -0.0 {
        return Some(f32::from_bits(1));
    }
    let bits = v.to_bits();
    let n = f32::from_bits(if v >= 0.0 { bits + 1 } else { bits - 1 });
    n.is_finite().then_some(n)
}

fn next_down(v: f32) -> Option<f32> {
    if v == f32::NEG_INFINITY {
        return None;
    }
    if v == 0.0 {
        return Some(f32::from_bits(0x8000_0001));
    }
    let bits = v.to_bits();
    let n = f32::from_bits(if v > 0.0 { bits - 1 } else { bits + 1 });
    n.is_finite().then_some(n)
}

// ── Thread-Local Scratch Buffers ─────────────────────────────────────────────

struct GraphScratch<V> {
    coords: Vec<[f32; 3]>,
    vals: Vec<V>,
}

impl<V: Copy> GraphScratch<V> {
    fn prepare_with(&mut self, nc: usize, nv: usize, zero: V) {
        if self.coords.len() < nc {
            self.coords.resize(nc, [0.0; 3]);
        }
        if self.vals.len() < nv {
            self.vals.resize(nv, zero);
        }
    }
}

impl GraphScratch<f32> {
    fn prepare(&mut self, nc: usize, nv: usize) {
        self.prepare_with(nc, nv, 0.0);
    }
}

impl GraphScratch<TaggedSample> {
    fn prepare(&mut self, nc: usize, nv: usize) {
        self.prepare_with(nc, nv, TaggedSample::plain(0.0));
    }
}

thread_local! {
    static GRAPH_F32_SCRATCH: RefCell<GraphScratch<f32>> = const {
        RefCell::new(GraphScratch { coords: Vec::new(), vals: Vec::new() })
    };
    static GRAPH_TAG_SCRATCH: RefCell<GraphScratch<TaggedSample>> = const {
        RefCell::new(GraphScratch { coords: Vec::new(), vals: Vec::new() })
    };
}

// ── Primal f32 Evaluation ────────────────────────────────────────────────────

#[inline]
fn len2(x: f32, y: f32) -> f32 {
    (x * x + y * y).sqrt()
}

#[inline]
fn len3(x: f32, y: f32, z: f32) -> f32 {
    (x * x + y * y + z * z).sqrt()
}

#[inline]
fn glsl_sign(v: f32) -> f32 {
    if v > 0.0 {
        1.0
    } else if v < 0.0 {
        -1.0
    } else {
        0.0
    }
}

#[inline]
fn smin_with_h(a: f32, b: f32, k: f32) -> (f32, f32) {
    if k <= 0.0 {
        return if a <= b { (a, 1.0) } else { (b, 0.0) };
    }
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    let d = b + (a - b) * h - k * h * (1.0 - h);
    (d, h)
}

#[inline]
fn chamfer_min(a: f32, b: f32, r: f32) -> f32 {
    a.min(b).min((a - r + b) * std::f32::consts::FRAC_1_SQRT_2)
}

#[inline]
fn steps_min(a: f32, b: f32, r: f32, n: f32) -> f32 {
    let s = r / n;
    let u = b - r;
    a.min(b)
        .min(0.5 * (u + a + ((u - a + s).rem_euclid(2.0 * s) - s).abs()))
}

#[inline]
fn cap_arc_nearest(rho: f32, y: f32, r: f32, h: f32, w: f32) -> (f32, f32) {
    let l = len2(rho, y);
    if l > 1e-9 {
        let (px, py) = (r * rho / l, r * y / l);
        if py >= h {
            (px, py)
        } else {
            (w, h)
        }
    } else {
        (0.0, r)
    }
}

fn eval_coord_common(op: &CoordOp, coords: &[[f32; 3]], get_val: impl Fn(u32) -> f32) -> [f32; 3] {
    match op {
        CoordOp::Translate { src, offset } => {
            let p = coords[*src as usize];
            [p[0] - offset[0], p[1] - offset[1], p[2] - offset[2]]
        }
        CoordOp::InvRotate { src, mat } => inv_rotate(mat, coords[*src as usize]),
        CoordOp::InvScale { src, inv_factor } => {
            let p = coords[*src as usize];
            [p[0] * inv_factor, p[1] * inv_factor, p[2] * inv_factor]
        }
        CoordOp::InvScaleNonUniform { src, inv_factors } => {
            let p = coords[*src as usize];
            [
                p[0] * inv_factors[0],
                p[1] * inv_factors[1],
                p[2] * inv_factors[2],
            ]
        }
        CoordOp::Mirror { src, axis } => {
            let p = coords[*src as usize];
            match axis {
                Axis::X => [p[0].abs(), p[1], p[2]],
                Axis::Y => [p[0], p[1].abs(), p[2]],
                Axis::Z => [p[0], p[1], p[2].abs()],
            }
        }
        CoordOp::MirrorPlane {
            src,
            normal,
            offset,
        } => {
            let p = coords[*src as usize];
            let d = (p[0] * normal[0] + p[1] * normal[1] + p[2] * normal[2] + offset).min(0.0);
            [
                p[0] - 2.0 * d * normal[0],
                p[1] - 2.0 * d * normal[1],
                p[2] - 2.0 * d * normal[2],
            ]
        }
        CoordOp::Repeat {
            src,
            spacing,
            count,
        } => {
            let p = coords[*src as usize];
            let map = |v: f32, s: f32, n: Option<u32>| -> f32 {
                if s <= 0.0 {
                    return v;
                }
                let cell = (v / s).round();
                let cell = match n {
                    Some(n) => cell.clamp(-(n as f32), n as f32),
                    None => cell,
                };
                v - s * cell
            };
            [
                map(p[0], spacing[0], count.map(|c| c[0])),
                map(p[1], spacing[1], count.map(|c| c[1])),
                map(p[2], spacing[2], count.map(|c| c[2])),
            ]
        }
        CoordOp::RepeatPolar { src, sector } => {
            let p = coords[*src as usize];
            let r = len2(p[0], p[2]);
            let a = p[2].atan2(p[0]);
            let a = (a + 0.5 * sector).rem_euclid(*sector) - 0.5 * sector;
            let (s, c) = a.sin_cos();
            [r * c, p[1], r * s]
        }
        CoordOp::Elongate { src, half_lengths } => {
            let p = coords[*src as usize];
            [
                (p[0].abs() - half_lengths[0]).max(0.0),
                (p[1].abs() - half_lengths[1]).max(0.0),
                (p[2].abs() - half_lengths[2]).max(0.0),
            ]
        }
        CoordOp::Twist { src, amount } => {
            let p = coords[*src as usize];
            let (s, c) = (amount * p[1]).sin_cos();
            [c * p[0] - s * p[2], p[1], s * p[0] + c * p[2]]
        }
        CoordOp::Bend { src, amount } => {
            let p = coords[*src as usize];
            let (s, c) = (amount * p[0]).sin_cos();
            [c * p[0] - s * p[1], s * p[0] + c * p[1], p[2]]
        }
        CoordOp::ProjectXZ { src } => {
            let p = coords[*src as usize];
            [p[0], 0.0, p[2]]
        }
        CoordOp::Warp {
            src,
            wx,
            wy,
            wz,
            amplitude,
        } => {
            let p = coords[*src as usize];
            [
                p[0] + get_val(*wx) * amplitude,
                p[1] + get_val(*wy) * amplitude,
                p[2] + get_val(*wz) * amplitude,
            ]
        }
    }
}

#[inline]
fn eval_coord_f32(op: &CoordOp, coords: &[[f32; 3]], vals: &[f32]) -> [f32; 3] {
    eval_coord_common(op, coords, |r| vals[r as usize])
}

#[inline]
fn eval_coord_tagged(op: &CoordOp, coords: &[[f32; 3]], vals: &[TaggedSample]) -> [f32; 3] {
    eval_coord_common(op, coords, |r| vals[r as usize].distance)
}

fn eval_primitive_or_leaf_f32(op: &EvalOp, p: [f32; 3]) -> Option<f32> {
    let [x, y, z] = p;
    Some(match op {
        EvalOp::Const(v) => *v,
        EvalOp::CoordAxis(Axis::X) => x,
        EvalOp::CoordAxis(Axis::Y) => y,
        EvalOp::CoordAxis(Axis::Z) => z,
        EvalOp::Sphere { radius } => len3(x, y, z) - radius,
        EvalOp::Box {
            half_extents: b,
            rounding: r,
        } => {
            let qx = x.abs() - (b[0] - r);
            let qy = y.abs() - (b[1] - r);
            let qz = z.abs() - (b[2] - r);
            len3(qx.max(0.0), qy.max(0.0), qz.max(0.0)) + qx.max(qy.max(qz)).min(0.0) - r
        }
        EvalOp::Ellipsoid { radii: r } => {
            let k0 = len3(x / r[0], y / r[1], z / r[2]);
            let k1 = len3(x / (r[0] * r[0]), y / (r[1] * r[1]), z / (r[2] * r[2]));
            if k1 > 0.0 {
                k0 * (k0 - 1.0) / k1
            } else {
                -r[0].min(r[1]).min(r[2])
            }
        }
        EvalOp::Torus {
            major_radius,
            minor_radius,
        } => len2(len2(x, z) - major_radius, y) - minor_radius,
        EvalOp::CappedTorus {
            major_radius,
            minor_radius,
            sin_a,
            cos_a,
        } => {
            let qx = x.abs();
            let k = if cos_a * qx > sin_a * z {
                qx * sin_a + z * cos_a
            } else {
                len2(qx, z)
            };
            (qx * qx + z * z + major_radius * major_radius - 2.0 * major_radius * k)
                .max(0.0)
                .add_y_sq(y)
                .sqrt()
                - minor_radius
        }
        EvalOp::Link {
            major_radius,
            minor_radius,
            half_length,
        } => {
            let qz = (z.abs() - half_length).max(0.0);
            len2(len2(x, qz) - major_radius, y) - minor_radius
        }
        EvalOp::Capsule { a, b, radius } => {
            let pa = [x - a[0], y - a[1], z - a[2]];
            let ba = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let dot_ba = ba[0] * ba[0] + ba[1] * ba[1] + ba[2] * ba[2];
            let h = if dot_ba > 0.0 {
                ((pa[0] * ba[0] + pa[1] * ba[1] + pa[2] * ba[2]) / dot_ba).clamp(0.0, 1.0)
            } else {
                0.0
            };
            len3(pa[0] - ba[0] * h, pa[1] - ba[1] * h, pa[2] - ba[2] * h) - radius
        }
        EvalOp::RoundCone { a, b, r1, r2 } => sd_round_cone(x, y, z, *a, *b, *r1, *r2),
        EvalOp::SolidAngle {
            radius,
            sin_a,
            cos_a,
        } => {
            let qx = len2(x, z);
            let qy = y;
            let l = len2(qx, qy) - radius;
            let dot_qc = (qx * sin_a + qy * cos_a).clamp(0.0, *radius);
            let m = len2(qx - sin_a * dot_qc, qy - cos_a * dot_qc);
            l.max(m * glsl_sign(cos_a * qx - sin_a * qy))
        }
        EvalOp::CutSphere {
            radius: r,
            height,
            rim_w,
        } => {
            let rho = len2(x, z);
            let seg_x = rho.clamp(0.0, *rim_w);
            let dist_seg = len2(rho - seg_x, y - height);
            let (ax, ay) = cap_arc_nearest(rho, y, *r, *height, *rim_w);
            let dist = dist_seg.min(len2(rho - ax, y - ay));
            if rho * rho + y * y <= r * r && y >= *height {
                -dist
            } else {
                dist
            }
        }
        EvalOp::CutHollowSphere {
            radius: r,
            height,
            rim_w,
            thickness,
        } => {
            let rho = len2(x, z);
            let (ax, ay) = cap_arc_nearest(rho, y, *r, *height, *rim_w);
            len2(rho - ax, y - ay) - thickness
        }
        EvalOp::CappedCylinder {
            radius,
            half_height,
        } => {
            let dx = len2(x, z) - radius;
            let dy = y.abs() - half_height;
            dx.max(dy).min(0.0) + len2(dx.max(0.0), dy.max(0.0))
        }
        EvalOp::InfiniteCylinder { radius } => len2(x, z) - radius,
        EvalOp::CappedCone {
            half_height,
            r1,
            r2,
        } => {
            let h = *half_height;
            let q = [len2(x, z), y];
            let k1 = [*r2, h];
            let k2 = [r2 - r1, 2.0 * h];
            let ca = [
                q[0] - q[0].min(if q[1] < 0.0 { *r1 } else { *r2 }),
                q[1].abs() - h,
            ];
            let dot_k2 = k2[0] * k2[0] + k2[1] * k2[1];
            let t = if dot_k2 > 0.0 {
                (((k1[0] - q[0]) * k2[0] + (k1[1] - q[1]) * k2[1]) / dot_k2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let cb = [q[0] - k1[0] + k2[0] * t, q[1] - k1[1] + k2[1] * t];
            let s = if cb[0] < 0.0 && ca[1] < 0.0 {
                -1.0
            } else {
                1.0
            };
            s * (ca[0] * ca[0] + ca[1] * ca[1])
                .min(cb[0] * cb[0] + cb[1] * cb[1])
                .sqrt()
        }
        EvalOp::Plane { normal, offset } => x * normal[0] + y * normal[1] + z * normal[2] + offset,
        EvalOp::Octahedron { size } => {
            let (px, py, pz) = (x.abs(), y.abs(), z.abs());
            let m = px + py + pz - size;
            let (qx, qy, qz) = if 3.0 * px < m {
                (px, py, pz)
            } else if 3.0 * py < m {
                (py, pz, px)
            } else if 3.0 * pz < m {
                (pz, px, py)
            } else {
                return Some(m * 0.577_350_26);
            };
            let k = (0.5 * (qz - qy + size)).clamp(0.0, *size);
            len3(qx, qy - size + k, qz - k)
        }
        EvalOp::HexPrism {
            radius,
            half_height,
        } => {
            const KX: f32 = -0.866_025_4;
            const KY: f32 = 0.5;
            const KZ: f32 = 0.577_350_27;
            let (mut px, py, mut pz) = (x.abs(), y.abs(), z.abs());
            let m = 2.0 * (KX * px + KY * pz).min(0.0);
            px -= m * KX;
            pz -= m * KY;
            let dx_c = px - px.clamp(-KZ * radius, KZ * radius);
            let d1 = len2(dx_c, pz - radius) * (pz - radius).signum();
            let d2 = py - half_height;
            d1.max(d2).min(0.0) + len2(d1.max(0.0), d2.max(0.0))
        }
        EvalOp::SuperPrism {
            half_extents: b,
            exponent,
        } => {
            let exp = exponent.max(1.0);
            let s = (x.abs() / b[0]).powf(exp) + (z.abs() / b[2]).powf(exp);
            let d_xz = (s.powf(1.0 / exp) - 1.0) * b[0].min(b[2]);
            let d_y = y.abs() - b[1];
            d_xz.max(d_y).min(0.0) + len2(d_xz.max(0.0), d_y.max(0.0))
        }
        EvalOp::BoxFrame {
            half_extents: b,
            thickness: e,
        } => {
            let pp = [x.abs() - b[0], y.abs() - b[1], z.abs() - b[2]];
            let q = [
                (pp[0] + e).abs() - e,
                (pp[1] + e).abs() - e,
                (pp[2] + e).abs() - e,
            ];
            let leg = |v: [f32; 3]| {
                len3(v[0].max(0.0), v[1].max(0.0), v[2].max(0.0))
                    + v[0].max(v[1]).max(v[2]).min(0.0)
            };
            leg([pp[0], q[1], q[2]])
                .min(leg([q[0], pp[1], q[2]]))
                .min(leg([q[0], q[1], pp[2]]))
        }
        EvalOp::InfiniteCone { sin_a, cos_a } => {
            let qx = len2(x, z);
            let qy = y;
            let dot_qc = (qx * sin_a + qy * cos_a).max(0.0);
            let d = len2(qx - sin_a * dot_qc, qy - cos_a * dot_qc);
            d * glsl_sign(qx * cos_a - qy * sin_a)
        }
        EvalOp::SquarePyramid { half_base, height } => {
            sd_square_pyramid(x, y, z, *half_base, *height)
        }
        EvalOp::Noise2 {
            basis,
            frequency,
            seed,
        } => match basis {
            NoiseBasis::Value => value_noise2(x * frequency, z * frequency, *seed),
            NoiseBasis::Perlin => perlin2(x * frequency, z * frequency, *seed),
        },
        EvalOp::Noise3 {
            basis,
            frequency,
            seed,
        } => match basis {
            NoiseBasis::Value => value_noise3(x * frequency, y * frequency, z * frequency, *seed),
            NoiseBasis::Perlin => perlin3(x * frequency, y * frequency, z * frequency, *seed),
        },
        EvalOp::Fbm2 {
            basis,
            frequency,
            seed,
            octaves,
            lacunarity,
            gain,
            ridged,
        } => fbm2_params(
            x,
            z,
            *seed,
            *frequency,
            *octaves,
            *lacunarity,
            *gain,
            *basis == NoiseBasis::Perlin,
            *ridged,
        ),
        EvalOp::Fbm3 {
            basis,
            frequency,
            seed,
            octaves,
            lacunarity,
            gain,
            ridged,
        } => fbm3_params(
            x,
            y,
            z,
            *seed,
            *frequency,
            *octaves,
            *lacunarity,
            *gain,
            *basis == NoiseBasis::Perlin,
            *ridged,
        ),
        EvalOp::Cells {
            frequency,
            seed,
            jitter,
            mode,
            threshold,
        } => {
            let raw = worley3(x, y, z, *frequency, *seed, *jitter, *mode);
            let scaled = match mode {
                CellMode::Value => raw,
                _ => raw / frequency.max(1e-6),
            };
            scaled - threshold
        }
        _ => return None,
    })
}

trait AddYSq {
    fn add_y_sq(self, y: f32) -> f32;
}
impl AddYSq for f32 {
    #[inline]
    fn add_y_sq(self, y: f32) -> f32 {
        self + y * y
    }
}

fn eval_unary_f32(op: UnaryMathOp, v: f32) -> f32 {
    match op {
        UnaryMathOp::Neg => -v,
        UnaryMathOp::Abs => v.abs(),
        UnaryMathOp::Sqrt => v.sqrt(),
        UnaryMathOp::Floor => v.floor(),
        UnaryMathOp::Ceil => v.ceil(),
        UnaryMathOp::Fract => v - v.floor(),
        UnaryMathOp::Sin => v.sin(),
        UnaryMathOp::Cos => v.cos(),
        UnaryMathOp::Exp => v.exp(),
        UnaryMathOp::Ln => v.ln(),
        UnaryMathOp::Sign => glsl_sign(v),
        UnaryMathOp::Saturate => v.clamp(0.0, 1.0),
    }
}

fn eval_binary_f32(op: BinaryMathOp, a: f32, b: f32) -> f32 {
    match op {
        BinaryMathOp::Add => a + b,
        BinaryMathOp::Sub => a - b,
        BinaryMathOp::Mul => a * b,
        BinaryMathOp::Div => a / b,
        BinaryMathOp::Min => a.min(b),
        BinaryMathOp::Max => a.max(b),
        BinaryMathOp::Pow => a.powf(b),
        BinaryMathOp::Mod => a.rem_euclid(b),
        BinaryMathOp::Atan2 => a.atan2(b),
    }
}

fn eval_op_f32(op: &EvalOp, p: [f32; 3], vals: &[f32]) -> f32 {
    if let Some(v) = eval_primitive_or_leaf_f32(op, p) {
        return v;
    }
    match op {
        EvalOp::Heightmap { base_y, height_reg } => p[1] - (base_y + vals[*height_reg as usize]),
        EvalOp::Unary { op, child } => eval_unary_f32(*op, vals[*child as usize]),
        EvalOp::Binary { op, a, b } => eval_binary_f32(*op, vals[*a as usize], vals[*b as usize]),
        EvalOp::Clamp { child, lo, hi } => vals[*child as usize].clamp(*lo, *hi),
        EvalOp::Smoothstep { child, edge0, edge1 } => {
            let t = ((vals[*child as usize] - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        }
        EvalOp::Step { child, edge } => {
            if vals[*child as usize] < *edge {
                0.0
            } else {
                1.0
            }
        }
        EvalOp::Remap {
            child,
            in_lo,
            in_hi,
            out_lo,
            out_hi,
            clamp,
        } => {
            let mut t = (vals[*child as usize] - in_lo) / (in_hi - in_lo);
            if *clamp {
                t = t.clamp(0.0, 1.0);
            }
            out_lo + (out_hi - out_lo) * t
        }
        EvalOp::Lerp { a, b, t } => {
            let va = vals[*a as usize];
            let vb = vals[*b as usize];
            va + (vb - va) * vals[*t as usize]
        }
        EvalOp::Select {
            cond,
            then_r,
            else_r,
        } => {
            if vals[*cond as usize] > 0.0 {
                vals[*then_r as usize]
            } else {
                vals[*else_r as usize]
            }
        }
        EvalOp::Union { children } => children
            .iter()
            .map(|&r| vals[r as usize])
            .fold(f32::INFINITY, f32::min),
        EvalOp::Intersect { children } => children
            .iter()
            .map(|&r| vals[r as usize])
            .fold(f32::NEG_INFINITY, f32::max),
        EvalOp::Subtract { a, b } => vals[*a as usize].max(-vals[*b as usize]),
        EvalOp::Xor { a, b } => {
            let da = vals[*a as usize];
            let db = vals[*b as usize];
            da.min(db).max(-da.max(db))
        }
        EvalOp::SmoothUnion { a, b, k } => smin_with_h(vals[*a as usize], vals[*b as usize], *k).0,
        EvalOp::SmoothSubtract { a, b, k } => {
            -smin_with_h(-vals[*a as usize], vals[*b as usize], *k).0
        }
        EvalOp::SmoothIntersect { a, b, k } => {
            -smin_with_h(-vals[*a as usize], -vals[*b as usize], *k).0
        }
        EvalOp::ChamferUnion { a, b, r } => chamfer_min(vals[*a as usize], vals[*b as usize], *r),
        EvalOp::ChamferSubtract { a, b, r } => {
            -chamfer_min(-vals[*a as usize], vals[*b as usize], *r)
        }
        EvalOp::ChamferIntersect { a, b, r } => {
            -chamfer_min(-vals[*a as usize], -vals[*b as usize], *r)
        }
        EvalOp::StepsUnion { a, b, r, steps } => {
            steps_min(vals[*a as usize], vals[*b as usize], *r, *steps)
        }
        EvalOp::Round { child, radius } => vals[*child as usize] - radius,
        EvalOp::Shell { child, thickness } => vals[*child as usize].abs() - thickness,
        EvalOp::ElongateFinish {
            child,
            half_lengths: h,
        } => {
            let qx = p[0].abs() - h[0];
            let qy = p[1].abs() - h[1];
            let qz = p[2].abs() - h[2];
            vals[*child as usize] + qx.max(qy.max(qz)).min(0.0)
        }
        EvalOp::ScaleDist { child, factor } => vals[*child as usize] * factor,
        EvalOp::Displace {
            child,
            offset,
            amplitude,
            ..
        } => vals[*child as usize] + vals[*offset as usize] * amplitude,
        EvalOp::RepeatPoints { child, offsets } => offsets
            .iter()
            .map(|o| child.eval(p[0] - o[0], p[1] - o[1], p[2] - o[2]))
            .fold(f32::INFINITY, f32::min),
        EvalOp::Tag { child, .. } => vals[*child as usize],
        EvalOp::Expr { program, inputs } => program.eval_f32(p[0], p[1], p[2], &|idx, qx, qy, qz| {
            inputs[idx].eval(qx, qy, qz)
        }),
        _ => unreachable!(),
    }
}

// ── Tagged Multi-Material Evaluation ─────────────────────────────────────────

fn eval_op_tagged(op: &EvalOp, p: [f32; 3], vals: &[TaggedSample]) -> TaggedSample {
    if let Some(v) = eval_primitive_or_leaf_f32(op, p) {
        return TaggedSample::plain(v);
    }
    match op {
        EvalOp::Tag { child, material_id } => {
            let mut s = vals[*child as usize];
            s.material_id = *material_id;
            s.blend_material_id = 0;
            s.blend_weight = 0.0;
            s
        }
        EvalOp::Union { children } => {
            let mut best = TaggedSample::plain(f32::INFINITY);
            for &r in children {
                let s = vals[r as usize];
                if s.distance < best.distance {
                    best = s;
                }
            }
            best
        }
        EvalOp::Intersect { children } => {
            let mut best = TaggedSample::plain(f32::NEG_INFINITY);
            for &r in children {
                let s = vals[r as usize];
                if s.distance > best.distance {
                    best = s;
                }
            }
            best
        }
        EvalOp::Subtract { a, b } => {
            let sa = vals[*a as usize];
            let sb = vals[*b as usize];
            let d = sa.distance.max(-sb.distance);
            TaggedSample {
                distance: d,
                material_id: sa.material_id,
                blend_material_id: sa.blend_material_id,
                blend_weight: sa.blend_weight,
            }
        }
        EvalOp::Xor { a, b } => {
            let sa = vals[*a as usize];
            let sb = vals[*b as usize];
            let d = sa.distance.min(sb.distance).max(-sa.distance.max(sb.distance));
            let winner = if sa.distance <= sb.distance { sa } else { sb };
            TaggedSample {
                distance: d,
                ..winner
            }
        }
        EvalOp::SmoothUnion { a, b, k } => {
            let sa = vals[*a as usize];
            let sb = vals[*b as usize];
            let (d, h) = smin_with_h(sa.distance, sb.distance, *k);
            if sa.material_id == sb.material_id {
                TaggedSample {
                    distance: d,
                    material_id: sa.material_id,
                    blend_material_id: if h >= 0.5 {
                        sa.blend_material_id
                    } else {
                        sb.blend_material_id
                    },
                    blend_weight: if h >= 0.5 {
                        sa.blend_weight
                    } else {
                        sb.blend_weight
                    },
                }
            } else if h >= 0.5 {
                TaggedSample {
                    distance: d,
                    material_id: sa.material_id,
                    blend_material_id: sb.material_id,
                    blend_weight: (1.0 - h).clamp(0.0, 0.5),
                }
            } else {
                TaggedSample {
                    distance: d,
                    material_id: sb.material_id,
                    blend_material_id: sa.material_id,
                    blend_weight: h.clamp(0.0, 0.5),
                }
            }
        }
        EvalOp::SmoothSubtract { a, b, k } => {
            let sa = vals[*a as usize];
            let sb = vals[*b as usize];
            let (neg_d, _) = smin_with_h(-sa.distance, sb.distance, *k);
            TaggedSample {
                distance: -neg_d,
                ..sa
            }
        }
        EvalOp::SmoothIntersect { a, b, k } => {
            let sa = vals[*a as usize];
            let sb = vals[*b as usize];
            let (neg_d, h) = smin_with_h(-sa.distance, -sb.distance, *k);
            let winner = if h >= 0.5 { sa } else { sb };
            TaggedSample {
                distance: -neg_d,
                ..winner
            }
        }
        EvalOp::ChamferUnion { a, b, r } => {
            let sa = vals[*a as usize];
            let sb = vals[*b as usize];
            let d = chamfer_min(sa.distance, sb.distance, *r);
            let winner = if sa.distance <= sb.distance { sa } else { sb };
            TaggedSample {
                distance: d,
                ..winner
            }
        }
        EvalOp::ChamferSubtract { a, b, r } => {
            let sa = vals[*a as usize];
            let sb = vals[*b as usize];
            let d = -chamfer_min(-sa.distance, sb.distance, *r);
            TaggedSample { distance: d, ..sa }
        }
        EvalOp::ChamferIntersect { a, b, r } => {
            let sa = vals[*a as usize];
            let sb = vals[*b as usize];
            let d = -chamfer_min(-sa.distance, -sb.distance, *r);
            let winner = if sa.distance >= sb.distance { sa } else { sb };
            TaggedSample {
                distance: d,
                ..winner
            }
        }
        EvalOp::StepsUnion { a, b, r, steps } => {
            let sa = vals[*a as usize];
            let sb = vals[*b as usize];
            let d = steps_min(sa.distance, sb.distance, *r, *steps);
            let winner = if sa.distance <= sb.distance { sa } else { sb };
            TaggedSample {
                distance: d,
                ..winner
            }
        }
        EvalOp::Round { child, radius } => {
            let mut s = vals[*child as usize];
            s.distance -= radius;
            s
        }
        EvalOp::Shell { child, thickness } => {
            let mut s = vals[*child as usize];
            s.distance = s.distance.abs() - thickness;
            s
        }
        EvalOp::ElongateFinish {
            child,
            half_lengths: h,
        } => {
            let mut s = vals[*child as usize];
            let qx = p[0].abs() - h[0];
            let qy = p[1].abs() - h[1];
            let qz = p[2].abs() - h[2];
            s.distance += qx.max(qy.max(qz)).min(0.0);
            s
        }
        EvalOp::ScaleDist { child, factor } => {
            let mut s = vals[*child as usize];
            s.distance *= factor;
            s
        }
        EvalOp::Displace {
            child,
            offset,
            amplitude,
            ..
        } => {
            let mut s = vals[*child as usize];
            s.distance += vals[*offset as usize].distance * amplitude;
            s
        }
        EvalOp::RepeatPoints { child, offsets } => {
            let mut best = TaggedSample::plain(f32::INFINITY);
            for o in offsets {
                let s = child.eval_tagged(p[0] - o[0], p[1] - o[1], p[2] - o[2]);
                if s.distance < best.distance {
                    best = s;
                }
            }
            best
        }
        EvalOp::Heightmap { base_y, height_reg } => {
            TaggedSample::plain(p[1] - (base_y + vals[*height_reg as usize].distance))
        }
        EvalOp::Unary { op, child } => {
            let mut s = vals[*child as usize];
            s.distance = eval_unary_f32(*op, s.distance);
            s
        }
        EvalOp::Binary { op, a, b } => {
            let sa = vals[*a as usize];
            let sb = vals[*b as usize];
            TaggedSample {
                distance: eval_binary_f32(*op, sa.distance, sb.distance),
                material_id: if sa.material_id != 0 {
                    sa.material_id
                } else {
                    sb.material_id
                },
                blend_material_id: 0,
                blend_weight: 0.0,
            }
        }
        EvalOp::Clamp { child, lo, hi } => {
            let mut s = vals[*child as usize];
            s.distance = s.distance.clamp(*lo, *hi);
            s
        }
        EvalOp::Smoothstep { child, edge0, edge1 } => {
            let t = ((vals[*child as usize].distance - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
            TaggedSample::plain(t * t * (3.0 - 2.0 * t))
        }
        EvalOp::Step { child, edge } => TaggedSample::plain(if vals[*child as usize].distance < *edge {
            0.0
        } else {
            1.0
        }),
        EvalOp::Remap {
            child,
            in_lo,
            in_hi,
            out_lo,
            out_hi,
            clamp,
        } => {
            let mut t = (vals[*child as usize].distance - in_lo) / (in_hi - in_lo);
            if *clamp {
                t = t.clamp(0.0, 1.0);
            }
            TaggedSample::plain(out_lo + (out_hi - out_lo) * t)
        }
        EvalOp::Lerp { a, b, t } => {
            let sa = vals[*a as usize];
            let sb = vals[*b as usize];
            let vt = vals[*t as usize].distance;
            TaggedSample {
                distance: sa.distance + (sb.distance - sa.distance) * vt,
                material_id: if vt < 0.5 {
                    sa.material_id
                } else {
                    sb.material_id
                },
                blend_material_id: 0,
                blend_weight: 0.0,
            }
        }
        EvalOp::Select {
            cond,
            then_r,
            else_r,
        } => {
            if vals[*cond as usize].distance > 0.0 {
                vals[*then_r as usize]
            } else {
                vals[*else_r as usize]
            }
        }
        EvalOp::Expr { program, inputs } => TaggedSample::plain(program.eval_f32(
            p[0],
            p[1],
            p[2],
            &|idx, qx, qy, qz| inputs[idx].eval(qx, qy, qz),
        )),
        _ => unreachable!(),
    }
}

// ── Conservative Interval Evaluation (Brick Pruning) ─────────────────────────

fn eval_coord_interval(op: &CoordOp, coords: &[Interval3], vals: &[Interval]) -> Interval3 {
    match op {
        CoordOp::Translate { src, offset } => coords[*src as usize].translate(*offset),
        CoordOp::InvRotate { src, mat } => coords[*src as usize].inv_rotate(mat),
        CoordOp::InvScale { src, inv_factor } => {
            let p = coords[*src as usize];
            Interval3::new(
                p.x.scale(*inv_factor),
                p.y.scale(*inv_factor),
                p.z.scale(*inv_factor),
            )
        }
        CoordOp::InvScaleNonUniform { src, inv_factors } => {
            let p = coords[*src as usize];
            Interval3::new(
                p.x.scale(inv_factors[0]),
                p.y.scale(inv_factors[1]),
                p.z.scale(inv_factors[2]),
            )
        }
        CoordOp::Mirror { src, axis } => {
            let p = coords[*src as usize];
            match axis {
                Axis::X => Interval3::new(p.x.abs(), p.y, p.z),
                Axis::Y => Interval3::new(p.x, p.y.abs(), p.z),
                Axis::Z => Interval3::new(p.x, p.y, p.z.abs()),
            }
        }
        CoordOp::Elongate { src, half_lengths } => {
            let p = coords[*src as usize];
            let z = Interval::point(0.0);
            Interval3::new(
                p.x.abs().sub(Interval::point(half_lengths[0])).max(z),
                p.y.abs().sub(Interval::point(half_lengths[1])).max(z),
                p.z.abs().sub(Interval::point(half_lengths[2])).max(z),
            )
        }
        CoordOp::ProjectXZ { src } => {
            let p = coords[*src as usize];
            Interval3::new(p.x, Interval::point(0.0), p.z)
        }
        CoordOp::Warp {
            src,
            wx,
            wy,
            wz,
            amplitude,
        } => {
            let p = coords[*src as usize];
            Interval3::new(
                p.x.add(vals[*wx as usize].scale(*amplitude)),
                p.y.add(vals[*wy as usize].scale(*amplitude)),
                p.z.add(vals[*wz as usize].scale(*amplitude)),
            )
        }
        CoordOp::Repeat {
            src,
            spacing,
            count,
        } => {
            let p = coords[*src as usize];
            let map = |iv: Interval, s: f32, n: Option<u32>| -> Interval {
                if s <= 0.0 || !iv.is_finite() {
                    return iv;
                }
                let c_lo = (iv.lo / s).round();
                let c_hi = (iv.hi / s).round();
                if c_lo == c_hi {
                    let cell = match n {
                        Some(n) => c_lo.clamp(-(n as f32), n as f32),
                        None => c_lo,
                    };
                    iv.sub(Interval::point(s * cell))
                } else {
                    match n {
                        Some(n) => {
                            let nf = n as f32;
                            let lo = (iv.lo - s * nf).min(-0.5 * s);
                            let hi = (iv.hi + s * nf).max(0.5 * s);
                            Interval::new(lo, hi)
                        }
                        None => Interval::new(-0.5 * s, 0.5 * s),
                    }
                }
            };
            Interval3::new(
                map(p.x, spacing[0], count.map(|c| c[0])),
                map(p.y, spacing[1], count.map(|c| c[1])),
                map(p.z, spacing[2], count.map(|c| c[2])),
            )
        }
        // Conservative fallback for non-trivial folding/twisting transforms
        CoordOp::MirrorPlane { .. }
        | CoordOp::RepeatPolar { .. }
        | CoordOp::Twist { .. }
        | CoordOp::Bend { .. } => {
            Interval3::new(Interval::ENTIRE, Interval::ENTIRE, Interval::ENTIRE)
        }
    }
}

fn eval_op_interval(op: &EvalOp, p: Interval3, vals: &[Interval]) -> Interval {
    match op {
        EvalOp::Const(v) => Interval::point(*v),
        EvalOp::CoordAxis(Axis::X) => p.x,
        EvalOp::CoordAxis(Axis::Y) => p.y,
        EvalOp::CoordAxis(Axis::Z) => p.z,
        EvalOp::Sphere { radius } => p.length().sub(Interval::point(*radius)),
        EvalOp::Box {
            half_extents: b,
            rounding: r,
        } => {
            let z = Interval::point(0.0);
            let qx = p.x.abs().sub(Interval::point(b[0] - r));
            let qy = p.y.abs().sub(Interval::point(b[1] - r));
            let qz = p.z.abs().sub(Interval::point(b[2] - r));
            let out = Interval::len3(qx.max(z), qy.max(z), qz.max(z));
            let inside = qx.max(qy.max(qz)).min(z);
            out.add(inside).sub(Interval::point(*r))
        }
        EvalOp::Torus {
            major_radius,
            minor_radius,
        } => {
            let qx = Interval::len2(p.x, p.z).sub(Interval::point(*major_radius));
            Interval::len2(qx, p.y).sub(Interval::point(*minor_radius))
        }
        EvalOp::CappedCylinder {
            radius,
            half_height,
        } => {
            let z = Interval::point(0.0);
            let dx = Interval::len2(p.x, p.z).sub(Interval::point(*radius));
            let dy = p.y.abs().sub(Interval::point(*half_height));
            dx.max(dy).min(z).add(Interval::len2(dx.max(z), dy.max(z)))
        }
        EvalOp::InfiniteCylinder { radius } => {
            Interval::len2(p.x, p.z).sub(Interval::point(*radius))
        }
        EvalOp::Plane { normal, offset } => p
            .x
            .scale(normal[0])
            .add(p.y.scale(normal[1]))
            .add(p.z.scale(normal[2]))
            .add(Interval::point(*offset)),
        EvalOp::Capsule { a, b, radius } => {
            // Conservative distance to bounding box of segment minus radius:
            // guaranteed lower bound <= actual distance!
            let min_x = a[0].min(b[0]);
            let max_x = a[0].max(b[0]);
            let min_y = a[1].min(b[1]);
            let max_y = a[1].max(b[1]);
            let min_z = a[2].min(b[2]);
            let max_z = a[2].max(b[2]);
            let hx = 0.5 * (max_x - min_x);
            let hy = 0.5 * (max_y - min_y);
            let hz = 0.5 * (max_z - min_z);
            let cx = 0.5 * (min_x + max_x);
            let cy = 0.5 * (min_y + max_y);
            let cz = 0.5 * (min_z + max_z);
            let z = Interval::point(0.0);
            let dx = p.x.sub(Interval::point(cx)).abs().sub(Interval::point(hx)).max(z);
            let dy = p.y.sub(Interval::point(cy)).abs().sub(Interval::point(hy)).max(z);
            let dz = p.z.sub(Interval::point(cz)).abs().sub(Interval::point(hz)).max(z);
            let lo = Interval::len3(dx, dy, dz).lo - radius;
            Interval::new(lo, f32::INFINITY)
        }
        EvalOp::Ellipsoid { radii: r } => {
            // Distance to enclosing sphere of max radius is a lower bound for lo!
            let r_max = r[0].max(r[1]).max(r[2]);
            let lo = p.length().lo - r_max;
            let r_min = r[0].min(r[1]).min(r[2]);
            let hi = p.length().hi - r_min;
            Interval::new(lo, hi)
        }
        EvalOp::Heightmap { base_y, height_reg } => {
            p.y.sub(Interval::point(*base_y).add(vals[*height_reg as usize]))
        }
        EvalOp::Noise2 { basis, .. } | EvalOp::Noise3 { basis, .. } => match basis {
            NoiseBasis::Value => Interval::new(0.0, 1.0),
            NoiseBasis::Perlin => Interval::new(-1.0, 1.0),
        },
        EvalOp::Fbm2 { ridged, .. } | EvalOp::Fbm3 { ridged, .. } => {
            if *ridged {
                Interval::new(0.0, 1.0)
            } else {
                Interval::new(-1.0, 1.0)
            }
        }
        EvalOp::Cells {
            frequency,
            mode,
            threshold,
            ..
        } => {
            let inv_f = 1.0 / frequency.max(1e-6);
            match mode {
                CellMode::Value => Interval::new(-threshold, 1.0 - threshold),
                _ => Interval::new(-threshold, 3.5 * inv_f - threshold),
            }
        }
        EvalOp::Unary { op, child } => {
            let c = vals[*child as usize];
            match op {
                UnaryMathOp::Neg => c.neg(),
                UnaryMathOp::Abs => c.abs(),
                UnaryMathOp::Sqrt => c.sqrt(),
                UnaryMathOp::Floor => c.floor(),
                UnaryMathOp::Ceil => c.ceil(),
                UnaryMathOp::Fract => c.fract(),
                UnaryMathOp::Sin => c.sin(),
                UnaryMathOp::Cos => c.cos(),
                UnaryMathOp::Exp => c.exp(),
                UnaryMathOp::Ln => c.ln(),
                UnaryMathOp::Sign => Interval::new(-1.0, 1.0),
                UnaryMathOp::Saturate => c.clamp(0.0, 1.0),
            }
        }
        EvalOp::Binary { op, a, b } => {
            let va = vals[*a as usize];
            let vb = vals[*b as usize];
            match op {
                BinaryMathOp::Add => va.add(vb),
                BinaryMathOp::Sub => va.sub(vb),
                BinaryMathOp::Mul => va.mul(vb),
                BinaryMathOp::Div => va.div(vb),
                BinaryMathOp::Min => va.min(vb),
                BinaryMathOp::Max => va.max(vb),
                BinaryMathOp::Atan2 => Interval::new(-std::f32::consts::PI, std::f32::consts::PI),
                BinaryMathOp::Pow | BinaryMathOp::Mod => Interval::ENTIRE,
            }
        }
        EvalOp::Clamp { child, lo, hi } => vals[*child as usize].clamp(*lo, *hi),
        EvalOp::Smoothstep { child, edge0, edge1 } => {
            vals[*child as usize].smoothstep(*edge0, *edge1)
        }
        EvalOp::Step { child, edge } => {
            let c = vals[*child as usize];
            if c.hi < *edge {
                Interval::point(0.0)
            } else if c.lo >= *edge {
                Interval::point(1.0)
            } else {
                Interval::new(0.0, 1.0)
            }
        }
        EvalOp::Remap {
            child,
            in_lo,
            in_hi,
            out_lo,
            out_hi,
            clamp,
        } => {
            let t = vals[*child as usize]
                .sub(Interval::point(*in_lo))
                .scale(1.0 / (in_hi - in_lo));
            let t = if *clamp { t.clamp(0.0, 1.0) } else { t };
            Interval::point(*out_lo).add(t.scale(out_hi - out_lo))
        }
        EvalOp::Lerp { a, b, t } => {
            let va = vals[*a as usize];
            let vb = vals[*b as usize];
            let vt = vals[*t as usize];
            if vt.lo >= 0.0 && vt.hi <= 1.0 {
                va.union_bounds(vb)
            } else {
                va.add(vb.sub(va).mul(vt))
            }
        }
        EvalOp::Select {
            cond,
            then_r,
            else_r,
        } => {
            let c = vals[*cond as usize];
            if c.lo > 0.0 {
                vals[*then_r as usize]
            } else if c.hi <= 0.0 {
                vals[*else_r as usize]
            } else {
                vals[*then_r as usize].union_bounds(vals[*else_r as usize])
            }
        }
        EvalOp::Union { children } => children
            .iter()
            .fold(Interval::point(f32::INFINITY), |acc, &r| acc.min(vals[r as usize])),
        EvalOp::Intersect { children } => children
            .iter()
            .fold(Interval::point(f32::NEG_INFINITY), |acc, &r| {
                acc.max(vals[r as usize])
            }),
        EvalOp::Subtract { a, b } => vals[*a as usize].max(vals[*b as usize].neg()),
        EvalOp::Xor { a, b } => {
            let va = vals[*a as usize];
            let vb = vals[*b as usize];
            va.min(vb).max(va.max(vb).neg())
        }
        EvalOp::SmoothUnion { a, b, k } => vals[*a as usize].smin(vals[*b as usize], *k),
        EvalOp::SmoothSubtract { a, b, k } => vals[*a as usize].smax(vals[*b as usize].neg(), *k),
        EvalOp::SmoothIntersect { a, b, k } => vals[*a as usize].smax(vals[*b as usize], *k),
        EvalOp::ChamferUnion { a, b, r } => {
            let va = vals[*a as usize];
            let vb = vals[*b as usize];
            let c = va
                .sub(Interval::point(*r))
                .add(vb)
                .scale(std::f32::consts::FRAC_1_SQRT_2);
            va.min(vb).min(c)
        }
        EvalOp::ChamferSubtract { a, b, r } => {
            let va = vals[*a as usize].neg();
            let vb = vals[*b as usize];
            let c = va
                .sub(Interval::point(*r))
                .add(vb)
                .scale(std::f32::consts::FRAC_1_SQRT_2);
            va.min(vb).min(c).neg()
        }
        EvalOp::ChamferIntersect { a, b, r } => {
            let va = vals[*a as usize].neg();
            let vb = vals[*b as usize].neg();
            let c = va
                .sub(Interval::point(*r))
                .add(vb)
                .scale(std::f32::consts::FRAC_1_SQRT_2);
            va.min(vb).min(c).neg()
        }
        EvalOp::StepsUnion { a, b, r, .. } => vals[*a as usize].min(vals[*b as usize]).expand(*r),
        EvalOp::Round { child, radius } => vals[*child as usize].sub(Interval::point(*radius)),
        EvalOp::Shell { child, thickness } => {
            vals[*child as usize].abs().sub(Interval::point(*thickness))
        }
        EvalOp::ElongateFinish {
            child,
            half_lengths: h,
        } => {
            let z = Interval::point(0.0);
            let qx = p.x.abs().sub(Interval::point(h[0]));
            let qy = p.y.abs().sub(Interval::point(h[1]));
            let qz = p.z.abs().sub(Interval::point(h[2]));
            vals[*child as usize].add(qx.max(qy.max(qz)).min(z))
        }
        EvalOp::ScaleDist { child, factor } => vals[*child as usize].scale(*factor),
        EvalOp::Displace {
            child,
            offset,
            amplitude,
            offset_range,
        } => {
            let off_iv = if vals[*offset as usize].is_finite() {
                vals[*offset as usize]
            } else if let Some([lo, hi]) = offset_range {
                Interval::new(*lo, *hi)
            } else {
                return Interval::ENTIRE;
            };
            vals[*child as usize].add(off_iv.scale(*amplitude))
        }
        EvalOp::RepeatPoints { child, offsets } => {
            let mut acc = Interval::point(f32::INFINITY);
            for o in offsets {
                let iv = child.eval_interval(p.translate(*o));
                acc = acc.min(iv);
                if acc.lo <= 0.0 && acc.hi <= 0.0 {
                    break;
                }
            }
            acc
        }
        EvalOp::Tag { child, .. } => vals[*child as usize],
        // Primitives without specialized interval formulas: safely widen to ENTIRE
        // (except we can cheaply prune outside their exact AABB if known!)
        EvalOp::CappedTorus {
            major_radius,
            minor_radius,
            ..
        } => aabb_lower_bound_interval(
            p,
            [-major_radius - minor_radius, -minor_radius, -major_radius - minor_radius],
            [major_radius + minor_radius, *minor_radius, major_radius + minor_radius],
        ),
        EvalOp::Link {
            major_radius,
            minor_radius,
            half_length,
        } => aabb_lower_bound_interval(
            p,
            [
                -major_radius - minor_radius,
                -minor_radius,
                -major_radius - minor_radius - half_length,
            ],
            [
                major_radius + minor_radius,
                *minor_radius,
                major_radius + minor_radius + half_length,
            ],
        ),
        EvalOp::RoundCone { a, b, r1, r2 } => aabb_lower_bound_interval(
            p,
            [
                (a[0] - r1).min(b[0] - r2),
                (a[1] - r1).min(b[1] - r2),
                (a[2] - r1).min(b[2] - r2),
            ],
            [
                (a[0] + r1).max(b[0] + r2),
                (a[1] + r1).max(b[1] + r2),
                (a[2] + r1).max(b[2] + r2),
            ],
        ),
        EvalOp::SolidAngle { radius, .. } => {
            aabb_lower_bound_interval(p, [-*radius; 3], [*radius; 3])
        }
        EvalOp::CutSphere { radius, height, .. } => {
            aabb_lower_bound_interval(p, [-*radius, *height, -*radius], [*radius, *radius, *radius])
        }
        EvalOp::CutHollowSphere {
            radius,
            height,
            thickness,
            ..
        } => aabb_lower_bound_interval(
            p,
            [-radius - thickness, height - thickness, -radius - thickness],
            [radius + thickness, radius + thickness, radius + thickness],
        ),
        EvalOp::CappedCone {
            half_height,
            r1,
            r2,
        } => {
            let r = r1.max(*r2);
            aabb_lower_bound_interval(p, [-r, -*half_height, -r], [r, *half_height, r])
        }
        EvalOp::Octahedron { size } => aabb_lower_bound_interval(p, [-*size; 3], [*size; 3]),
        EvalOp::HexPrism {
            radius,
            half_height,
        } => {
            let rx = radius * 1.154_700_5;
            aabb_lower_bound_interval(p, [-rx, -*half_height, -*radius], [rx, *half_height, *radius])
        }
        EvalOp::SuperPrism {
            half_extents: b, ..
        }
        | EvalOp::BoxFrame {
            half_extents: b, ..
        } => aabb_lower_bound_interval(p, [-b[0], -b[1], -b[2]], *b),
        EvalOp::SquarePyramid { half_base, height } => aabb_lower_bound_interval(
            p,
            [-*half_base, -0.5 * height, -*half_base],
            [*half_base, 0.5 * height, *half_base],
        ),
        EvalOp::InfiniteCone { .. } | EvalOp::Expr { .. } => Interval::ENTIRE,
    }
}

/// Returns `[min_dist_to_aabb, +inf]`: whenever a brick is strictly outside an
/// exact primitive's bounding box, `lo > 0` safely skips the entire brick!
fn aabb_lower_bound_interval(p: Interval3, min: [f32; 3], max: [f32; 3]) -> Interval {
    if !p.is_finite() {
        return Interval::ENTIRE;
    }
    let dx = (min[0] - p.x.hi).max(p.x.lo - max[0]).max(0.0);
    let dy = (min[1] - p.y.hi).max(p.y.lo - max[1]).max(0.0);
    let dz = (min[2] - p.z.hi).max(p.z.lo - max[2]).max(0.0);
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    if dist > 0.0 {
        Interval::new(dist, f32::INFINITY)
    } else {
        Interval::ENTIRE
    }
}

// ── Dual-Number Autodiff (Exact Analytic Gradients) ──────────────────────────

fn eval_coord_dual(op: &CoordOp, coords: &[[Dual3; 3]], vals: &[Dual3]) -> Option<[Dual3; 3]> {
    match op {
        CoordOp::Translate { src, offset } => {
            let p = coords[*src as usize];
            Some([
                p[0].sub(Dual3::constant(offset[0])),
                p[1].sub(Dual3::constant(offset[1])),
                p[2].sub(Dual3::constant(offset[2])),
            ])
        }
        CoordOp::InvRotate { src, mat } => {
            let p = coords[*src as usize];
            let dot_row = |col: usize| {
                p[0].mul(Dual3::constant(mat[0][col]))
                    .add(p[1].mul(Dual3::constant(mat[1][col])))
                    .add(p[2].mul(Dual3::constant(mat[2][col])))
            };
            Some([dot_row(0), dot_row(1), dot_row(2)])
        }
        CoordOp::InvScale { src, inv_factor } => {
            let p = coords[*src as usize];
            let s = Dual3::constant(*inv_factor);
            Some([p[0].mul(s), p[1].mul(s), p[2].mul(s)])
        }
        CoordOp::InvScaleNonUniform { src, inv_factors } => {
            let p = coords[*src as usize];
            Some([
                p[0].mul(Dual3::constant(inv_factors[0])),
                p[1].mul(Dual3::constant(inv_factors[1])),
                p[2].mul(Dual3::constant(inv_factors[2])),
            ])
        }
        CoordOp::Mirror { src, axis } => {
            let p = coords[*src as usize];
            Some(match axis {
                Axis::X => [p[0].abs(), p[1], p[2]],
                Axis::Y => [p[0], p[1].abs(), p[2]],
                Axis::Z => [p[0], p[1], p[2].abs()],
            })
        }
        CoordOp::Twist { src, amount } => {
            let p = coords[*src as usize];
            let a = p[1].mul(Dual3::constant(*amount));
            let s = a.sin();
            let c = a.cos();
            Some([
                c.mul(p[0]).sub(s.mul(p[2])),
                p[1],
                s.mul(p[0]).add(c.mul(p[2])),
            ])
        }
        CoordOp::Bend { src, amount } => {
            let p = coords[*src as usize];
            let a = p[0].mul(Dual3::constant(*amount));
            let s = a.sin();
            let c = a.cos();
            Some([
                c.mul(p[0]).sub(s.mul(p[1])),
                s.mul(p[0]).add(c.mul(p[1])),
                p[2],
            ])
        }
        CoordOp::Warp {
            src,
            wx,
            wy,
            wz,
            amplitude,
        } => {
            let p = coords[*src as usize];
            let amp = Dual3::constant(*amplitude);
            Some([
                p[0].add(vals[*wx as usize].mul(amp)),
                p[1].add(vals[*wy as usize].mul(amp)),
                p[2].add(vals[*wz as usize].mul(amp)),
            ])
        }
        _ => None, // fall back cleanly to ULP-safe numerical normal
    }
}

fn eval_op_dual(op: &EvalOp, p: [Dual3; 3], vals: &[Dual3]) -> Option<Dual3> {
    let zero = Dual3::constant(0.0);
    Some(match op {
        EvalOp::Const(v) => Dual3::constant(*v),
        EvalOp::CoordAxis(Axis::X) => p[0],
        EvalOp::CoordAxis(Axis::Y) => p[1],
        EvalOp::CoordAxis(Axis::Z) => p[2],
        EvalOp::Sphere { radius } => p[0]
            .mul(p[0])
            .add(p[1].mul(p[1]))
            .add(p[2].mul(p[2]))
            .sqrt()
            .sub(Dual3::constant(*radius)),
        EvalOp::Box {
            half_extents: b,
            rounding: r,
        } => {
            let qx = p[0].abs().sub(Dual3::constant(b[0] - r));
            let qy = p[1].abs().sub(Dual3::constant(b[1] - r));
            let qz = p[2].abs().sub(Dual3::constant(b[2] - r));
            let ox = qx.max(zero);
            let oy = qy.max(zero);
            let oz = qz.max(zero);
            let out = ox.mul(ox).add(oy.mul(oy)).add(oz.mul(oz)).sqrt();
            out.add(qx.max(qy.max(qz)).min(zero))
                .sub(Dual3::constant(*r))
        }
        EvalOp::Torus {
            major_radius,
            minor_radius,
        } => {
            let qx = p[0]
                .mul(p[0])
                .add(p[2].mul(p[2]))
                .sqrt()
                .sub(Dual3::constant(*major_radius));
            qx.mul(qx)
                .add(p[1].mul(p[1]))
                .sqrt()
                .sub(Dual3::constant(*minor_radius))
        }
        EvalOp::CappedCylinder {
            radius,
            half_height,
        } => {
            let dx = p[0]
                .mul(p[0])
                .add(p[2].mul(p[2]))
                .sqrt()
                .sub(Dual3::constant(*radius));
            let dy = p[1].abs().sub(Dual3::constant(*half_height));
            let ox = dx.max(zero);
            let oy = dy.max(zero);
            dx.max(dy).min(zero).add(ox.mul(ox).add(oy.mul(oy)).sqrt())
        }
        EvalOp::InfiniteCylinder { radius } => p[0]
            .mul(p[0])
            .add(p[2].mul(p[2]))
            .sqrt()
            .sub(Dual3::constant(*radius)),
        EvalOp::Plane { normal, offset } => p[0]
            .mul(Dual3::constant(normal[0]))
            .add(p[1].mul(Dual3::constant(normal[1])))
            .add(p[2].mul(Dual3::constant(normal[2])))
            .add(Dual3::constant(*offset)),
        EvalOp::Unary { op, child } => {
            let c = vals[*child as usize];
            match op {
                UnaryMathOp::Neg => c.neg(),
                UnaryMathOp::Abs => c.abs(),
                UnaryMathOp::Sqrt => c.sqrt(),
                UnaryMathOp::Sin => c.sin(),
                UnaryMathOp::Cos => c.cos(),
                UnaryMathOp::Exp => c.exp(),
                UnaryMathOp::Ln => c.ln(),
                UnaryMathOp::Saturate => c.clamp_scalar(zero, Dual3::constant(1.0)),
                _ => return None,
            }
        }
        EvalOp::Binary { op, a, b } => {
            let va = vals[*a as usize];
            let vb = vals[*b as usize];
            match op {
                BinaryMathOp::Add => va.add(vb),
                BinaryMathOp::Sub => va.sub(vb),
                BinaryMathOp::Mul => va.mul(vb),
                BinaryMathOp::Div => va.div(vb),
                BinaryMathOp::Min => va.min(vb),
                BinaryMathOp::Max => va.max(vb),
                BinaryMathOp::Pow => va.pow(vb),
                BinaryMathOp::Atan2 => va.atan2(vb),
                BinaryMathOp::Mod => return None,
            }
        }
        EvalOp::Union { children } => {
            let mut best = Dual3::constant(f32::INFINITY);
            for &r in children {
                best = best.min(vals[r as usize]);
            }
            best
        }
        EvalOp::Intersect { children } => {
            let mut best = Dual3::constant(f32::NEG_INFINITY);
            for &r in children {
                best = best.max(vals[r as usize]);
            }
            best
        }
        EvalOp::Subtract { a, b } => vals[*a as usize].max(vals[*b as usize].neg()),
        EvalOp::SmoothUnion { a, b, k } => {
            vals[*a as usize].smin(vals[*b as usize], Dual3::constant(*k))
        }
        EvalOp::SmoothSubtract { a, b, k } => {
            vals[*a as usize].smax(vals[*b as usize].neg(), Dual3::constant(*k))
        }
        EvalOp::SmoothIntersect { a, b, k } => {
            vals[*a as usize].smax(vals[*b as usize], Dual3::constant(*k))
        }
        EvalOp::Round { child, radius } => vals[*child as usize].sub(Dual3::constant(*radius)),
        EvalOp::Shell { child, thickness } => {
            vals[*child as usize].abs().sub(Dual3::constant(*thickness))
        }
        EvalOp::ScaleDist { child, factor } => vals[*child as usize].mul(Dual3::constant(*factor)),
        EvalOp::Displace {
            child,
            offset,
            amplitude,
            ..
        } => vals[*child as usize].add(vals[*offset as usize].mul(Dual3::constant(*amplitude))),
        EvalOp::Tag { child, .. } => vals[*child as usize],
        EvalOp::Expr { program, inputs } => {
            if !program.is_differentiable() {
                return None;
            }
            let (v, d) = program.eval_dual(p[0].v, p[1].v, p[2].v, &|idx, q| {
                // Chain rule for nested input fields
                if let Some((iv, id)) = inputs[idx].graph.eval_dual(0, q[0].v, q[1].v, q[2].v) {
                    Dual3 {
                        v: iv,
                        d: [
                            id[0] * q[0].d[0] + id[1] * q[1].d[0] + id[2] * q[2].d[0],
                            id[0] * q[0].d[1] + id[1] * q[1].d[1] + id[2] * q[2].d[1],
                            id[0] * q[0].d[2] + id[1] * q[1].d[2] + id[2] * q[2].d[2],
                        ],
                    }
                } else {
                    Dual3 {
                        v: f32::NAN,
                        d: [f32::NAN; 3],
                    }
                }
            })?;
            // Apply outer coordinate Jacobian
            Dual3 {
                v,
                d: [
                    d[0] * p[0].d[0] + d[1] * p[1].d[0] + d[2] * p[2].d[0],
                    d[0] * p[0].d[1] + d[1] * p[1].d[1] + d[2] * p[2].d[1],
                    d[0] * p[0].d[2] + d[1] * p[1].d[2] + d[2] * p[2].d[2],
                ],
            }
        }
        _ => return None,
    })
}

// ── Exact Complex Primitive Helpers ──────────────────────────────────────────

fn sd_round_cone(x: f32, y: f32, z: f32, a: [f32; 3], b: [f32; 3], r1: f32, r2: f32) -> f32 {
    let a64 = [a[0] as f64, a[1] as f64, a[2] as f64];
    let b64 = [b[0] as f64, b[1] as f64, b[2] as f64];
    let p = [x as f64, y as f64, z as f64];
    let ba = [b64[0] - a64[0], b64[1] - a64[1], b64[2] - a64[2]];
    let l2 = ba[0] * ba[0] + ba[1] * ba[1] + ba[2] * ba[2];
    let rr = r1 as f64 - r2 as f64;
    let sphere_dist = |c: [f64; 3], rad: f64| {
        let dx = p[0] - c[0];
        let dy = p[1] - c[1];
        let dz = p[2] - c[2];
        (dx * dx + dy * dy + dz * dz).sqrt() - rad
    };
    if l2.sqrt() <= rr.abs() {
        if r1 >= r2 {
            sphere_dist(a64, r1 as f64) as f32
        } else {
            sphere_dist(b64, r2 as f64) as f32
        }
    } else {
        let a2 = l2 - rr * rr;
        let il2 = 1.0 / l2;
        let pa = [p[0] - a64[0], p[1] - a64[1], p[2] - a64[2]];
        let y_ = pa[0] * ba[0] + pa[1] * ba[1] + pa[2] * ba[2];
        let z_ = y_ - l2;
        let px = [
            pa[0] * l2 - ba[0] * y_,
            pa[1] * l2 - ba[1] * y_,
            pa[2] * l2 - ba[2] * y_,
        ];
        let x2 = px[0] * px[0] + px[1] * px[1] + px[2] * px[2];
        let y2 = y_ * y_ * l2;
        let z2 = z_ * z_ * l2;
        let k = rr.signum() * rr * rr * x2;
        let dist = if z_.signum() * a2 * z2 > k {
            (x2 + z2).max(0.0).sqrt() * il2 - r2 as f64
        } else if y_.signum() * a2 * y2 < k {
            (x2 + y2).max(0.0).sqrt() * il2 - r1 as f64
        } else {
            ((x2 * a2 * il2).max(0.0).sqrt() + y_ * rr) * il2 - r1 as f64
        };
        dist as f32
    }
}

fn sd_square_pyramid(px: f32, py: f32, pz: f32, half_base: f32, height: f32) -> f32 {
    type V3 = [f64; 3];
    #[inline]
    fn sub(a: V3, b: V3) -> V3 {
        [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
    }
    #[inline]
    fn dot(a: V3, b: V3) -> f64 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }
    #[inline]
    fn length(v: V3) -> f64 {
        dot(v, v).sqrt()
    }
    #[inline]
    fn seg(a: V3, b: V3, t: f64) -> V3 {
        [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ]
    }
    fn tri_dist(p: V3, a: V3, b: V3, c: V3) -> f64 {
        let ab = sub(b, a);
        let ac = sub(c, a);
        let ap = sub(p, a);
        let d1 = dot(ab, ap);
        let d2 = dot(ac, ap);
        if d1 <= 0.0 && d2 <= 0.0 {
            return length(ap);
        }
        let bp = sub(p, b);
        let d3 = dot(ab, bp);
        let d4 = dot(ac, bp);
        if d3 >= 0.0 && d4 <= d3 {
            return length(bp);
        }
        let vc = d1 * d4 - d3 * d2;
        if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
            return length(sub(p, seg(a, b, d1 / (d1 - d3))));
        }
        let cp = sub(p, c);
        let d5 = dot(ab, cp);
        let d6 = dot(ac, cp);
        if d6 >= 0.0 && d5 <= d6 {
            return length(cp);
        }
        let vb = d5 * d2 - d1 * d6;
        if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
            return length(sub(p, seg(a, c, d2 / (d2 - d6))));
        }
        let va = d3 * d6 - d5 * d4;
        if va <= 0.0 && d4 >= d3 && d5 >= d6 {
            let t = (d4 - d3) / ((d4 - d3) + (d5 - d6));
            return length(sub(p, seg(b, c, t)));
        }
        let denom = 1.0 / (va + vb + vc);
        let v = vb * denom;
        let w = vc * denom;
        length(sub(
            p,
            [
                a[0] + ab[0] * v + ac[0] * w,
                a[1] + ab[1] * v + ac[1] * w,
                a[2] + ab[2] * v + ac[2] * w,
            ],
        ))
    }

    let p = [px as f64, py as f64, pz as f64];
    let h = height as f64;
    let r = half_base as f64;
    let base_y = -0.5 * h;
    let apex_y = 0.5 * h;
    let apex = [0.0, apex_y, 0.0];
    let corners = [
        [-r, base_y, -r],
        [r, base_y, -r],
        [r, base_y, r],
        [-r, base_y, r],
    ];
    let mut dist = f64::INFINITY;
    for i in 0..4 {
        dist = dist.min(tri_dist(p, corners[i], corners[(i + 1) % 4], apex));
    }
    let dx = (p[0].abs() - r).max(0.0);
    let dz = (p[2].abs() - r).max(0.0);
    let dy = p[1] - base_y;
    dist = dist.min((dx * dx + dy * dy + dz * dz).sqrt());
    let section = r * ((apex_y - p[1]) / h);
    let inside = p[1] >= base_y && p[1] <= apex_y && p[0].abs() <= section && p[2].abs() <= section;
    (if inside { -dist } else { dist }) as f32
}
