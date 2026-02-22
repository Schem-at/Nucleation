pub mod brushes;
pub mod enums;
pub mod shapes;

pub use brushes::*;
pub use enums::*;
pub use shapes::*;

use crate::universal_schematic::UniversalSchematic;

pub struct BuildingTool<'a> {
    schematic: &'a mut UniversalSchematic,
}

impl<'a> BuildingTool<'a> {
    pub fn new(schematic: &'a mut UniversalSchematic) -> Self {
        Self { schematic }
    }

    /// Fill a shape with a brush using generic trait objects (pure Rust API).
    pub fn fill(&mut self, shape: &impl Shape, brush: &impl Brush) {
        let (min_x, min_y, min_z, max_x, max_y, max_z) = shape.bounds();
        self.schematic
            .ensure_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));

        shape.for_each_point(|x, y, z| {
            let normal = shape.normal_at(x, y, z);
            if let Some(block) = brush.get_block(x, y, z, normal) {
                self.schematic.set_block(x, y, z, &block);
            }
        });
    }

    /// Fill a ShapeEnum with a BrushEnum, passing parametric `t` to brushes that support it.
    pub fn fill_enum(&mut self, shape: &ShapeEnum, brush: &BrushEnum) {
        let (min_x, min_y, min_z, max_x, max_y, max_z) = shape.bounds();
        self.schematic
            .ensure_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));

        shape.for_each_point(|x, y, z| {
            let normal = shape.normal_at(x, y, z);
            let t = shape.parameter_at(x, y, z);
            if let Some(block) = brush.get_block_with_parameter(x, y, z, normal, t) {
                self.schematic.set_block(x, y, z, &block);
            }
        });
    }

    /// Repeat a shape+brush fill at regular offset intervals.
    /// Creates `count` copies of the shape, each offset by `offset * i` from the original.
    pub fn rstack(
        &mut self,
        shape: &ShapeEnum,
        brush: &BrushEnum,
        count: usize,
        offset: (i32, i32, i32),
    ) {
        for i in 0..count {
            let dx = offset.0 * i as i32;
            let dy = offset.1 * i as i32;
            let dz = offset.2 * i as i32;
            let translated = TranslatedShape::new(shape, dx, dy, dz);
            let (min_x, min_y, min_z, max_x, max_y, max_z) = translated.bounds();
            self.schematic
                .ensure_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));

            translated.for_each_point(|x, y, z| {
                let normal = translated.normal_at(x, y, z);
                let t = shape.parameter_at(x - dx, y - dy, z - dz);
                if let Some(block) = brush.get_block_with_parameter(x, y, z, normal, t) {
                    self.schematic.set_block(x, y, z, &block);
                }
            });
        }
    }
}

/// Internal helper that wraps a ShapeEnum with a translation offset.
struct TranslatedShape<'a> {
    inner: &'a ShapeEnum,
    dx: i32,
    dy: i32,
    dz: i32,
}

impl<'a> TranslatedShape<'a> {
    fn new(inner: &'a ShapeEnum, dx: i32, dy: i32, dz: i32) -> Self {
        Self { inner, dx, dy, dz }
    }
}

impl Shape for TranslatedShape<'_> {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        self.inner.contains(x - self.dx, y - self.dy, z - self.dz)
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        self.inner
            .points()
            .into_iter()
            .map(|(x, y, z)| (x + self.dx, y + self.dy, z + self.dz))
            .collect()
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        self.inner.normal_at(x - self.dx, y - self.dy, z - self.dz)
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        let (min_x, min_y, min_z, max_x, max_y, max_z) = self.inner.bounds();
        (
            min_x + self.dx,
            min_y + self.dy,
            min_z + self.dz,
            max_x + self.dx,
            max_y + self.dy,
            max_z + self.dz,
        )
    }

    fn for_each_point<F>(&self, mut f: F)
    where
        F: FnMut(i32, i32, i32),
    {
        let dx = self.dx;
        let dy = self.dy;
        let dz = self.dz;
        self.inner.for_each_point(|x, y, z| {
            f(x + dx, y + dy, z + dz);
        });
    }
}
