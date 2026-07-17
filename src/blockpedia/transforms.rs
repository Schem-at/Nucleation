use crate::blockpedia::{BlockState, BlockpediaError, Result, BLOCKS};
use std::collections::HashMap;

/// Block transformation operations for rotation, material variants, and shape modifications
pub struct BlockTransforms;

/// Represents a direction in Minecraft
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

/// Represents a rotation amount
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    /// No rotation (0 degrees)
    None,
    /// 90 degrees clockwise
    Clockwise90,
    /// 180 degrees
    Half,
    /// 270 degrees clockwise (90 counter-clockwise)
    Clockwise270,
}

/// Represents different block shapes/variants
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockShape {
    /// Full block (default)
    Full,
    /// Stair variant
    Stairs,
    /// Slab variant (half block)
    Slab,
    /// Wall variant
    Wall,
    /// Fence variant
    Fence,
    /// Fence gate variant
    FenceGate,
    /// Door variant
    Door,
    /// Trapdoor variant
    Trapdoor,
    /// Button variant
    Button,
    /// Pressure plate variant
    PressurePlate,
}

impl Direction {
    /// Parse direction from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "north" => Some(Direction::North),
            "south" => Some(Direction::South),
            "east" => Some(Direction::East),
            "west" => Some(Direction::West),
            "up" => Some(Direction::Up),
            "down" => Some(Direction::Down),
            _ => None,
        }
    }

    /// Convert direction to string
    pub fn to_string(&self) -> &'static str {
        match self {
            Direction::North => "north",
            Direction::South => "south",
            Direction::East => "east",
            Direction::West => "west",
            Direction::Up => "up",
            Direction::Down => "down",
        }
    }

    /// Rotate direction clockwise by 90 degrees
    pub fn rotate_clockwise(self) -> Self {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
            Direction::Up => Direction::Up,
            Direction::Down => Direction::Down,
        }
    }

    /// Apply rotation to direction
    pub fn apply_rotation(self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => self,
            Rotation::Clockwise90 => self.rotate_clockwise(),
            Rotation::Half => self.rotate_clockwise().rotate_clockwise(),
            Rotation::Clockwise270 => self
                .rotate_clockwise()
                .rotate_clockwise()
                .rotate_clockwise(),
        }
    }

    /// Get opposite direction
    pub fn opposite(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }
}

impl BlockTransforms {
    /// Rotate a block state by the specified rotation
    pub fn rotate_block(block_state: &BlockState, rotation: Rotation) -> Result<BlockState> {
        if rotation == Rotation::None {
            return Ok(block_state.clone());
        }

        let block_id = block_state.to_string();
        let (base_id, properties) = Self::parse_block_state(&block_id)?;

        let mut new_properties = properties.clone();

        // Handle directional properties
        if let Some(facing) = properties.get("facing") {
            if let Some(direction) = Direction::from_str(facing) {
                let new_direction = direction.apply_rotation(rotation);
                new_properties.insert("facing".to_string(), new_direction.to_string().to_string());
            }
        }

        // Handle axis property (for logs, pillars, etc.)
        if let Some(axis) = properties.get("axis") {
            let new_axis = match (axis.as_str(), rotation) {
                ("x", Rotation::Clockwise90) => "z",
                ("z", Rotation::Clockwise90) => "x",
                ("x", Rotation::Clockwise270) => "z",
                ("z", Rotation::Clockwise270) => "x",
                ("x", Rotation::Half) => "x",
                ("z", Rotation::Half) => "z",
                ("y", _) => "y", // Y axis doesn't rotate horizontally
                _ => axis,
            };
            new_properties.insert("axis".to_string(), new_axis.to_string());
        }

        // Handle shape property for stairs
        if let Some(shape) = properties.get("shape") {
            let new_shape = Self::rotate_stair_shape(shape, rotation);
            new_properties.insert("shape".to_string(), new_shape);
        }

        // Reconstruct block state
        Self::build_block_state(&base_id, &new_properties)
    }

    /// Get material variant of a block (e.g., oak_stairs -> stone_stairs)
    pub fn get_material_variant(
        block_state: &BlockState,
        target_material: &str,
    ) -> Result<BlockState> {
        let block_id = block_state.to_string();
        let (base_id, properties) = Self::parse_block_state(&block_id)?;

        // Extract the shape/type from the block ID
        let shape = Self::extract_block_shape(&base_id)?;

        // Build new block ID with target material
        let new_base_id = match shape {
            BlockShape::Full => format!("minecraft:{}", target_material),
            BlockShape::Stairs => format!("minecraft:{}_stairs", target_material),
            BlockShape::Slab => format!("minecraft:{}_slab", target_material),
            BlockShape::Wall => format!("minecraft:{}_wall", target_material),
            BlockShape::Fence => format!("minecraft:{}_fence", target_material),
            BlockShape::FenceGate => format!("minecraft:{}_fence_gate", target_material),
            BlockShape::Door => format!("minecraft:{}_door", target_material),
            BlockShape::Trapdoor => format!("minecraft:{}_trapdoor", target_material),
            BlockShape::Button => format!("minecraft:{}_button", target_material),
            BlockShape::PressurePlate => format!("minecraft:{}_pressure_plate", target_material),
        };

        // Check if the target block exists
        if !BLOCKS.contains_key(&new_base_id) {
            return Err(BlockpediaError::block_not_found(&new_base_id));
        }

        // Get the target block's valid properties
        let target_block = BLOCKS.get(&new_base_id).unwrap();
        let mut valid_properties = HashMap::new();

        // Only keep properties that are valid for the target block
        for (key, value) in &properties {
            if target_block.has_property(key) {
                if let Some(valid_values) = target_block.get_property_values(key) {
                    if valid_values.contains(value) {
                        valid_properties.insert(key.clone(), value.clone());
                    } else {
                        // Try to use a sensible default
                        if let Some(default_value) = valid_values.first() {
                            valid_properties.insert(key.clone(), default_value.clone());
                        }
                    }
                }
            }
        }

        Self::build_block_state(&new_base_id, &valid_properties)
    }

    /// Get shape variant of a block (e.g., stone -> stone_stairs)
    pub fn get_shape_variant(
        block_state: &BlockState,
        target_shape: BlockShape,
    ) -> Result<BlockState> {
        let block_id = block_state.to_string();
        let (base_id, properties) = Self::parse_block_state(&block_id)?;

        // Extract the material from the block ID
        let material = Self::extract_material(&base_id)?;

        // Build new block ID with target shape
        let new_base_id = match target_shape {
            BlockShape::Full => format!("minecraft:{}", material),
            BlockShape::Stairs => format!("minecraft:{}_stairs", material),
            BlockShape::Slab => format!("minecraft:{}_slab", material),
            BlockShape::Wall => format!("minecraft:{}_wall", material),
            BlockShape::Fence => format!("minecraft:{}_fence", material),
            BlockShape::FenceGate => format!("minecraft:{}_fence_gate", material),
            BlockShape::Door => format!("minecraft:{}_door", material),
            BlockShape::Trapdoor => format!("minecraft:{}_trapdoor", material),
            BlockShape::Button => format!("minecraft:{}_button", material),
            BlockShape::PressurePlate => format!("minecraft:{}_pressure_plate", material),
        };

        // Check if the target block exists
        if !BLOCKS.contains_key(&new_base_id) {
            return Err(BlockpediaError::block_not_found(&new_base_id));
        }

        // Build with default properties for the new shape
        let target_block = BLOCKS.get(&new_base_id).unwrap();
        let mut new_properties = HashMap::new();

        // Add appropriate default properties for the shape
        match target_shape {
            BlockShape::Stairs => {
                new_properties.insert("facing".to_string(), "north".to_string());
                new_properties.insert("half".to_string(), "bottom".to_string());
                new_properties.insert("shape".to_string(), "straight".to_string());
            }
            BlockShape::Slab => {
                new_properties.insert("type".to_string(), "bottom".to_string());
            }
            BlockShape::Door => {
                new_properties.insert("facing".to_string(), "north".to_string());
                new_properties.insert("half".to_string(), "lower".to_string());
                new_properties.insert("hinge".to_string(), "left".to_string());
                new_properties.insert("open".to_string(), "false".to_string());
                new_properties.insert("powered".to_string(), "false".to_string());
            }
            BlockShape::Trapdoor => {
                new_properties.insert("facing".to_string(), "north".to_string());
                new_properties.insert("half".to_string(), "bottom".to_string());
                new_properties.insert("open".to_string(), "false".to_string());
                new_properties.insert("powered".to_string(), "false".to_string());
                new_properties.insert("waterlogged".to_string(), "false".to_string());
            }
            _ => {
                // For other shapes, use the block's default state
                for (key, value) in target_block.default_state {
                    new_properties.insert(key.to_string(), value.to_string());
                }
            }
        }

        // Try to preserve compatible properties from the original block
        for (key, value) in &properties {
            if target_block.has_property(key) {
                if let Some(valid_values) = target_block.get_property_values(key) {
                    if valid_values.contains(value) {
                        new_properties.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        Self::build_block_state(&new_base_id, &new_properties)
    }

    /// Find all available material variants for a given block shape
    pub fn find_material_variants(block_state: &BlockState) -> Result<Vec<String>> {
        let block_id = block_state.to_string();
        let (base_id, _) = Self::parse_block_state(&block_id)?;

        let shape = Self::extract_block_shape(&base_id)?;
        let mut variants = Vec::new();

        // Search for all blocks with the same shape
        for block_id in BLOCKS.keys() {
            if let Ok(block_shape) = Self::extract_block_shape(block_id) {
                if block_shape == shape {
                    if let Ok(material) = Self::extract_material(block_id) {
                        variants.push(material);
                    }
                }
            }
        }

        variants.sort();
        variants.dedup();
        Ok(variants)
    }

    /// Find all available shape variants for a given material
    pub fn find_shape_variants(block_state: &BlockState) -> Result<Vec<BlockShape>> {
        let block_id = block_state.to_string();
        let (base_id, _) = Self::parse_block_state(&block_id)?;

        let material = Self::extract_material(&base_id)?;
        let mut variants = Vec::new();

        // Search for all shapes with this material
        for block_id in BLOCKS.keys() {
            if let Ok(block_material) = Self::extract_material(block_id) {
                if block_material == material {
                    if let Ok(shape) = Self::extract_block_shape(block_id) {
                        variants.push(shape);
                    }
                }
            }
        }

        variants.sort_by_key(|shape| format!("{:?}", shape));
        variants.dedup();
        Ok(variants)
    }

    // Helper methods

    fn parse_block_state(block_id: &str) -> Result<(String, HashMap<String, String>)> {
        if let Some(bracket_pos) = block_id.find('[') {
            let base_id = block_id[..bracket_pos].to_string();
            let properties_str = &block_id[bracket_pos + 1..];

            if !properties_str.ends_with(']') {
                return Err(BlockpediaError::parse_failed(
                    block_id,
                    "missing closing bracket",
                ));
            }

            let properties_str = &properties_str[..properties_str.len() - 1];
            let mut properties = HashMap::new();

            if !properties_str.is_empty() {
                for prop_pair in properties_str.split(',') {
                    let parts: Vec<&str> = prop_pair.split('=').collect();
                    if parts.len() == 2 {
                        properties.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
                    }
                }
            }

            Ok((base_id, properties))
        } else {
            Ok((block_id.to_string(), HashMap::new()))
        }
    }

    fn build_block_state(
        base_id: &str,
        properties: &HashMap<String, String>,
    ) -> Result<BlockState> {
        let mut state = BlockState::new(base_id)?;

        for (key, value) in properties {
            state = state.with(key, value)?;
        }

        Ok(state)
    }

    fn extract_block_shape(block_id: &str) -> Result<BlockShape> {
        let id = block_id.strip_prefix("minecraft:").unwrap_or(block_id);

        if id.ends_with("_stairs") {
            Ok(BlockShape::Stairs)
        } else if id.ends_with("_slab") {
            Ok(BlockShape::Slab)
        } else if id.ends_with("_wall") {
            Ok(BlockShape::Wall)
        } else if id.ends_with("_fence_gate") {
            Ok(BlockShape::FenceGate)
        } else if id.ends_with("_fence") {
            Ok(BlockShape::Fence)
        } else if id.ends_with("_door") {
            Ok(BlockShape::Door)
        } else if id.ends_with("_trapdoor") {
            Ok(BlockShape::Trapdoor)
        } else if id.ends_with("_button") {
            Ok(BlockShape::Button)
        } else if id.ends_with("_pressure_plate") {
            Ok(BlockShape::PressurePlate)
        } else {
            Ok(BlockShape::Full)
        }
    }

    fn extract_material(block_id: &str) -> Result<String> {
        let id = block_id.strip_prefix("minecraft:").unwrap_or(block_id);

        // Remove common suffixes to get the base material
        let material = if id.ends_with("_stairs") {
            id.strip_suffix("_stairs").unwrap()
        } else if id.ends_with("_slab") {
            id.strip_suffix("_slab").unwrap()
        } else if id.ends_with("_wall") {
            id.strip_suffix("_wall").unwrap()
        } else if id.ends_with("_fence_gate") {
            id.strip_suffix("_fence_gate").unwrap()
        } else if id.ends_with("_fence") {
            id.strip_suffix("_fence").unwrap()
        } else if id.ends_with("_door") {
            id.strip_suffix("_door").unwrap()
        } else if id.ends_with("_trapdoor") {
            id.strip_suffix("_trapdoor").unwrap()
        } else if id.ends_with("_button") {
            id.strip_suffix("_button").unwrap()
        } else if id.ends_with("_pressure_plate") {
            id.strip_suffix("_pressure_plate").unwrap()
        } else if id.ends_with("_planks") {
            id.strip_suffix("_planks").unwrap()
        } else if id.ends_with("_log") {
            id.strip_suffix("_log").unwrap()
        } else if id.ends_with("_wood") {
            id.strip_suffix("_wood").unwrap()
        } else {
            id
        };

        Ok(material.to_string())
    }

    fn rotate_stair_shape(shape: &str, rotation: Rotation) -> String {
        match (shape, rotation) {
            ("straight", _) => "straight".to_string(),
            ("inner_left", Rotation::Clockwise90) => "inner_right".to_string(),
            ("inner_right", Rotation::Clockwise90) => "outer_right".to_string(),
            ("outer_right", Rotation::Clockwise90) => "outer_left".to_string(),
            ("outer_left", Rotation::Clockwise90) => "inner_left".to_string(),
            ("inner_left", Rotation::Half) => "outer_right".to_string(),
            ("inner_right", Rotation::Half) => "outer_left".to_string(),
            ("outer_right", Rotation::Half) => "inner_left".to_string(),
            ("outer_left", Rotation::Half) => "inner_right".to_string(),
            ("inner_left", Rotation::Clockwise270) => "outer_left".to_string(),
            ("inner_right", Rotation::Clockwise270) => "inner_left".to_string(),
            ("outer_right", Rotation::Clockwise270) => "inner_right".to_string(),
            ("outer_left", Rotation::Clockwise270) => "outer_right".to_string(),
            _ => shape.to_string(),
        }
    }
}

// Convenience functions for common operations
impl BlockState {
    /// Rotate this block state by 90 degrees clockwise
    pub fn rotate_clockwise(&self) -> Result<BlockState> {
        BlockTransforms::rotate_block(self, Rotation::Clockwise90)
    }

    /// Rotate this block state by 180 degrees
    pub fn rotate_180(&self) -> Result<BlockState> {
        BlockTransforms::rotate_block(self, Rotation::Half)
    }

    /// Rotate this block state by 270 degrees clockwise (90 counter-clockwise)
    pub fn rotate_counter_clockwise(&self) -> Result<BlockState> {
        BlockTransforms::rotate_block(self, Rotation::Clockwise270)
    }

    /// Get a material variant of this block (e.g., oak_stairs -> stone_stairs)
    pub fn with_material(&self, material: &str) -> Result<BlockState> {
        BlockTransforms::get_material_variant(self, material)
    }

    /// Get a shape variant of this block (e.g., stone -> stone_stairs)
    pub fn with_shape(&self, shape: BlockShape) -> Result<BlockState> {
        BlockTransforms::get_shape_variant(self, shape)
    }

    /// Find all available material variants for this block's shape
    pub fn available_materials(&self) -> Result<Vec<String>> {
        BlockTransforms::find_material_variants(self)
    }

    /// Find all available shape variants for this block's material
    pub fn available_shapes(&self) -> Result<Vec<BlockShape>> {
        BlockTransforms::find_shape_variants(self)
    }
}
