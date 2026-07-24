mod bezier;
mod composite;
mod cone;
mod cuboid;
mod curve;
mod cylinder;
mod disk;
mod ellipsoid;
mod hollow;
mod line;
mod plane;
mod polygon_prism;
mod pyramid;
mod sdf_shape;
mod sphere;
mod torus;
mod triangle;

pub use bezier::BezierCurve;
pub use composite::{Difference, Intersection, Union};
pub use cone::Cone;
pub use cuboid::Cuboid;
pub use curve::{Curve3D, TubePath};
pub use cylinder::Cylinder;
pub use disk::Disk;
pub use ellipsoid::Ellipsoid;
pub use hollow::Hollow;
pub use line::Line;
pub use plane::Plane;
pub use polygon_prism::PolygonPrism;
pub use pyramid::Pyramid;
pub use sdf_shape::SdfShape;
pub use sphere::Sphere;
pub use torus::Torus;
pub use triangle::Triangle;

// The mesh shape lives in the voxelize module (building must not hard-depend
// on the gltf/image stack); re-exported here so it sits beside its peers.
#[cfg(feature = "voxelize")]
pub use crate::voxelize::MeshShape;

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

#[cfg(test)]
mod sampled_curve_tests {
    use super::{Curve3D, ParametricShape, Shape, TubePath};
    use std::collections::HashSet;

    #[test]
    fn closed_curve_includes_the_closing_segment_and_wraps_parameterisation() {
        let curve = Curve3D::new(
            vec![(0.0, 0.0, 0.0), (4.0, 0.0, 0.0), (4.0, 0.0, 4.0)],
            true,
        )
        .unwrap();
        let tube = TubePath::new(curve, 0.6).unwrap();

        assert!(tube.contains(2, 0, 2), "closing diagonal must be voxelised");
        let t = tube.parameter_at(2, 0, 2);
        assert!(t > 0.66 && t < 1.0, "closing segment t was {t}");
    }

    #[test]
    fn sampled_curve_rejects_too_few_points_and_non_positive_radius() {
        assert!(Curve3D::new(vec![(0.0, 0.0, 0.0)], false).is_err());
        let curve = Curve3D::new(vec![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0)], false).unwrap();
        assert!(TubePath::new(curve, 0.0).is_err());
    }

    #[test]
    fn tube_parameter_follows_arc_length_not_point_index() {
        let curve = Curve3D::new(
            vec![(0.0, 0.0, 0.0), (9.0, 0.0, 0.0), (10.0, 0.0, 0.0)],
            false,
        )
        .unwrap();
        let tube = TubePath::new(curve, 0.6).unwrap();

        assert!((tube.parameter_at(5, 0, 0) - 0.5).abs() < 1e-6);
        assert!((tube.parameter_at(9, 0, 0) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn thick_off_lattice_tube_enumerates_every_contained_voxel() {
        let curve = Curve3D::new(vec![(0.49, -0.37, 0.21), (10.73, 6.19, 4.81)], false).unwrap();
        let tube = TubePath::new(curve, 2.85).unwrap();
        let enumerated: HashSet<_> = tube.points().into_iter().collect();
        let (min_x, min_y, min_z, max_x, max_y, max_z) = tube.bounds();

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    if tube.contains(x, y, z) {
                        assert!(
                            enumerated.contains(&(x, y, z)),
                            "contained voxel ({x}, {y}, {z}) was not enumerated"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn sampled_curve_rejects_finite_inputs_that_overflow_or_exceed_voxel_work_budgets() {
        assert!(Curve3D::new(vec![(f64::MAX, 0.0, 0.0), (-f64::MAX, 0.0, 0.0)], false,).is_err());

        let ordinary = Curve3D::new(vec![(0.0, 0.0, 0.0), (10.0, 0.0, 0.0)], false).unwrap();
        assert!(TubePath::new(ordinary, f64::MAX).is_err());

        let enormous_span =
            Curve3D::new(vec![(0.0, 0.0, 0.0), (1_000_000_000.0, 0.0, 0.0)], false).unwrap();
        assert!(TubePath::new(enormous_span, 1.0).is_err());
    }
}
