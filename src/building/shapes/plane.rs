use super::Shape;

#[derive(Clone)]
pub struct Plane {
    pub origin: (f64, f64, f64),
    pub u_vec: (f64, f64, f64),
    pub v_vec: (f64, f64, f64),
    pub u_extent: f64,
    pub v_extent: f64,
    pub thickness: f64,
    u_len_sq: f64,
    v_len_sq: f64,
    normal: (f64, f64, f64),
}

impl Plane {
    pub fn new(
        origin: (f64, f64, f64),
        u_vec: (f64, f64, f64),
        v_vec: (f64, f64, f64),
        u_extent: f64,
        v_extent: f64,
        thickness: f64,
    ) -> Self {
        let u_len_sq = u_vec.0 * u_vec.0 + u_vec.1 * u_vec.1 + u_vec.2 * u_vec.2;
        let v_len_sq = v_vec.0 * v_vec.0 + v_vec.1 * v_vec.1 + v_vec.2 * v_vec.2;

        // Normal = cross(u_vec, v_vec), normalized
        let nx = u_vec.1 * v_vec.2 - u_vec.2 * v_vec.1;
        let ny = u_vec.2 * v_vec.0 - u_vec.0 * v_vec.2;
        let nz = u_vec.0 * v_vec.1 - u_vec.1 * v_vec.0;
        let n_len = (nx * nx + ny * ny + nz * nz).sqrt();
        let normal = if n_len == 0.0 {
            (0.0, 1.0, 0.0)
        } else {
            (nx / n_len, ny / n_len, nz / n_len)
        };

        Self {
            origin,
            u_vec,
            v_vec,
            u_extent,
            v_extent,
            thickness,
            u_len_sq,
            v_len_sq,
            normal,
        }
    }
}

impl Shape for Plane {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        let dx = x as f64 - self.origin.0;
        let dy = y as f64 - self.origin.1;
        let dz = z as f64 - self.origin.2;

        // Distance from plane
        let plane_dist = (dx * self.normal.0 + dy * self.normal.1 + dz * self.normal.2).abs();
        if plane_dist > self.thickness / 2.0 {
            return false;
        }

        // Project onto u and v axes
        let u = if self.u_len_sq > 0.0 {
            (dx * self.u_vec.0 + dy * self.u_vec.1 + dz * self.u_vec.2) / self.u_len_sq
        } else {
            0.0
        };

        let v = if self.v_len_sq > 0.0 {
            (dx * self.v_vec.0 + dy * self.v_vec.1 + dz * self.v_vec.2) / self.v_len_sq
        } else {
            0.0
        };

        u.abs() <= self.u_extent && v.abs() <= self.v_extent
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = Vec::new();
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, _x: i32, _y: i32, _z: i32) -> (f64, f64, f64) {
        self.normal
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        let u_len = self.u_len_sq.sqrt();
        let v_len = self.v_len_sq.sqrt();
        let r = (u_len * self.u_extent + v_len * self.v_extent + self.thickness).ceil() as i32 + 1;
        let cx = self.origin.0.round() as i32;
        let cy = self.origin.1.round() as i32;
        let cz = self.origin.2.round() as i32;
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
