use crate::building::{
    BilinearGradientBrush, BlockPalette, BrushEnum, BuildingTool, ColorBrush, Cuboid,
    InterpolationSpace, LinearGradientBrush, PointGradientBrush, ShadedBrush, ShapeEnum,
    SolidBrush, Sphere,
};
use crate::python::schematic::PySchematic;
use crate::BlockState;
use pyo3::prelude::*;
use std::sync::Arc;

// ============================================================================
// Shapes
// ============================================================================

/// A wrapper for any shape (Sphere, Cuboid, etc.)
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
}

// ============================================================================
// Brushes
// ============================================================================

/// A wrapper for any brush (Solid, Gradient, Shaded, etc.)
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
        tool.fill(&shape.inner, &brush.inner);
    }
}
