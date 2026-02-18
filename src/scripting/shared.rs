use crate::building::{BuildingTool, Cuboid, SolidBrush, Sphere};
use crate::formats::manager::get_manager;
use crate::BlockState;
use crate::UniversalSchematic;

/// Shared scripting schematic wrapper. All schematic logic lives here —
/// both Lua and JS engines are thin adapters that delegate to these methods.
#[derive(Debug)]
pub struct ScriptingSchematic {
    pub inner: UniversalSchematic,
}

impl ScriptingSchematic {
    // -- Lifecycle --

    pub fn new(name: Option<String>) -> Self {
        Self {
            inner: UniversalSchematic::new(name.unwrap_or_else(|| "Unnamed".to_string())),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, String> {
        let data = std::fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
        let manager = get_manager();
        let manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
        let schematic = manager
            .read(&data)
            .map_err(|e| format!("Failed to parse schematic: {}", e))?;
        Ok(Self { inner: schematic })
    }

    // -- Metadata --

    pub fn get_name(&self) -> String {
        self.inner
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "Unnamed".to_string())
    }

    pub fn set_name(&mut self, name: &str) {
        self.inner.metadata.name = Some(name.to_string());
    }

    pub fn get_author(&self) -> String {
        self.inner.metadata.author.clone().unwrap_or_default()
    }

    pub fn set_author(&mut self, author: &str) {
        self.inner.metadata.author = Some(author.to_string());
    }

    pub fn get_description(&self) -> String {
        self.inner.metadata.description.clone().unwrap_or_default()
    }

    pub fn set_description(&mut self, desc: &str) {
        self.inner.metadata.description = Some(desc.to_string());
    }

    // -- Blocks --

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, name: &str) {
        self.inner.set_block_str(x, y, z, name);
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<String> {
        self.inner.get_block(x, y, z).map(|b| b.to_string())
    }

    // -- Building --

    pub fn fill_cuboid(&mut self, min: (i32, i32, i32), max: (i32, i32, i32), block_name: &str) {
        let shape = Cuboid::new(min, max);
        let brush = SolidBrush::new(BlockState::new(block_name.to_string()));
        let mut tool = BuildingTool::new(&mut self.inner);
        tool.fill(&shape, &brush);
    }

    pub fn fill_sphere(&mut self, center: (i32, i32, i32), radius: f64, block_name: &str) {
        let shape = Sphere::new(center, radius);
        let brush = SolidBrush::new(BlockState::new(block_name.to_string()));
        let mut tool = BuildingTool::new(&mut self.inner);
        tool.fill(&shape, &brush);
    }

    // -- Info --

    pub fn get_dimensions(&self) -> (i32, i32, i32) {
        self.inner.get_dimensions()
    }

    pub fn get_block_count(&self) -> i32 {
        self.inner.total_blocks()
    }

    pub fn get_volume(&self) -> i32 {
        self.inner.total_volume()
    }

    // -- Transforms --

    pub fn flip_x(&mut self) {
        self.inner.flip_x();
    }

    pub fn flip_y(&mut self) {
        self.inner.flip_y();
    }

    pub fn flip_z(&mut self) {
        self.inner.flip_z();
    }

    pub fn rotate_x(&mut self, degrees: i32) {
        self.inner.rotate_x(degrees);
    }

    pub fn rotate_y(&mut self, degrees: i32) {
        self.inner.rotate_y(degrees);
    }

    pub fn rotate_z(&mut self, degrees: i32) {
        self.inner.rotate_z(degrees);
    }

    // -- Export --

    pub fn to_schematic(&self) -> Result<Vec<u8>, String> {
        let manager = get_manager();
        let manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
        manager
            .write("schematic", &self.inner, None)
            .map_err(|e| format!("Export error: {}", e))
    }

    pub fn to_litematic(&self) -> Result<Vec<u8>, String> {
        let manager = get_manager();
        let manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
        manager
            .write("litematic", &self.inner, None)
            .map_err(|e| format!("Export error: {}", e))
    }

    pub fn save_as(&self, format: &str) -> Result<Vec<u8>, String> {
        let manager = get_manager();
        let manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
        manager
            .write(format, &self.inner, None)
            .map_err(|e| format!("Export error: {}", e))
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let manager = get_manager();
        let manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
        let data = manager
            .write_auto(path, &self.inner, None)
            .map_err(|e| format!("Export error: {}", e))?;
        std::fs::write(path, data).map_err(|e| format!("Failed to write file: {}", e))
    }

    // -- Iteration --

    pub fn get_all_blocks(&self) -> Vec<(i32, i32, i32, String)> {
        self.inner
            .iter_blocks()
            .filter(|(_, bs)| !bs.name.contains("air"))
            .map(|(pos, bs)| (pos.x, pos.y, pos.z, bs.to_string()))
            .collect()
    }

    pub fn get_region_names(&self) -> Vec<String> {
        self.inner.get_region_names()
    }
}
