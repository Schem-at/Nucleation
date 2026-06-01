//! Alignment: find the translation that best maps A-cells onto B-cells.

use std::collections::HashMap;

use rustfft::{num_complex::Complex, FftPlanner};

use crate::diff::{AlignOptions, Cell, IVec3};

/// Hough anchor-voting. Returns (best translation, peak/runner-up margin).
/// margin is `f32::INFINITY` for a single candidate, `0.0` when there are no
/// shared anchor tokens (caller may fall back to FFT).
pub fn hough_translate(a: &[Cell], b: &[Cell], opts: &AlignOptions) -> (IVec3, f32) {
    let mut a_count: HashMap<&str, usize> = HashMap::new();
    for c in a {
        *a_count.entry(c.1.as_str()).or_default() += 1;
    }
    let mut b_count: HashMap<&str, usize> = HashMap::new();
    let mut b_by_token: HashMap<&str, Vec<IVec3>> = HashMap::new();
    for c in b {
        *b_count.entry(c.1.as_str()).or_default() += 1;
        b_by_token.entry(c.1.as_str()).or_default().push(c.0);
    }

    let mut votes: HashMap<IVec3, u32> = HashMap::new();
    for c in a {
        let tok = c.1.as_str();
        if a_count[tok] > opts.anchor_max_count {
            continue;
        }
        if b_count.get(tok).copied().unwrap_or(usize::MAX) > opts.anchor_max_count {
            continue;
        }
        if let Some(bs) = b_by_token.get(tok) {
            for bp in bs {
                let t = (bp.0 - c.0 .0, bp.1 - c.0 .1, bp.2 - c.0 .2);
                *votes.entry(t).or_default() += 1;
            }
        }
    }

    if votes.is_empty() {
        return ((0, 0, 0), 0.0);
    }
    let mut ranked: Vec<(IVec3, u32)> = votes.into_iter().collect();
    ranked.sort_by(|x, y| y.1.cmp(&x.1));
    let best = ranked[0];
    let margin = match ranked.get(1) {
        Some(second) if second.1 > 0 => best.1 as f32 / second.1 as f32,
        _ => f32::INFINITY,
    };
    (best.0, margin)
}

/// Dense occupancy grid (1.0 = occupied) from cell positions, plus its min
/// corner. None if empty or larger than `limit` on any axis.
fn occupancy(cells: &[Cell], limit: usize) -> Option<([usize; 3], Vec<f32>, IVec3)> {
    if cells.is_empty() {
        return None;
    }
    let mn = cells.iter().fold((i32::MAX, i32::MAX, i32::MAX), |m, c| {
        (m.0.min(c.0 .0), m.1.min(c.0 .1), m.2.min(c.0 .2))
    });
    let mx = cells.iter().fold((i32::MIN, i32::MIN, i32::MIN), |m, c| {
        (m.0.max(c.0 .0), m.1.max(c.0 .1), m.2.max(c.0 .2))
    });
    let dims = [
        (mx.0 - mn.0 + 1) as usize,
        (mx.1 - mn.1 + 1) as usize,
        (mx.2 - mn.2 + 1) as usize,
    ];
    if dims.iter().any(|&d| d > limit) {
        return None;
    }
    let mut data = vec![0.0f32; dims[0] * dims[1] * dims[2]];
    for c in cells {
        let (i, j, k) = (
            (c.0 .0 - mn.0) as usize,
            (c.0 .1 - mn.1) as usize,
            (c.0 .2 - mn.2) as usize,
        );
        data[i + dims[0] * (j + dims[1] * k)] = 1.0;
    }
    Some((dims, data, mn))
}

fn fft3d(cube: &mut [Complex<f32>], n: [usize; 3], inverse: bool) {
    let mut planner = FftPlanner::<f32>::new();
    let idx = |i: usize, j: usize, k: usize| i + n[0] * (j + n[1] * k);
    let fx = if inverse {
        planner.plan_fft_inverse(n[0])
    } else {
        planner.plan_fft_forward(n[0])
    };
    let mut lx = vec![Complex::new(0.0, 0.0); n[0]];
    for k in 0..n[2] {
        for j in 0..n[1] {
            for i in 0..n[0] {
                lx[i] = cube[idx(i, j, k)];
            }
            fx.process(&mut lx);
            for i in 0..n[0] {
                cube[idx(i, j, k)] = lx[i];
            }
        }
    }
    let fy = if inverse {
        planner.plan_fft_inverse(n[1])
    } else {
        planner.plan_fft_forward(n[1])
    };
    let mut ly = vec![Complex::new(0.0, 0.0); n[1]];
    for k in 0..n[2] {
        for i in 0..n[0] {
            for j in 0..n[1] {
                ly[j] = cube[idx(i, j, k)];
            }
            fy.process(&mut ly);
            for j in 0..n[1] {
                cube[idx(i, j, k)] = ly[j];
            }
        }
    }
    let fz = if inverse {
        planner.plan_fft_inverse(n[2])
    } else {
        planner.plan_fft_forward(n[2])
    };
    let mut lz = vec![Complex::new(0.0, 0.0); n[2]];
    for j in 0..n[1] {
        for i in 0..n[0] {
            for k in 0..n[2] {
                lz[k] = cube[idx(i, j, k)];
            }
            fz.process(&mut lz);
            for k in 0..n[2] {
                cube[idx(i, j, k)] = lz[k];
            }
        }
    }
}

/// FFT cross-correlation: translation aligning A-cells onto B-cells, or None if
/// either is empty / too large.
pub fn fft_translate(a: &[Cell], b: &[Cell], limit: usize) -> Option<IVec3> {
    let (da, ga, mn_a) = occupancy(a, limit)?;
    let (db, gb, mn_b) = occupancy(b, limit)?;
    let n = [da[0] + db[0], da[1] + db[1], da[2] + db[2]];
    let total = n[0] * n[1] * n[2];
    let mut ca = vec![Complex::new(0.0, 0.0); total];
    let mut cb = vec![Complex::new(0.0, 0.0); total];
    let idx = |i: usize, j: usize, k: usize| i + n[0] * (j + n[1] * k);
    for k in 0..da[2] {
        for j in 0..da[1] {
            for i in 0..da[0] {
                ca[idx(i, j, k)] = Complex::new(ga[i + da[0] * (j + da[1] * k)], 0.0);
            }
        }
    }
    for k in 0..db[2] {
        for j in 0..db[1] {
            for i in 0..db[0] {
                cb[idx(i, j, k)] = Complex::new(gb[i + db[0] * (j + db[1] * k)], 0.0);
            }
        }
    }
    fft3d(&mut ca, n, false);
    fft3d(&mut cb, n, false);
    let mut prod: Vec<Complex<f32>> = ca.iter().zip(&cb).map(|(x, y)| x.conj() * y).collect();
    fft3d(&mut prod, n, true);
    let mut best = (0usize, f32::MIN);
    for (i, v) in prod.iter().enumerate() {
        if v.re > best.1 {
            best = (i, v.re);
        }
    }
    let li = best.0 % n[0];
    let lj = (best.0 / n[0]) % n[1];
    let lk = best.0 / (n[0] * n[1]);
    let unwrap = |v: usize, m: usize| {
        if v > m / 2 {
            v as i32 - m as i32
        } else {
            v as i32
        }
    };
    let s = (unwrap(li, n[0]), unwrap(lj, n[1]), unwrap(lk, n[2]));
    Some((
        mn_b.0 - mn_a.0 + s.0,
        mn_b.1 - mn_a.1 + s.1,
        mn_b.2 - mn_a.2 + s.2,
    ))
}
