use crate::building::{
    BezierCurve, BilinearGradientBrush, BlockPalette, BrushEnum, BuildingTool, ColorBrush, Cone,
    Cuboid, CurveGradientBrush, Cylinder, Difference, Disk, Ellipsoid, Hollow, InterpolationSpace,
    Intersection, Line, LinearGradientBrush, Plane, PointGradientBrush, Pyramid, ShadedBrush,
    ShapeEnum, SolidBrush, Sphere, Torus, Triangle, Union,
};
use crate::python::schematic::PySchematic;
use crate::BlockState;
use pyo3::prelude::*;
use std::sync::Arc;

// ============================================================================
// Shapes
// ============================================================================

/// A wrapper for any shape (Sphere, Cuboid, Line, etc.)
#[pyclass(name = "Shape")]
pub struct PyShape {
    pub(crate) inner: ShapeEnum,
}

#[pymethods]
impl PyShape {
    /// Create a new Sphere shape
    #[staticmethod]
    pub fn sphere(cx: i32, cy: i32, cz: i32, radius: f64) -> Self {
        Self {
            inner: ShapeEnum::Sphere(Sphere::new((cx, cy, cz), radius)),
        }
    }

    /// Create a new Cuboid shape
    #[staticmethod]
    pub fn cuboid(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> Self {
        Self {
            inner: ShapeEnum::Cuboid(Cuboid::new((min_x, min_y, min_z), (max_x, max_y, max_z))),
        }
    }

    /// Create an Ellipsoid shape
    #[staticmethod]
    pub fn ellipsoid(cx: i32, cy: i32, cz: i32, rx: f64, ry: f64, rz: f64) -> Self {
        Self {
            inner: ShapeEnum::Ellipsoid(Ellipsoid::new((cx, cy, cz), (rx, ry, rz))),
        }
    }

    /// Create a Cylinder with arbitrary axis
    #[staticmethod]
    pub fn cylinder(
        bx: f64,
        by: f64,
        bz: f64,
        ax: f64,
        ay: f64,
        az: f64,
        radius: f64,
        height: f64,
    ) -> Self {
        Self {
            inner: ShapeEnum::Cylinder(Cylinder::new((bx, by, bz), (ax, ay, az), radius, height)),
        }
    }

    /// Create a Cylinder between two points
    #[staticmethod]
    pub fn cylinder_between(
        x1: f64,
        y1: f64,
        z1: f64,
        x2: f64,
        y2: f64,
        z2: f64,
        radius: f64,
    ) -> Self {
        Self {
            inner: ShapeEnum::Cylinder(Cylinder::between((x1, y1, z1), (x2, y2, z2), radius)),
        }
    }

    /// Create a Cone shape
    #[staticmethod]
    pub fn cone(
        ax: f64,
        ay: f64,
        az: f64,
        dx: f64,
        dy: f64,
        dz: f64,
        radius: f64,
        height: f64,
    ) -> Self {
        Self {
            inner: ShapeEnum::Cone(Cone::new((ax, ay, az), (dx, dy, dz), radius, height)),
        }
    }

    /// Create a Torus shape
    #[staticmethod]
    pub fn torus(
        cx: f64,
        cy: f64,
        cz: f64,
        major_r: f64,
        minor_r: f64,
        ax: f64,
        ay: f64,
        az: f64,
    ) -> Self {
        Self {
            inner: ShapeEnum::Torus(Torus::new((cx, cy, cz), major_r, minor_r, (ax, ay, az))),
        }
    }

    /// Create a Pyramid shape
    #[staticmethod]
    pub fn pyramid(
        bx: f64,
        by: f64,
        bz: f64,
        half_w: f64,
        half_d: f64,
        height: f64,
        ax: f64,
        ay: f64,
        az: f64,
    ) -> Self {
        Self {
            inner: ShapeEnum::Pyramid(Pyramid::new(
                (bx, by, bz),
                (half_w, half_d),
                height,
                (ax, ay, az),
            )),
        }
    }

    /// Create a Disk shape (flat cylinder)
    #[staticmethod]
    #[pyo3(signature = (cx, cy, cz, radius, nx, ny, nz, thickness=1.0))]
    pub fn disk(
        cx: f64,
        cy: f64,
        cz: f64,
        radius: f64,
        nx: f64,
        ny: f64,
        nz: f64,
        thickness: f64,
    ) -> Self {
        Self {
            inner: ShapeEnum::Disk(Disk::new((cx, cy, cz), radius, (nx, ny, nz), thickness)),
        }
    }

    /// Create a bounded Plane shape
    #[staticmethod]
    #[pyo3(signature = (ox, oy, oz, ux, uy, uz, vx, vy, vz, u_ext, v_ext, thickness=1.0))]
    pub fn plane(
        ox: f64,
        oy: f64,
        oz: f64,
        ux: f64,
        uy: f64,
        uz: f64,
        vx: f64,
        vy: f64,
        vz: f64,
        u_ext: f64,
        v_ext: f64,
        thickness: f64,
    ) -> Self {
        Self {
            inner: ShapeEnum::Plane(Plane::new(
                (ox, oy, oz),
                (ux, uy, uz),
                (vx, vy, vz),
                u_ext,
                v_ext,
                thickness,
            )),
        }
    }

    /// Create a filled Triangle shape
    #[staticmethod]
    #[pyo3(signature = (ax, ay, az, bx, by, bz, cx, cy, cz, thickness=1.0))]
    pub fn triangle(
        ax: f64,
        ay: f64,
        az: f64,
        bx: f64,
        by: f64,
        bz: f64,
        cx: f64,
        cy: f64,
        cz: f64,
        thickness: f64,
    ) -> Self {
        Self {
            inner: ShapeEnum::Triangle(Triangle::new(
                (ax, ay, az),
                (bx, by, bz),
                (cx, cy, cz),
                thickness,
            )),
        }
    }

    /// Create a 3D Line with thickness
    #[staticmethod]
    #[pyo3(signature = (x1, y1, z1, x2, y2, z2, thickness=1.0))]
    pub fn line(x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64, thickness: f64) -> Self {
        Self {
            inner: ShapeEnum::Line(Line::new((x1, y1, z1), (x2, y2, z2), thickness)),
        }
    }

    /// Create a Bezier curve with thickness
    #[staticmethod]
    #[pyo3(signature = (control_points, thickness=1.0, resolution=32))]
    pub fn bezier(control_points: Vec<(f64, f64, f64)>, thickness: f64, resolution: u32) -> Self {
        Self {
            inner: ShapeEnum::BezierCurve(BezierCurve::new(control_points, thickness, resolution)),
        }
    }

    /// Create a Hollow version of any shape
    #[staticmethod]
    #[pyo3(signature = (shape, thickness=1))]
    pub fn hollow(shape: &PyShape, thickness: u32) -> Self {
        Self {
            inner: ShapeEnum::Hollow(Hollow::new(shape.inner.clone(), thickness)),
        }
    }

    /// Create a Union of two shapes (a OR b)
    #[staticmethod]
    pub fn union(a: &PyShape, b: &PyShape) -> Self {
        Self {
            inner: ShapeEnum::Union(Union::new(a.inner.clone(), b.inner.clone())),
        }
    }

    /// Create an Intersection of two shapes (a AND b)
    #[staticmethod]
    pub fn intersection(a: &PyShape, b: &PyShape) -> Self {
        Self {
            inner: ShapeEnum::Intersection(Intersection::new(a.inner.clone(), b.inner.clone())),
        }
    }

    /// Create a Difference of two shapes (a AND NOT b)
    #[staticmethod]
    pub fn difference(a: &PyShape, b: &PyShape) -> Self {
        Self {
            inner: ShapeEnum::Difference(Difference::new(a.inner.clone(), b.inner.clone())),
        }
    }
}

// ============================================================================
// Brushes
// ============================================================================

/// A wrapper for any brush (Solid, Gradient, Shaded, CurveGradient, etc.)
#[pyclass(name = "Brush")]
pub struct PyBrush {
    pub(crate) inner: BrushEnum,
}

#[pymethods]
impl PyBrush {
    /// Create a solid brush with a specific block
    #[staticmethod]
    pub fn solid(block_state: &str) -> PyResult<Self> {
        let block = BlockState::new(block_state.to_string());
        Ok(Self {
            inner: BrushEnum::Solid(SolidBrush::new(block)),
        })
    }

    /// Create a color brush (matches closest block to RGB color)
    #[staticmethod]
    pub fn color(r: u8, g: u8, b: u8, palette_filter: Option<Vec<String>>) -> Self {
        let brush = if let Some(keywords) = palette_filter {
            let palette = Arc::new(BlockPalette::new_filtered(|f| {
                keywords.iter().any(|k| f.id.contains(k))
            }));
            ColorBrush::with_palette(r, g, b, palette)
        } else {
            ColorBrush::new(r, g, b)
        };

        Self {
            inner: BrushEnum::Color(brush),
        }
    }

    /// Create a linear gradient brush
    /// Space: 0 = RGB, 1 = Oklab
    #[staticmethod]
    pub fn linear_gradient(
        x1: i32,
        y1: i32,
        z1: i32,
        r1: u8,
        g1: u8,
        b1: u8,
        x2: i32,
        y2: i32,
        z2: i32,
        r2: u8,
        g2: u8,
        b2: u8,
        space: Option<u8>,
        palette_filter: Option<Vec<String>>,
    ) -> Self {
        let interp_space = match space {
            Some(1) => InterpolationSpace::Oklab,
            _ => InterpolationSpace::Rgb,
        };

        let mut brush =
            LinearGradientBrush::new((x1, y1, z1), (r1, g1, b1), (x2, y2, z2), (r2, g2, b2))
                .with_space(interp_space);

        if let Some(keywords) = palette_filter {
            let palette = Arc::new(BlockPalette::new_filtered(|f| {
                keywords.iter().any(|k| f.id.contains(k))
            }));
            brush = brush.with_palette(palette);
        }

        Self {
            inner: BrushEnum::Linear(brush),
        }
    }

    /// Create a bilinear gradient brush (4-corner quad)
    #[staticmethod]
    pub fn bilinear_gradient(
        ox: i32,
        oy: i32,
        oz: i32,
        ux: i32,
        uy: i32,
        uz: i32,
        vx: i32,
        vy: i32,
        vz: i32,
        r00: u8,
        g00: u8,
        b00: u8,
        r10: u8,
        g10: u8,
        b10: u8,
        r01: u8,
        g01: u8,
        b01: u8,
        r11: u8,
        g11: u8,
        b11: u8,
        space: Option<u8>,
        palette_filter: Option<Vec<String>>,
    ) -> Self {
        let interp_space = match space {
            Some(1) => InterpolationSpace::Oklab,
            _ => InterpolationSpace::Rgb,
        };

        let mut brush = BilinearGradientBrush::new(
            (ox, oy, oz),
            (ux, uy, uz),
            (vx, vy, vz),
            (r00, g00, b00),
            (r10, g10, b10),
            (r01, g01, b01),
            (r11, g11, b11),
        )
        .with_space(interp_space);

        if let Some(keywords) = palette_filter {
            let palette = Arc::new(BlockPalette::new_filtered(|f| {
                keywords.iter().any(|k| f.id.contains(k))
            }));
            brush = brush.with_palette(palette);
        }

        Self {
            inner: BrushEnum::Bilinear(brush),
        }
    }

    /// Create a shaded brush (Lambertian shading)
    /// light_dir: [x, y, z] vector
    #[staticmethod]
    pub fn shaded(
        r: u8,
        g: u8,
        b: u8,
        lx: f64,
        ly: f64,
        lz: f64,
        palette_filter: Option<Vec<String>>,
    ) -> Self {
        let mut brush = ShadedBrush::new((r, g, b), (lx, ly, lz));

        if let Some(keywords) = palette_filter {
            let palette = Arc::new(BlockPalette::new_filtered(|f| {
                keywords.iter().any(|k| f.id.contains(k))
            }));
            brush = brush.with_palette(palette);
        }

        Self {
            inner: BrushEnum::Shaded(brush),
        }
    }

    /// Create a point cloud gradient brush using Inverse Distance Weighting (IDW)
    /// points: List of ((x, y, z), (r, g, b)) tuples
    /// falloff: Power parameter (default 2.0 if None)
    #[staticmethod]
    pub fn point_gradient(
        points: Vec<((i32, i32, i32), (u8, u8, u8))>,
        falloff: Option<f64>,
        space: Option<u8>,
        palette_filter: Option<Vec<String>>,
    ) -> Self {
        let interp_space = match space {
            Some(1) => InterpolationSpace::Oklab,
            _ => InterpolationSpace::Rgb,
        };

        let mut brush = PointGradientBrush::new(points)
            .with_space(interp_space)
            .with_falloff(falloff.unwrap_or(2.0));

        if let Some(keywords) = palette_filter {
            let palette = Arc::new(BlockPalette::new_filtered(|f| {
                keywords.iter().any(|k| f.id.contains(k))
            }));
            brush = brush.with_palette(palette);
        }

        Self {
            inner: BrushEnum::Point(brush),
        }
    }

    /// Create a curve gradient brush that interpolates colors along parametric shapes.
    /// stops: List of (position, (r, g, b)) where position is 0.0-1.0
    /// Space: 0 = RGB, 1 = Oklab
    #[staticmethod]
    pub fn curve_gradient(
        stops: Vec<(f64, (u8, u8, u8))>,
        space: Option<u8>,
        palette_filter: Option<Vec<String>>,
    ) -> Self {
        let interp_space = match space {
            Some(1) => InterpolationSpace::Oklab,
            _ => InterpolationSpace::Rgb,
        };

        let mut brush = CurveGradientBrush::new(stops).with_space(interp_space);

        if let Some(keywords) = palette_filter {
            let palette = Arc::new(BlockPalette::new_filtered(|f| {
                keywords.iter().any(|k| f.id.contains(k))
            }));
            brush = brush.with_palette(palette);
        }

        Self {
            inner: BrushEnum::CurveGradient(brush),
        }
    }
}

// ============================================================================
// Tool
// ============================================================================

#[pyclass(name = "BuildingTool")]
pub struct PyBuildingTool;

#[pymethods]
impl PyBuildingTool {
    #[staticmethod]
    pub fn fill(schematic: &mut PySchematic, shape: &PyShape, brush: &PyBrush) {
        let mut tool = BuildingTool::new(&mut schematic.inner);
        tool.fill_enum(&shape.inner, &brush.inner);
    }

    /// Repeat a shape+brush fill at regular offset intervals
    #[staticmethod]
    pub fn rstack(
        schematic: &mut PySchematic,
        shape: &PyShape,
        brush: &PyBrush,
        count: usize,
        offset_x: i32,
        offset_y: i32,
        offset_z: i32,
    ) {
        let mut tool = BuildingTool::new(&mut schematic.inner);
        tool.rstack(
            &shape.inner,
            &brush.inner,
            count,
            (offset_x, offset_y, offset_z),
        );
    }
}
