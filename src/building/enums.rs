use crate::building::{
    BezierCurve, BilinearGradientBrush, Brush, ColorBrush, Cone, Cuboid, CurveGradientBrush,
    Cylinder, Difference, Disk, Ellipsoid, Hollow, Intersection, Line, LinearGradientBrush,
    MultiPointGradientBrush, ParametricShape, Plane, PointGradientBrush, Pyramid, ShadedBrush,
    Shape, SolidBrush, Sphere, Torus, Triangle, Union,
};
use crate::BlockState;

// ============================================================================
// Delegate macro for ShapeEnum
// ============================================================================

macro_rules! delegate_shape {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            ShapeEnum::Sphere(s) => s.$method($($arg),*),
            ShapeEnum::Cuboid(s) => s.$method($($arg),*),
            ShapeEnum::Ellipsoid(s) => s.$method($($arg),*),
            ShapeEnum::Cylinder(s) => s.$method($($arg),*),
            ShapeEnum::Cone(s) => s.$method($($arg),*),
            ShapeEnum::Torus(s) => s.$method($($arg),*),
            ShapeEnum::Pyramid(s) => s.$method($($arg),*),
            ShapeEnum::Disk(s) => s.$method($($arg),*),
            ShapeEnum::Plane(s) => s.$method($($arg),*),
            ShapeEnum::Triangle(s) => s.$method($($arg),*),
            ShapeEnum::Line(s) => s.$method($($arg),*),
            ShapeEnum::BezierCurve(s) => s.$method($($arg),*),
            ShapeEnum::Hollow(s) => s.$method($($arg),*),
            ShapeEnum::Union(s) => s.$method($($arg),*),
            ShapeEnum::Intersection(s) => s.$method($($arg),*),
            ShapeEnum::Difference(s) => s.$method($($arg),*),
        }
    };
}

// ============================================================================
// Shapes
// ============================================================================

#[derive(Clone)]
pub enum ShapeEnum {
    Sphere(Sphere),
    Cuboid(Cuboid),
    Ellipsoid(Ellipsoid),
    Cylinder(Cylinder),
    Cone(Cone),
    Torus(Torus),
    Pyramid(Pyramid),
    Disk(Disk),
    Plane(Plane),
    Triangle(Triangle),
    Line(Line),
    BezierCurve(BezierCurve),
    Hollow(Hollow),
    Union(Union),
    Intersection(Intersection),
    Difference(Difference),
}

impl ShapeEnum {
    /// Returns parametric position t in [0, 1] for parametric shapes, None for non-parametric.
    pub fn parameter_at(&self, x: i32, y: i32, z: i32) -> Option<f64> {
        match self {
            Self::Line(s) => Some(s.parameter_at(x, y, z)),
            Self::Cylinder(s) => Some(s.parameter_at(x, y, z)),
            Self::Cone(s) => Some(s.parameter_at(x, y, z)),
            Self::Torus(s) => Some(s.parameter_at(x, y, z)),
            Self::Pyramid(s) => Some(s.parameter_at(x, y, z)),
            Self::BezierCurve(s) => Some(s.parameter_at(x, y, z)),
            Self::Hollow(s) => s.inner.parameter_at(x, y, z),
            Self::Union(s) => {
                if s.a.contains(x, y, z) {
                    s.a.parameter_at(x, y, z)
                } else {
                    s.b.parameter_at(x, y, z)
                }
            }
            Self::Intersection(s) => s.a.parameter_at(x, y, z),
            Self::Difference(s) => s.a.parameter_at(x, y, z),
            _ => None,
        }
    }
}

impl Shape for ShapeEnum {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        delegate_shape!(self, contains, x, y, z)
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        delegate_shape!(self, points)
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        delegate_shape!(self, normal_at, x, y, z)
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        delegate_shape!(self, bounds)
    }

    fn for_each_point<F>(&self, f: F)
    where
        F: FnMut(i32, i32, i32),
    {
        delegate_shape!(self, for_each_point, f)
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
    CurveGradient(CurveGradientBrush),
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
            BrushEnum::CurveGradient(b) => b.get_block(x, y, z, normal),
        }
    }
}

impl BrushEnum {
    /// Get a block using both spatial and parametric information.
    /// For CurveGradientBrush, the `t` parameter takes priority when available.
    pub fn get_block_with_parameter(
        &self,
        x: i32,
        y: i32,
        z: i32,
        normal: (f64, f64, f64),
        t: Option<f64>,
    ) -> Option<BlockState> {
        match self {
            BrushEnum::CurveGradient(b) => b.get_block_parametric(x, y, z, normal, t),
            _ => self.get_block(x, y, z, normal),
        }
    }
}
