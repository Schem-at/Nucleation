use super::Shape;

#[derive(Clone)]
pub struct Ellipsoid {
    pub center: (i32, i32, i32),
    pub radii: (f64, f64, f64),
}

impl Ellipsoid {
    pub fn new(center: (i32, i32, i32), radii: (f64, f64, f64)) -> Self {
        Self { center, radii }
    }
}

impl Shape for Ellipsoid {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        let dx = (x - self.center.0) as f64 / self.radii.0;
        let dy = (y - self.center.1) as f64 / self.radii.1;
        let dz = (z - self.center.2) as f64 / self.radii.2;
        dx * dx + dy * dy + dz * dz <= 1.0
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = Vec::new();
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        let nx = (x - self.center.0) as f64 / (self.radii.0 * self.radii.0);
        let ny = (y - self.center.1) as f64 / (self.radii.1 * self.radii.1);
        let nz = (z - self.center.2) as f64 / (self.radii.2 * self.radii.2);
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len == 0.0 {
            (0.0, 1.0, 0.0)
        } else {
            (nx / len, ny / len, nz / len)
        }
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        let rx = self.radii.0.ceil() as i32;
        let ry = self.radii.1.ceil() as i32;
        let rz = self.radii.2.ceil() as i32;
        (
            self.center.0 - rx,
            self.center.1 - ry,
            self.center.2 - rz,
            self.center.0 + rx,
            self.center.1 + ry,
            self.center.2 + rz,
        )
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
