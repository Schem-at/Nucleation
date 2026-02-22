use super::Shape;

#[derive(Clone)]
pub struct Sphere {
    pub center: (i32, i32, i32),
    pub radius: f64,
}

impl Sphere {
    pub fn new(center: (i32, i32, i32), radius: f64) -> Self {
        Self { center, radius }
    }
}

impl Shape for Sphere {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        let dx = x - self.center.0;
        let dy = y - self.center.1;
        let dz = z - self.center.2;
        (dx * dx + dy * dy + dz * dz) as f64 <= self.radius * self.radius
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = Vec::new();
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        let dx = x as f64 - self.center.0 as f64;
        let dy = y as f64 - self.center.1 as f64;
        let dz = z as f64 - self.center.2 as f64;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if dist == 0.0 {
            (0.0, 1.0, 0.0)
        } else {
            (dx / dist, dy / dist, dz / dist)
        }
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        let r = self.radius.ceil() as i32;
        (
            self.center.0 - r,
            self.center.1 - r,
            self.center.2 - r,
            self.center.0 + r,
            self.center.1 + r,
            self.center.2 + r,
        )
    }

    fn for_each_point<F>(&self, mut f: F)
    where
        F: FnMut(i32, i32, i32),
    {
        let (min_x, min_y, min_z, max_x, max_y, max_z) = self.bounds();
        let r2 = self.radius * self.radius;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    let dx = x - self.center.0;
                    let dy = y - self.center.1;
                    let dz = z - self.center.2;
                    if (dx * dx + dy * dy + dz * dz) as f64 <= r2 {
                        f(x, y, z);
                    }
                }
            }
        }
    }
}
