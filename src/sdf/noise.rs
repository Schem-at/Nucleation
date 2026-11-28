//! Deterministic seeded procedural noise: value noise, Perlin gradient noise,
//! fractal Brownian motion (standard & ridged), and Worley/cellular fields.
//!
//! All lattice hashing uses pure 32-bit integer arithmetic so identical
//! `(coordinates, seed)` always produce bit-identical results across native
//! platforms, WASM, and every language binding.

use serde::{Deserialize, Serialize};

/// Safe coordinate headroom below `i32::MAX` for lattice hashing.
pub(crate) const MAX_SAFE_LATTICE_COORD: f64 = i32::MAX as f64 - 1024.0;

/// What a Worley / cellular noise node outputs per point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum CellMode {
    /// Distance to the nearest feature point (rounded blobs).
    F1,
    /// Distance to the second-nearest feature point.
    F2,
    /// `F2 - F1`: zero on cell walls, classic Voronoi crack field.
    #[default]
    F2MinusF1,
    /// Deterministic per-cell constant in `[0, 1)`: Voronoi mosaic.
    Value,
}

/// Pure integer 3D lattice hash returning a `u32`.
#[inline]
pub fn hash_u32_3(x: i32, y: i32, z: i32, seed: i32) -> u32 {
    let mut h = (x as u32)
        .wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(1_103_515_245))
        .wrapping_add((z as u32).wrapping_mul(668_265_263))
        .wrapping_add((seed as u32).wrapping_mul(1_442_695_041));
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}

/// Integer hash over a 3D lattice point + seed in `[0, 1)`.
#[inline]
pub fn hash01_3(x: i32, y: i32, z: i32, seed: i32) -> f32 {
    (hash_u32_3(x, y, z, seed) & 0x7FFF_FFFF) as f32 / 0x8000_0000u32 as f32
}

/// 2D lattice hash in `[0, 1)`.
#[inline]
pub fn hash01_2(x: i32, z: i32, seed: i32) -> f32 {
    hash01_3(x, 0, z, seed)
}

#[inline]
fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn quintic(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn floor_i(v: f32) -> i32 {
    v.floor() as i32
}

#[inline]
fn coords_safe(coords: &[f32]) -> bool {
    coords
        .iter()
        .all(|&c| c.is_finite() && (c as f64).abs() < MAX_SAFE_LATTICE_COORD)
}

/// Trilinearly-interpolated value noise in `[0, 1)`.
pub fn value_noise3(x: f32, y: f32, z: f32, seed: i32) -> f32 {
    if !coords_safe(&[x, y, z]) {
        return 0.5;
    }
    let (x0, y0, z0) = (floor_i(x), floor_i(y), floor_i(z));
    let (fx, fy, fz) = (x - x0 as f32, y - y0 as f32, z - z0 as f32);
    let (u, v, w) = (smoothstep(fx), smoothstep(fy), smoothstep(fz));

    let c000 = hash01_3(x0, y0, z0, seed);
    let c100 = hash01_3(x0.wrapping_add(1), y0, z0, seed);
    let c010 = hash01_3(x0, y0.wrapping_add(1), z0, seed);
    let c110 = hash01_3(x0.wrapping_add(1), y0.wrapping_add(1), z0, seed);
    let c001 = hash01_3(x0, y0, z0.wrapping_add(1), seed);
    let c101 = hash01_3(x0.wrapping_add(1), y0, z0.wrapping_add(1), seed);
    let c011 = hash01_3(x0, y0.wrapping_add(1), z0.wrapping_add(1), seed);
    let c111 = hash01_3(
        x0.wrapping_add(1),
        y0.wrapping_add(1),
        z0.wrapping_add(1),
        seed,
    );

    let x00 = c000 + (c100 - c000) * u;
    let x10 = c010 + (c110 - c010) * u;
    let x01 = c001 + (c101 - c001) * u;
    let x11 = c011 + (c111 - c011) * u;
    let y0v = x00 + (x10 - x00) * v;
    let y1v = x01 + (x11 - x01) * v;
    y0v + (y1v - y0v) * w
}

/// Bilinearly-interpolated 2D value noise in `[0, 1)`.
pub fn value_noise2(x: f32, z: f32, seed: i32) -> f32 {
    if !coords_safe(&[x, z]) {
        return 0.5;
    }
    let (x0, z0) = (floor_i(x), floor_i(z));
    let (fx, fz) = (x - x0 as f32, z - z0 as f32);
    let (u, v) = (smoothstep(fx), smoothstep(fz));

    let a = hash01_2(x0, z0, seed);
    let b = hash01_2(x0.wrapping_add(1), z0, seed);
    let c = hash01_2(x0, z0.wrapping_add(1), seed);
    let d = hash01_2(x0.wrapping_add(1), z0.wrapping_add(1), seed);

    (a + (b - a) * u) * (1.0 - v) + (c + (d - c) * u) * v
}

/// 2D Perlin gradient dot product using 8 unit-ish directions.
#[inline]
fn grad2(hash: u32, dx: f32, dz: f32) -> f32 {
    // 8 gradients spaced by 45 degrees (normalized so max dot over unit square <= 1)
    const INV_SQRT2: f32 = std::f32::consts::FRAC_1_SQRT_2;
    match hash & 7 {
        0 => dx,
        1 => -dx,
        2 => dz,
        3 => -dz,
        4 => (dx + dz) * INV_SQRT2,
        5 => (-dx + dz) * INV_SQRT2,
        6 => (dx - dz) * INV_SQRT2,
        _ => (-dx - dz) * INV_SQRT2,
    }
}

/// 2D Perlin gradient noise in `[-1, 1]`.
pub fn perlin2(x: f32, z: f32, seed: i32) -> f32 {
    if !coords_safe(&[x, z]) {
        return 0.0;
    }
    let (x0, z0) = (floor_i(x), floor_i(z));
    let (fx, fz) = (x - x0 as f32, z - z0 as f32);
    let (u, v) = (quintic(fx), quintic(fz));

    let g00 = grad2(hash_u32_3(x0, 0, z0, seed), fx, fz);
    let g10 = grad2(hash_u32_3(x0.wrapping_add(1), 0, z0, seed), fx - 1.0, fz);
    let g01 = grad2(hash_u32_3(x0, 0, z0.wrapping_add(1), seed), fx, fz - 1.0);
    let g11 = grad2(
        hash_u32_3(x0.wrapping_add(1), 0, z0.wrapping_add(1), seed),
        fx - 1.0,
        fz - 1.0,
    );

    let nx0 = g00 + (g10 - g00) * u;
    let nx1 = g01 + (g11 - g01) * u;
    // Theoretical max of 2D unit-gradient Perlin is ~0.707; scale by sqrt(2) and clamp to [-1, 1].
    ((nx0 + (nx1 - nx0) * v) * std::f32::consts::SQRT_2).clamp(-1.0, 1.0)
}

/// 3D Perlin gradient dot product (Ken Perlin's 12 cube-edge directions).
#[inline]
fn grad3(hash: u32, dx: f32, dy: f32, dz: f32) -> f32 {
    match hash & 15 {
        0 => dx + dy,
        1 => -dx + dy,
        2 => dx - dy,
        3 => -dx - dy,
        4 => dx + dz,
        5 => -dx + dz,
        6 => dx - dz,
        7 => -dx - dz,
        8 => dy + dz,
        9 => -dy + dz,
        10 => dy - dz,
        11 => -dy - dz,
        12 => dx + dy,
        13 => -dy + dz,
        14 => -dx + dy,
        _ => -dy - dz,
    }
}

/// 3D Perlin gradient noise in `[-1, 1]`.
pub fn perlin3(x: f32, y: f32, z: f32, seed: i32) -> f32 {
    if !coords_safe(&[x, y, z]) {
        return 0.0;
    }
    let (x0, y0, z0) = (floor_i(x), floor_i(y), floor_i(z));
    let (fx, fy, fz) = (x - x0 as f32, y - y0 as f32, z - z0 as f32);
    let (u, v, w) = (quintic(fx), quintic(fy), quintic(fz));

    let x1 = x0.wrapping_add(1);
    let y1 = y0.wrapping_add(1);
    let z1 = z0.wrapping_add(1);

    let c000 = grad3(hash_u32_3(x0, y0, z0, seed), fx, fy, fz);
    let c100 = grad3(hash_u32_3(x1, y0, z0, seed), fx - 1.0, fy, fz);
    let c010 = grad3(hash_u32_3(x0, y1, z0, seed), fx, fy - 1.0, fz);
    let c110 = grad3(hash_u32_3(x1, y1, z0, seed), fx - 1.0, fy - 1.0, fz);
    let c001 = grad3(hash_u32_3(x0, y0, z1, seed), fx, fy, fz - 1.0);
    let c101 = grad3(hash_u32_3(x1, y0, z1, seed), fx - 1.0, fy, fz - 1.0);
    let c011 = grad3(hash_u32_3(x0, y1, z1, seed), fx, fy - 1.0, fz - 1.0);
    let c111 = grad3(hash_u32_3(x1, y1, z1, seed), fx - 1.0, fy - 1.0, fz - 1.0);

    let x00 = c000 + (c100 - c000) * u;
    let x10 = c010 + (c110 - c010) * u;
    let x01 = c001 + (c101 - c001) * u;
    let x11 = c011 + (c111 - c011) * u;
    let y0v = x00 + (x10 - x00) * v;
    let y1v = x01 + (x11 - x01) * v;
    (y0v + (y1v - y0v) * w).clamp(-1.0, 1.0)
}

/// 3D fractal Brownian motion (value noise) in `[-1, 1]`.
pub fn fbm3(x: f32, y: f32, z: f32, seed: i32, frequency: f32, octaves: u32) -> f32 {
    fbm3_params(x, y, z, seed, frequency, octaves, 2.0, 0.5, false, false)
}

/// 2D fractal Brownian motion (value noise) in `[-1, 1]`.
pub fn fbm2(x: f32, z: f32, seed: i32, frequency: f32, octaves: u32) -> f32 {
    fbm2_params(x, z, seed, frequency, octaves, 2.0, 0.5, false, false)
}

/// Configurable 3D FBM (Perlin or Value, standard or ridged).
/// Standard returns `[-1, 1]`; ridged returns `[0, 1]`.
#[allow(clippy::too_many_arguments)]
pub fn fbm3_params(
    x: f32,
    y: f32,
    z: f32,
    seed: i32,
    frequency: f32,
    octaves: u32,
    lacunarity: f32,
    gain: f32,
    use_perlin: bool,
    ridged: bool,
) -> f32 {
    let octaves = octaves.clamp(1, 8);
    let mut amp = 1.0f32;
    let mut freq = frequency;
    let mut sum = 0.0f32;
    let mut norm = 0.0f32;
    for o in 0..octaves {
        let s = seed.wrapping_add(o as i32 * 101);
        let sx = x * freq;
        let sy = y * freq;
        let sz = z * freq;
        if !coords_safe(&[sx, sy, sz]) {
            break;
        }
        let sample = if use_perlin {
            perlin3(sx, sy, sz, s)
        } else {
            value_noise3(sx, sy, sz, s) * 2.0 - 1.0
        };
        let contribution = if ridged {
            let r = 1.0 - sample.abs();
            r * r
        } else {
            sample
        };
        sum += amp * contribution;
        norm += amp;
        amp *= gain;
        freq *= lacunarity;
    }
    if norm <= 0.0 {
        0.0
    } else if ridged {
        (sum / norm).clamp(0.0, 1.0)
    } else {
        (sum / norm).clamp(-1.0, 1.0)
    }
}

/// Configurable 2D FBM (Perlin or Value, standard or ridged).
/// Standard returns `[-1, 1]`; ridged returns `[0, 1]`.
#[allow(clippy::too_many_arguments)]
pub fn fbm2_params(
    x: f32,
    z: f32,
    seed: i32,
    frequency: f32,
    octaves: u32,
    lacunarity: f32,
    gain: f32,
    use_perlin: bool,
    ridged: bool,
) -> f32 {
    let octaves = octaves.clamp(1, 8);
    let mut amp = 1.0f32;
    let mut freq = frequency;
    let mut sum = 0.0f32;
    let mut norm = 0.0f32;
    for o in 0..octaves {
        let s = seed.wrapping_add(o as i32 * 101);
        let sx = x * freq;
        let sz = z * freq;
        if !coords_safe(&[sx, sz]) {
            break;
        }
        let sample = if use_perlin {
            perlin2(sx, sz, s)
        } else {
            value_noise2(sx, sz, s) * 2.0 - 1.0
        };
        let contribution = if ridged {
            let r = 1.0 - sample.abs();
            r * r
        } else {
            sample
        };
        sum += amp * contribution;
        norm += amp;
        amp *= gain;
        freq *= lacunarity;
    }
    if norm <= 0.0 {
        0.0
    } else if ridged {
        (sum / norm).clamp(0.0, 1.0)
    } else {
        (sum / norm).clamp(-1.0, 1.0)
    }
}

/// 3D Worley / cellular noise.
///
/// Output range:
/// - `F1`: `[0, sqrt(3) * (1 + jitter/2)]` (approx `[0, 2.6]` for `jitter = 1`)
/// - `F2`: `[0, sqrt(3) * 2]`
/// - `F2MinusF1`: `[0, sqrt(3) * 2]`
/// - `Value`: `[0, 1]`
pub fn worley3(
    x: f32,
    y: f32,
    z: f32,
    frequency: f32,
    seed: i32,
    jitter: f32,
    mode: CellMode,
) -> f32 {
    let sx = x * frequency;
    let sy = y * frequency;
    let sz = z * frequency;
    if !coords_safe(&[sx, sy, sz]) {
        return 0.0;
    }
    let ix = sx.floor() as i32;
    let iy = sy.floor() as i32;
    let iz = sz.floor() as i32;
    let fx = sx - ix as f32;
    let fy = sy - iy as f32;
    let fz = sz - iz as f32;

    let mut f1 = f32::INFINITY;
    let mut f2 = f32::INFINITY;
    let mut best_cell = (ix, iy, iz);
    let j = jitter.clamp(0.0, 1.0);
    let center_bias = 0.5 * (1.0 - j);

    for dz in -1..=1 {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let cx = ix.wrapping_add(dx);
                let cy = iy.wrapping_add(dy);
                let cz = iz.wrapping_add(dz);
                let px = dx as f32 + center_bias + hash01_3(cx, cy, cz, seed) * j;
                let py =
                    dy as f32 + center_bias + hash01_3(cx, cy, cz, seed.wrapping_add(17)) * j;
                let pz =
                    dz as f32 + center_bias + hash01_3(cx, cy, cz, seed.wrapping_add(31)) * j;
                let ddx = px - fx;
                let ddy = py - fy;
                let ddz = pz - fz;
                let d2 = ddx * ddx + ddy * ddy + ddz * ddz;
                if d2 < f1 {
                    f2 = f1;
                    f1 = d2;
                    best_cell = (cx, cy, cz);
                } else if d2 < f2 {
                    f2 = d2;
                }
            }
        }
    }

    match mode {
        CellMode::F1 => f1.sqrt(),
        CellMode::F2 => f2.sqrt(),
        CellMode::F2MinusF1 => f2.sqrt() - f1.sqrt(),
        CellMode::Value => hash01_3(
            best_cell.0,
            best_cell.1,
            best_cell.2,
            seed.wrapping_add(73),
        ),
    }
}
