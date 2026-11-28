//! Dense 3D signed distance grid backing [`super::Field::from_schematic`] and
//! [`super::Field::from_glb_bytes`].
//!
//! Uses the linear-time separable 3D Euclidean Distance Transform
//! (Felzenszwalb & Huttenlocher, 2012) to compute exact signed distances from
//! any binary solid voxel mask in O(W · H · D), then evaluates with smooth
//! trilinear interpolation, analytic `Dual3` gradients, and conservative box
//! intervals.

use super::expr::Dual3;
use super::interval::{Interval, Interval3};
use super::Aabb;
use std::hash::{Hash, Hasher};

/// Dense 3D signed distance volume evaluated at voxel centers `origin + (i + 0.5) * voxel_size`.
#[derive(Debug, Clone)]
pub struct VoxelDistanceGrid {
    pub origin: [f32; 3],
    pub voxel_size: f32,
    pub dims: [usize; 3],
    pub values: Vec<f32>,
    pub min_val: f32,
    pub max_val: f32,
    /// Stable content hash so identical grids deduplicate in CSE.
    pub content_hash: u64,
}

impl PartialEq for VoxelDistanceGrid {
    fn eq(&self, other: &Self) -> bool {
        self.content_hash == other.content_hash
            && self.dims == other.dims
            && self.voxel_size.to_bits() == other.voxel_size.to_bits()
            && self.origin.map(f32::to_bits) == other.origin.map(f32::to_bits)
            && self.values.len() == other.values.len()
            && self
                .values
                .iter()
                .zip(&other.values)
                .all(|(a, b)| a.to_bits() == b.to_bits())
    }
}

impl Eq for VoxelDistanceGrid {}

impl Hash for VoxelDistanceGrid {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.content_hash.hash(state);
    }
}

impl VoxelDistanceGrid {
    /// Build an exact signed distance grid from a flat `solid[x + w * (z + d * y)]` mask.
    ///
    /// Negative values lie inside solid voxels; positive values lie in air.
    pub fn from_solid_mask(
        origin: [f32; 3],
        voxel_size: f32,
        dims: [usize; 3],
        solid: &[bool],
    ) -> Self {
        let [nx, ny, nz] = dims;
        let total = nx * ny * nz;
        assert_eq!(solid.len(), total);

        let inf = 1.0e20f32;
        let mut outside_sq = vec![inf; total];
        let mut inside_sq = vec![inf; total];
        for i in 0..total {
            if solid[i] {
                outside_sq[i] = 0.0;
            } else {
                inside_sq[i] = 0.0;
            }
        }

        edt_3d_inplace(&mut outside_sq, nx, ny, nz);
        edt_3d_inplace(&mut inside_sq, nx, ny, nz);

        let mut values = Vec::with_capacity(total);
        let mut min_val = f32::INFINITY;
        let mut max_val = f32::NEG_INFINITY;
        for i in 0..total {
            // Shift by -0.5 so the zero crossing sits cleanly on voxel boundaries
            let d = if solid[i] {
                -(inside_sq[i].sqrt() - 0.5).max(0.05) * voxel_size
            } else {
                (outside_sq[i].sqrt() - 0.5).max(0.05) * voxel_size
            };
            min_val = min_val.min(d);
            max_val = max_val.max(d);
            values.push(d);
        }

        Self::from_values(origin, voxel_size, dims, values)
    }

    pub fn from_values(
        origin: [f32; 3],
        voxel_size: f32,
        dims: [usize; 3],
        values: Vec<f32>,
    ) -> Self {
        let mut min_val = f32::INFINITY;
        let mut max_val = f32::NEG_INFINITY;
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for &c in &origin {
            h ^= c.to_bits() as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h ^= voxel_size.to_bits() as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
        for &d in &dims {
            h ^= d as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        for &v in &values {
            min_val = min_val.min(v);
            max_val = max_val.max(v);
            h ^= v.to_bits() as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        Self {
            origin,
            voxel_size,
            dims,
            values,
            min_val,
            max_val,
            content_hash: h,
        }
    }

    pub fn aabb(&self) -> Aabb {
        let max = [
            self.origin[0] + self.dims[0] as f32 * self.voxel_size,
            self.origin[1] + self.dims[1] as f32 * self.voxel_size,
            self.origin[2] + self.dims[2] as f32 * self.voxel_size,
        ];
        Aabb::new(self.origin, max).expect("valid voxel grid bounds")
    }

    #[inline]
    fn idx(&self, x: usize, y: usize, z: usize) -> usize {
        x + self.dims[0] * (z + self.dims[2] * y)
    }

    /// Smooth trilinear SDF evaluation, with exact exterior AABB distance outside the grid.
    #[inline]
    pub fn eval_f32(&self, x: f32, y: f32, z: f32) -> f32 {
        let [nx, ny, nz] = self.dims;
        let inv = 1.0 / self.voxel_size;
        let gx = (x - self.origin[0]) * inv - 0.5;
        let gy = (y - self.origin[1]) * inv - 0.5;
        let gz = (z - self.origin[2]) * inv - 0.5;

        let max_x = (nx - 1) as f32;
        let max_y = (ny - 1) as f32;
        let max_z = (nz - 1) as f32;

        let cx = gx.clamp(0.0, max_x);
        let cy = gy.clamp(0.0, max_y);
        let cz = gz.clamp(0.0, max_z);

        let dx_out = (gx - cx) * self.voxel_size;
        let dy_out = (gy - cy) * self.voxel_size;
        let dz_out = (gz - cz) * self.voxel_size;
        let out_dist = (dx_out * dx_out + dy_out * dy_out + dz_out * dz_out).sqrt();

        let x0 = (cx.floor() as usize).min(nx - 1);
        let y0 = (cy.floor() as usize).min(ny - 1);
        let z0 = (cz.floor() as usize).min(nz - 1);
        let x1 = (x0 + 1).min(nx - 1);
        let y1 = (y0 + 1).min(ny - 1);
        let z1 = (z0 + 1).min(nz - 1);

        let tx = cx - x0 as f32;
        let ty = cy - y0 as f32;
        let tz = cz - z0 as f32;

        let c000 = self.values[self.idx(x0, y0, z0)];
        let c100 = self.values[self.idx(x1, y0, z0)];
        let c010 = self.values[self.idx(x0, y1, z0)];
        let c110 = self.values[self.idx(x1, y1, z0)];
        let c001 = self.values[self.idx(x0, y0, z1)];
        let c101 = self.values[self.idx(x1, y0, z1)];
        let c011 = self.values[self.idx(x0, y1, z1)];
        let c111 = self.values[self.idx(x1, y1, z1)];

        let c00 = c000 + (c100 - c000) * tx;
        let c10 = c010 + (c110 - c010) * tx;
        let c01 = c001 + (c101 - c001) * tx;
        let c11 = c011 + (c111 - c011) * tx;
        let c0 = c00 + (c10 - c00) * ty;
        let c1 = c01 + (c11 - c01) * ty;
        let sample = c0 + (c1 - c0) * tz;

        if out_dist > 0.0 {
            sample.max(0.0) + out_dist
        } else {
            sample
        }
    }

    /// Analytic trilinear gradient via forward-mode `Dual3`.
    #[inline]
    pub fn eval_dual(&self, p: [Dual3; 3]) -> Dual3 {
        let [nx, ny, nz] = self.dims;
        let inv = 1.0 / self.voxel_size;
        let gx = (p[0].v - self.origin[0]) * inv - 0.5;
        let gy = (p[1].v - self.origin[1]) * inv - 0.5;
        let gz = (p[2].v - self.origin[2]) * inv - 0.5;

        let max_x = (nx - 1) as f32;
        let max_y = (ny - 1) as f32;
        let max_z = (nz - 1) as f32;

        let cx = gx.clamp(0.0, max_x);
        let cy = gy.clamp(0.0, max_y);
        let cz = gz.clamp(0.0, max_z);

        let x0 = (cx.floor() as usize).min(nx - 1);
        let y0 = (cy.floor() as usize).min(ny - 1);
        let z0 = (cz.floor() as usize).min(nz - 1);
        let x1 = (x0 + 1).min(nx - 1);
        let y1 = (y0 + 1).min(ny - 1);
        let z1 = (z0 + 1).min(nz - 1);

        let tx = cx - x0 as f32;
        let ty = cy - y0 as f32;
        let tz = cz - z0 as f32;

        let c000 = self.values[self.idx(x0, y0, z0)];
        let c100 = self.values[self.idx(x1, y0, z0)];
        let c010 = self.values[self.idx(x0, y1, z0)];
        let c110 = self.values[self.idx(x1, y1, z0)];
        let c001 = self.values[self.idx(x0, y0, z1)];
        let c101 = self.values[self.idx(x1, y0, z1)];
        let c011 = self.values[self.idx(x0, y1, z1)];
        let c111 = self.values[self.idx(x1, y1, z1)];

        let c00 = c000 + (c100 - c000) * tx;
        let c10 = c010 + (c110 - c010) * tx;
        let c01 = c001 + (c101 - c001) * tx;
        let c11 = c011 + (c111 - c011) * tx;
        let c0 = c00 + (c10 - c00) * ty;
        let c1 = c01 + (c11 - c01) * ty;
        let val = c0 + (c1 - c0) * tz;

        let dfdx = (((c100 - c000) * (1.0 - ty) + (c110 - c010) * ty) * (1.0 - tz)
            + ((c101 - c001) * (1.0 - ty) + (c111 - c011) * ty) * tz)
            * inv;
        let dfdy = ((c10 - c00) * (1.0 - tz) + (c11 - c01) * tz) * inv;
        let dfdz = (c1 - c0) * inv;

        let dx_out = (gx - cx) * self.voxel_size;
        let dy_out = (gy - cy) * self.voxel_size;
        let dz_out = (gz - cz) * self.voxel_size;
        let out_dist = (dx_out * dx_out + dy_out * dy_out + dz_out * dz_out).sqrt();

        let (v, g) = if out_dist > 1e-6 {
            (
                val.max(0.0) + out_dist,
                [dx_out / out_dist, dy_out / out_dist, dz_out / out_dist],
            )
        } else {
            (val, [dfdx, dfdy, dfdz])
        };

        Dual3 {
            v,
            d: [
                g[0] * p[0].d[0] + g[1] * p[1].d[0] + g[2] * p[2].d[0],
                g[0] * p[0].d[1] + g[1] * p[1].d[1] + g[2] * p[2].d[1],
                g[0] * p[0].d[2] + g[1] * p[1].d[2] + g[2] * p[2].d[2],
            ],
        }
    }

    /// Conservative interval bound over an arbitrary 3D box (enables O(1) `8×8×8` brick pruning!).
    #[inline]
    pub fn eval_interval(&self, b: Interval3) -> Interval {
        let aabb = self.aabb();
        // Closest point distance from box b to the grid AABB
        let dx = if b.x.hi < aabb.min[0] {
            aabb.min[0] - b.x.hi
        } else if b.x.lo > aabb.max[0] {
            b.x.lo - aabb.max[0]
        } else {
            0.0
        };
        let dy = if b.y.hi < aabb.min[1] {
            aabb.min[1] - b.y.hi
        } else if b.y.lo > aabb.max[1] {
            b.y.lo - aabb.max[1]
        } else {
            0.0
        };
        let dz = if b.z.hi < aabb.min[2] {
            aabb.min[2] - b.z.hi
        } else if b.z.lo > aabb.max[2] {
            b.z.lo - aabb.max[2]
        } else {
            0.0
        };
        let min_out = (dx * dx + dy * dy + dz * dz).sqrt();
        if min_out > 0.0 {
            return Interval::new(min_out, f32::INFINITY);
        }

        // Box overlaps the grid: scan overlapped grid cells (tiny for an 8x8x8 brick!)
        let [nx, ny, nz] = self.dims;
        let inv = 1.0 / self.voxel_size;
        let ix0 = (((b.x.lo - self.origin[0]) * inv - 0.5).floor() as isize).clamp(0, nx as isize - 1) as usize;
        let ix1 = (((b.x.hi - self.origin[0]) * inv - 0.5).ceil() as isize).clamp(0, nx as isize - 1) as usize;
        let iy0 = (((b.y.lo - self.origin[1]) * inv - 0.5).floor() as isize).clamp(0, ny as isize - 1) as usize;
        let iy1 = (((b.y.hi - self.origin[1]) * inv - 0.5).ceil() as isize).clamp(0, ny as isize - 1) as usize;
        let iz0 = (((b.z.lo - self.origin[2]) * inv - 0.5).floor() as isize).clamp(0, nz as isize - 1) as usize;
        let iz1 = (((b.z.hi - self.origin[2]) * inv - 0.5).ceil() as isize).clamp(0, nz as isize - 1) as usize;

        let cells = (ix1 - ix0 + 1) * (iy1 - iy0 + 1) * (iz1 - iz0 + 1);
        if cells > 4096 {
            return Interval::new(self.min_val, f32::INFINITY);
        }
        let mut lo = f32::INFINITY;
        let mut hi = f32::NEG_INFINITY;
        for y in iy0..=iy1 {
            for z in iz0..=iz1 {
                let row = self.dims[0] * (z + self.dims[2] * y);
                for x in ix0..=ix1 {
                    let v = self.values[row + x];
                    lo = lo.min(v);
                    hi = hi.max(v);
                }
            }
        }
        Interval::new(lo, hi.max(0.0) + self.voxel_size)
    }
}

/// 1D squared Euclidean distance transform (Felzenszwalb & Huttenlocher, 2012).
fn edt_1d(f: &[f32], d: &mut [f32], v: &mut [usize], z: &mut [f32]) {
    let n = f.len();
    if n == 0 {
        return;
    }
    let mut k = 0usize;
    v[0] = 0;
    z[0] = f32::NEG_INFINITY;
    z[1] = f32::INFINITY;

    for q in 1..n {
        let fq = f[q];
        let qf = q as f32;
        loop {
            let vk = v[k];
            let vkf = vk as f32;
            let s = ((fq + qf * qf) - (f[vk] + vkf * vkf)) / (2.0 * (qf - vkf));
            if s <= z[k] {
                if k == 0 {
                    v[0] = q;
                    z[1] = f32::INFINITY;
                    break;
                }
                k -= 1;
            } else {
                k += 1;
                v[k] = q;
                z[k] = s;
                z[k + 1] = f32::INFINITY;
                break;
            }
        }
    }

    let mut k = 0usize;
    for q in 0..n {
        let qf = q as f32;
        while z[k + 1] < qf {
            k += 1;
        }
        let vk = v[k];
        let diff = qf - vk as f32;
        d[q] = diff * diff + f[vk];
    }
}

fn edt_3d_inplace(grid: &mut [f32], nx: usize, ny: usize, nz: usize) {
    let max_n = nx.max(ny).max(nz);
    let mut f = vec![0.0f32; max_n];
    let mut d = vec![0.0f32; max_n];
    let mut v = vec![0usize; max_n];
    let mut z = vec![0.0f32; max_n + 1];

    // X pass
    for y in 0..ny {
        for z_i in 0..nz {
            let base = nx * (z_i + nz * y);
            f[..nx].copy_from_slice(&grid[base..base + nx]);
            edt_1d(&f[..nx], &mut d[..nx], &mut v[..nx], &mut z[..=nx]);
            grid[base..base + nx].copy_from_slice(&d[..nx]);
        }
    }

    // Z pass
    for y in 0..ny {
        for x in 0..nx {
            for z_i in 0..nz {
                f[z_i] = grid[x + nx * (z_i + nz * y)];
            }
            edt_1d(&f[..nz], &mut d[..nz], &mut v[..nz], &mut z[..=nz]);
            for z_i in 0..nz {
                grid[x + nx * (z_i + nz * y)] = d[z_i];
            }
        }
    }

    // Y pass
    for z_i in 0..nz {
        for x in 0..nx {
            for y in 0..ny {
                f[y] = grid[x + nx * (z_i + nz * y)];
            }
            edt_1d(&f[..ny], &mut d[..ny], &mut v[..ny], &mut z[..=ny]);
            for y in 0..ny {
                grid[x + nx * (z_i + nz * y)] = d[y];
            }
        }
    }
}
