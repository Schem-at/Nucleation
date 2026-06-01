//! Translation-invariant FFT footprint for fast fuzzy duplicate lookup.
//!
//! The occupancy grid is normalized to its min corner, padded/cropped into a
//! fixed `N^3` cube, 3D-FFT'd, and the magnitudes of the low-frequency block are
//! kept and L2-normalized into a fixed-length feature vector. Builds larger than
//! `N` on any axis are cropped (documented limit).

use rustfft::{num_complex::Complex, FftPlanner};

use crate::fingerprint::voxel::occupancy_grid;
use crate::fingerprint::FingerprintSpec;
use crate::universal_schematic::UniversalSchematic;

const N: usize = 32; // canonical padded cube edge
const LOW: usize = 8; // low-frequency block edge kept per axis (8^3 = 512 dims)

#[derive(Clone, Debug, PartialEq)]
pub struct Footprint(pub Vec<f32>);

impl Footprint {
    pub fn distance(&self, other: &Footprint) -> f32 {
        self.0
            .iter()
            .zip(&other.0)
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f32>()
            .sqrt()
    }
}

/// Source sample range `[s0, s1)` for target voxel `t` of `td` along an axis of
/// `d` source voxels. Width is always ≥1 (nearest sample when upscaling).
fn resample_range(t: usize, td: usize, d: usize) -> (usize, usize) {
    let s0 = t * d / td;
    let s1 = ((t + 1) * d / td).max(s0 + 1).min(d);
    (s0, s1)
}

pub fn footprint(schem: &UniversalSchematic, spec: &FingerprintSpec) -> Footprint {
    let grid = occupancy_grid(schem, spec);
    let [dx, dy, dz] = grid.dims;
    let mut cube = vec![Complex::<f32>::new(0.0, 0.0); N * N * N];

    // Uniformly scale the build to fit the longest axis into N (box-filter
    // resample — averages, so no voxels are dropped even for builds > N). One
    // scale factor for all axes preserves aspect ratio, and fitting to N makes
    // the descriptor scale-invariant. Placement is irrelevant: the FFT
    // magnitude is translation-invariant.
    if dx > 0 && dy > 0 && dz > 0 {
        let maxd = dx.max(dy).max(dz);
        let fit = |d: usize| (d * N).div_ceil(maxd).clamp(1, N); // scale to fit, ≤ N
        let (tx, ty, tz) = (fit(dx), fit(dy), fit(dz));
        let src = |i: usize, j: usize, k: usize| grid.data[i + dx * (j + dy * k)];
        for tk in 0..tz {
            let (k0, k1) = resample_range(tk, tz, dz);
            for tj in 0..ty {
                let (j0, j1) = resample_range(tj, ty, dy);
                for ti in 0..tx {
                    let (i0, i1) = resample_range(ti, tx, dx);
                    let mut sum = 0.0f32;
                    let mut cnt = 0u32;
                    for k in k0..k1 {
                        for j in j0..j1 {
                            for i in i0..i1 {
                                sum += src(i, j, k);
                                cnt += 1;
                            }
                        }
                    }
                    cube[ti + N * (tj + N * tk)] = Complex::new(sum / cnt as f32, 0.0);
                }
            }
        }
    }

    fft3d(&mut cube);
    let mut feat = Vec::with_capacity(LOW * LOW * LOW);
    for k in 0..LOW {
        for j in 0..LOW {
            for i in 0..LOW {
                feat.push(cube[i + N * (j + N * k)].norm());
            }
        }
    }
    // Drop the DC bin (index 0 = total occupancy / "mass"). It otherwise
    // dominates the L2 distance, making the metric track size more than shape;
    // size already lives in `signature`. The footprint is now a pure shape
    // descriptor.
    feat[0] = 0.0;
    let norm = feat.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in feat.iter_mut() {
            *x /= norm;
        }
    }
    Footprint(feat)
}

fn fft3d(cube: &mut [Complex<f32>]) {
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(N);
    let mut line = vec![Complex::new(0.0, 0.0); N];
    // X
    for k in 0..N {
        for j in 0..N {
            for i in 0..N {
                line[i] = cube[i + N * (j + N * k)];
            }
            fft.process(&mut line);
            for i in 0..N {
                cube[i + N * (j + N * k)] = line[i];
            }
        }
    }
    // Y
    for k in 0..N {
        for i in 0..N {
            for j in 0..N {
                line[j] = cube[i + N * (j + N * k)];
            }
            fft.process(&mut line);
            for j in 0..N {
                cube[i + N * (j + N * k)] = line[j];
            }
        }
    }
    // Z
    for j in 0..N {
        for i in 0..N {
            for k in 0..N {
                line[k] = cube[i + N * (j + N * k)];
            }
            fft.process(&mut line);
            for k in 0..N {
                cube[i + N * (j + N * k)] = line[k];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::testgen::{filled_box, translated};
    use crate::fingerprint::FingerprintSpec;

    #[test]
    fn footprint_translation_invariant_and_ranks_duplicates() {
        let spec = FingerprintSpec::structural();
        let a = filled_box((0, 0, 0), (4, 3, 2), "minecraft:stone");
        let a_shift = translated(&a, (25, -7, 11));
        let other = filled_box((0, 0, 0), (2, 2, 8), "minecraft:stone");

        let fa = footprint(&a, &spec);
        let fa2 = footprint(&a_shift, &spec);
        let fo = footprint(&other, &spec);

        assert!(fa.distance(&fa2) < 1e-3, "shift should be ~invariant");
        assert!(
            fa.distance(&fo) > fa.distance(&fa2),
            "unrelated build is farther"
        );
    }

    #[test]
    fn footprint_is_scale_invariant_and_aspect_aware() {
        let spec = FingerprintSpec::structural();
        // Same shape (a line), two scales; vs a different aspect (a flat slab).
        let line_a = footprint(&filled_box((0, 0, 0), (0, 0, 7), "minecraft:stone"), &spec);
        let line_b = footprint(&filled_box((0, 0, 0), (1, 1, 15), "minecraft:stone"), &spec);
        let flat = footprint(&filled_box((0, 0, 0), (7, 7, 0), "minecraft:stone"), &spec);

        // 2x-scaled line ≈ the line; the flat slab is much farther (aspect ratio
        // is preserved, absolute size is not).
        assert!(line_a.distance(&line_b) < 1e-2, "scale-invariant");
        assert!(
            line_a.distance(&flat) > 10.0 * line_a.distance(&line_b).max(1e-6),
            "different aspect ratio is clearly farther"
        );
    }
}
