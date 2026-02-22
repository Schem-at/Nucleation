use super::Shape;
use crate::building::enums::ShapeEnum;

#[derive(Clone)]
pub struct Hollow {
    pub inner: Box<ShapeEnum>,
    pub thickness: u32,
}

impl Hollow {
    pub fn new(inner: ShapeEnum, thickness: u32) -> Self {
        Self {
            inner: Box::new(inner),
            thickness,
        }
    }
}

impl Shape for Hollow {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        if !self.inner.contains(x, y, z) {
            return false;
        }
        // Check if at least one 6-face-neighbor within thickness is outside inner shape
        let t = self.thickness as i32;
        for d in 1..=t {
            if !self.inner.contains(x + d, y, z)
                || !self.inner.contains(x - d, y, z)
                || !self.inner.contains(x, y + d, z)
                || !self.inner.contains(x, y - d, z)
                || !self.inner.contains(x, y, z + d)
                || !self.inner.contains(x, y, z - d)
            {
                return true;
            }
        }
        false
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = Vec::new();
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        self.inner.normal_at(x, y, z)
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        self.inner.bounds()
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
