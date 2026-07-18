//! [`MeshShape`]: a fitted [`MeshModel`] as a building [`Shape`], with a
//! uniform-grid spatial index for ray parity tests and nearest-triangle
//! queries (normals + texture lookups).

use super::model::{MeshModel, MeshTriangle, TextureImage};
use crate::building::Shape;
use std::sync::Arc;

/// Uniform spatial grid over the triangles (cell size = 1 voxel).
struct TriGrid {
    min: [f32; 3],
    dims: [i32; 3],
    /// `dims.x * dims.y * dims.z` buckets of triangle indices.
    cells: Vec<Vec<u32>>,
}

impl TriGrid {
    const CELL: f32 = 1.0;

    fn build(triangles: &[MeshTriangle], min: [f32; 3], max: [f32; 3]) -> Self {
        let dims = [
            (((max[0] - min[0]) / Self::CELL).ceil() as i32).max(1),
            (((max[1] - min[1]) / Self::CELL).ceil() as i32).max(1),
            (((max[2] - min[2]) / Self::CELL).ceil() as i32).max(1),
        ];
        let mut cells = vec![Vec::new(); (dims[0] * dims[1] * dims[2]) as usize];
        for (idx, tri) in triangles.iter().enumerate() {
            let mut tmin = [f32::INFINITY; 3];
            let mut tmax = [f32::NEG_INFINITY; 3];
            for p in &tri.positions {
                for a in 0..3 {
                    tmin[a] = tmin[a].min(p[a]);
                    tmax[a] = tmax[a].max(p[a]);
                }
            }
            let lo = [
                Self::clamp_axis(dims, 0, ((tmin[0] - min[0]) / Self::CELL).floor() as i32),
                Self::clamp_axis(dims, 1, ((tmin[1] - min[1]) / Self::CELL).floor() as i32),
                Self::clamp_axis(dims, 2, ((tmin[2] - min[2]) / Self::CELL).floor() as i32),
            ];
            let hi = [
                Self::clamp_axis(dims, 0, ((tmax[0] - min[0]) / Self::CELL).floor() as i32),
                Self::clamp_axis(dims, 1, ((tmax[1] - min[1]) / Self::CELL).floor() as i32),
                Self::clamp_axis(dims, 2, ((tmax[2] - min[2]) / Self::CELL).floor() as i32),
            ];
            for cx in lo[0]..=hi[0] {
                for cy in lo[1]..=hi[1] {
                    for cz in lo[2]..=hi[2] {
                        let i = ((cx * dims[1] + cy) * dims[2] + cz) as usize;
                        cells[i].push(idx as u32);
                    }
                }
            }
        }
        Self { min, dims, cells }
    }

    fn clamp_axis(dims: [i32; 3], axis: usize, v: i32) -> i32 {
        v.clamp(0, dims[axis] - 1)
    }

    fn cell_of(&self, p: [f32; 3]) -> [i32; 3] {
        [
            Self::clamp_axis(self.dims, 0, ((p[0] - self.min[0]) / Self::CELL).floor() as i32),
            Self::clamp_axis(self.dims, 1, ((p[1] - self.min[1]) / Self::CELL).floor() as i32),
            Self::clamp_axis(self.dims, 2, ((p[2] - self.min[2]) / Self::CELL).floor() as i32),
        ]
    }

    fn bucket(&self, c: [i32; 3]) -> &[u32] {
        &self.cells[((c[0] * self.dims[1] + c[1]) * self.dims[2] + c[2]) as usize]
    }
}

/// A triangle mesh (loaded from GLB/OBJ, already [`MeshModel::fit`]ted into
/// voxel space) usable as a building [`Shape`].
///
/// `contains` is a solid parity test at the voxel center: axis rays along
/// +x/+y/+z count proper triangle crossings (Möller–Trumbore, ray origins
/// jittered 1e-4 on the perpendicular axes to dodge edge grazing) and the
/// three parities take a majority vote. Robust on closed meshes; open or
/// self-intersecting meshes get a best-effort answer.
///
/// Note that parity honors real wall thickness: a hollow, double-walled
/// model (e.g. an actual vessel with inner and outer surfaces) voxelizes as
/// its thin solid walls, not as a filled volume — sub-voxel walls can then
/// capture few voxel centers. That is the geometrically correct answer, not
/// a bug; scale the model up or use a single-surface mesh for a filled solid.
///
/// Cloning is cheap (the triangle data and grid are shared via `Arc`).
#[derive(Clone)]
pub struct MeshShape {
    data: Arc<MeshData>,
}

struct MeshData {
    triangles: Vec<MeshTriangle>,
    materials: Vec<Option<TextureImage>>,
    grid: TriGrid,
    /// Inclusive voxel bounds of the fitted AABB.
    bounds: (i32, i32, i32, i32, i32, i32),
    aabb_min: [f32; 3],
    aabb_max: [f32; 3],
}

const JITTER: f32 = 1e-4;

impl MeshShape {
    /// Index a (typically fitted) model for voxel queries.
    pub fn new(model: MeshModel) -> Self {
        let (min, max) = model.aabb().unwrap_or(([0.0; 3], [0.0; 3]));
        let grid = TriGrid::build(&model.triangles, min, max);
        // Voxel (x, y, z) covers [x, x+1); keep every voxel whose cube
        // intersects the AABB.
        let bounds = (
            min[0].floor() as i32,
            min[1].floor() as i32,
            min[2].floor() as i32,
            (max[0].ceil() as i32 - 1).max(min[0].floor() as i32),
            (max[1].ceil() as i32 - 1).max(min[1].floor() as i32),
            (max[2].ceil() as i32 - 1).max(min[2].floor() as i32),
        );
        Self {
            data: Arc::new(MeshData {
                triangles: model.triangles,
                materials: model.materials,
                grid,
                bounds,
                aabb_min: min,
                aabb_max: max,
            }),
        }
    }

    /// Number of triangles in the indexed mesh.
    pub fn triangle_count(&self) -> usize {
        self.data.triangles.len()
    }

    /// Parity (crossing count mod 2) of an axis-aligned ray from `origin`
    /// toward +axis, walked through the grid row.
    fn axis_ray_parity(&self, origin: [f32; 3], axis: usize) -> bool {
        let d = &self.data;
        // Jitter the two perpendicular axes to avoid hitting edges/vertices.
        // Deliberately asymmetric: equal offsets would keep the ray exactly on
        // 45-degree face diagonals (a quad's shared edge), double-counting the
        // crossing.
        let (p1, p2) = ((axis + 1) % 3, (axis + 2) % 3);
        let mut o = origin;
        o[p1] += JITTER;
        o[p2] -= 1.31 * JITTER;

        let start = d.grid.cell_of(o);
        let mut candidates: Vec<u32> = Vec::new();
        let mut c = start;
        for a in start[axis]..d.grid.dims[axis] {
            c[axis] = a;
            candidates.extend_from_slice(d.grid.bucket(c));
        }
        candidates.sort_unstable();
        candidates.dedup();

        let mut dir = [0f32; 3];
        dir[axis] = 1.0;
        let mut crossings = 0u32;
        for &t in &candidates {
            if ray_triangle_t(o, dir, &d.triangles[t as usize].positions)
                .is_some_and(|t| t > 1e-6)
            {
                crossings += 1;
            }
        }
        crossings % 2 == 1
    }

    /// Nearest triangle to `p`: `(triangle index, closest point, distance)`.
    /// Grid-accelerated expanding-ring search. `None` for an empty mesh.
    fn nearest_triangle(&self, p: [f32; 3]) -> Option<(usize, [f32; 3], f32)> {
        let d = &self.data;
        if d.triangles.is_empty() {
            return None;
        }
        let start = d.grid.cell_of(p);
        let max_r = d.grid.dims[0].max(d.grid.dims[1]).max(d.grid.dims[2]);
        let mut best: Option<(usize, [f32; 3], f32)> = None;
        let mut seen = vec![false; d.triangles.len()];
        for r in 0..=max_r {
            // Any cell beyond Chebyshev ring `r` is at least `(r) * CELL`
            // away from a point inside the start cell's ring-0 cube, so once
            // the best distance is under that we can stop.
            if let Some((_, _, dist)) = best {
                if dist <= (r as f32 - 1.0).max(0.0) * TriGrid::CELL {
                    break;
                }
            }
            let mut any_cell = false;
            for cx in (start[0] - r).max(0)..=(start[0] + r).min(d.grid.dims[0] - 1) {
                for cy in (start[1] - r).max(0)..=(start[1] + r).min(d.grid.dims[1] - 1) {
                    for cz in (start[2] - r).max(0)..=(start[2] + r).min(d.grid.dims[2] - 1) {
                        let on_shell = (cx - start[0]).abs() == r
                            || (cy - start[1]).abs() == r
                            || (cz - start[2]).abs() == r;
                        if !on_shell {
                            continue;
                        }
                        any_cell = true;
                        for &t in d.grid.bucket([cx, cy, cz]) {
                            let ti = t as usize;
                            if seen[ti] {
                                continue;
                            }
                            seen[ti] = true;
                            let q = closest_point_on_triangle(p, &d.triangles[ti].positions);
                            let dist = distance(p, q);
                            if best.is_none_or(|(_, _, bd)| dist < bd) {
                                best = Some((ti, q, dist));
                            }
                        }
                    }
                }
            }
            if !any_cell && best.is_some() {
                break;
            }
        }
        best
    }

    /// Interpolated surface color at the voxel's nearest surface point:
    /// nearest triangle → barycentric UVs → bilinear texture sample of that
    /// triangle's material. `None` when the triangle has no usable UVs or
    /// its material has no texture (constant-color materials always work).
    pub fn surface_color(&self, x: i32, y: i32, z: i32) -> Option<[u8; 3]> {
        let p = [x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5];
        let (ti, q, _) = self.nearest_triangle(p)?;
        let tri = &self.data.triangles[ti];
        let img = self
            .data
            .materials
            .get(tri.material? as usize)?
            .as_ref()?;
        if img.width == 1 && img.height == 1 {
            return Some([img.pixels[0], img.pixels[1], img.pixels[2]]);
        }
        let uvs = tri.uvs?;
        let (u, v, w) = barycentric(q, &tri.positions);
        let uv = [
            uvs[0][0] * u + uvs[1][0] * v + uvs[2][0] * w,
            uvs[0][1] * u + uvs[1][1] * v + uvs[2][1] * w,
        ];
        Some(img.sample_bilinear(uv[0], uv[1]))
    }
}

impl Shape for MeshShape {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        let d = &self.data;
        let c = [x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5];
        for a in 0..3 {
            if c[a] < d.aabb_min[a] - JITTER || c[a] > d.aabb_max[a] + JITTER {
                return false;
            }
        }
        let votes = (0..3)
            .filter(|&axis| self.axis_ray_parity(c, axis))
            .count();
        votes >= 2
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = Vec::new();
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        let p = [x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5];
        match self.nearest_triangle(p) {
            Some((ti, _, _)) => {
                let t = &self.data.triangles[ti].positions;
                let e1 = sub(t[1], t[0]);
                let e2 = sub(t[2], t[0]);
                let n = cross(e1, e2);
                let len = (n[0] as f64).hypot(n[1] as f64).hypot(n[2] as f64);
                if len < 1e-12 {
                    (0.0, 1.0, 0.0)
                } else {
                    (n[0] as f64 / len, n[1] as f64 / len, n[2] as f64 / len)
                }
            }
            None => (0.0, 1.0, 0.0),
        }
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        self.data.bounds
    }

    fn for_each_point<F>(&self, mut f: F)
    where
        F: FnMut(i32, i32, i32),
    {
        let (x0, y0, z0, x1, y1, z1) = self.data.bounds;
        for x in x0..=x1 {
            for y in y0..=y1 {
                for z in z0..=z1 {
                    if self.contains(x, y, z) {
                        f(x, y, z);
                    }
                }
            }
        }
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    dot(sub(a, b), sub(a, b)).sqrt()
}

/// Möller–Trumbore: `t` of the ray/triangle intersection, `None` on miss or
/// (near-)parallel rays.
fn ray_triangle_t(origin: [f32; 3], dir: [f32; 3], tri: &[[f32; 3]; 3]) -> Option<f32> {
    const EPS: f32 = 1e-9;
    let e1 = sub(tri[1], tri[0]);
    let e2 = sub(tri[2], tri[0]);
    let pvec = cross(dir, e2);
    let det = dot(e1, pvec);
    if det.abs() < EPS {
        return None;
    }
    let inv_det = 1.0 / det;
    let tvec = sub(origin, tri[0]);
    let u = dot(tvec, pvec) * inv_det;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let qvec = cross(tvec, e1);
    let v = dot(dir, qvec) * inv_det;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    Some(dot(e2, qvec) * inv_det)
}

/// Closest point on a triangle to `p` (Ericson, *Real-Time Collision
/// Detection* §5.1.5).
fn closest_point_on_triangle(p: [f32; 3], tri: &[[f32; 3]; 3]) -> [f32; 3] {
    let [a, b, c] = *tri;
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = sub(p, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [a[0] + ab[0] * v, a[1] + ab[1] * v, a[2] + ab[2] * v];
    }
    let cp = sub(p, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return [a[0] + ac[0] * w, a[1] + ac[1] * w, a[2] + ac[2] * w];
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return [
            b[0] + (c[0] - b[0]) * w,
            b[1] + (c[1] - b[1]) * w,
            b[2] + (c[2] - b[2]) * w,
        ];
    }
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    [
        a[0] + ab[0] * v + ac[0] * w,
        a[1] + ab[1] * v + ac[1] * w,
        a[2] + ab[2] * v + ac[2] * w,
    ]
}

/// Barycentric weights of point `q` (assumed on the triangle's plane).
fn barycentric(q: [f32; 3], tri: &[[f32; 3]; 3]) -> (f32, f32, f32) {
    let v0 = sub(tri[1], tri[0]);
    let v1 = sub(tri[2], tri[0]);
    let v2 = sub(q, tri[0]);
    let d00 = dot(v0, v0);
    let d01 = dot(v0, v1);
    let d11 = dot(v1, v1);
    let d20 = dot(v2, v0);
    let d21 = dot(v2, v1);
    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < 1e-12 {
        return (1.0, 0.0, 0.0);
    }
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    (1.0 - v - w, v, w)
}
