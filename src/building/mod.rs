pub mod brushes;
pub mod shapes;

pub use brushes::*;
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
        for (x, y, z) in shape.points() {
            let normal = shape.normal_at(x, y, z);
            if let Some(block) = brush.get_block(x, y, z, normal) {
                self.schematic.set_block(x, y, z, block);
            }
        }
    }
}
