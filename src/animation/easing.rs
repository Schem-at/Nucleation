//! Easing curves.
//!
//! Every curve maps `t` in `0..=1` to an eased progress value. Most stay inside
//! `0..=1`; `Back` and `Elastic` deliberately overshoot, which is the point of
//! them. [`Easing::CubicBezier`] is the escape hatch — any CSS/anime.js curve
//! can be expressed with it, so this enum does not have to chase their
//! catalogue forever.

use serde::{Deserialize, Serialize};

/// The polynomial/transcendental family a directional easing is built from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Power {
    Quad,
    Cubic,
    Quart,
    Quint,
    Sine,
    Expo,
    Circ,
}

impl Power {
    /// The "ease-in" shape of this family: slow start, fast finish.
    fn ease_in(self, t: f32) -> f32 {
        match self {
            Power::Quad => t * t,
            Power::Cubic => t * t * t,
            Power::Quart => t * t * t * t,
            Power::Quint => t * t * t * t * t,
            Power::Sine => 1.0 - (t * core::f32::consts::FRAC_PI_2).cos(),
            Power::Expo => {
                if t <= 0.0 {
                    0.0
                } else {
                    (2.0f32).powf(10.0 * (t - 1.0))
                }
            }
            Power::Circ => 1.0 - (1.0 - t * t).max(0.0).sqrt(),
        }
    }
}

/// Default overshoot for the `Back` curves, matching the CSS/anime.js constant.
pub const DEFAULT_BACK_OVERSHOOT: f32 = 1.701_58;

/// An easing curve.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum Easing {
    #[default]
    Linear,
    In(Power),
    Out(Power),
    InOut(Power),
    /// Anticipation: pulls back before moving. `f32` is the overshoot amount.
    InBack(f32),
    /// Overshoots the target, then settles.
    OutBack(f32),
    InOutBack(f32),
    InElastic {
        amplitude: f32,
        period: f32,
    },
    OutElastic {
        amplitude: f32,
        period: f32,
    },
    InOutElastic {
        amplitude: f32,
        period: f32,
    },
    InBounce,
    OutBounce,
    InOutBounce,
    /// Quantise progress into `n` discrete steps.
    Steps(u32),
    /// Arbitrary curve through control points `(x1, y1)` and `(x2, y2)`,
    /// the same parameterisation as CSS `cubic-bezier()`.
    CubicBezier(f32, f32, f32, f32),
}

impl Easing {
    /// Evaluate the curve. `t` is clamped to `0..=1`; the result may leave that
    /// range for overshooting curves.
    pub fn eval(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match *self {
            Easing::Linear => t,
            Easing::In(p) => p.ease_in(t),
            Easing::Out(p) => 1.0 - p.ease_in(1.0 - t),
            Easing::InOut(p) => {
                if t < 0.5 {
                    p.ease_in(t * 2.0) / 2.0
                } else {
                    1.0 - p.ease_in((1.0 - t) * 2.0) / 2.0
                }
            }
            Easing::InBack(c1) => {
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            }
            Easing::OutBack(c1) => {
                let c3 = c1 + 1.0;
                let u = t - 1.0;
                1.0 + c3 * u * u * u + c1 * u * u
            }
            Easing::InOutBack(c1) => {
                let c2 = c1 * 1.525;
                if t < 0.5 {
                    let u = 2.0 * t;
                    (u * u * ((c2 + 1.0) * u - c2)) / 2.0
                } else {
                    let u = 2.0 * t - 2.0;
                    (u * u * ((c2 + 1.0) * u + c2) + 2.0) / 2.0
                }
            }
            Easing::InElastic { amplitude, period } => elastic_in(t, amplitude, period),
            Easing::OutElastic { amplitude, period } => {
                1.0 - elastic_in(1.0 - t, amplitude, period)
            }
            Easing::InOutElastic { amplitude, period } => {
                if t < 0.5 {
                    elastic_in(t * 2.0, amplitude, period) / 2.0
                } else {
                    1.0 - elastic_in((1.0 - t) * 2.0, amplitude, period) / 2.0
                }
            }
            Easing::OutBounce => bounce_out(t),
            Easing::InBounce => 1.0 - bounce_out(1.0 - t),
            Easing::InOutBounce => {
                if t < 0.5 {
                    (1.0 - bounce_out(1.0 - 2.0 * t)) / 2.0
                } else {
                    (1.0 + bounce_out(2.0 * t - 1.0)) / 2.0
                }
            }
            Easing::Steps(n) => {
                if n == 0 {
                    t
                } else {
                    let n = n as f32;
                    ((t * n).floor() / n).min(1.0)
                }
            }
            Easing::CubicBezier(x1, y1, x2, y2) => cubic_bezier(t, x1, y1, x2, y2),
        }
    }

    /// `OutBack` with the conventional overshoot — the usual "pop in" curve.
    pub fn out_back() -> Self {
        Easing::OutBack(DEFAULT_BACK_OVERSHOOT)
    }

    /// `OutElastic` with conventional parameters.
    pub fn out_elastic() -> Self {
        Easing::OutElastic {
            amplitude: 1.0,
            period: 0.3,
        }
    }
}

fn elastic_in(t: f32, amplitude: f32, period: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    let period = if period == 0.0 { 0.3 } else { period };
    let amplitude = amplitude.max(1.0);
    let s = period / (2.0 * core::f32::consts::PI) * (1.0 / amplitude).asin();
    let u = t - 1.0;
    -(amplitude
        * (2.0f32).powf(10.0 * u)
        * ((u - s) * (2.0 * core::f32::consts::PI) / period).sin())
}

fn bounce_out(t: f32) -> f32 {
    const N: f32 = 7.5625;
    const D: f32 = 2.75;
    if t < 1.0 / D {
        N * t * t
    } else if t < 2.0 / D {
        let t = t - 1.5 / D;
        N * t * t + 0.75
    } else if t < 2.5 / D {
        let t = t - 2.25 / D;
        N * t * t + 0.9375
    } else {
        let t = t - 2.625 / D;
        N * t * t + 0.984_375
    }
}

/// Evaluate a CSS-style cubic bezier at progress `x`.
///
/// The curve is defined parametrically, so `x` must first be inverted to the
/// bezier's own parameter. Newton-Raphson converges in a few iterations for the
/// well-behaved curves people actually write; bisection is the fallback when the
/// derivative is too flat to trust.
fn cubic_bezier(x: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    fn calc(a: f32, b: f32, t: f32) -> f32 {
        let inv = 1.0 - t;
        3.0 * inv * inv * t * a + 3.0 * inv * t * t * b + t * t * t
    }
    fn slope(a: f32, b: f32, t: f32) -> f32 {
        let inv = 1.0 - t;
        3.0 * inv * inv * a + 6.0 * inv * t * (b - a) + 3.0 * t * t * (1.0 - b)
    }
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let mut t = x;
    for _ in 0..8 {
        let err = calc(x1, x2, t) - x;
        if err.abs() < 1e-6 {
            return calc(y1, y2, t);
        }
        let d = slope(x1, x2, t);
        if d.abs() < 1e-6 {
            break;
        }
        t -= err / d;
    }
    // Bisection fallback.
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    let mut t = x;
    for _ in 0..32 {
        let v = calc(x1, x2, t);
        if (v - x).abs() < 1e-6 {
            break;
        }
        if v > x {
            hi = t;
        } else {
            lo = t;
        }
        t = (lo + hi) / 2.0;
    }
    calc(y1, y2, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every curve must be pinned at both ends, or staggered animations drift.
    #[test]
    fn all_curves_are_pinned_at_both_ends() {
        let curves = [
            Easing::Linear,
            Easing::In(Power::Quad),
            Easing::Out(Power::Cubic),
            Easing::InOut(Power::Sine),
            Easing::In(Power::Expo),
            Easing::Out(Power::Expo),
            Easing::In(Power::Circ),
            Easing::InBack(DEFAULT_BACK_OVERSHOOT),
            Easing::out_back(),
            Easing::InOutBack(DEFAULT_BACK_OVERSHOOT),
            Easing::out_elastic(),
            Easing::InElastic {
                amplitude: 1.0,
                period: 0.3,
            },
            Easing::OutBounce,
            Easing::InBounce,
            Easing::InOutBounce,
            Easing::Steps(4),
            Easing::CubicBezier(0.42, 0.0, 0.58, 1.0),
        ];
        for c in curves {
            assert!(c.eval(0.0).abs() < 1e-4, "{c:?} must be 0 at t=0");
            assert!((c.eval(1.0) - 1.0).abs() < 1e-4, "{c:?} must be 1 at t=1");
        }
    }

    #[test]
    fn input_is_clamped() {
        assert_eq!(Easing::Linear.eval(-5.0), 0.0);
        assert_eq!(Easing::Linear.eval(5.0), 1.0);
    }

    #[test]
    fn ease_in_starts_slower_than_linear() {
        for p in [Power::Quad, Power::Cubic, Power::Quart, Power::Quint] {
            assert!(
                Easing::In(p).eval(0.25) < 0.25,
                "{p:?} ease-in should lag linear early"
            );
            assert!(
                Easing::Out(p).eval(0.25) > 0.25,
                "{p:?} ease-out should lead linear early"
            );
        }
    }

    #[test]
    fn inout_is_symmetric_about_the_midpoint() {
        for p in [Power::Quad, Power::Cubic, Power::Sine] {
            let e = Easing::InOut(p);
            assert!((e.eval(0.5) - 0.5).abs() < 1e-4, "{p:?} midpoint");
            for t in [0.1f32, 0.25, 0.4] {
                let a = e.eval(t);
                let b = 1.0 - e.eval(1.0 - t);
                assert!((a - b).abs() < 1e-4, "{p:?} asymmetric at {t}");
            }
        }
    }

    /// Back and Elastic are *supposed* to leave 0..1 — that overshoot is the effect.
    #[test]
    fn back_and_elastic_overshoot() {
        assert!(
            Easing::out_back().eval(0.7) > 1.0,
            "out_back should overshoot past 1"
        );
        assert!(
            Easing::InBack(DEFAULT_BACK_OVERSHOOT).eval(0.3) < 0.0,
            "in_back should dip below 0"
        );
    }

    #[test]
    fn steps_quantises() {
        let e = Easing::Steps(4);
        assert_eq!(e.eval(0.0), 0.0);
        assert_eq!(e.eval(0.24), 0.0);
        assert_eq!(e.eval(0.26), 0.25);
        assert_eq!(e.eval(0.51), 0.5);
        assert_eq!(e.eval(1.0), 1.0);
        // Degenerate step count must not divide by zero.
        assert_eq!(Easing::Steps(0).eval(0.37), 0.37);
    }

    /// A bezier matching `ease-in-out` should track the analytic curve closely.
    #[test]
    fn cubic_bezier_matches_known_values() {
        let e = Easing::CubicBezier(0.42, 0.0, 0.58, 1.0);
        assert!((e.eval(0.5) - 0.5).abs() < 1e-3, "symmetric curve midpoint");
        // Linear control points reproduce linear.
        let lin = Easing::CubicBezier(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0);
        for t in [0.1f32, 0.3, 0.5, 0.75, 0.9] {
            assert!((lin.eval(t) - t).abs() < 2e-3, "linear bezier at {t}");
        }
    }

    #[test]
    fn monotonic_curves_never_decrease() {
        let curves = [
            Easing::Linear,
            Easing::In(Power::Cubic),
            Easing::Out(Power::Expo),
            Easing::InOut(Power::Quad),
            Easing::CubicBezier(0.42, 0.0, 0.58, 1.0),
        ];
        for c in curves {
            let mut prev = f32::NEG_INFINITY;
            for i in 0..=100 {
                let v = c.eval(i as f32 / 100.0);
                assert!(v >= prev - 1e-5, "{c:?} decreased at t={i}");
                prev = v;
            }
        }
    }

    #[test]
    fn evaluation_is_deterministic() {
        let e = Easing::out_elastic();
        for i in 0..=50 {
            let t = i as f32 / 50.0;
            assert_eq!(e.eval(t), e.eval(t));
        }
    }
}
