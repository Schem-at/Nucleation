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

    pub fn fill(&mut self, shape: &impl Shape, brush: &impl Brush) {
        // Optimization: Ensure bounds are large enough before filling
        let (min_x, min_y, min_z, max_x, max_y, max_z) = shape.bounds();
        self.schematic
            .ensure_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));

        shape.for_each_point(|x, y, z| {
            let normal = shape.normal_at(x, y, z);
            if let Some(block) = brush.get_block(x, y, z, normal) {
                // Since we pre-expanded, this call should be faster
                self.schematic.set_block(x, y, z, &block);
            }
        });
    }
}
