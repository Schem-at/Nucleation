use super::Shape;

#[derive(Clone)]
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
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, _x: i32, _y: i32, _z: i32) -> (f64, f64, f64) {
        (0.0, 1.0, 0.0) // Simplified normal
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        (
            self.min.0, self.min.1, self.min.2, self.max.0, self.max.1, self.max.2,
        )
    }

    fn for_each_point<F>(&self, mut f: F)
    where
        F: FnMut(i32, i32, i32),
    {
        for x in self.min.0..=self.max.0 {
            for y in self.min.1..=self.max.1 {
                for z in self.min.2..=self.max.2 {
                    f(x, y, z);
                }
            }
        }
    }
}
