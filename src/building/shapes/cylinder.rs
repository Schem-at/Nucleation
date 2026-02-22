use super::{ParametricShape, Shape};

#[derive(Clone)]
pub struct Cylinder {
    pub base_center: (f64, f64, f64),
    pub axis: (f64, f64, f64),
    pub radius: f64,
    pub height: f64,
    axis_len: f64,
}

impl Cylinder {
    pub fn new(
        base_center: (f64, f64, f64),
        axis: (f64, f64, f64),
        radius: f64,
        height: f64,
    ) -> Self {
        let axis_len = (axis.0 * axis.0 + axis.1 * axis.1 + axis.2 * axis.2).sqrt();
        Self {
            base_center,
            axis,
            radius,
            height,
            axis_len,
        }
    }

    /// Convenience: create a cylinder between two points with given radius.
    pub fn between(p1: (f64, f64, f64), p2: (f64, f64, f64), radius: f64) -> Self {
        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        let dz = p2.2 - p1.2;
        let height = (dx * dx + dy * dy + dz * dz).sqrt();
        let axis = if height > 0.0 {
            (dx / height, dy / height, dz / height)
        } else {
            (0.0, 1.0, 0.0)
        };
        Self::new(p1, axis, radius, height)
    }

    fn normalized_axis(&self) -> (f64, f64, f64) {
        if self.axis_len == 0.0 {
            (0.0, 1.0, 0.0)
        } else {
            (
                self.axis.0 / self.axis_len,
                self.axis.1 / self.axis_len,
                self.axis.2 / self.axis_len,
            )
        }
    }

    fn project(&self, x: i32, y: i32, z: i32) -> (f64, f64) {
        let ax = self.normalized_axis();
        let dx = x as f64 - self.base_center.0;
        let dy = y as f64 - self.base_center.1;
        let dz = z as f64 - self.base_center.2;
        let axial = dx * ax.0 + dy * ax.1 + dz * ax.2;
        let rx = dx - axial * ax.0;
        let ry = dy - axial * ax.1;
        let rz = dz - axial * ax.2;
        let radial = (rx * rx + ry * ry + rz * rz).sqrt();
        (axial, radial)
    }
}

impl Shape for Cylinder {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        let (axial, radial) = self.project(x, y, z);
        axial >= 0.0 && axial <= self.height && radial <= self.radius
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = Vec::new();
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        let ax = self.normalized_axis();
        let dx = x as f64 - self.base_center.0;
        let dy = y as f64 - self.base_center.1;
        let dz = z as f64 - self.base_center.2;
        let axial = dx * ax.0 + dy * ax.1 + dz * ax.2;
        let rx = dx - axial * ax.0;
        let ry = dy - axial * ax.1;
        let rz = dz - axial * ax.2;
        let radial = (rx * rx + ry * ry + rz * rz).sqrt();
        if radial == 0.0 {
            ax
        } else {
            (rx / radial, ry / radial, rz / radial)
        }
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        let ax = self.normalized_axis();
        let end = (
            self.base_center.0 + ax.0 * self.height,
            self.base_center.1 + ax.1 * self.height,
            self.base_center.2 + ax.2 * self.height,
        );
        let r = self.radius.ceil() as i32 + 1;
        let min_x = (self.base_center.0.min(end.0)).floor() as i32 - r;
        let min_y = (self.base_center.1.min(end.1)).floor() as i32 - r;
        let min_z = (self.base_center.2.min(end.2)).floor() as i32 - r;
        let max_x = (self.base_center.0.max(end.0)).ceil() as i32 + r;
        let max_y = (self.base_center.1.max(end.1)).ceil() as i32 + r;
        let max_z = (self.base_center.2.max(end.2)).ceil() as i32 + r;
        (min_x, min_y, min_z, max_x, max_y, max_z)
    }

    fn for_each_point<F>(&self, mut f: F)
    where
        F: FnMut(i32, i32, i32),
    {
        let (min_x, min_y, min_z, max_x, max_y, max_z) = self.bounds();
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    if self.contains(x, y, z) {
                        f(x, y, z);
                    }
                }
            }
        }
    }
}

impl ParametricShape for Cylinder {
    fn parameter_at(&self, x: i32, y: i32, z: i32) -> f64 {
        let (axial, _) = self.project(x, y, z);
        if self.height == 0.0 {
            0.0
        } else {
            (axial / self.height).clamp(0.0, 1.0)
        }
    }
}
