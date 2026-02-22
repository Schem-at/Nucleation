use super::Shape;

#[derive(Clone)]
pub struct Disk {
    pub center: (f64, f64, f64),
    pub radius: f64,
    pub normal: (f64, f64, f64),
    pub thickness: f64,
    normal_len: f64,
}

impl Disk {
    pub fn new(
        center: (f64, f64, f64),
        radius: f64,
        normal: (f64, f64, f64),
        thickness: f64,
    ) -> Self {
        let normal_len = (normal.0 * normal.0 + normal.1 * normal.1 + normal.2 * normal.2).sqrt();
        Self {
            center,
            radius,
            normal,
            thickness,
            normal_len,
        }
    }

    fn normalized_normal(&self) -> (f64, f64, f64) {
        if self.normal_len == 0.0 {
            (0.0, 1.0, 0.0)
        } else {
            (
                self.normal.0 / self.normal_len,
                self.normal.1 / self.normal_len,
                self.normal.2 / self.normal_len,
            )
        }
    }
}

impl Shape for Disk {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        let n = self.normalized_normal();
        let dx = x as f64 - self.center.0;
        let dy = y as f64 - self.center.1;
        let dz = z as f64 - self.center.2;

        let plane_dist = (dx * n.0 + dy * n.1 + dz * n.2).abs();
        if plane_dist > self.thickness / 2.0 {
            return false;
        }

        let rx = dx - (dx * n.0 + dy * n.1 + dz * n.2) * n.0;
        let ry = dy - (dx * n.0 + dy * n.1 + dz * n.2) * n.1;
        let rz = dz - (dx * n.0 + dy * n.1 + dz * n.2) * n.2;
        let radial = (rx * rx + ry * ry + rz * rz).sqrt();
        radial <= self.radius
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = Vec::new();
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, _x: i32, _y: i32, _z: i32) -> (f64, f64, f64) {
        self.normalized_normal()
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        let r = self.radius.ceil() as i32 + 1;
        let t = (self.thickness / 2.0).ceil() as i32 + 1;
        let b = r + t;
        let cx = self.center.0.round() as i32;
        let cy = self.center.1.round() as i32;
        let cz = self.center.2.round() as i32;
        (cx - b, cy - b, cz - b, cx + b, cy + b, cz + b)
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
