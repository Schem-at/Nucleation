use super::{ParametricShape, Shape};

#[derive(Clone)]
pub struct BezierCurve {
    pub control_points: Vec<(f64, f64, f64)>,
    pub thickness: f64,
    pub resolution: u32,
    segments: Vec<((f64, f64, f64), (f64, f64, f64))>,
    segment_lengths: Vec<f64>,
    total_length: f64,
}

impl BezierCurve {
    pub fn new(control_points: Vec<(f64, f64, f64)>, thickness: f64, resolution: u32) -> Self {
        let resolution = resolution.max(2);
        let mut segments = Vec::new();
        let mut segment_lengths = Vec::new();
        let mut total_length = 0.0;

        for i in 0..resolution {
            let t0 = i as f64 / resolution as f64;
            let t1 = (i + 1) as f64 / resolution as f64;
            let p0 = evaluate_bezier(&control_points, t0);
            let p1 = evaluate_bezier(&control_points, t1);
            let dx = p1.0 - p0.0;
            let dy = p1.1 - p0.1;
            let dz = p1.2 - p0.2;
            let len = (dx * dx + dy * dy + dz * dz).sqrt();
            segments.push((p0, p1));
            segment_lengths.push(len);
            total_length += len;
        }

        Self {
            control_points,
            thickness,
            resolution,
            segments,
            segment_lengths,
            total_length,
        }
    }

    fn closest_point_info(&self, x: i32, y: i32, z: i32) -> (f64, f64) {
        let px = x as f64;
        let py = y as f64;
        let pz = z as f64;

        let mut best_dist = f64::MAX;
        let mut best_t = 0.0;
        let mut cumulative_len = 0.0;

        for (i, (p0, p1)) in self.segments.iter().enumerate() {
            let dx = p1.0 - p0.0;
            let dy = p1.1 - p0.1;
            let dz = p1.2 - p0.2;
            let seg_len_sq = dx * dx + dy * dy + dz * dz;

            let local_t = if seg_len_sq == 0.0 {
                0.0
            } else {
                let vx = px - p0.0;
                let vy = py - p0.1;
                let vz = pz - p0.2;
                ((vx * dx + vy * dy + vz * dz) / seg_len_sq).clamp(0.0, 1.0)
            };

            let closest_x = p0.0 + local_t * dx;
            let closest_y = p0.1 + local_t * dy;
            let closest_z = p0.2 + local_t * dz;
            let rx = px - closest_x;
            let ry = py - closest_y;
            let rz = pz - closest_z;
            let dist = (rx * rx + ry * ry + rz * rz).sqrt();

            if dist < best_dist {
                best_dist = dist;
                if self.total_length > 0.0 {
                    best_t =
                        (cumulative_len + local_t * self.segment_lengths[i]) / self.total_length;
                }
            }
            cumulative_len += self.segment_lengths[i];
        }

        (best_dist, best_t.clamp(0.0, 1.0))
    }
}

impl Shape for BezierCurve {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        let (dist, _) = self.closest_point_info(x, y, z);
        dist <= self.thickness / 2.0
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = Vec::new();
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        let px = x as f64;
        let py = y as f64;
        let pz = z as f64;

        let mut best_dist = f64::MAX;
        let mut best_closest = (0.0, 0.0, 0.0);

        for (p0, p1) in &self.segments {
            let dx = p1.0 - p0.0;
            let dy = p1.1 - p0.1;
            let dz = p1.2 - p0.2;
            let seg_len_sq = dx * dx + dy * dy + dz * dz;
            let local_t = if seg_len_sq == 0.0 {
                0.0
            } else {
                let vx = px - p0.0;
                let vy = py - p0.1;
                let vz = pz - p0.2;
                ((vx * dx + vy * dy + vz * dz) / seg_len_sq).clamp(0.0, 1.0)
            };
            let cx = p0.0 + local_t * dx;
            let cy = p0.1 + local_t * dy;
            let cz = p0.2 + local_t * dz;
            let rx = px - cx;
            let ry = py - cy;
            let rz = pz - cz;
            let dist = (rx * rx + ry * ry + rz * rz).sqrt();
            if dist < best_dist {
                best_dist = dist;
                best_closest = (cx, cy, cz);
            }
        }

        let nx = px - best_closest.0;
        let ny = py - best_closest.1;
        let nz = pz - best_closest.2;
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len == 0.0 {
            (0.0, 1.0, 0.0)
        } else {
            (nx / len, ny / len, nz / len)
        }
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        if self.segments.is_empty() {
            return (0, 0, 0, 0, 0, 0);
        }
        let r = (self.thickness / 2.0).ceil() as i32 + 1;
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut min_z = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        let mut max_z = f64::MIN;

        for (p0, p1) in &self.segments {
            min_x = min_x.min(p0.0).min(p1.0);
            min_y = min_y.min(p0.1).min(p1.1);
            min_z = min_z.min(p0.2).min(p1.2);
            max_x = max_x.max(p0.0).max(p1.0);
            max_y = max_y.max(p0.1).max(p1.1);
            max_z = max_z.max(p0.2).max(p1.2);
        }

        (
            min_x.floor() as i32 - r,
            min_y.floor() as i32 - r,
            min_z.floor() as i32 - r,
            max_x.ceil() as i32 + r,
            max_y.ceil() as i32 + r,
            max_z.ceil() as i32 + r,
        )
    }

    fn for_each_point<F>(&self, mut f: F)
    where
        F: FnMut(i32, i32, i32),
    {
        let (min_x, min_y, min_z, max_x, max_y, max_z) = self.bounds();
        let half_t = self.thickness / 2.0;
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    let (dist, _) = self.closest_point_info(x, y, z);
                    if dist <= half_t {
                        f(x, y, z);
                    }
                }
            }
        }
    }
}

impl ParametricShape for BezierCurve {
    fn parameter_at(&self, x: i32, y: i32, z: i32) -> f64 {
        let (_, t) = self.closest_point_info(x, y, z);
        t
    }
}

fn evaluate_bezier(points: &[(f64, f64, f64)], t: f64) -> (f64, f64, f64) {
    if points.len() == 1 {
        return points[0];
    }
    // De Casteljau's algorithm
    let mut work: Vec<(f64, f64, f64)> = points.to_vec();
    let n = work.len();
    for level in 1..n {
        for i in 0..n - level {
            work[i] = (
                work[i].0 * (1.0 - t) + work[i + 1].0 * t,
                work[i].1 * (1.0 - t) + work[i + 1].1 * t,
                work[i].2 * (1.0 - t) + work[i + 1].2 * t,
            );
        }
    }
    work[0]
}
