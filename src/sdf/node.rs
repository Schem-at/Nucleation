//! SDF node tree: a serde JSON AST of signed-distance primitives, operators,
//! transforms, and noise modifiers.
//!
//! Distance functions follow Inigo Quilez's reference formulations
//! (<https://iquilezles.org/articles/distfunctions/>). Nodes marked
//! *approximate* return a lower bound rather than an exact Euclidean
//! distance — safe for inside/outside sampling, imprecise for sphere tracing.

use super::noise::{fbm3, hash01_3, value_noise3};
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
    /// Exact. Line segment `a`→`b` with radius.
    Capsule {
        a: [f32; 3],
        b: [f32; 3],
        radius: f32,
    },
    /// Exact. Y-axis aligned.
    CappedCylinder {
        radius: f32,
        half_height: f32,
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

#[inline]
fn len3(x: f32, y: f32, z: f32) -> f32 {
    (x * x + y * y + z * z).sqrt()
}

#[inline]
fn len2(x: f32, y: f32) -> f32 {
    (x * x + y * y).sqrt()
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
    /// Parse a node tree from its JSON representation.
    pub fn from_json(json: &str) -> Result<SdfNode, String> {
        serde_json::from_str(json).map_err(|e| format!("Invalid SDF JSON: {e}"))
    }

    /// Serialize this tree to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("SDF serialization failed: {e}"))
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

            SdfNode::CappedCylinder {
                radius,
                half_height,
            } => {
                let dx = len2(x, z) - radius;
                let dy = y.abs() - half_height;
                dx.max(dy).min(0.0) + len2(dx.max(0.0), dy.max(0.0))
            }

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
            SdfNode::CappedCylinder {
                radius,
                half_height,
            } => sym(*radius, *half_height, *radius),
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
            } => sym(*radius, *half_height, *radius),
            SdfNode::SuperPrism {
                half_extents: b, ..
            } => sym(b[0], b[1], b[2]),

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
        }
    }
}
