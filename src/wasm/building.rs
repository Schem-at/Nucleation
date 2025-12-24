use crate::building::{
    BilinearGradientBrush, BlockPalette, BrushEnum, BuildingTool, ColorBrush, Cuboid,
    InterpolationSpace, LinearGradientBrush, PointGradientBrush, ShadedBrush, ShapeEnum,
    SolidBrush, Sphere,
};
use crate::wasm::schematic::SchematicWrapper;
use crate::BlockState;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

// ============================================================================
// Shapes
// ============================================================================

/// A wrapper for any shape (Sphere, Cuboid, etc.)
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
}

// ============================================================================
// Brushes
// ============================================================================

/// A wrapper for any brush (Solid, Gradient, Shaded, etc.)
#[wasm_bindgen]
pub struct BrushWrapper {
    pub(crate) inner: BrushEnum,
}

#[wasm_bindgen]
impl BrushWrapper {
    /// Create a solid brush with a specific block
    pub fn solid(block_state: &str) -> Result<BrushWrapper, JsValue> {
        // We need to parse properties if they exist
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
    /// Palette: optional list of block IDs to restrict matching to.
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
    /// light_dir: [x, y, z] vector
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
    /// positions: Flat array [x1, y1, z1, x2, y2, z2, ...]
    /// colors: Flat array [r1, g1, b1, r2, g2, b2, ...]
    /// falloff: Power parameter (default 2.0 if None)
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
        tool.fill(&shape.inner, &brush.inner);
    }
}
