use super::{ParametricShape, Shape};

#[derive(Clone)]
pub struct Pyramid {
    pub base_center: (f64, f64, f64),
    pub base_half_size: (f64, f64),
    pub height: f64,
    pub axis: (f64, f64, f64),
    axis_len: f64,
}

impl Pyramid {
    pub fn new(
        base_center: (f64, f64, f64),
        base_half_size: (f64, f64),
        height: f64,
        axis: (f64, f64, f64),
    ) -> Self {
        let axis_len = (axis.0 * axis.0 + axis.1 * axis.1 + axis.2 * axis.2).sqrt();
        Self {
            base_center,
            base_half_size,
            height,
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

    /// Build a local coordinate frame: axis = up, u = right, v = forward
    fn local_frame(&self) -> ((f64, f64, f64), (f64, f64, f64), (f64, f64, f64)) {
        let up = self.normalized_axis();
        // Pick a non-parallel vector for cross product
        let ref_vec = if up.1.abs() < 0.9 {
            (0.0, 1.0, 0.0)
        } else {
            (1.0, 0.0, 0.0)
        };
        // u = normalize(cross(up, ref))
        let u = cross(up, ref_vec);
        let u_len = (u.0 * u.0 + u.1 * u.1 + u.2 * u.2).sqrt();
        let u = (u.0 / u_len, u.1 / u_len, u.2 / u_len);
        // v = cross(u, up) — not cross(up, u) — for right-handed frame
        let v = cross(u, up);
        (up, u, v)
    }
}

fn cross(a: (f64, f64, f64), b: (f64, f64, f64)) -> (f64, f64, f64) {
    (
        a.1 * b.2 - a.2 * b.1,
        a.2 * b.0 - a.0 * b.2,
        a.0 * b.1 - a.1 * b.0,
    )
}

impl Shape for Pyramid {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        let (up, u_dir, v_dir) = self.local_frame();
        let dx = x as f64 - self.base_center.0;
        let dy = y as f64 - self.base_center.1;
        let dz = z as f64 - self.base_center.2;

        let h = dx * up.0 + dy * up.1 + dz * up.2;
        if h < 0.0 || h > self.height {
            return false;
        }

        let u_coord = dx * u_dir.0 + dy * u_dir.1 + dz * u_dir.2;
        let v_coord = dx * v_dir.0 + dy * v_dir.1 + dz * v_dir.2;

        // Cross-section shrinks linearly with height
        let t = 1.0 - h / self.height;
        u_coord.abs() <= self.base_half_size.0 * t && v_coord.abs() <= self.base_half_size.1 * t
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = Vec::new();
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        let (up, u_dir, v_dir) = self.local_frame();
        let dx = x as f64 - self.base_center.0;
        let dy = y as f64 - self.base_center.1;
        let dz = z as f64 - self.base_center.2;

        let h = dx * up.0 + dy * up.1 + dz * up.2;
        let u_coord = dx * u_dir.0 + dy * u_dir.1 + dz * u_dir.2;
        let v_coord = dx * v_dir.0 + dy * v_dir.1 + dz * v_dir.2;
        let t = 1.0 - h / self.height;
        let u_edge = self.base_half_size.0 * t;
        let v_edge = self.base_half_size.1 * t;

        // Nearest face normal
        let u_dist = u_edge - u_coord.abs();
        let v_dist = v_edge - v_coord.abs();

        if u_dist < v_dist {
            let sign = u_coord.signum();
            (u_dir.0 * sign, u_dir.1 * sign, u_dir.2 * sign)
        } else {
            let sign = v_coord.signum();
            (v_dir.0 * sign, v_dir.1 * sign, v_dir.2 * sign)
        }
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        let r = self.base_half_size.0.max(self.base_half_size.1).ceil() as i32 + 1;
        let h = self.height.ceil() as i32 + 1;
        let cx = self.base_center.0.round() as i32;
        let cy = self.base_center.1.round() as i32;
        let cz = self.base_center.2.round() as i32;
        (cx - r, cy - r, cz - r, cx + r, cy + h, cz + r)
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

impl ParametricShape for Pyramid {
    fn parameter_at(&self, x: i32, y: i32, z: i32) -> f64 {
        let up = self.normalized_axis();
        let dx = x as f64 - self.base_center.0;
        let dy = y as f64 - self.base_center.1;
        let dz = z as f64 - self.base_center.2;
        let h = dx * up.0 + dy * up.1 + dz * up.2;
        if self.height == 0.0 {
            0.0
        } else {
            (h / self.height).clamp(0.0, 1.0)
        }
    }
}
