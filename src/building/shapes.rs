pub trait Shape {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool;
    // Returns an iterator over all points in the shape
    fn points(&self) -> Vec<(i32, i32, i32)>;
    // Returns the surface normal at the given point (normalized)
    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64);
}

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
        let r = self.radius.ceil() as i32;

        for x in (self.center.0 - r)..=(self.center.0 + r) {
            for y in (self.center.1 - r)..=(self.center.1 + r) {
                for z in (self.center.2 - r)..=(self.center.2 + r) {
                    if self.contains(x, y, z) {
                        points.push((x, y, z));
                    }
                }
            }
        }
        points
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        let dx = x as f64 - self.center.0 as f64;
        let dy = y as f64 - self.center.1 as f64;
        let dz = z as f64 - self.center.2 as f64;
        let len = (dx * dx + dy * dy + dz * dz).sqrt();
        if len == 0.0 {
            (0.0, 1.0, 0.0)
        } else {
            (dx / len, dy / len, dz / len)
        }
    }
}

pub struct Cuboid {
    pub min: (i32, i32, i32),
    pub max: (i32, i32, i32),
}

impl Cuboid {
    pub fn new(p1: (i32, i32, i32), p2: (i32, i32, i32)) -> Self {
        let min = (p1.0.min(p2.0), p1.1.min(p2.1), p1.2.min(p2.2));
        let max = (p1.0.max(p2.0), p1.1.max(p2.1), p1.2.max(p2.2));
        Self { min, max }
    }
}

impl Shape for Cuboid {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        x >= self.min.0
            && x <= self.max.0
            && y >= self.min.1
            && y <= self.max.1
            && z >= self.min.2
            && z <= self.max.2
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = Vec::new();
        for x in self.min.0..=self.max.0 {
            for y in self.min.1..=self.max.1 {
                for z in self.min.2..=self.max.2 {
                    points.push((x, y, z));
                }
            }
        }
        points
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        // For a cuboid, points inside don't really have a "surface normal" in the same way.
        // We can approximate by finding which face is closest.
        let min_d = [
            (x - self.min.0).abs(),
            (self.max.0 - x).abs(),
            (y - self.min.1).abs(),
            (self.max.1 - y).abs(),
            (z - self.min.2).abs(),
            (self.max.2 - z).abs(),
        ];

        let mut min_val = min_d[0];
        let mut min_idx = 0;
        for (i, val) in min_d.iter().enumerate() {
            if *val < min_val {
                min_val = *val;
                min_idx = i;
            }
        }

        match min_idx {
            0 => (-1.0, 0.0, 0.0),
            1 => (1.0, 0.0, 0.0),
            2 => (0.0, -1.0, 0.0),
            3 => (0.0, 1.0, 0.0),
            4 => (0.0, 0.0, -1.0),
            5 => (0.0, 0.0, 1.0),
            _ => (0.0, 1.0, 0.0),
        }
    }
}
