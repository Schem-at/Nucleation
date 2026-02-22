use super::{ParametricShape, Shape};

#[derive(Clone)]
pub struct Cone {
    pub apex: (f64, f64, f64),
    pub axis: (f64, f64, f64),
    pub base_radius: f64,
    pub height: f64,
    axis_len: f64,
}

impl Cone {
    pub fn new(
        apex: (f64, f64, f64),
        axis: (f64, f64, f64),
        base_radius: f64,
        height: f64,
    ) -> Self {
        let axis_len = (axis.0 * axis.0 + axis.1 * axis.1 + axis.2 * axis.2).sqrt();
        Self {
            apex,
            axis,
            base_radius,
            height,
            axis_len,
        }
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
        let dx = x as f64 - self.apex.0;
        let dy = y as f64 - self.apex.1;
        let dz = z as f64 - self.apex.2;
        // Axial distance from apex along axis direction
        let axial = dx * ax.0 + dy * ax.1 + dz * ax.2;
        let rx = dx - axial * ax.0;
        let ry = dy - axial * ax.1;
        let rz = dz - axial * ax.2;
        let radial = (rx * rx + ry * ry + rz * rz).sqrt();
        (axial, radial)
    }
}

impl Shape for Cone {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        let (axial, radial) = self.project(x, y, z);
        if axial < 0.0 || axial > self.height {
            return false;
        }
        // Radius linearly tapers from 0 at apex to base_radius at height
        let allowed_radius = self.base_radius * (axial / self.height);
        radial <= allowed_radius
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = Vec::new();
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        let ax = self.normalized_axis();
        let dx = x as f64 - self.apex.0;
        let dy = y as f64 - self.apex.1;
        let dz = z as f64 - self.apex.2;
        let axial = dx * ax.0 + dy * ax.1 + dz * ax.2;
        let rx = dx - axial * ax.0;
        let ry = dy - axial * ax.1;
        let rz = dz - axial * ax.2;
        let radial = (rx * rx + ry * ry + rz * rz).sqrt();
        if radial == 0.0 {
            ax
        } else {
            // Normal tilts outward from the surface
            let slope = self.base_radius / self.height;
            let nr = 1.0;
            let na = -slope;
            let len = (nr * nr + na * na).sqrt();
            let radial_dir = (rx / radial, ry / radial, rz / radial);
            let nx = radial_dir.0 * (nr / len) + ax.0 * (na / len);
            let ny = radial_dir.1 * (nr / len) + ax.1 * (na / len);
            let nz = radial_dir.2 * (nr / len) + ax.2 * (na / len);
            (nx, ny, nz)
        }
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        let ax = self.normalized_axis();
        let base_center = (
            self.apex.0 + ax.0 * self.height,
            self.apex.1 + ax.1 * self.height,
            self.apex.2 + ax.2 * self.height,
        );
        let r = self.base_radius.ceil() as i32 + 1;
        let min_x = (self.apex.0.min(base_center.0)).floor() as i32 - r;
        let min_y = (self.apex.1.min(base_center.1)).floor() as i32 - r;
        let min_z = (self.apex.2.min(base_center.2)).floor() as i32 - r;
        let max_x = (self.apex.0.max(base_center.0)).ceil() as i32 + r;
        let max_y = (self.apex.1.max(base_center.1)).ceil() as i32 + r;
        let max_z = (self.apex.2.max(base_center.2)).ceil() as i32 + r;
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

impl ParametricShape for Cone {
    fn parameter_at(&self, x: i32, y: i32, z: i32) -> f64 {
        let (axial, _) = self.project(x, y, z);
        if self.height == 0.0 {
            0.0
        } else {
            (axial / self.height).clamp(0.0, 1.0)
        }
    }
}
