use super::{ParametricShape, Shape};

#[derive(Clone)]
pub struct Line {
    pub start: (f64, f64, f64),
    pub end: (f64, f64, f64),
    pub thickness: f64,
    length_sq: f64,
}

impl Line {
    pub fn new(start: (f64, f64, f64), end: (f64, f64, f64), thickness: f64) -> Self {
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let dz = end.2 - start.2;
        Self {
            start,
            end,
            thickness,
            length_sq: dx * dx + dy * dy + dz * dz,
        }
    }

    fn project_t(&self, x: i32, y: i32, z: i32) -> f64 {
        if self.length_sq == 0.0 {
            return 0.0;
        }
        let dx = self.end.0 - self.start.0;
        let dy = self.end.1 - self.start.1;
        let dz = self.end.2 - self.start.2;
        let vx = x as f64 - self.start.0;
        let vy = y as f64 - self.start.1;
        let vz = z as f64 - self.start.2;
        let dot = vx * dx + vy * dy + vz * dz;
        (dot / self.length_sq).clamp(0.0, 1.0)
    }

    fn distance_to_segment(&self, x: i32, y: i32, z: i32) -> f64 {
        let t = self.project_t(x, y, z);
        let dx = self.end.0 - self.start.0;
        let dy = self.end.1 - self.start.1;
        let dz = self.end.2 - self.start.2;
        let closest_x = self.start.0 + t * dx;
        let closest_y = self.start.1 + t * dy;
        let closest_z = self.start.2 + t * dz;
        let rx = x as f64 - closest_x;
        let ry = y as f64 - closest_y;
        let rz = z as f64 - closest_z;
        (rx * rx + ry * ry + rz * rz).sqrt()
    }
}

impl Shape for Line {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        // Bresenham fast path for thin lines
        if self.thickness <= 0.5 {
            return self.distance_to_segment(x, y, z) <= 0.87; // ~sqrt(3)/2 for voxel diag
        }
        self.distance_to_segment(x, y, z) <= self.thickness / 2.0
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        // For thin lines, use Bresenham's algorithm
        if self.thickness <= 0.5 {
            return bresenham_3d(
                self.start.0.round() as i32,
                self.start.1.round() as i32,
                self.start.2.round() as i32,
                self.end.0.round() as i32,
                self.end.1.round() as i32,
                self.end.2.round() as i32,
            );
        }
        let mut points = Vec::new();
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        let t = self.project_t(x, y, z);
        let dx = self.end.0 - self.start.0;
        let dy = self.end.1 - self.start.1;
        let dz = self.end.2 - self.start.2;
        let closest_x = self.start.0 + t * dx;
        let closest_y = self.start.1 + t * dy;
        let closest_z = self.start.2 + t * dz;
        let rx = x as f64 - closest_x;
        let ry = y as f64 - closest_y;
        let rz = z as f64 - closest_z;
        let len = (rx * rx + ry * ry + rz * rz).sqrt();
        if len == 0.0 {
            (0.0, 1.0, 0.0)
        } else {
            (rx / len, ry / len, rz / len)
        }
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        let r = (self.thickness / 2.0).ceil() as i32 + 1;
        let min_x = (self.start.0.min(self.end.0)).floor() as i32 - r;
        let min_y = (self.start.1.min(self.end.1)).floor() as i32 - r;
        let min_z = (self.start.2.min(self.end.2)).floor() as i32 - r;
        let max_x = (self.start.0.max(self.end.0)).ceil() as i32 + r;
        let max_y = (self.start.1.max(self.end.1)).ceil() as i32 + r;
        let max_z = (self.start.2.max(self.end.2)).ceil() as i32 + r;
        (min_x, min_y, min_z, max_x, max_y, max_z)
    }

    fn for_each_point<F>(&self, mut f: F)
    where
        F: FnMut(i32, i32, i32),
    {
        if self.thickness <= 0.5 {
            for (x, y, z) in bresenham_3d(
                self.start.0.round() as i32,
                self.start.1.round() as i32,
                self.start.2.round() as i32,
                self.end.0.round() as i32,
                self.end.1.round() as i32,
                self.end.2.round() as i32,
            ) {
                f(x, y, z);
            }
            return;
        }
        let (min_x, min_y, min_z, max_x, max_y, max_z) = self.bounds();
        let half_t = self.thickness / 2.0;
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    if self.distance_to_segment(x, y, z) <= half_t {
                        f(x, y, z);
                    }
                }
            }
        }
    }
}

impl ParametricShape for Line {
    fn parameter_at(&self, x: i32, y: i32, z: i32) -> f64 {
        self.project_t(x, y, z)
    }
}

fn bresenham_3d(x0: i32, y0: i32, z0: i32, x1: i32, y1: i32, z1: i32) -> Vec<(i32, i32, i32)> {
    let mut points = Vec::new();
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let dz = (z1 - z0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let sz = if z0 < z1 { 1 } else { -1 };

    let dm = dx.max(dy).max(dz);
    let mut x = x0;
    let mut y = y0;
    let mut z = z0;

    let mut err_x = dm / 2;
    let mut err_y = dm / 2;
    let mut err_z = dm / 2;

    for _ in 0..=dm {
        points.push((x, y, z));
        err_x -= dx;
        err_y -= dy;
        err_z -= dz;
        if err_x < 0 {
            err_x += dm;
            x += sx;
        }
        if err_y < 0 {
            err_y += dm;
            y += sy;
        }
        if err_z < 0 {
            err_z += dm;
            z += sz;
        }
    }
    points
}
