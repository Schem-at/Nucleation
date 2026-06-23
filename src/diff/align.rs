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

/// Dense occupancy grid (1.0 = occupied) from cell positions, plus its (fine)
/// min corner. Cells are block-pooled by `stride` (≥1): a coarse cell is
/// occupied if any fine cell falls in it. None if empty or, after pooling,
/// larger than `limit` on any axis.
fn occupancy(cells: &[Cell], limit: usize, stride: usize) -> Option<([usize; 3], Vec<f32>, IVec3)> {
    if cells.is_empty() {
        return None;
    }
    let stride = stride.max(1);
    let mn = cells.iter().fold((i32::MAX, i32::MAX, i32::MAX), |m, c| {
        (m.0.min(c.0 .0), m.1.min(c.0 .1), m.2.min(c.0 .2))
    });
    let mx = cells.iter().fold((i32::MIN, i32::MIN, i32::MIN), |m, c| {
        (m.0.max(c.0 .0), m.1.max(c.0 .1), m.2.max(c.0 .2))
    });
    let dims = [
        (mx.0 - mn.0) as usize / stride + 1,
        (mx.1 - mn.1) as usize / stride + 1,
        (mx.2 - mn.2) as usize / stride + 1,
    ];
    if dims.iter().any(|&d| d > limit) {
        return None;
    }
    let mut data = vec![0.0f32; dims[0] * dims[1] * dims[2]];
    for c in cells {
        let (i, j, k) = (
            (c.0 .0 - mn.0) as usize / stride,
            (c.0 .1 - mn.1) as usize / stride,
            (c.0 .2 - mn.2) as usize / stride,
        );
        data[i + dims[0] * (j + dims[1] * k)] = 1.0;
    }
    Some((dims, data, mn))
}

/// Cross-correlate two occupancy grids and return the integer shift (in grid
/// cells) that best maps `a` onto `b` (peak of the FFT cross-correlation).
fn correlate(da: [usize; 3], ga: &[f32], db: [usize; 3], gb: &[f32]) -> IVec3 {
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
    (unwrap(li, n[0]), unwrap(lj, n[1]), unwrap(lk, n[2]))
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

/// FFT cross-correlation: exact translation aligning A-cells onto B-cells, or
/// None if either is empty / larger than `limit` on any axis.
pub fn fft_translate(a: &[Cell], b: &[Cell], limit: usize) -> Option<IVec3> {
    let (da, ga, mn_a) = occupancy(a, limit, 1)?;
    let (db, gb, mn_b) = occupancy(b, limit, 1)?;
    let s = correlate(da, &ga, db, &gb);
    Some((
        mn_b.0 - mn_a.0 + s.0,
        mn_b.1 - mn_a.1 + s.1,
        mn_b.2 - mn_a.2 + s.2,
    ))
}

/// Coarse FFT alignment for builds too large for the exact grid: block-pool the
/// occupancy by an integer `stride` so both grids fit `limit`, correlate, and
/// return `(approximate translation, stride)`. The caller refines the offset
/// within ±stride to snap to the exact translation. None if either is empty.
///
/// Peak memory is bounded: each pooled grid is ≤ `limit` on every axis.
pub fn fft_translate_downsampled(a: &[Cell], b: &[Cell], limit: usize) -> Option<(IVec3, usize)> {
    if a.is_empty() || b.is_empty() || limit < 2 {
        return None;
    }
    let span = |cells: &[Cell]| -> i32 {
        let mn = cells.iter().fold((i32::MAX, i32::MAX, i32::MAX), |m, c| {
            (m.0.min(c.0 .0), m.1.min(c.0 .1), m.2.min(c.0 .2))
        });
        let mx = cells.iter().fold((i32::MIN, i32::MIN, i32::MIN), |m, c| {
            (m.0.max(c.0 .0), m.1.max(c.0 .1), m.2.max(c.0 .2))
        });
        (mx.0 - mn.0).max(mx.1 - mn.1).max(mx.2 - mn.2)
    };
    let m = span(a).max(span(b)).max(0) as usize;
    // stride so that span/stride + 1 <= limit on every axis.
    let stride = (m / limit) + 1;
    let (da, ga, mn_a) = occupancy(a, limit, stride)?;
    let (db, gb, mn_b) = occupancy(b, limit, stride)?;
    let s = correlate(da, &ga, db, &gb);
    let off = (
        mn_b.0 - mn_a.0 + s.0 * stride as i32,
        mn_b.1 - mn_a.1 + s.1 * stride as i32,
        mn_b.2 - mn_a.2 + s.2 * stride as i32,
    );
    Some((off, stride))
}
