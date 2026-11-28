use super::Shape;
use crate::sdf::{CompiledField, Field, Interval3};
use std::sync::Arc;

const BRICK_SIZE: i32 = 8;

/// A compiled [`Field`] wrapped as a building [`Shape`].
///
/// Uses conservative 8×8×8 interval pruning in `for_each_point` to skip empty
/// regions without per-voxel evaluation, and forward-mode dual-number
/// gradients (falling back to ULP-safe central differences) in `normal_at`.
#[derive(Clone)]
pub struct SdfShape {
    compiled: Arc<CompiledField>,
    bounds: (i32, i32, i32, i32, i32, i32),
}

impl SdfShape {
    /// Wrap a [`Field`], inferring inclusive block bounds from its conservative
    /// AABB. Returns `None` when the field is unbounded (use [`Self::with_bounds`]).
    pub fn new(field: Field) -> Option<Self> {
        field.validate().ok()?;
        let b = field.bounds()?;
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
            compiled: Arc::new(field.compile()),
            bounds,
        })
    }

    /// Wrap a [`Field`] with explicit inclusive block bounds.
    pub fn with_bounds(field: Field, min: (i32, i32, i32), max: (i32, i32, i32)) -> Option<Self> {
        field.validate().ok()?;
        let bounds = (min.0, min.1, min.2, max.0, max.1, max.2);
        validate_bounds(bounds)?;
        Some(Self {
            compiled: Arc::new(field.compile()),
            bounds,
        })
    }

    #[inline]
    fn eval_center(&self, x: i32, y: i32, z: i32) -> f32 {
        self.compiled
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
        self.compiled
            .normal(fx, fy, fz, 0.5)
            .map(|n| (n[0] as f64, n[1] as f64, n[2] as f64))
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
        let mut bx = x0;
        while bx <= x1 {
            let bx1 = (bx + BRICK_SIZE - 1).min(x1);
            let mut by = y0;
            while by <= y1 {
                let by1 = (by + BRICK_SIZE - 1).min(y1);
                let mut bz = z0;
                while bz <= z1 {
                    let bz1 = (bz + BRICK_SIZE - 1).min(z1);
                    let box3 = Interval3::from_box(
                        [bx as f32 + 0.5, by as f32 + 0.5, bz as f32 + 0.5],
                        [bx1 as f32 + 0.5, by1 as f32 + 0.5, bz1 as f32 + 0.5],
                    );
                    let iv = self.compiled.eval_interval(box3);
                    if !iv.strictly_positive() {
                        if iv.non_positive() {
                            for x in bx..=bx1 {
                                for y in by..=by1 {
                                    for z in bz..=bz1 {
                                        f(x, y, z);
                                    }
                                }
                            }
                        } else {
                            for x in bx..=bx1 {
                                for y in by..=by1 {
                                    for z in bz..=bz1 {
                                        if self.eval_center(x, y, z) <= 0.0 {
                                            f(x, y, z);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    bz = bz1 + 1;
                }
                by = by1 + 1;
            }
            bx = bx1 + 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SdfShape, Shape};
    use crate::sdf::Field;

    #[test]
    fn normal_at_extreme_valid_coordinate_uses_representable_neighbors() {
        let shape = SdfShape::with_bounds(
            Field::plane([1.0, 0.0, 0.0], 0.0).unwrap(),
            (i32::MAX, 0, 0),
            (i32::MAX, 0, 0),
        )
        .unwrap();
        let normal = shape.normal_at(i32::MAX, 0, 0);
        assert!((normal.0 - 1.0).abs() < 1e-6);
        assert!(normal.1.abs() < 1e-6);
        assert!(normal.2.abs() < 1e-6);
    }
}
