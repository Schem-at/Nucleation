use super::Shape;

/// A vertical prism: a closed 2D polygon in the X/Z plane extruded between two
/// Y levels. The footprint is filled by an even-odd point-in-polygon test at
/// each voxel center, so any simple polygon works — building footprints, lake
/// outlines, plot boundaries. Vertices are `(x, z)` world coordinates in order
/// (winding does not matter); the ring is closed implicitly.
#[derive(Clone)]
pub struct PolygonPrism {
    verts: Vec<(f64, f64)>,
    y_min: i32,
    y_max: i32,
    min_x: i32,
    min_z: i32,
    max_x: i32,
    max_z: i32,
}

impl PolygonPrism {
    pub fn new(verts: Vec<(f64, f64)>, y0: i32, y1: i32) -> Self {
        let (y_min, y_max) = (y0.min(y1), y0.max(y1));
        let (mut lo_x, mut lo_z, mut hi_x, mut hi_z) = (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        );
        for &(x, z) in &verts {
            lo_x = lo_x.min(x);
            lo_z = lo_z.min(z);
            hi_x = hi_x.max(x);
            hi_z = hi_z.max(z);
        }
        // Empty polygon -> a degenerate, contains-nothing box.
        if !lo_x.is_finite() {
            lo_x = 0.0;
            lo_z = 0.0;
            hi_x = -1.0;
            hi_z = -1.0;
        }
        Self {
            verts,
            y_min,
            y_max,
            min_x: lo_x.floor() as i32,
            min_z: lo_z.floor() as i32,
            max_x: hi_x.ceil() as i32,
            max_z: hi_z.ceil() as i32,
        }
    }

    /// Even-odd ray cast in the X/Z plane.
    fn footprint_contains(&self, px: f64, pz: f64) -> bool {
        let v = &self.verts;
        let n = v.len();
        if n < 3 {
            return false;
        }
        let mut inside = false;
        let mut j = n - 1;
        for i in 0..n {
            let (xi, zi) = v[i];
            let (xj, zj) = v[j];
            if (zi > pz) != (zj > pz) {
                let xint = (xj - xi) * (pz - zi) / (zj - zi) + xi;
                if px < xint {
                    inside = !inside;
                }
            }
            j = i;
        }
        inside
    }
}

impl Shape for PolygonPrism {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        y >= self.y_min
            && y <= self.y_max
            && self.footprint_contains(x as f64 + 0.5, z as f64 + 0.5)
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = Vec::new();
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, _x: i32, _y: i32, _z: i32) -> (f64, f64, f64) {
        (0.0, 1.0, 0.0)
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        (
            self.min_x, self.y_min, self.min_z, self.max_x, self.y_max, self.max_z,
        )
    }

    fn for_each_point<F>(&self, mut f: F)
    where
        F: FnMut(i32, i32, i32),
    {
        // Test the footprint once per (x, z) column, then fill the whole
        // Y range — far cheaper than a point-in-polygon test per voxel.
        for x in self.min_x..=self.max_x {
            for z in self.min_z..=self.max_z {
                if self.footprint_contains(x as f64 + 0.5, z as f64 + 0.5) {
                    for y in self.y_min..=self.y_max {
                        f(x, y, z);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> PolygonPrism {
        // 0..10 square footprint, y 0..4.
        PolygonPrism::new(
            vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)],
            0,
            4,
        )
    }

    #[test]
    fn fills_footprint_and_height() {
        let p = square();
        assert!(p.contains(5, 2, 5), "center of the prism is solid");
        assert!(p.contains(1, 0, 1), "corner interior is solid");
        assert!(!p.contains(5, 5, 5), "above the top is empty");
        assert!(!p.contains(5, -1, 5), "below the base is empty");
        assert!(!p.contains(11, 2, 5), "outside the footprint is empty");
        assert_eq!(p.bounds(), (0, 0, 0, 10, 4, 10));
    }

    #[test]
    fn concave_polygon_respects_the_notch() {
        // An L-shape: the notch corner must be empty.
        let l = PolygonPrism::new(
            vec![
                (0.0, 0.0),
                (10.0, 0.0),
                (10.0, 4.0),
                (4.0, 4.0),
                (4.0, 10.0),
                (0.0, 10.0),
            ],
            0,
            2,
        );
        assert!(l.contains(2, 1, 2), "inside the L arm");
        assert!(l.contains(8, 1, 2), "inside the L foot");
        assert!(!l.contains(8, 1, 8), "the notch is empty");
    }
}
