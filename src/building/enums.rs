use crate::building::{
    BilinearGradientBrush, Brush, ColorBrush, Cuboid, LinearGradientBrush, MultiPointGradientBrush,
    PointGradientBrush, ShadedBrush, Shape, SolidBrush, Sphere,
};
use crate::BlockState;

// ============================================================================
// Shapes
// ============================================================================

#[derive(Clone)]
pub enum ShapeEnum {
    Sphere(Sphere),
    Cuboid(Cuboid),
}

impl Shape for ShapeEnum {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        match self {
            ShapeEnum::Sphere(s) => s.contains(x, y, z),
            ShapeEnum::Cuboid(c) => c.contains(x, y, z),
        }
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        match self {
            ShapeEnum::Sphere(s) => s.points(),
            ShapeEnum::Cuboid(c) => c.points(),
        }
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        match self {
            ShapeEnum::Sphere(s) => s.normal_at(x, y, z),
            ShapeEnum::Cuboid(c) => c.normal_at(x, y, z),
        }
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        match self {
            ShapeEnum::Sphere(s) => s.bounds(),
            ShapeEnum::Cuboid(c) => c.bounds(),
        }
    }

    fn for_each_point<F>(&self, f: F)
    where
        F: FnMut(i32, i32, i32),
    {
        match self {
            ShapeEnum::Sphere(s) => s.for_each_point(f),
            ShapeEnum::Cuboid(c) => c.for_each_point(f),
        }
    }
}

// ============================================================================
// Brushes
// ============================================================================

#[derive(Clone)]
pub enum BrushEnum {
    Solid(SolidBrush),
    Color(ColorBrush),
    Linear(LinearGradientBrush),
    Bilinear(BilinearGradientBrush),
    Point(PointGradientBrush),
    MultiPoint(MultiPointGradientBrush),
    Shaded(ShadedBrush),
}

impl Brush for BrushEnum {
    fn get_block(&self, x: i32, y: i32, z: i32, normal: (f64, f64, f64)) -> Option<BlockState> {
        match self {
            BrushEnum::Solid(b) => b.get_block(x, y, z, normal),
            BrushEnum::Color(b) => b.get_block(x, y, z, normal),
            BrushEnum::Linear(b) => b.get_block(x, y, z, normal),
            BrushEnum::Bilinear(b) => b.get_block(x, y, z, normal),
            BrushEnum::Point(b) => b.get_block(x, y, z, normal),
            BrushEnum::MultiPoint(b) => b.get_block(x, y, z, normal),
            BrushEnum::Shaded(b) => b.get_block(x, y, z, normal),
        }
    }
}
