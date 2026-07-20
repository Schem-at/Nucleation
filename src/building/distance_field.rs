//! Depth-from-surface distance field over an occupancy grid, plus a surface
//! normal read from the field's gradient.
//!
//! An SDF shape hands you the distance to its surface (and thus a normal) for
//! free, but arbitrary geometry, an imported schematic, voxelized model, or map
//! data, has no analytic surface. A `DistanceField` recovers both by a distance
//! transform of the occupancy: every solid voxel learns how many blocks it sits
//! below the surface, and the gradient of that depth gives the outward normal.
//! Materials can then key on depth (strata, a glowing core, a glass shell) and
//! on slope (grass on the flats, stone on the steep faces) over any build.

use std::collections::VecDeque;

const NB: [(i32, i32, i32); 6] = [
    (1, 0, 0),
    (-1, 0, 0),
    (0, 1, 0),
    (0, -1, 0),
    (0, 0, 1),
    (0, 0, -1),
];

pub struct DistanceField {
    min: (i32, i32, i32),
    dims: (usize, usize, usize),
    /// 0 = empty; n >= 1 = solid, n blocks below the nearest surface.
    depth: Vec<i32>,
}

impl DistanceField {
    fn lin(&self, dx: usize, dy: usize, dz: usize) -> usize {
        let (w, h, _) = self.dims;
        (dz * h + dy) * w + dx
    }

    fn local(&self, x: i32, y: i32, z: i32) -> Option<(usize, usize, usize)> {
        let (dx, dy, dz) = (x - self.min.0, y - self.min.1, z - self.min.2);
        if dx < 0 || dy < 0 || dz < 0 {
            return None;
        }
        let (dx, dy, dz) = (dx as usize, dy as usize, dz as usize);
        let (w, h, d) = self.dims;
        if dx >= w || dy >= h || dz >= d {
            return None;
        }
        Some((dx, dy, dz))
    }

    /// Build from an occupancy closure over an inclusive world-space box. A voxel
    /// is a surface voxel when it is solid and touches empty space (or the box
    /// edge); depth grows inward by one per block via multi-source BFS.
    pub fn from_occupancy(
        min: (i32, i32, i32),
        max: (i32, i32, i32),
        occupied: impl Fn(i32, i32, i32) -> bool,
    ) -> Self {
        let dims = (
            (max.0 - min.0 + 1).max(0) as usize,
            (max.1 - min.1 + 1).max(0) as usize,
            (max.2 - min.2 + 1).max(0) as usize,
        );
        let (w, h, d) = (dims.0 as i32, dims.1 as i32, dims.2 as i32);
        let lin = |x: i32, y: i32, z: i32| ((z * h + y) * w + x) as usize;
        let inb = |x: i32, y: i32, z: i32| x >= 0 && y >= 0 && z >= 0 && x < w && y < h && z < d;
        let mut depth = vec![0i32; dims.0 * dims.1 * dims.2];

        // -1 marks solid-but-unassigned.
        for z in 0..d {
            for y in 0..h {
                for x in 0..w {
                    if occupied(min.0 + x, min.1 + y, min.2 + z) {
                        depth[lin(x, y, z)] = -1;
                    }
                }
            }
        }

        let mut q: VecDeque<(i32, i32, i32)> = VecDeque::new();
        for z in 0..d {
            for y in 0..h {
                for x in 0..w {
                    if depth[lin(x, y, z)] != -1 {
                        continue;
                    }
                    let surface = NB.iter().any(|&(dx, dy, dz)| {
                        let (nx, ny, nz) = (x + dx, y + dy, z + dz);
                        !inb(nx, ny, nz) || depth[lin(nx, ny, nz)] == 0
                    });
                    if surface {
                        depth[lin(x, y, z)] = 1;
                        q.push_back((x, y, z));
                    }
                }
            }
        }
        while let Some((x, y, z)) = q.pop_front() {
            let cur = depth[lin(x, y, z)];
            for &(dx, dy, dz) in &NB {
                let (nx, ny, nz) = (x + dx, y + dy, z + dz);
                if inb(nx, ny, nz) && depth[lin(nx, ny, nz)] == -1 {
                    depth[lin(nx, ny, nz)] = cur + 1;
                    q.push_back((nx, ny, nz));
                }
            }
        }
        DistanceField { min, dims, depth }
    }

    /// Blocks below the surface at a voxel: 0 for empty/outside, 1 at the
    /// surface, increasing inward.
    pub fn depth_at(&self, x: i32, y: i32, z: i32) -> i32 {
        self.local(x, y, z)
            .map(|(dx, dy, dz)| self.depth[self.lin(dx, dy, dz)].max(0))
            .unwrap_or(0)
    }

    /// Outward surface normal at a voxel: the negated, normalized gradient of the
    /// depth field (depth rises inward, so the outward direction is `-gradient`).
    /// Falls back to straight up for a fully interior or empty voxel.
    pub fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        let d = |xx, yy, zz| self.depth_at(xx, yy, zz) as f64;
        let gx = d(x + 1, y, z) - d(x - 1, y, z);
        let gy = d(x, y + 1, z) - d(x, y - 1, z);
        let gz = d(x, y, z + 1) - d(x, y, z - 1);
        let (nx, ny, nz) = (-gx, -gy, -gz);
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len < 1e-9 {
            (0.0, 1.0, 0.0)
        } else {
            (nx / len, ny / len, nz / len)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_grows_inward_and_top_normal_points_up() {
        // A solid 7x7x7 cube.
        let f = DistanceField::from_occupancy((0, 0, 0), (6, 6, 6), |_, _, _| true);
        assert_eq!(f.depth_at(0, 0, 0), 1); // corner is surface
        assert_eq!(f.depth_at(3, 3, 3), 4); // center is deepest
        assert_eq!(f.depth_at(3, 6, 3), 1); // top face is surface
        assert_eq!(f.depth_at(-1, 0, 0), 0); // outside
                                             // The top-face normal points up.
        let (_, ny, _) = f.normal_at(3, 6, 3);
        assert!(ny > 0.7, "top normal should point up, got ny={ny}");
    }
}
