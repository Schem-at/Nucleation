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

pub fn footprint(schem: &UniversalSchematic, spec: &FingerprintSpec) -> Footprint {
    let grid = occupancy_grid(schem, spec);
    let mut cube = vec![Complex::<f32>::new(0.0, 0.0); N * N * N];
    let [dx, dy, dz] = grid.dims;
    for k in 0..dz.min(N) {
        for j in 0..dy.min(N) {
            for i in 0..dx.min(N) {
                let v = grid.data[i + dx * (j + dy * k)];
                cube[i + N * (j + N * k)] = Complex::new(v, 0.0);
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
}
