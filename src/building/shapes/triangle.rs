use super::Shape;

#[derive(Clone)]
pub struct Triangle {
    pub a: (f64, f64, f64),
    pub b: (f64, f64, f64),
    pub c: (f64, f64, f64),
    pub thickness: f64,
    normal: (f64, f64, f64),
}

impl Triangle {
    pub fn new(a: (f64, f64, f64), b: (f64, f64, f64), c: (f64, f64, f64), thickness: f64) -> Self {
        let ab = (b.0 - a.0, b.1 - a.1, b.2 - a.2);
        let ac = (c.0 - a.0, c.1 - a.1, c.2 - a.2);
        let nx = ab.1 * ac.2 - ab.2 * ac.1;
        let ny = ab.2 * ac.0 - ab.0 * ac.2;
        let nz = ab.0 * ac.1 - ab.1 * ac.0;
        let n_len = (nx * nx + ny * ny + nz * nz).sqrt();
        let normal = if n_len == 0.0 {
            (0.0, 1.0, 0.0)
        } else {
            (nx / n_len, ny / n_len, nz / n_len)
        };
        Self {
            a,
            b,
            c,
            thickness,
            normal,
        }
    }
}

impl Shape for Triangle {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        let px = x as f64;
        let py = y as f64;
        let pz = z as f64;

        // Check plane distance
        let dx = px - self.a.0;
        let dy = py - self.a.1;
        let dz = pz - self.a.2;
        let plane_dist = (dx * self.normal.0 + dy * self.normal.1 + dz * self.normal.2).abs();
        if plane_dist > self.thickness / 2.0 {
            return false;
        }

        // Barycentric test: project onto triangle plane
        let v0 = (
            self.b.0 - self.a.0,
            self.b.1 - self.a.1,
            self.b.2 - self.a.2,
        );
        let v1 = (
            self.c.0 - self.a.0,
            self.c.1 - self.a.1,
            self.c.2 - self.a.2,
        );
        let v2 = (dx, dy, dz);

        let d00 = v0.0 * v0.0 + v0.1 * v0.1 + v0.2 * v0.2;
        let d01 = v0.0 * v1.0 + v0.1 * v1.1 + v0.2 * v1.2;
        let d11 = v1.0 * v1.0 + v1.1 * v1.1 + v1.2 * v1.2;
        let d20 = v2.0 * v0.0 + v2.1 * v0.1 + v2.2 * v0.2;
        let d21 = v2.0 * v1.0 + v2.1 * v1.1 + v2.2 * v1.2;

        let denom = d00 * d11 - d01 * d01;
        if denom.abs() < 1e-10 {
            return false;
        }

        let u = (d11 * d20 - d01 * d21) / denom;
        let v = (d00 * d21 - d01 * d20) / denom;

        u >= 0.0 && v >= 0.0 && u + v <= 1.0
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
        let t = (self.thickness / 2.0).ceil() as i32 + 1;
        let min_x = self.a.0.min(self.b.0).min(self.c.0).floor() as i32 - t;
        let min_y = self.a.1.min(self.b.1).min(self.c.1).floor() as i32 - t;
        let min_z = self.a.2.min(self.b.2).min(self.c.2).floor() as i32 - t;
        let max_x = self.a.0.max(self.b.0).max(self.c.0).ceil() as i32 + t;
        let max_y = self.a.1.max(self.b.1).max(self.c.1).ceil() as i32 + t;
        let max_z = self.a.2.max(self.b.2).max(self.c.2).ceil() as i32 + t;
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
