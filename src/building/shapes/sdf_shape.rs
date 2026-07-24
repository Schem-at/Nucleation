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
            checked_floor(b.min[0])?,
            checked_floor(b.min[1])?,
            checked_floor(b.min[2])?,
            checked_ceil(b.max[0])?,
            checked_ceil(b.max[1])?,
            checked_ceil(b.max[2])?,
        );
        validate_bounds(bounds)?;
        Some(Self {
            node: Arc::new(node),
            bounds,
        })
    }

    /// Wrap a tree with explicit sampling bounds (inclusive block coords).
    pub fn with_bounds(node: SdfNode, min: (i32, i32, i32), max: (i32, i32, i32)) -> Option<Self> {
        let bounds = (min.0, min.1, min.2, max.0, max.1, max.2);
        validate_bounds(bounds)?;
        Some(Self {
            node: Arc::new(node),
            bounds,
        })
    }

    fn eval_center(&self, x: i32, y: i32, z: i32) -> f32 {
        self.node
            .eval(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5)
    }
}

fn checked_floor(value: f32) -> Option<i32> {
    checked_rounded(value, f32::floor)
}

fn checked_ceil(value: f32) -> Option<i32> {
    checked_rounded(value, f32::ceil)
}

fn checked_rounded(value: f32, round: fn(f32) -> f32) -> Option<i32> {
    let rounded = round(value);
    if !rounded.is_finite()
        || f64::from(rounded) < f64::from(i32::MIN)
        || f64::from(rounded) > f64::from(i32::MAX)
    {
        return None;
    }
    Some(rounded as i32)
}

fn validate_bounds(bounds: (i32, i32, i32, i32, i32, i32)) -> Option<u64> {
    crate::sdf::checked_sample_volume(
        [bounds.0, bounds.1, bounds.2],
        [bounds.3, bounds.4, bounds.5],
    )
    .ok()
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
        let (fx, fy, fz) = (x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
        crate::sdf::numerical_normal(&self.node, [fx, fy, fz], 0.5)
            .map(|normal| (normal[0], normal[1], normal[2]))
            .unwrap_or((0.0, 1.0, 0.0))
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

#[cfg(test)]
mod tests {
    use super::{SdfShape, Shape};
    use crate::sdf::SdfNode;

    #[test]
    fn normal_at_extreme_valid_coordinate_uses_representable_neighbors() {
        let shape = SdfShape::with_bounds(
            SdfNode::Plane {
                normal: [1.0, 0.0, 0.0],
                offset: 0.0,
            },
            (i32::MAX, 0, 0),
            (i32::MAX, 0, 0),
        )
        .unwrap();
        let normal = shape.normal_at(i32::MAX, 0, 0);
        assert!((normal.0 - 1.0).abs() < 1e-9);
        assert!(normal.1.abs() < 1e-9);
        assert!(normal.2.abs() < 1e-9);
    }
}
