use crate::building::{
    BezierCurve, BilinearGradientBrush, BlockPalette, BrushEnum, BuildingTool, ColorBrush, Cone,
    Cuboid, CurveGradientBrush, Cylinder, Difference, Disk, Ellipsoid, Hollow, InterpolationSpace,
    Intersection, Line, LinearGradientBrush, Plane, PointGradientBrush, Pyramid, ShadedBrush,
    ShapeEnum, SolidBrush, Sphere, Torus, Triangle, Union,
};
use crate::wasm::schematic::SchematicWrapper;
use crate::BlockState;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

// ============================================================================
// Shapes
// ============================================================================

/// A wrapper for any shape (Sphere, Cuboid, Line, etc.)
#[wasm_bindgen]
pub struct ShapeWrapper {
    pub(crate) inner: ShapeEnum,
}

#[wasm_bindgen]
impl ShapeWrapper {
    /// Create a new Sphere shape
    pub fn sphere(cx: i32, cy: i32, cz: i32, radius: f64) -> Self {
        Self {
            inner: ShapeEnum::Sphere(Sphere::new((cx, cy, cz), radius)),
        }
    }

    /// Create a new Cuboid shape
    pub fn cuboid(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> Self {
        Self {
            inner: ShapeEnum::Cuboid(Cuboid::new((min_x, min_y, min_z), (max_x, max_y, max_z))),
        }
    }

    /// Create an Ellipsoid shape
    pub fn ellipsoid(cx: i32, cy: i32, cz: i32, rx: f64, ry: f64, rz: f64) -> Self {
        Self {
            inner: ShapeEnum::Ellipsoid(Ellipsoid::new((cx, cy, cz), (rx, ry, rz))),
        }
    }

    /// Create a Cylinder with arbitrary axis
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
    #[wasm_bindgen(js_name = cylinderBetween)]
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
    pub fn disk(
        cx: f64,
        cy: f64,
        cz: f64,
        radius: f64,
        nx: f64,
        ny: f64,
        nz: f64,
        thickness: Option<f64>,
    ) -> Self {
        Self {
            inner: ShapeEnum::Disk(Disk::new(
                (cx, cy, cz),
                radius,
                (nx, ny, nz),
                thickness.unwrap_or(1.0),
            )),
        }
    }

    /// Create a bounded Plane shape
    #[wasm_bindgen(js_name = plane)]
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
        thickness: Option<f64>,
    ) -> Self {
        Self {
            inner: ShapeEnum::Plane(Plane::new(
                (ox, oy, oz),
                (ux, uy, uz),
                (vx, vy, vz),
                u_ext,
                v_ext,
                thickness.unwrap_or(1.0),
            )),
        }
    }

    /// Create a filled Triangle shape
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
        thickness: Option<f64>,
    ) -> Self {
        Self {
            inner: ShapeEnum::Triangle(Triangle::new(
                (ax, ay, az),
                (bx, by, bz),
                (cx, cy, cz),
                thickness.unwrap_or(1.0),
            )),
        }
    }

    /// Create a 3D Line with thickness
    pub fn line(
        x1: f64,
        y1: f64,
        z1: f64,
        x2: f64,
        y2: f64,
        z2: f64,
        thickness: Option<f64>,
    ) -> Self {
        Self {
            inner: ShapeEnum::Line(Line::new(
                (x1, y1, z1),
                (x2, y2, z2),
                thickness.unwrap_or(1.0),
            )),
        }
    }

    /// Create a Bezier curve with thickness
    pub fn bezier(
        control_points_flat: Vec<f64>,
        thickness: Option<f64>,
        resolution: Option<u32>,
    ) -> Result<ShapeWrapper, JsValue> {
        if control_points_flat.len() % 3 != 0 {
            return Err(JsValue::from_str(
                "control_points_flat must have length divisible by 3",
            ));
        }
        let points: Vec<(f64, f64, f64)> = control_points_flat
            .chunks(3)
            .map(|c| (c[0], c[1], c[2]))
            .collect();
        Ok(Self {
            inner: ShapeEnum::BezierCurve(BezierCurve::new(
                points,
                thickness.unwrap_or(1.0),
                resolution.unwrap_or(32),
            )),
        })
    }

    /// Create a Hollow version of any shape
    pub fn hollow(shape: &ShapeWrapper, thickness: Option<u32>) -> Self {
        Self {
            inner: ShapeEnum::Hollow(Hollow::new(shape.inner.clone(), thickness.unwrap_or(1))),
        }
    }

    /// Create a Union of two shapes (a OR b)
    #[wasm_bindgen(js_name = union)]
    pub fn union_shapes(a: &ShapeWrapper, b: &ShapeWrapper) -> Self {
        Self {
            inner: ShapeEnum::Union(Union::new(a.inner.clone(), b.inner.clone())),
        }
    }

    /// Create an Intersection of two shapes (a AND b)
    pub fn intersection(a: &ShapeWrapper, b: &ShapeWrapper) -> Self {
        Self {
            inner: ShapeEnum::Intersection(Intersection::new(a.inner.clone(), b.inner.clone())),
        }
    }

    /// Create a Difference of two shapes (a AND NOT b)
    pub fn difference(a: &ShapeWrapper, b: &ShapeWrapper) -> Self {
        Self {
            inner: ShapeEnum::Difference(Difference::new(a.inner.clone(), b.inner.clone())),
        }
    }
}

// ============================================================================
// Brushes
// ============================================================================

/// A wrapper for any brush (Solid, Gradient, Shaded, CurveGradient, etc.)
#[wasm_bindgen]
pub struct BrushWrapper {
    pub(crate) inner: BrushEnum,
}

#[wasm_bindgen]
impl BrushWrapper {
    /// Create a solid brush with a specific block
    pub fn solid(block_state: &str) -> Result<BrushWrapper, JsValue> {
        let block = if block_state.contains('[') {
            BlockState::new(block_state.to_string())
        } else {
            BlockState::new(block_state.to_string())
        };

        Ok(Self {
            inner: BrushEnum::Solid(SolidBrush::new(block)),
        })
    }

    /// Create a color brush (matches closest block to RGB color)
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

    /// Create a shaded brush (Lambertian shading)
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

    /// Create a bilinear gradient brush (4-corner quad)
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

    /// Create a point cloud gradient brush using Inverse Distance Weighting (IDW)
    pub fn point_gradient(
        positions: Vec<i32>,
        colors: Vec<u8>,
        falloff: Option<f64>,
        space: Option<u8>,
        palette_filter: Option<Vec<String>>,
    ) -> Result<BrushWrapper, JsValue> {
        if positions.len() % 3 != 0
            || colors.len() % 3 != 0
            || positions.len() / 3 != colors.len() / 3
        {
            return Err(JsValue::from_str(
                "Positions and colors arrays must match in length (3 components per point)",
            ));
        }

        let count = positions.len() / 3;
        let mut points = Vec::with_capacity(count);

        for i in 0..count {
            points.push((
                (positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]),
                (colors[i * 3], colors[i * 3 + 1], colors[i * 3 + 2]),
            ));
        }

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

        Ok(Self {
            inner: BrushEnum::Point(brush),
        })
    }

    /// Create a curve gradient brush that interpolates colors along parametric shapes.
    /// stops_flat: Flat array [pos1, r1, g1, b1, pos2, r2, g2, b2, ...]
    /// Space: 0 = RGB, 1 = Oklab
    #[wasm_bindgen(js_name = curveGradient)]
    pub fn curve_gradient(
        stops_flat: Vec<f64>,
        space: Option<u8>,
        palette_filter: Option<Vec<String>>,
    ) -> Result<BrushWrapper, JsValue> {
        if stops_flat.len() % 4 != 0 {
            return Err(JsValue::from_str(
                "stops_flat must have length divisible by 4 (position, r, g, b per stop)",
            ));
        }

        let stops: Vec<(f64, (u8, u8, u8))> = stops_flat
            .chunks(4)
            .map(|c| (c[0], (c[1] as u8, c[2] as u8, c[3] as u8)))
            .collect();

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

        Ok(Self {
            inner: BrushEnum::CurveGradient(brush),
        })
    }
}

// ============================================================================
// Tool
// ============================================================================

#[wasm_bindgen]
pub struct WasmBuildingTool;

#[wasm_bindgen]
impl WasmBuildingTool {
    /// Apply a brush to a shape on the given schematic
    pub fn fill(schematic: &mut SchematicWrapper, shape: &ShapeWrapper, brush: &BrushWrapper) {
        let mut tool = BuildingTool::new(&mut schematic.0);
        tool.fill_enum(&shape.inner, &brush.inner);
    }

    /// Repeat a shape+brush fill at regular offset intervals
    pub fn rstack(
        schematic: &mut SchematicWrapper,
        shape: &ShapeWrapper,
        brush: &BrushWrapper,
        count: usize,
        offset_x: i32,
        offset_y: i32,
        offset_z: i32,
    ) {
        let mut tool = BuildingTool::new(&mut schematic.0);
        tool.rstack(
            &shape.inner,
            &brush.inner,
            count,
            (offset_x, offset_y, offset_z),
        );
    }
}
