//! Deterministic seeded value noise + FBM.
//!
//! All lattice hashing is pure integer math (no float intermediates), so the
//! same (coordinates, seed) always produce the same value on every platform,
//! architecture, and binding — a hard requirement for reproducible terrain.

/// Integer hash (Wang-style avalanche) over a 3D lattice point + seed → [0, 1).
#[inline]
pub fn hash01_3(x: i32, y: i32, z: i32, seed: i32) -> f32 {
    let mut h = (x as u32)
        .wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(1_103_515_245))
        .wrapping_add((z as u32).wrapping_mul(668_265_263))
        .wrapping_add((seed as u32).wrapping_mul(1_442_695_041));
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    (h & 0x7FFF_FFFF) as f32 / 0x8000_0000u32 as f32
}

/// 2D lattice hash → [0, 1).
#[inline]
pub fn hash01_2(x: i32, z: i32, seed: i32) -> f32 {
    hash01_3(x, 0, z, seed)
}

#[inline]
fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn floor_i(v: f32) -> i32 {
    v.floor() as i32
}

/// Trilinearly-interpolated value noise in [0, 1).
pub fn value_noise3(x: f32, y: f32, z: f32, seed: i32) -> f32 {
    let (x0, y0, z0) = (floor_i(x), floor_i(y), floor_i(z));
    let (fx, fy, fz) = (x - x0 as f32, y - y0 as f32, z - z0 as f32);
    let (u, v, w) = (smoothstep(fx), smoothstep(fy), smoothstep(fz));

    let c000 = hash01_3(x0, y0, z0, seed);
    let c100 = hash01_3(x0 + 1, y0, z0, seed);
    let c010 = hash01_3(x0, y0 + 1, z0, seed);
    let c110 = hash01_3(x0 + 1, y0 + 1, z0, seed);
    let c001 = hash01_3(x0, y0, z0 + 1, seed);
    let c101 = hash01_3(x0 + 1, y0, z0 + 1, seed);
    let c011 = hash01_3(x0, y0 + 1, z0 + 1, seed);
    let c111 = hash01_3(x0 + 1, y0 + 1, z0 + 1, seed);

    let x00 = c000 + (c100 - c000) * u;
    let x10 = c010 + (c110 - c010) * u;
    let x01 = c001 + (c101 - c001) * u;
    let x11 = c011 + (c111 - c011) * u;
    let y0v = x00 + (x10 - x00) * v;
    let y1v = x01 + (x11 - x01) * v;
    y0v + (y1v - y0v) * w
}

/// Bilinearly-interpolated 2D value noise in [0, 1).
pub fn value_noise2(x: f32, z: f32, seed: i32) -> f32 {
    let (x0, z0) = (floor_i(x), floor_i(z));
    let (fx, fz) = (x - x0 as f32, z - z0 as f32);
    let (u, v) = (smoothstep(fx), smoothstep(fz));

    let a = hash01_2(x0, z0, seed);
    let b = hash01_2(x0 + 1, z0, seed);
    let c = hash01_2(x0, z0 + 1, seed);
    let d = hash01_2(x0 + 1, z0 + 1, seed);

    (a + (b - a) * u) * (1.0 - v) + (c + (d - c) * u) * v
}

/// 3D fractal Brownian motion in [-1, 1]. Lacunarity 2.0, gain 0.5.
pub fn fbm3(x: f32, y: f32, z: f32, seed: i32, frequency: f32, octaves: u32) -> f32 {
    let octaves = octaves.clamp(1, 8);
    let mut amp = 1.0f32;
    let mut freq = frequency;
    let mut sum = 0.0f32;
    let mut norm = 0.0f32;
    for o in 0..octaves {
        let s = seed.wrapping_add(o as i32 * 101);
        sum += amp * (value_noise3(x * freq, y * freq, z * freq, s) * 2.0 - 1.0);
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / norm
}

/// 2D fractal Brownian motion in [-1, 1].
pub fn fbm2(x: f32, z: f32, seed: i32, frequency: f32, octaves: u32) -> f32 {
    let octaves = octaves.clamp(1, 8);
    let mut amp = 1.0f32;
    let mut freq = frequency;
    let mut sum = 0.0f32;
    let mut norm = 0.0f32;
    for o in 0..octaves {
        let s = seed.wrapping_add(o as i32 * 101);
        sum += amp * (value_noise2(x * freq, z * freq, s) * 2.0 - 1.0);
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / norm
}
