//! Conservative interval arithmetic for spatial brick / octree pruning.
//!
//! For any 3D box `[x0, x1] × [y0, y1] × [z0, z1]`, evaluating a compiled
//! field over intervals produces `[lo, hi]` guaranteed to contain every value
//! `f(x, y, z)` takes inside that box.
//!
//! - `lo > 0.0` proves the entire box is strictly outside the surface (air) → skip!
//! - `hi <= 0.0` proves the entire box is strictly inside the surface (solid).
//!
//! Whenever an operation cannot prove a tighter bound (or encounters NaN /
//! division across zero), it widens conservatively to [`Interval::ENTIRE`],
//! guaranteeing zero false skips.

/// Closed interval `[lo, hi]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub lo: f32,
    pub hi: f32,
}

impl Interval {
    pub const ENTIRE: Self = Self {
        lo: f32::NEG_INFINITY,
        hi: f32::INFINITY,
    };

    #[inline]
    pub fn point(v: f32) -> Self {
        if v.is_nan() {
            Self::ENTIRE
        } else {
            Self { lo: v, hi: v }
        }
    }

    #[inline]
    pub fn new(lo: f32, hi: f32) -> Self {
        if lo.is_nan() || hi.is_nan() {
            Self::ENTIRE
        } else if lo <= hi {
            Self { lo, hi }
        } else {
            Self { lo: hi, hi: lo }
        }
    }

    #[inline]
    pub fn is_finite(self) -> bool {
        self.lo.is_finite() && self.hi.is_finite()
    }

    /// True when every point in the interval is strictly positive (guaranteed air).
    #[inline]
    pub fn strictly_positive(self) -> bool {
        self.lo > 0.0
    }

    /// True when every point in the interval is `<= 0.0` (guaranteed solid).
    #[inline]
    pub fn non_positive(self) -> bool {
        self.hi <= 0.0
    }

    #[inline]
    pub fn contains_zero(self) -> bool {
        self.lo <= 0.0 && self.hi >= 0.0
    }

    /// Conservative expansion by `pad >= 0`.
    #[inline]
    pub fn expand(self, pad: f32) -> Self {
        if !pad.is_finite() {
            return Self::ENTIRE;
        }
        let p = pad.abs();
        Self::new(self.lo - p, self.hi + p)
    }

    #[inline]
    pub fn add(self, o: Self) -> Self {
        if !self.is_finite() || !o.is_finite() {
            return Self::ENTIRE;
        }
        // Small ULP margin guards against round-to-nearest tightening bounds.
        Self::new(
            next_down_safe(self.lo + o.lo),
            next_up_safe(self.hi + o.hi),
        )
    }

    #[inline]
    pub fn sub(self, o: Self) -> Self {
        if !self.is_finite() || !o.is_finite() {
            return Self::ENTIRE;
        }
        Self::new(
            next_down_safe(self.lo - o.hi),
            next_up_safe(self.hi - o.lo),
        )
    }

    #[inline]
    pub fn neg(self) -> Self {
        Self::new(-self.hi, -self.lo)
    }

    #[inline]
    pub fn mul(self, o: Self) -> Self {
        if !self.is_finite() || !o.is_finite() {
            return Self::ENTIRE;
        }
        let p1 = self.lo * o.lo;
        let p2 = self.lo * o.hi;
        let p3 = self.hi * o.lo;
        let p4 = self.hi * o.hi;
        let lo = p1.min(p2).min(p3).min(p4);
        let hi = p1.max(p2).max(p3).max(p4);
        if lo.is_nan() || hi.is_nan() {
            Self::ENTIRE
        } else {
            Self::new(next_down_safe(lo), next_up_safe(hi))
        }
    }

    #[inline]
    pub fn scale(self, s: f32) -> Self {
        self.mul(Self::point(s))
    }

    #[inline]
    pub fn div(self, o: Self) -> Self {
        if !self.is_finite() || !o.is_finite() || o.contains_zero() {
            return Self::ENTIRE;
        }
        let inv = Self::new(next_down_safe(1.0 / o.hi), next_up_safe(1.0 / o.lo));
        self.mul(inv)
    }

    #[inline]
    pub fn abs(self) -> Self {
        if self.lo >= 0.0 {
            self
        } else if self.hi <= 0.0 {
            self.neg()
        } else {
            Self::new(0.0, self.hi.max(-self.lo))
        }
    }

    #[inline]
    pub fn square(self) -> Self {
        if !self.is_finite() {
            return Self::new(0.0, f32::INFINITY);
        }
        let a = self.abs();
        Self::new(
            next_down_safe(a.lo * a.lo).max(0.0),
            next_up_safe(a.hi * a.hi),
        )
    }

    #[inline]
    pub fn sqrt(self) -> Self {
        if self.hi < 0.0 {
            return Self::ENTIRE;
        }
        let lo = if self.lo <= 0.0 {
            0.0
        } else {
            next_down_safe(self.lo.sqrt()).max(0.0)
        };
        let hi = if self.hi.is_infinite() {
            f32::INFINITY
        } else {
            next_up_safe(self.hi.sqrt())
        };
        Self::new(lo, hi)
    }

    #[inline]
    pub fn min(self, o: Self) -> Self {
        Self::new(self.lo.min(o.lo), self.hi.min(o.hi))
    }

    #[inline]
    pub fn max(self, o: Self) -> Self {
        Self::new(self.lo.max(o.lo), self.hi.max(o.hi))
    }

    #[inline]
    pub fn union_bounds(self, o: Self) -> Self {
        Self::new(self.lo.min(o.lo), self.hi.max(o.hi))
    }

    #[inline]
    pub fn clamp(self, lo: f32, hi: f32) -> Self {
        Self::new(self.lo.clamp(lo, hi), self.hi.clamp(lo, hi))
    }

    #[inline]
    pub fn floor(self) -> Self {
        Self::new(self.lo.floor(), self.hi.floor())
    }

    #[inline]
    pub fn ceil(self) -> Self {
        Self::new(self.lo.ceil(), self.hi.ceil())
    }

    #[inline]
    pub fn fract(self) -> Self {
        if self.is_finite() && (self.hi - self.lo) < 1.0 && self.lo.floor() == self.hi.floor() {
            Self::new(self.lo.rem_euclid(1.0), self.hi.rem_euclid(1.0))
        } else {
            Self::new(0.0, 1.0)
        }
    }

    #[inline]
    pub fn sin(self) -> Self {
        if !self.is_finite() || self.hi - self.lo >= std::f32::consts::TAU {
            return Self::new(-1.0, 1.0);
        }
        // Conservative bound: check endpoints and critical points (pi/2 + k*pi).
        let mut lo = self.lo.sin().min(self.hi.sin());
        let mut hi = self.lo.sin().max(self.hi.sin());
        let half_pi = std::f32::consts::FRAC_PI_2;
        let k_start = ((self.lo - half_pi) / std::f32::consts::PI).ceil() as i64;
        let k_end = ((self.hi - half_pi) / std::f32::consts::PI).floor() as i64;
        if k_end - k_start <= 4 {
            for k in k_start..=k_end {
                if k.rem_euclid(2) == 0 {
                    hi = 1.0;
                } else {
                    lo = -1.0;
                }
            }
        } else {
            return Self::new(-1.0, 1.0);
        }
        Self::new(next_down_safe(lo).max(-1.0), next_up_safe(hi).min(1.0))
    }

    #[inline]
    pub fn cos(self) -> Self {
        self.add(Self::point(std::f32::consts::FRAC_PI_2)).sin()
    }

    #[inline]
    pub fn exp(self) -> Self {
        if !self.is_finite() {
            return Self::new(0.0, f32::INFINITY);
        }
        Self::new(
            next_down_safe(self.lo.exp()).max(0.0),
            next_up_safe(self.hi.exp()),
        )
    }

    #[inline]
    pub fn ln(self) -> Self {
        if self.lo <= 0.0 || !self.is_finite() {
            return Self::ENTIRE;
        }
        Self::new(next_down_safe(self.lo.ln()), next_up_safe(self.hi.ln()))
    }

    #[inline]
    pub fn smoothstep(self, edge0: f32, edge1: f32) -> Self {
        if (edge1 - edge0).abs() <= f32::EPSILON {
            return Self::new(0.0, 1.0);
        }
        let s = |v: f32| {
            let t = ((v - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        };
        if edge0 < edge1 {
            Self::new(s(self.lo), s(self.hi))
        } else {
            Self::new(s(self.hi), s(self.lo))
        }
    }

    /// Polynomial smooth min satisfies: `min(a, b) - 0.25 * k <= smin(a, b, k) <= min(a, b)`.
    #[inline]
    pub fn smin(self, o: Self, k: f32) -> Self {
        let m = self.min(o);
        if k <= 0.0 {
            m
        } else {
            Self::new(next_down_safe(m.lo - 0.25 * k), m.hi)
        }
    }

    #[inline]
    pub fn smax(self, o: Self, k: f32) -> Self {
        self.neg().smin(o.neg(), k).neg()
    }

    #[inline]
    pub fn len2(x: Self, y: Self) -> Self {
        x.square().add(y.square()).sqrt()
    }

    #[inline]
    pub fn len3(x: Self, y: Self, z: Self) -> Self {
        x.square().add(y.square()).add(z.square()).sqrt()
    }
}

/// 3D interval box `[x] × [y] × [z]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval3 {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl Interval3 {
    #[inline]
    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn from_box(min: [f32; 3], max: [f32; 3]) -> Self {
        Self {
            x: Interval::new(min[0], max[0]),
            y: Interval::new(min[1], max[1]),
            z: Interval::new(min[2], max[2]),
        }
    }

    #[inline]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    #[inline]
    pub fn length(self) -> Interval {
        Interval::len3(self.x, self.y, self.z)
    }

    #[inline]
    pub fn translate(self, offset: [f32; 3]) -> Self {
        Self {
            x: self.x.sub(Interval::point(offset[0])),
            y: self.y.sub(Interval::point(offset[1])),
            z: self.z.sub(Interval::point(offset[2])),
        }
    }

    #[inline]
    pub fn scale(self, factor: f32) -> Self {
        let inv = 1.0 / factor;
        Self {
            x: self.x.scale(inv),
            y: self.y.scale(inv),
            z: self.z.scale(inv),
        }
    }

    /// Transform an interval box by the transpose (inverse) of rotation matrix `m`.
    pub fn inv_rotate(self, m: &[[f32; 3]; 3]) -> Self {
        if !self.is_finite() {
            return Self::new(Interval::ENTIRE, Interval::ENTIRE, Interval::ENTIRE);
        }
        // Row j of M^T is [m[0][j], m[1][j], m[2][j]]
        let rx = self
            .x
            .scale(m[0][0])
            .add(self.y.scale(m[1][0]))
            .add(self.z.scale(m[2][0]));
        let ry = self
            .x
            .scale(m[0][1])
            .add(self.y.scale(m[1][1]))
            .add(self.z.scale(m[2][1]));
        let rz = self
            .x
            .scale(m[0][2])
            .add(self.y.scale(m[1][2]))
            .add(self.z.scale(m[2][2]));
        Self::new(rx, ry, rz)
    }
}

#[inline]
fn next_up_safe(v: f32) -> f32 {
    if !v.is_finite() {
        return v;
    }
    if v == 0.0 {
        return f32::from_bits(1);
    }
    let bits = v.to_bits();
    let next = f32::from_bits(if v > 0.0 { bits + 1 } else { bits - 1 });
    if next.is_finite() {
        next
    } else {
        f32::INFINITY
    }
}

#[inline]
fn next_down_safe(v: f32) -> f32 {
    if !v.is_finite() {
        return v;
    }
    if v == 0.0 {
        return f32::from_bits(0x8000_0001);
    }
    let bits = v.to_bits();
    let next = f32::from_bits(if v > 0.0 { bits - 1 } else { bits + 1 });
    if next.is_finite() {
        next
    } else {
        f32::NEG_INFINITY
    }
}
