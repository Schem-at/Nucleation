use super::Shape;
use crate::building::enums::ShapeEnum;

#[derive(Clone)]
pub struct Union {
    pub a: Box<ShapeEnum>,
    pub b: Box<ShapeEnum>,
}

impl Union {
    pub fn new(a: ShapeEnum, b: ShapeEnum) -> Self {
        Self {
            a: Box::new(a),
            b: Box::new(b),
        }
    }
}

impl Shape for Union {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        self.a.contains(x, y, z) || self.b.contains(x, y, z)
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = Vec::new();
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        if self.a.contains(x, y, z) {
            self.a.normal_at(x, y, z)
        } else {
            self.b.normal_at(x, y, z)
        }
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        let (a0, a1, a2, a3, a4, a5) = self.a.bounds();
        let (b0, b1, b2, b3, b4, b5) = self.b.bounds();
        (
            a0.min(b0),
            a1.min(b1),
            a2.min(b2),
            a3.max(b3),
            a4.max(b4),
            a5.max(b5),
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

#[derive(Clone)]
pub struct Intersection {
    pub a: Box<ShapeEnum>,
    pub b: Box<ShapeEnum>,
}

impl Intersection {
    pub fn new(a: ShapeEnum, b: ShapeEnum) -> Self {
        Self {
            a: Box::new(a),
            b: Box::new(b),
        }
    }
}

impl Shape for Intersection {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        self.a.contains(x, y, z) && self.b.contains(x, y, z)
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = Vec::new();
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        self.a.normal_at(x, y, z)
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        let (a0, a1, a2, a3, a4, a5) = self.a.bounds();
        let (b0, b1, b2, b3, b4, b5) = self.b.bounds();
        (
            a0.max(b0),
            a1.max(b1),
            a2.max(b2),
            a3.min(b3),
            a4.min(b4),
            a5.min(b5),
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

#[derive(Clone)]
pub struct Difference {
    pub a: Box<ShapeEnum>,
    pub b: Box<ShapeEnum>,
}

impl Difference {
    pub fn new(a: ShapeEnum, b: ShapeEnum) -> Self {
        Self {
            a: Box::new(a),
            b: Box::new(b),
        }
    }
}

impl Shape for Difference {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        self.a.contains(x, y, z) && !self.b.contains(x, y, z)
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = Vec::new();
        self.for_each_point(|x, y, z| points.push((x, y, z)));
        points
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        self.a.normal_at(x, y, z)
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        self.a.bounds()
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
