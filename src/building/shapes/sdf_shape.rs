use super::Shape;
use crate::sdf::SdfNode;
use std::sync::Arc;

/// An SDF tree used as a building [`Shape`] — the bridge between the two
/// geometry systems: any JSON distance-field tree (primitives, smooth
/// booleans, noise) becomes fillable with every brush, including masked
/// fills and the normal-driven shaded brush (normals come from the field
/// gradient, so smooth-blended surfaces shade smoothly).
///
/// Voxel membership matches the SDF sampler exactly: the field is evaluated
/// at the block center (`x + 0.5`), solid where `eval <= 0`.
#[derive(Clone)]
pub struct SdfShape {
    node: Arc<SdfNode>,
    bounds: (i32, i32, i32, i32, i32, i32),
}

impl SdfShape {
    /// Wrap a tree, sampling inside its own AABB. `None` when the tree is
    /// unbounded (e.g. a bare `plane`) — use [`Self::with_bounds`] then.
    pub fn new(node: SdfNode) -> Option<Self> {
        let b = node.bounds()?;
        let bounds = (
            b.min[0].floor() as i32,
            b.min[1].floor() as i32,
            b.min[2].floor() as i32,
            b.max[0].ceil() as i32,
            b.max[1].ceil() as i32,
            b.max[2].ceil() as i32,
        );
        Some(Self {
            node: Arc::new(node),
            bounds,
        })
    }

    /// Wrap a tree with explicit sampling bounds (inclusive block coords).
    pub fn with_bounds(node: SdfNode, min: (i32, i32, i32), max: (i32, i32, i32)) -> Self {
        Self {
            node: Arc::new(node),
            bounds: (min.0, min.1, min.2, max.0, max.1, max.2),
        }
    }

    fn eval_center(&self, x: i32, y: i32, z: i32) -> f32 {
        self.node
            .eval(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5)
    }
}

impl Shape for SdfShape {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        let (x0, y0, z0, x1, y1, z1) = self.bounds;
        x >= x0
            && x <= x1
            && y >= y0
            && y <= y1
            && z >= z0
            && z <= z1
            && self.eval_center(x, y, z) <= 0.0
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = Vec::new();
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        // Central-difference gradient of the field at the block center.
        let (fx, fy, fz) = (x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
        const H: f32 = 0.5;
        let nx = self.node.eval(fx + H, fy, fz) - self.node.eval(fx - H, fy, fz);
        let ny = self.node.eval(fx, fy + H, fz) - self.node.eval(fx, fy - H, fz);
        let nz = self.node.eval(fx, fy, fz + H) - self.node.eval(fx, fy, fz - H);
        let len = ((nx * nx + ny * ny + nz * nz) as f64).sqrt();
        if len < 1e-9 {
            (0.0, 1.0, 0.0)
        } else {
            (nx as f64 / len, ny as f64 / len, nz as f64 / len)
        }
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        self.bounds
    }

    fn for_each_point<F>(&self, mut f: F)
    where
        F: FnMut(i32, i32, i32),
    {
        let (x0, y0, z0, x1, y1, z1) = self.bounds;
        for x in x0..=x1 {
            for y in y0..=y1 {
                for z in z0..=z1 {
                    if self.eval_center(x, y, z) <= 0.0 {
                        f(x, y, z);
                    }
                }
            }
        }
    }
}
