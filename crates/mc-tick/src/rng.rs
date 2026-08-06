//! Vanilla's random source, bit-for-bit.
//!
//! `LegacyRandomSource` is `java.util.Random`'s 48-bit LCG behind Mojang's
//! `BitRandomSource` interface, with a `MarsagliaPolarGaussian` bolted on and
//! one extension vanilla leans on for item physics: `triangle(mean, dev)`.
//!
//! The engine is deterministic by default; a simulation only draws from this
//! when a seed is set (`Simulation::set_rng_seed`). Reproducibility is the
//! contract: the same structure, seed and actions produce the same world.
//! Matching a *live* server draw-for-draw is explicitly not the contract —
//! a real `ServerLevel.random` is shared with everything else in the world,
//! so its draw order depends on state this engine does not model.
//!
//! Constants verified against `java.util.Random` (Java 25) — the unit tests
//! below pin sequences produced by the real thing.

/// `java.util.Random` / `LegacyRandomSource`: the 48-bit LCG.
#[derive(Debug, Clone, PartialEq)]
pub struct JavaRandom {
    seed: u64,
    /// Marsaglia polar generates pairs; the spare is cached, as vanilla does.
    cached_gaussian: Option<f64>,
}

const MULTIPLIER: u64 = 0x5DEECE66D;
const INCREMENT: u64 = 0xB;
const MASK: u64 = (1 << 48) - 1;

impl JavaRandom {
    /// Seed exactly as `Random.setSeed`: scramble with the multiplier.
    pub fn new(seed: i64) -> Self {
        Self {
            seed: (seed as u64 ^ MULTIPLIER) & MASK,
            cached_gaussian: None,
        }
    }

    /// The LCG step: `next(bits)`.
    fn next(&mut self, bits: u32) -> i32 {
        self.seed = self.seed.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT) & MASK;
        (self.seed >> (48 - bits)) as i64 as i32
    }

    /// `Random.nextInt(bound)` — power-of-two fast path, rejection loop otherwise.
    pub fn next_int(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");
        let m = bound - 1;
        let mut r = self.next(31);
        if bound & m == 0 {
            r = ((i64::from(bound) * i64::from(r)) >> 31) as i32;
        } else {
            let mut u = r;
            loop {
                r = u % bound;
                if u.wrapping_sub(r).wrapping_add(m) >= 0 {
                    break;
                }
                u = self.next(31);
            }
        }
        r
    }

    /// `Random.nextLong`.
    pub fn next_long(&mut self) -> i64 {
        (i64::from(self.next(32)) << 32).wrapping_add(i64::from(self.next(32)))
    }

    /// `Random.nextFloat`.
    pub fn next_float(&mut self) -> f32 {
        self.next(24) as f32 / (1u32 << 24) as f32
    }

    /// `Random.nextDouble`.
    pub fn next_double(&mut self) -> f64 {
        let high = (i64::from(self.next(26))) << 27;
        let low = i64::from(self.next(27));
        (high + low) as f64 * f64::powi(2.0, -53)
    }

    /// `MarsagliaPolarGaussian.nextGaussian` — polar method, cached spare.
    pub fn next_gaussian(&mut self) -> f64 {
        if let Some(spare) = self.cached_gaussian.take() {
            return spare;
        }
        loop {
            let d0 = 2.0 * self.next_double() - 1.0;
            let d1 = 2.0 * self.next_double() - 1.0;
            let d2 = d0 * d0 + d1 * d1;
            if d2 < 1.0 && d2 != 0.0 {
                let d3 = (-2.0 * d2.ln() / d2).sqrt();
                self.cached_gaussian = Some(d1 * d3);
                return d0 * d3;
            }
        }
    }

    /// `RandomSource.triangle(mean, deviation)` — the item-jitter distribution.
    pub fn triangle(&mut self, mean: f64, deviation: f64) -> f64 {
        mean + deviation * (self.next_double() - self.next_double())
    }

    /// Uniform double in `[min, max)` — `Mth.nextDouble`.
    pub fn next_double_between(&mut self, min: f64, max: f64) -> f64 {
        min + self.next_double() * (max - min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Every expected value below was printed by `java.util.Random` (Java 25).

    #[test]
    fn next_int_matches_java() {
        let mut r = JavaRandom::new(12345);
        assert_eq!([r.next_int(10), r.next_int(10), r.next_int(10)], [1, 0, 1]);
        let mut r = JavaRandom::new(12345);
        assert_eq!([r.next_int(16), r.next_int(16), r.next_int(16)], [5, 8, 14]);
        let mut r = JavaRandom::new(42);
        assert_eq!([r.next_int(10), r.next_int(10), r.next_int(10)], [0, 3, 8]);
        let mut r = JavaRandom::new(42);
        assert_eq!(
            [r.next_int(16), r.next_int(16), r.next_int(16)],
            [11, 0, 10]
        );
        // bound 1 consumes a draw (vanilla's reservoir sampling relies on it).
        let mut r = JavaRandom::new(12345);
        assert_eq!(r.next_int(1), 0);
        assert_eq!(r.next_int(2), 1);
    }

    #[test]
    fn next_double_matches_java() {
        let mut r = JavaRandom::new(12345);
        assert_eq!(r.next_double(), 0.3618031071604718);
        assert_eq!(r.next_double(), 0.932993485288541);
        assert_eq!(r.next_double(), 0.8330913489710237);
        let mut r = JavaRandom::new(42);
        assert_eq!(r.next_double(), 0.7275636800328681);
        assert_eq!(r.next_double(), 0.6832234717598454);
        assert_eq!(r.next_double(), 0.30871945533265976);
    }

    #[test]
    fn next_long_matches_java() {
        let mut r = JavaRandom::new(12345);
        assert_eq!(r.next_long(), 6674089274190705457);
        assert_eq!(r.next_long(), -1236052134575208584);
        let mut r = JavaRandom::new(42);
        assert_eq!(r.next_long(), -5025562857975149833);
        assert_eq!(r.next_long(), -5843495416241995736);
    }

    #[test]
    fn next_float_matches_java() {
        let mut r = JavaRandom::new(12345);
        assert!((r.next_float() - 0.361803055).abs() < 1e-9);
        assert!((r.next_float() - 0.513209522).abs() < 1e-9);
        let mut r = JavaRandom::new(42);
        assert!((r.next_float() - 0.727563679).abs() < 1e-9);
        assert!((r.next_float() - 0.0546652079).abs() < 1e-9);
    }

    #[test]
    fn triangle_matches_java_composition() {
        let mut r = JavaRandom::new(12345);
        assert_eq!(r.triangle(0.2, 0.103365), 0.14095890656479215);
        let mut r = JavaRandom::new(42);
        assert_eq!(r.triangle(0.2, 0.103365), 0.204583225628141);
    }
}
