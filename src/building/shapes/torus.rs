use super::{ParametricShape, Shape};

#[derive(Clone)]
pub struct Torus {
    pub center: (f64, f64, f64),
    pub major_radius: f64,
    pub minor_radius: f64,
    pub axis: (f64, f64, f64),
    axis_len: f64,
}

impl Torus {
    pub fn new(
        center: (f64, f64, f64),
        major_radius: f64,
        minor_radius: f64,
        axis: (f64, f64, f64),
    ) -> Self {
        let axis_len = (axis.0 * axis.0 + axis.1 * axis.1 + axis.2 * axis.2).sqrt();
        Self {
            center,
            major_radius,
            minor_radius,
            axis,
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

    fn local_coords(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        let ax = self.normalized_axis();
        let dx = x as f64 - self.center.0;
        let dy = y as f64 - self.center.1;
        let dz = z as f64 - self.center.2;

        // Project onto axis to get "height" (axial component)
        let h = dx * ax.0 + dy * ax.1 + dz * ax.2;

        // Radial component in the plane perpendicular to axis
        let px = dx - h * ax.0;
        let py = dy - h * ax.1;
        let pz = dz - h * ax.2;
        let planar_dist = (px * px + py * py + pz * pz).sqrt();

        (planar_dist, h, planar_dist.atan2(1.0))
    }
}

impl Shape for Torus {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        let (planar_dist, h, _) = self.local_coords(x, y, z);
        let d = planar_dist - self.major_radius;
        d * d + h * h <= self.minor_radius * self.minor_radius
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = Vec::new();
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        let ax = self.normalized_axis();
        let dx = x as f64 - self.center.0;
        let dy = y as f64 - self.center.1;
        let dz = z as f64 - self.center.2;

        let h = dx * ax.0 + dy * ax.1 + dz * ax.2;
        let px = dx - h * ax.0;
        let py = dy - h * ax.1;
        let pz = dz - h * ax.2;
        let planar_dist = (px * px + py * py + pz * pz).sqrt();

        if planar_dist == 0.0 {
            return (0.0, 1.0, 0.0);
        }

        // Direction from tube center to point
        let tube_center_x = px / planar_dist * self.major_radius;
        let tube_center_y = py / planar_dist * self.major_radius;
        let tube_center_z = pz / planar_dist * self.major_radius;

        let nx = dx - tube_center_x;
        let ny = dy - tube_center_y;
        let nz = dz - tube_center_z;
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len == 0.0 {
            (0.0, 1.0, 0.0)
        } else {
            (nx / len, ny / len, nz / len)
        }
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        let total = self.major_radius + self.minor_radius;
        let r = total.ceil() as i32 + 1;
        let cx = self.center.0.round() as i32;
        let cy = self.center.1.round() as i32;
        let cz = self.center.2.round() as i32;
        (cx - r, cy - r, cz - r, cx + r, cy + r, cz + r)
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

impl ParametricShape for Torus {
    fn parameter_at(&self, x: i32, y: i32, z: i32) -> f64 {
        let ax = self.normalized_axis();
        let dx = x as f64 - self.center.0;
        let dy = y as f64 - self.center.1;
        let dz = z as f64 - self.center.2;
        let h = dx * ax.0 + dy * ax.1 + dz * ax.2;
        let px = dx - h * ax.0;
        let py = dy - h * ax.1;

        // Angle around the major circle, mapped to [0, 1]
        let angle = py.atan2(px); // -PI to PI
        ((angle + std::f64::consts::PI) / (2.0 * std::f64::consts::PI)).clamp(0.0, 1.0)
    }
}
