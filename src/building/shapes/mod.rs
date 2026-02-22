mod bezier;
mod composite;
mod cone;
mod cuboid;
mod cylinder;
mod disk;
mod ellipsoid;
mod hollow;
mod line;
mod plane;
mod pyramid;
mod sphere;
mod torus;
mod triangle;

pub use bezier::BezierCurve;
pub use composite::{Difference, Intersection, Union};
pub use cone::Cone;
pub use cuboid::Cuboid;
pub use cylinder::Cylinder;
pub use disk::Disk;
pub use ellipsoid::Ellipsoid;
pub use hollow::Hollow;
pub use line::Line;
pub use plane::Plane;
pub use pyramid::Pyramid;
pub use sphere::Sphere;
pub use torus::Torus;
pub use triangle::Triangle;

pub trait Shape {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool;
    fn points(&self) -> Vec<(i32, i32, i32)>;
    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64);
    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32);
    fn for_each_point<F>(&self, f: F)
    where
        F: FnMut(i32, i32, i32);
}

pub trait ParametricShape: Shape {
    /// Returns parametric position t in [0, 1] for the given voxel.
    fn parameter_at(&self, x: i32, y: i32, z: i32) -> f64;
}
