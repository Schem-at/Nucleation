//! SDF node tree: a serde JSON AST of signed-distance primitives, operators,
//! transforms, and noise modifiers.
//!
//! Distance functions follow Inigo Quilez's reference formulations
//! (<https://iquilezles.org/articles/distfunctions/>). Nodes marked
//! *approximate* return a lower bound rather than an exact Euclidean
//! distance — safe for inside/outside sampling, imprecise for sphere tracing.

use super::noise::{fbm3, hash01_3, value_noise3};
use super::program::Program;
use serde::{Deserialize, Serialize};

/// What a `Cells` (Worley / cellular) node returns per point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum CellMode {
    /// Distance to the nearest seed (rounded blobs around each cell center).
    F1,
    /// Distance to the second-nearest seed.
    F2,
    /// `F2 - F1`: small on cell boundaries, the classic Voronoi crack field.
    #[default]
    F2MinusF1,
    /// A per-cell pseudo-random constant in `[0, 1)`: the Voronoi mosaic.
    Value,
}

/// Axis selector for mirror operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    X,
    Y,
    Z,
}

/// Axis-aligned bounding box in continuous space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Aabb {
    fn grow(self, amount: f32) -> Aabb {
        Aabb {
            min: [
                self.min[0] - amount,
                self.min[1] - amount,
                self.min[2] - amount,
            ],
            max: [
                self.max[0] + amount,
                self.max[1] + amount,
                self.max[2] + amount,
            ],
        }
    }

    fn union(self, other: Aabb) -> Aabb {
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

    fn intersection(self, other: Aabb) -> Aabb {
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

/// One node of the SDF tree. Serialized as `{"type": "...", ...}` JSON.
///
/// Primitives are centered at the origin; use [`SdfNode::Translate`] /
/// [`SdfNode::Rotate`] / [`SdfNode::Scale`] to position them.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SdfNode {
    // ── Primitives ─────────────────────────────────────────────────────────
    /// Exact.
    Sphere {
        radius: f32,
    },
    /// Exact. `halfExtents` are the FULL half-extents including `rounding`.
    Box {
        half_extents: [f32; 3],
        #[serde(default)]
        rounding: f32,
    },
    /// Exact. Ring in the XZ plane.
    Torus {
        major_radius: f32,
        minor_radius: f32,
    },
    /// Exact. Ring in the XZ plane cut down to an arc: `cap_angle` (degrees, in
    /// `(0, 180]`) is the half-aperture measured from the +X axis, mirrored
    /// across X by symmetry. `cap_angle: 180` is identical to [`SdfNode::Torus`].
    CappedTorus {
        major_radius: f32,
        minor_radius: f32,
        cap_angle: f32,
    },
    /// Exact. Chain-link shape: a torus (`major_radius`/`minor_radius`, ring in
    /// XZ, tube toward Y) stretched by `half_length` along Z, capped by two
    /// half-tori. `half_length: 0` is identical to [`SdfNode::Torus`].
    Link {
        major_radius: f32,
        minor_radius: f32,
        half_length: f32,
    },
    /// Exact. Line segment `a`→`b` with radius.
    Capsule {
        a: [f32; 3],
        b: [f32; 3],
        radius: f32,
    },
    /// Exact. Convex hull of two spheres (`radius` `r1` at `a`, `r2` at `b`):
    /// a capsule with a linear taper between the two end radii instead of a
    /// constant one. `r1 == r2` is identical to [`SdfNode::Capsule`].
    RoundCone {
        a: [f32; 3],
        b: [f32; 3],
        r1: f32,
        r2: f32,
    },
    /// Exact. Sphere of `radius` intersected with an infinite cone of
    /// half-aperture `angle` (degrees, in `(0, 180)`) from the +Y axis, apex
    /// at the origin. The apex is always a boundary corner (distance 0)
    /// regardless of `angle`; use [`SdfNode::Sphere`] for a full sphere
    /// rather than `angle: 180`, which degenerates the cone constraint away
    /// everywhere except that single point.
    SolidAngle {
        radius: f32,
        angle: f32,
    },
    /// Exact. Sphere of `radius` cut by the plane `y = height`, keeping the
    /// cap above it (`y >= height`): a dome. `height: 0` is a hemisphere;
    /// `height` near `radius` is a shallow cap; `height` near `-radius` is
    /// nearly the full sphere.
    CutSphere {
        radius: f32,
        height: f32,
    },
    /// Exact. Open (hollow) shell of the same dome as [`SdfNode::CutSphere`]:
    /// only the spherical cap surface is solid, offset by `thickness`, with
    /// no flat floor — a bowl.
    CutHollowSphere {
        radius: f32,
        height: f32,
        thickness: f32,
    },
    /// Exact. Y-axis aligned.
    CappedCylinder {
        radius: f32,
        half_height: f32,
    },
    /// Exact but unbounded along Y — sampling requires explicit bounds.
    InfiniteCylinder {
        radius: f32,
    },
    /// Exact (iq's sdCappedCone). Y-axis aligned; `r1` bottom, `r2` top.
    CappedCone {
        half_height: f32,
        r1: f32,
        r2: f32,
    },
    /// Exact but unbounded — sampling requires explicit bounds.
    Plane {
        normal: [f32; 3],
        #[serde(default)]
        offset: f32,
    },
    /// *Approximate* (iq's bound formulation; underestimates near the poles).
    Ellipsoid {
        radii: [f32; 3],
    },
    /// Exact.
    Octahedron {
        size: f32,
    },
    /// Exact. Hexagonal cross-section in XZ, extruded along Y.
    HexPrism {
        radius: f32,
        half_height: f32,
    },
    /// *Approximate*: superellipse cross-section in XZ (`(|x|/hx)^p + (|z|/hz)^p ≤ 1`)
    /// extruded along Y with flat top/bottom. The flat-plateau primitive.
    SuperPrism {
        half_extents: [f32; 3],
        exponent: f32,
    },
    /// Exact. Hollow wireframe of a box: only the 12 edge beams (each
    /// `thickness` thick) are solid. `half_extents` are the outer half-extents
    /// of the box the frame is cut from.
    BoxFrame {
        half_extents: [f32; 3],
        thickness: f32,
    },
    /// Exact but unbounded — sampling requires explicit bounds. Single-nappe
    /// cone, apex at the origin, axis +Y, half-aperture `angle` (degrees,
    /// strictly in `(0, 90)`) measured from that axis.
    InfiniteCone {
        angle: f32,
    },
    /// Exact. Square-base pyramid, vertically centered: base (half-extent
    /// `half_base` in X/Z) at `y = -height/2`, apex at `y = height/2`. IQ's
    /// exact pyramid SDF, uniformly scaled to the given base/height.
    SquarePyramid {
        half_base: f32,
        height: f32,
    },

    // ── Operators ──────────────────────────────────────────────────────────
    Union {
        children: Vec<SdfNode>,
    },
    Intersect {
        children: Vec<SdfNode>,
    },
    /// `a` minus `b`.
    Subtract {
        a: Box<SdfNode>,
        b: Box<SdfNode>,
    },
    SmoothUnion {
        a: Box<SdfNode>,
        b: Box<SdfNode>,
        k: f32,
    },
    SmoothSubtract {
        a: Box<SdfNode>,
        b: Box<SdfNode>,
        k: f32,
    },
    SmoothIntersect {
        a: Box<SdfNode>,
        b: Box<SdfNode>,
        k: f32,
    },
    /// Rounds (inflates) the child surface outward by `radius`.
    Round {
        child: Box<SdfNode>,
        radius: f32,
    },
    /// Hollow shell (onion) of the child surface with given thickness.
    Shell {
        child: Box<SdfNode>,
        thickness: f32,
    },
    /// Symmetric difference (XOR) of two children: `max(min(a,b), -max(a,b))`.
    /// Exact outside both shapes and inside exactly one; only a conservative
    /// bound (like [`SdfNode::Intersect`]) where their interiors overlap.
    Xor {
        a: Box<SdfNode>,
        b: Box<SdfNode>,
    },
    /// Stretches the child outward by `half_lengths` along each axis before
    /// evaluating it (IQ's corrected `opElongate`: `q = abs(p) -
    /// half_lengths`, then `child(max(q,0)) + min(max(q.x,q.y,q.z),0)`).
    /// Exact for convex children, a conservative bound otherwise.
    Elongate {
        child: Box<SdfNode>,
        half_lengths: [f32; 3],
    },

    // ── Transforms ─────────────────────────────────────────────────────────
    Translate {
        child: Box<SdfNode>,
        offset: [f32; 3],
    },
    /// Euler angles in degrees, applied to the object in X, then Y, then Z order.
    Rotate {
        child: Box<SdfNode>,
        angles: [f32; 3],
    },
    /// Uniform scale.
    Scale {
        child: Box<SdfNode>,
        factor: f32,
    },
    /// Mirrors across the plane orthogonal to `axis` (evaluates `abs(coord)`).
    Mirror {
        child: Box<SdfNode>,
        axis: Axis,
    },
    /// Infinite (or counted) repetition. `spacing` 0 on an axis disables
    /// repetition on that axis. With `count = [nx, ny, nz]` the pattern is
    /// clamped to that many instances per side of the origin (bounded).
    Repeat {
        child: Box<SdfNode>,
        spacing: [f32; 3],
        #[serde(default)]
        count: Option<[u32; 3]>,
    },

    // ── Noise ──────────────────────────────────────────────────────────────
    /// Adds seeded FBM value noise to the child's distance (surface displacement).
    /// *Approximate*: bounds grow by `amplitude`.
    Displace {
        child: Box<SdfNode>,
        amplitude: f32,
        frequency: f32,
        seed: i32,
        #[serde(default = "default_octaves")]
        octaves: u32,
    },
    /// Domain-warps the sample point with seeded value noise before
    /// evaluating the child. *Approximate*: bounds grow by `amplitude`.
    Warp {
        child: Box<SdfNode>,
        amplitude: f32,
        frequency: f32,
        seed: i32,
    },
    /// Cellular / Worley noise: a jittered seed point per grid cell, returning a
    /// scalar per sample chosen by `mode` (F1, F2, F2-F1, or a per-cell value),
    /// minus `threshold`. Unbounded on its own (wrap it in `sdfBounded` or
    /// intersect it with a bounded shape); as an SDF it is solid where the
    /// returned value is negative, so `mode: f2MinusF1` with a small `threshold`
    /// makes a Voronoi foam of cell walls, and as a field brush its raw value
    /// (`threshold: 0`) paints Voronoi patterns.
    Cells {
        #[serde(default = "default_cell_frequency")]
        frequency: f32,
        #[serde(default)]
        seed: i32,
        #[serde(default = "default_jitter")]
        jitter: f32,
        #[serde(default)]
        mode: CellMode,
        #[serde(default)]
        threshold: f32,
    },

    /// A portable, sandboxed custom field: a validated, serde-serializable
    /// stack-based typed bytecode [`Program`] with explicit finite bounds
    /// and distance-kind metadata. See [`super::program`].
    Program {
        program: Box<Program>,
    },
}

fn default_cell_frequency() -> f32 {
    0.1
}

fn default_jitter() -> f32 {
    1.0
}

fn default_octaves() -> u32 {
    3
}

/// Maximum nesting depth an [`SdfNode`] tree may reach for
/// [`SdfNode::validate`] to accept it. Well above any realistically
/// authored tree, but bounds recursion depth against malicious/generated
/// JSON (e.g. a long chain of single-child wrapper nodes).
const MAX_TREE_DEPTH: u32 = 64;

/// Maximum total node count an [`SdfNode`] tree may reach for
/// [`SdfNode::validate`] to accept it. Bounds the work a wide tree (e.g. a
/// `Union`/`Intersect` with many children) can force regardless of depth.
const MAX_TREE_NODES: u32 = 5_000;

/// All values finite.
fn finite_all(values: &[f32]) -> Result<(), String> {
    if values.iter().all(|v| v.is_finite()) {
        Ok(())
    } else {
        Err("value must be finite".into())
    }
}

/// All values finite and strictly positive.
fn positive_all(values: &[f32]) -> Result<(), String> {
    finite_all(values)?;
    if values.iter().all(|v| *v > 0.0) {
        Ok(())
    } else {
        Err("value must be positive".into())
    }
}

/// All values finite and non-negative.
fn non_negative_all(values: &[f32]) -> Result<(), String> {
    finite_all(values)?;
    if values.iter().all(|v| *v >= 0.0) {
        Ok(())
    } else {
        Err("value must be non-negative".into())
    }
}

#[inline]
fn len3(x: f32, y: f32, z: f32) -> f32 {
    (x * x + y * y + z * z).sqrt()
}

#[inline]
fn len2(x: f32, y: f32) -> f32 {
    (x * x + y * y).sqrt()
}

/// Nearest point, in the (radial, y) half-plane, on the sphere-cap arc of
/// radius `r` cut at `h` (rim at `(w, h)`, `w = sqrt(r*r - h*h)`): the
/// radial projection of `(rho, y)` onto the full circle, clamped to the
/// rim when that projection would fall below the cut.
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

/// GLSL `sign`: unlike `f32::signum`, zero maps to zero rather than +1.
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
fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// iq polynomial smooth min.
#[inline]
fn smin(a: f32, b: f32, k: f32) -> f32 {
    if k <= 0.0 {
        return a.min(b);
    }
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    mix(b, a, h) - k * h * (1.0 - h)
}

#[inline]
fn smax(a: f32, b: f32, k: f32) -> f32 {
    -smin(-a, -b, k)
}

/// iq's exact square-pyramid SDF for the unit pyramid: base half-extent
/// `0.5` in X/Z at `y = 0`, apex at `(0, h, 0)`.
#[inline]
fn sd_pyramid_unit(px: f32, py: f32, pz: f32, h: f32) -> f32 {
    let m2 = h * h + 0.25;

    let (mut ax, mut az) = (px.abs(), pz.abs());
    if az > ax {
        std::mem::swap(&mut ax, &mut az);
    }
    ax -= 0.5;
    az -= 0.5;

    let qx = az;
    let qy = h * py - 0.5 * ax;
    let qz = h * ax + 0.5 * py;

    let s = (-qx).max(0.0);
    let t = ((qy - 0.5 * az) / (m2 + 0.25)).clamp(0.0, 1.0);

    let a = m2 * (qx + s) * (qx + s) + qy * qy;
    let b = m2 * (qx + t * m2) * (qx + t * m2) + (qy - m2 * t) * (qy - m2 * t);

    let d2 = if qy.min(-qx * m2 - qy * 0.5) > 0.0 {
        0.0
    } else {
        a.min(b)
    };

    ((d2 + qz * qz) / m2).max(0.0).sqrt() * glsl_sign(qz.max(-py))
}

/// Column-major 3x3 rotation helpers (row-vector free, plain arrays).
fn rot_matrix(deg: [f32; 3]) -> [[f32; 3]; 3] {
    let (rx, ry, rz) = (
        deg[0].to_radians(),
        deg[1].to_radians(),
        deg[2].to_radians(),
    );
    let (sx, cx) = rx.sin_cos();
    let (sy, cy) = ry.sin_cos();
    let (sz, cz) = rz.sin_cos();
    // R = Rz * Ry * Rx (object rotated X first, then Y, then Z)
    [
        [cz * cy, cz * sy * sx - sz * cx, cz * sy * cx + sz * sx],
        [sz * cy, sz * sy * sx + cz * cx, sz * sy * cx - cz * sx],
        [-sy, cy * sx, cy * cx],
    ]
}

/// Multiply the TRANSPOSE (= inverse for rotations) of `m` with `p`.
#[inline]
fn inv_rotate(m: &[[f32; 3]; 3], p: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * p[0] + m[1][0] * p[1] + m[2][0] * p[2],
        m[0][1] * p[0] + m[1][1] * p[1] + m[2][1] * p[2],
        m[0][2] * p[0] + m[1][2] * p[1] + m[2][2] * p[2],
    ]
}

#[inline]
fn rotate_point(m: &[[f32; 3]; 3], p: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2],
        m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2],
        m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2],
    ]
}

impl SdfNode {
    /// Parse a node tree from its JSON representation, then recursively
    /// [`validate`](SdfNode::validate) it.
    pub fn from_json(json: &str) -> Result<SdfNode, String> {
        let node: SdfNode =
            serde_json::from_str(json).map_err(|e| format!("Invalid SDF JSON: {e}"))?;
        node.validate()?;
        Ok(node)
    }

    /// Serialize this tree to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("SDF serialization failed: {e}"))
    }

    /// Recursively validate every node's parameters: finite values, valid
    /// signs/ranges, enum-specific constraints, transform sanity, and a
    /// bound on tree depth/size. Mirrors the checks the typed `Sdf` builder
    /// API applies at construction time (see `bridge/sdf.rs`), so a JSON
    /// tree and an equivalent typed tree are held to the same standard.
    ///
    /// Called automatically by [`SdfNode::from_json`]; hand-built trees
    /// (e.g. deserialized manually, or assembled outside the typed builder)
    /// can call this directly before evaluating or sampling them.
    pub fn validate(&self) -> Result<(), String> {
        let mut budget = MAX_TREE_NODES;
        self.validate_at(0, &mut budget)
    }

    fn validate_at(&self, depth: u32, budget: &mut u32) -> Result<(), String> {
        if depth > MAX_TREE_DEPTH {
            return Err(format!(
                "SDF tree exceeds the maximum nesting depth of {MAX_TREE_DEPTH}"
            ));
        }
        if *budget == 0 {
            return Err(format!(
                "SDF tree exceeds the maximum node count of {MAX_TREE_NODES}"
            ));
        }
        *budget -= 1;

        match self {
            SdfNode::Sphere { radius } => positive_all(&[*radius])?,

            SdfNode::Box {
                half_extents,
                rounding,
            } => {
                positive_all(half_extents)?;
                non_negative_all(&[*rounding])?;
                if *rounding > half_extents[0].min(half_extents[1]).min(half_extents[2]) {
                    return Err("box rounding cannot exceed its smallest half-extent".into());
                }
            }

            SdfNode::Torus {
                major_radius,
                minor_radius,
            } => positive_all(&[*major_radius, *minor_radius])?,

            SdfNode::CappedTorus {
                major_radius,
                minor_radius,
                cap_angle,
            } => {
                positive_all(&[*major_radius, *minor_radius])?;
                finite_all(&[*cap_angle])?;
                if *cap_angle <= 0.0 || *cap_angle > 180.0 {
                    return Err("capped torus cap_angle must be in (0, 180]".into());
                }
            }

            SdfNode::Link {
                major_radius,
                minor_radius,
                half_length,
            } => {
                positive_all(&[*major_radius, *minor_radius])?;
                non_negative_all(&[*half_length])?;
            }

            SdfNode::Capsule { a, b, radius } => {
                finite_all(a)?;
                finite_all(b)?;
                positive_all(&[*radius])?;
            }

            SdfNode::RoundCone { a, b, r1, r2 } => {
                finite_all(a)?;
                finite_all(b)?;
                positive_all(&[*r1, *r2])?;
                if a == b {
                    return Err("round cone endpoints must not coincide".into());
                }
            }

            SdfNode::SolidAngle { radius, angle } => {
                positive_all(&[*radius])?;
                finite_all(&[*angle])?;
                if *angle <= 0.0 || *angle >= 180.0 {
                    return Err("solid angle must be in (0, 180) degrees".into());
                }
            }

            SdfNode::CutSphere { radius, height } => {
                positive_all(&[*radius])?;
                finite_all(&[*height])?;
                if *height < -*radius || *height > *radius {
                    return Err("cut sphere height must be within [-radius, radius]".into());
                }
            }

            SdfNode::CutHollowSphere {
                radius,
                height,
                thickness,
            } => {
                positive_all(&[*radius, *thickness])?;
                finite_all(&[*height])?;
                if *height < -*radius || *height > *radius {
                    return Err("cut hollow sphere height must be within [-radius, radius]".into());
                }
            }

            SdfNode::CappedCylinder {
                radius,
                half_height,
            } => positive_all(&[*radius, *half_height])?,

            SdfNode::InfiniteCylinder { radius } => positive_all(&[*radius])?,

            SdfNode::InfiniteCone { angle } => {
                finite_all(&[*angle])?;
                if *angle <= 0.0 || *angle >= 90.0 {
                    return Err("infinite cone angle must be in (0, 90) degrees".into());
                }
            }

            SdfNode::SquarePyramid { half_base, height } => {
                positive_all(&[*half_base, *height])?;
            }

            SdfNode::CappedCone {
                half_height,
                r1,
                r2,
            } => {
                positive_all(&[*half_height])?;
                non_negative_all(&[*r1, *r2])?;
                if *r1 == 0.0 && *r2 == 0.0 {
                    return Err("capped cone radii cannot both be zero".into());
                }
            }

            SdfNode::Plane { normal, offset } => {
                finite_all(normal)?;
                finite_all(&[*offset])?;
                let length = ((normal[0] as f64).powi(2)
                    + (normal[1] as f64).powi(2)
                    + (normal[2] as f64).powi(2))
                .sqrt();
                if !length.is_finite() || length <= f64::from(f32::EPSILON) {
                    return Err("plane normal must not be degenerate".into());
                }
            }

            SdfNode::Ellipsoid { radii } => positive_all(radii)?,

            SdfNode::Octahedron { size } => positive_all(&[*size])?,

            SdfNode::HexPrism {
                radius,
                half_height,
            } => positive_all(&[*radius, *half_height])?,

            SdfNode::SuperPrism {
                half_extents,
                exponent,
            } => {
                positive_all(half_extents)?;
                positive_all(&[*exponent])?;
            }

            SdfNode::BoxFrame {
                half_extents,
                thickness,
            } => {
                positive_all(half_extents)?;
                non_negative_all(&[*thickness])?;
                if *thickness > half_extents[0].min(half_extents[1]).min(half_extents[2]) {
                    return Err("box frame thickness cannot exceed its smallest half-extent".into());
                }
            }

            SdfNode::Union { children } | SdfNode::Intersect { children } => {
                for child in children {
                    child.validate_at(depth + 1, budget)?;
                }
            }

            SdfNode::Subtract { a, b } => {
                a.validate_at(depth + 1, budget)?;
                b.validate_at(depth + 1, budget)?;
            }

            SdfNode::SmoothUnion { a, b, k }
            | SdfNode::SmoothSubtract { a, b, k }
            | SdfNode::SmoothIntersect { a, b, k } => {
                positive_all(&[*k])?;
                a.validate_at(depth + 1, budget)?;
                b.validate_at(depth + 1, budget)?;
            }

            SdfNode::Round { child, radius } => {
                non_negative_all(&[*radius])?;
                child.validate_at(depth + 1, budget)?;
            }

            SdfNode::Shell { child, thickness } => {
                positive_all(&[*thickness])?;
                child.validate_at(depth + 1, budget)?;
            }

            SdfNode::Translate { child, offset } => {
                finite_all(offset)?;
                child.validate_at(depth + 1, budget)?;
            }

            SdfNode::Rotate { child, angles } => {
                finite_all(angles)?;
                child.validate_at(depth + 1, budget)?;
            }

            SdfNode::Scale { child, factor } => {
                positive_all(&[*factor])?;
                child.validate_at(depth + 1, budget)?;
            }

            SdfNode::Mirror { child, .. } => {
                child.validate_at(depth + 1, budget)?;
            }

            SdfNode::Repeat {
                child,
                spacing,
                count: _,
            } => {
                finite_all(spacing)?;
                if spacing.iter().any(|s| *s < 0.0) || spacing.iter().all(|s| *s == 0.0) {
                    return Err("repeat spacing must be non-negative and not all zero".into());
                }
                child.validate_at(depth + 1, budget)?;
            }

            SdfNode::Displace {
                child,
                amplitude,
                frequency,
                octaves,
                seed: _,
            } => {
                non_negative_all(&[*amplitude])?;
                positive_all(&[*frequency])?;
                if !(1..=8).contains(octaves) {
                    return Err("displace octaves must be in 1..=8".into());
                }
                child.validate_at(depth + 1, budget)?;
            }

            SdfNode::Warp {
                child,
                amplitude,
                frequency,
                seed: _,
            } => {
                non_negative_all(&[*amplitude])?;
                positive_all(&[*frequency])?;
                child.validate_at(depth + 1, budget)?;
            }

            SdfNode::Cells {
                frequency,
                jitter,
                threshold,
                seed: _,
                mode: _,
            } => {
                positive_all(&[*frequency])?;
                non_negative_all(&[*jitter])?;
                finite_all(&[*threshold])?;
            }

            SdfNode::Program { program } => {
                super::program::validate(program.data()).map_err(String::from)?;
            }
        }
        Ok(())
    }

    /// Signed distance at a point (negative = inside).
    pub fn eval(&self, x: f32, y: f32, z: f32) -> f32 {
        match self {
            SdfNode::Sphere { radius } => len3(x, y, z) - radius,

            SdfNode::Box {
                half_extents: b,
                rounding,
            } => {
                let r = rounding.max(0.0).min(b[0].min(b[1]).min(b[2]));
                let qx = x.abs() - (b[0] - r);
                let qy = y.abs() - (b[1] - r);
                let qz = z.abs() - (b[2] - r);
                let outside = len3(qx.max(0.0), qy.max(0.0), qz.max(0.0));
                let inside = qx.max(qy.max(qz)).min(0.0);
                outside + inside - r
            }

            SdfNode::Torus {
                major_radius,
                minor_radius,
            } => {
                let qx = len2(x, z) - major_radius;
                len2(qx, y) - minor_radius
            }

            SdfNode::CappedTorus {
                major_radius: ra,
                minor_radius: rb,
                cap_angle,
            } => {
                let (sin_a, cos_a) = cap_angle.to_radians().sin_cos();
                let px = x.abs();
                let pz = z;
                let k = if cos_a * px > sin_a * pz {
                    px * sin_a + pz * cos_a
                } else {
                    len2(px, pz)
                };
                (px * px + pz * pz + y * y + ra * ra - 2.0 * ra * k)
                    .max(0.0)
                    .sqrt()
                    - rb
            }

            SdfNode::Link {
                major_radius,
                minor_radius,
                half_length,
            } => {
                let qz = (z.abs() - half_length).max(0.0);
                let ring = len2(x, qz) - major_radius;
                len2(ring, y) - minor_radius
            }

            SdfNode::Capsule { a, b, radius } => {
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

            SdfNode::RoundCone { a, b, r1, r2 } => {
                // iq's sdRoundCone: convex hull of two spheres, exact.
                let ba = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let l2 = ba[0] * ba[0] + ba[1] * ba[1] + ba[2] * ba[2];
                let rr = r1 - r2;
                let a2 = l2 - rr * rr;
                let il2 = 1.0 / l2;

                let pa = [x - a[0], y - a[1], z - a[2]];
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
                if z_.signum() * a2 * z2 > k {
                    (x2 + z2).max(0.0).sqrt() * il2 - r2
                } else if y_.signum() * a2 * y2 < k {
                    (x2 + y2).max(0.0).sqrt() * il2 - r1
                } else {
                    ((x2 * a2 * il2).max(0.0).sqrt() + y_ * rr) * il2 - r1
                }
            }

            SdfNode::SolidAngle { radius, angle } => {
                // iq's sdSolidAngle: sphere intersected with a cone from the origin.
                let (sin_a, cos_a) = angle.to_radians().sin_cos();
                let qx = len2(x, z);
                let qy = y;
                let l = len2(qx, qy) - radius;
                let dot_qc = (qx * sin_a + qy * cos_a).clamp(0.0, *radius);
                let mx = qx - sin_a * dot_qc;
                let my = qy - cos_a * dot_qc;
                let m = len2(mx, my);
                l.max(m * glsl_sign(cos_a * qx - sin_a * qy))
            }

            SdfNode::CutSphere { radius: r, height } => {
                let rho = len2(x, z);
                let w = (r * r - height * height).max(0.0).sqrt();
                let seg_x = rho.clamp(0.0, w);
                let dist_seg = len2(rho - seg_x, y - height);
                let (ax, ay) = cap_arc_nearest(rho, y, *r, *height, w);
                let dist_arc = len2(rho - ax, y - ay);
                let dist = dist_seg.min(dist_arc);
                let inside = rho * rho + y * y <= r * r && y >= *height;
                if inside {
                    -dist
                } else {
                    dist
                }
            }

            SdfNode::CutHollowSphere {
                radius: r,
                height,
                thickness,
            } => {
                let rho = len2(x, z);
                let w = (r * r - height * height).max(0.0).sqrt();
                let (ax, ay) = cap_arc_nearest(rho, y, *r, *height, w);
                len2(rho - ax, y - ay) - thickness
            }

            SdfNode::CappedCylinder {
                radius,
                half_height,
            } => {
                let dx = len2(x, z) - radius;
                let dy = y.abs() - half_height;
                dx.max(dy).min(0.0) + len2(dx.max(0.0), dy.max(0.0))
            }

            SdfNode::InfiniteCylinder { radius } => len2(x, z) - radius,

            SdfNode::CappedCone {
                half_height,
                r1,
                r2,
            } => {
                // iq sdCappedCone
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

            SdfNode::Plane { normal, offset } => {
                let n = *normal;
                let l = len3(n[0], n[1], n[2]).max(1e-9);
                (x * n[0] + y * n[1] + z * n[2]) / l + offset
            }

            SdfNode::Ellipsoid { radii: r } => {
                let k0 = len3(x / r[0], y / r[1], z / r[2]);
                let k1 = len3(x / (r[0] * r[0]), y / (r[1] * r[1]), z / (r[2] * r[2]));
                if k1 > 0.0 {
                    k0 * (k0 - 1.0) / k1
                } else {
                    -r[0].min(r[1]).min(r[2])
                }
            }

            SdfNode::Octahedron { size } => {
                let (px, py, pz) = (x.abs(), y.abs(), z.abs());
                let m = px + py + pz - size;
                // iq exact variant
                let (qx, qy, qz) = if 3.0 * px < m {
                    (px, py, pz)
                } else if 3.0 * py < m {
                    (py, pz, px)
                } else if 3.0 * pz < m {
                    (pz, px, py)
                } else {
                    return m * 0.577_350_26;
                };
                let k = (0.5 * (qz - qy + size)).clamp(0.0, *size);
                len3(qx, qy - size + k, qz - k)
            }

            SdfNode::HexPrism {
                radius,
                half_height,
            } => {
                // Hex cross-section in XZ, height along Y (iq sdHexPrism reoriented)
                const KX: f32 = -0.866_025_4;
                const KY: f32 = 0.5;
                const KZ: f32 = 0.577_350_27;
                let (mut px, py, mut pz) = (x.abs(), y.abs(), z.abs());
                let dot = KX * px + KY * pz;
                let m = 2.0 * dot.min(0.0);
                px -= m * KX;
                pz -= m * KY;
                let dx_clamped = px - px.clamp(-KZ * radius, KZ * radius);
                let d1 = len2(dx_clamped, pz - radius) * (pz - radius).signum();
                let d2 = py - half_height;
                d1.max(d2).min(0.0) + len2(d1.max(0.0), d2.max(0.0))
            }

            SdfNode::SuperPrism {
                half_extents: b,
                exponent,
            } => {
                let p = exponent.max(1.0);
                let s = (x.abs() / b[0]).powf(p) + (z.abs() / b[2]).powf(p);
                // Scaled implicit → approximate radial distance in the XZ plane
                let d_xz = (s.powf(1.0 / p) - 1.0) * b[0].min(b[2]);
                let d_y = y.abs() - b[1];
                d_xz.max(d_y).min(0.0) + len2(d_xz.max(0.0), d_y.max(0.0))
            }

            SdfNode::BoxFrame {
                half_extents: b,
                thickness: e,
            } => {
                let p = [x.abs() - b[0], y.abs() - b[1], z.abs() - b[2]];
                let q = [
                    (p[0] + e).abs() - e,
                    (p[1] + e).abs() - e,
                    (p[2] + e).abs() - e,
                ];
                // iq sdBoxFrame: three axis-aligned "leg" distances, each keeping
                // one coordinate at the un-shrunk `p` value.
                let leg = |v: [f32; 3]| {
                    len3(v[0].max(0.0), v[1].max(0.0), v[2].max(0.0))
                        + v[0].max(v[1]).max(v[2]).min(0.0)
                };
                let d1 = leg([p[0], q[1], q[2]]);
                let d2 = leg([q[0], p[1], q[2]]);
                let d3 = leg([q[0], q[1], p[2]]);
                d1.min(d2).min(d3)
            }

            SdfNode::InfiniteCone { angle } => {
                // iq's sdCone (infinite, single nappe), axis +Y, apex at origin.
                let (sin_a, cos_a) = angle.to_radians().sin_cos();
                let qx = len2(x, z);
                let qy = y;
                let dot_qc = (qx * sin_a + qy * cos_a).max(0.0);
                let mx = qx - sin_a * dot_qc;
                let my = qy - cos_a * dot_qc;
                let d = len2(mx, my);
                d * glsl_sign(qx * cos_a - qy * sin_a)
            }

            SdfNode::SquarePyramid { half_base, height } => {
                let hb = half_base.max(1e-9);
                let scale = 2.0 * hb;
                let h = height / scale;
                let px = x / scale;
                let py = (y + height * 0.5) / scale;
                let pz = z / scale;
                sd_pyramid_unit(px, py, pz, h) * scale
            }

            SdfNode::Union { children } => children
                .iter()
                .map(|c| c.eval(x, y, z))
                .fold(f32::INFINITY, f32::min),

            SdfNode::Intersect { children } => children
                .iter()
                .map(|c| c.eval(x, y, z))
                .fold(f32::NEG_INFINITY, f32::max),

            SdfNode::Subtract { a, b } => a.eval(x, y, z).max(-b.eval(x, y, z)),

            SdfNode::SmoothUnion { a, b, k } => smin(a.eval(x, y, z), b.eval(x, y, z), *k),
            SdfNode::SmoothSubtract { a, b, k } => smax(a.eval(x, y, z), -b.eval(x, y, z), *k),
            SdfNode::SmoothIntersect { a, b, k } => smax(a.eval(x, y, z), b.eval(x, y, z), *k),

            SdfNode::Round { child, radius } => child.eval(x, y, z) - radius,
            SdfNode::Shell { child, thickness } => child.eval(x, y, z).abs() - thickness,

            SdfNode::Xor { a, b } => {
                let da = a.eval(x, y, z);
                let db = b.eval(x, y, z);
                da.min(db).max(-da.max(db))
            }

            SdfNode::Elongate {
                child,
                half_lengths: h,
            } => {
                let qx = x.abs() - h[0];
                let qy = y.abs() - h[1];
                let qz = z.abs() - h[2];
                let d = child.eval(qx.max(0.0), qy.max(0.0), qz.max(0.0));
                d + qx.max(qy.max(qz)).min(0.0)
            }

            SdfNode::Translate { child, offset } => {
                child.eval(x - offset[0], y - offset[1], z - offset[2])
            }

            SdfNode::Rotate { child, angles } => {
                let m = rot_matrix(*angles);
                let p = inv_rotate(&m, [x, y, z]);
                child.eval(p[0], p[1], p[2])
            }

            SdfNode::Scale { child, factor } => {
                let f = if *factor == 0.0 { 1e-9 } else { *factor };
                child.eval(x / f, y / f, z / f) * f.abs()
            }

            SdfNode::Mirror { child, axis } => match axis {
                Axis::X => child.eval(x.abs(), y, z),
                Axis::Y => child.eval(x, y.abs(), z),
                Axis::Z => child.eval(x, y, z.abs()),
            },

            SdfNode::Repeat {
                child,
                spacing,
                count,
            } => {
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
                let n = count.map(|c| (c[0], c[1], c[2]));
                child.eval(
                    map(x, spacing[0], n.map(|c| c.0)),
                    map(y, spacing[1], n.map(|c| c.1)),
                    map(z, spacing[2], n.map(|c| c.2)),
                )
            }

            SdfNode::Displace {
                child,
                amplitude,
                frequency,
                seed,
                octaves,
            } => child.eval(x, y, z) + fbm3(x, y, z, *seed, *frequency, *octaves) * amplitude,

            SdfNode::Warp {
                child,
                amplitude,
                frequency,
                seed,
            } => {
                let wx = (value_noise3(x * frequency, y * frequency, z * frequency, *seed) * 2.0
                    - 1.0)
                    * amplitude;
                let wy = (value_noise3(
                    x * frequency,
                    y * frequency,
                    z * frequency,
                    seed.wrapping_add(7919),
                ) * 2.0
                    - 1.0)
                    * amplitude;
                let wz = (value_noise3(
                    x * frequency,
                    y * frequency,
                    z * frequency,
                    seed.wrapping_add(104_729),
                ) * 2.0
                    - 1.0)
                    * amplitude;
                child.eval(x + wx, y + wy, z + wz)
            }
            SdfNode::Cells {
                frequency,
                seed,
                jitter,
                mode,
                threshold,
            } => {
                let (px, py, pz) = (x * frequency, y * frequency, z * frequency);
                let (bx, by, bz) = (px.floor() as i32, py.floor() as i32, pz.floor() as i32);
                let (mut f1, mut f2) = (f32::INFINITY, f32::INFINITY);
                let mut best = (bx, by, bz);
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        for dz in -1..=1 {
                            let (gx, gy, gz) = (bx + dx, by + dy, bz + dz);
                            let sx = gx as f32 + jitter * hash01_3(gx, gy, gz, *seed);
                            let sy =
                                gy as f32 + jitter * hash01_3(gx, gy, gz, seed.wrapping_add(1));
                            let sz =
                                gz as f32 + jitter * hash01_3(gx, gy, gz, seed.wrapping_add(2));
                            let d =
                                ((px - sx).powi(2) + (py - sy).powi(2) + (pz - sz).powi(2)).sqrt();
                            if d < f1 {
                                f2 = f1;
                                f1 = d;
                                best = (gx, gy, gz);
                            } else if d < f2 {
                                f2 = d;
                            }
                        }
                    }
                }
                let inv_f = 1.0 / frequency.max(1e-6);
                let raw = match mode {
                    CellMode::F1 => f1 * inv_f,
                    CellMode::F2 => f2 * inv_f,
                    CellMode::F2MinusF1 => (f2 - f1) * inv_f,
                    CellMode::Value => hash01_3(best.0, best.1, best.2, seed.wrapping_add(7)),
                };
                raw - threshold
            }

            SdfNode::Program { program } => program.eval(x, y, z),
        }
    }

    /// Conservative AABB, or `None` when the node is unbounded (e.g. planes,
    /// uncounted repeats). Displace/warp/smooth bounds are grown estimates.
    pub fn bounds(&self) -> Option<Aabb> {
        fn sym(hx: f32, hy: f32, hz: f32) -> Option<Aabb> {
            Some(Aabb {
                min: [-hx, -hy, -hz],
                max: [hx, hy, hz],
            })
        }
        match self {
            SdfNode::Sphere { radius } => sym(*radius, *radius, *radius),
            SdfNode::Box {
                half_extents: b, ..
            } => sym(b[0], b[1], b[2]),
            SdfNode::Torus {
                major_radius,
                minor_radius,
            } => sym(
                major_radius + minor_radius,
                *minor_radius,
                major_radius + minor_radius,
            ),
            SdfNode::CappedTorus {
                major_radius,
                minor_radius,
                ..
            } => sym(
                major_radius + minor_radius,
                *minor_radius,
                major_radius + minor_radius,
            ),
            SdfNode::Link {
                major_radius,
                minor_radius,
                half_length,
            } => sym(
                major_radius + minor_radius,
                *minor_radius,
                major_radius + minor_radius + half_length,
            ),
            SdfNode::Capsule { a, b, radius } => Some(Aabb {
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
            SdfNode::RoundCone { a, b, r1, r2 } => Some(Aabb {
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
            SdfNode::SolidAngle { radius, .. } => sym(*radius, *radius, *radius),
            SdfNode::CutSphere { radius: r, height } => {
                let w = (r * r - height * height).max(0.0).sqrt();
                Some(Aabb {
                    min: [-w, *height, -w],
                    max: [w, *r, w],
                })
            }
            SdfNode::CutHollowSphere {
                radius: r,
                height,
                thickness,
            } => {
                let w = (r * r - height * height).max(0.0).sqrt();
                Some(Aabb {
                    min: [-w - thickness, height - thickness, -w - thickness],
                    max: [w + thickness, r + thickness, w + thickness],
                })
            }
            SdfNode::CappedCylinder {
                radius,
                half_height,
            } => sym(*radius, *half_height, *radius),
            SdfNode::InfiniteCylinder { .. } => None,
            SdfNode::CappedCone {
                half_height,
                r1,
                r2,
            } => {
                let r = r1.max(*r2);
                sym(r, *half_height, r)
            }
            SdfNode::Plane { .. } => None,
            SdfNode::Ellipsoid { radii } => sym(radii[0], radii[1], radii[2]),
            SdfNode::Octahedron { size } => sym(*size, *size, *size),
            SdfNode::HexPrism {
                radius,
                half_height,
            } => {
                // `radius` is the hexagon's apothem (inradius) toward the
                // flat edges (Z), matching the exact surface there. Along X
                // the hexagon's corners reach the circumradius instead:
                // 2 * radius / sqrt(3) (apothem / cos(30°)).
                const INV_COS_30: f32 = 1.154_700_5; // 2 / sqrt(3)
                sym(*radius * INV_COS_30, *half_height, *radius)
            }
            SdfNode::SuperPrism {
                half_extents: b, ..
            } => sym(b[0], b[1], b[2]),
            SdfNode::BoxFrame {
                half_extents: b, ..
            } => sym(b[0], b[1], b[2]),
            SdfNode::InfiniteCone { .. } => None,
            SdfNode::SquarePyramid { half_base, height } => {
                sym(*half_base, height * 0.5, *half_base)
            }

            SdfNode::Union { children } => {
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
            SdfNode::Intersect { children } => {
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
            SdfNode::Subtract { a, .. } => a.bounds(),
            SdfNode::SmoothUnion { a, b, k } => Some(a.bounds()?.union(b.bounds()?).grow(*k)),
            SdfNode::SmoothSubtract { a, b: _, k } => a.bounds().map(|bb| bb.grow(*k)),
            SdfNode::SmoothIntersect { a, b, k } => match (a.bounds(), b.bounds()) {
                (Some(ab), Some(bb)) => Some(ab.intersection(bb).grow(*k)),
                (Some(ab), None) => Some(ab.grow(*k)),
                (None, Some(bb)) => Some(bb.grow(*k)),
                (None, None) => None,
            },
            SdfNode::Round { child, radius } => child.bounds().map(|b| b.grow(*radius)),
            SdfNode::Shell { child, thickness } => child.bounds().map(|b| b.grow(*thickness)),
            SdfNode::Xor { a, b } => match (a.bounds(), b.bounds()) {
                (Some(ab), Some(bb)) => Some(ab.union(bb)),
                _ => None,
            },
            SdfNode::Elongate {
                child,
                half_lengths: h,
            } => child.bounds().map(|b| Aabb {
                min: [b.min[0] - h[0], b.min[1] - h[1], b.min[2] - h[2]],
                max: [b.max[0] + h[0], b.max[1] + h[1], b.max[2] + h[2]],
            }),

            SdfNode::Translate { child, offset } => child.bounds().map(|b| Aabb {
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
            SdfNode::Rotate { child, angles } => {
                let b = child.bounds()?;
                let m = rot_matrix(*angles);
                let mut min = [f32::INFINITY; 3];
                let mut max = [f32::NEG_INFINITY; 3];
                for i in 0..8 {
                    let corner = [
                        if i & 1 == 0 { b.min[0] } else { b.max[0] },
                        if i & 2 == 0 { b.min[1] } else { b.max[1] },
                        if i & 4 == 0 { b.min[2] } else { b.max[2] },
                    ];
                    let r = rotate_point(&m, corner);
                    for a in 0..3 {
                        min[a] = min[a].min(r[a]);
                        max[a] = max[a].max(r[a]);
                    }
                }
                Some(Aabb { min, max })
            }
            SdfNode::Scale { child, factor } => {
                let f = factor.abs();
                child.bounds().map(|b| Aabb {
                    min: [b.min[0] * f, b.min[1] * f, b.min[2] * f],
                    max: [b.max[0] * f, b.max[1] * f, b.max[2] * f],
                })
            }
            SdfNode::Mirror { child, axis } => {
                let b = child.bounds()?;
                let i = match axis {
                    Axis::X => 0,
                    Axis::Y => 1,
                    Axis::Z => 2,
                };
                let hi = b.max[i].abs().max(b.min[i].abs());
                let mut min = b.min;
                let mut max = b.max;
                min[i] = -hi;
                max[i] = hi;
                Some(Aabb { min, max })
            }
            SdfNode::Repeat {
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
                    // Unbounded repetition on any active axis → unbounded
                    None => {
                        if spacing.iter().all(|&s| s <= 0.0) {
                            Some(b)
                        } else {
                            None
                        }
                    }
                }
            }
            SdfNode::Displace {
                child, amplitude, ..
            } => child.bounds().map(|b| b.grow(amplitude.abs())),
            SdfNode::Warp {
                child, amplitude, ..
            } => child.bounds().map(|b| b.grow(amplitude.abs())),
            SdfNode::Cells { .. } => None,
            SdfNode::Program { program } => Some(program.aabb()),
        }
    }
}
