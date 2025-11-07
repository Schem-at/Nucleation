use crate::BlockState;

/// Axis enum for transformations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

/// Direction mapping for Minecraft blocks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

impl Direction {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "north" => Some(Direction::North),
            "south" => Some(Direction::South),
            "east" => Some(Direction::East),
            "west" => Some(Direction::West),
            "up" => Some(Direction::Up),
            "down" => Some(Direction::Down),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Direction::North => "north",
            Direction::South => "south",
            Direction::East => "east",
            Direction::West => "west",
            Direction::Up => "up",
            Direction::Down => "down",
        }
    }

    /// Apply flip transformation to direction
    pub fn flip(&self, axis: Axis) -> Direction {
        match axis {
            Axis::X => match self {
                Direction::East => Direction::West,
                Direction::West => Direction::East,
                _ => *self,
            },
            Axis::Y => match self {
                Direction::Up => Direction::Down,
                Direction::Down => Direction::Up,
                _ => *self,
            },
            Axis::Z => match self {
                Direction::North => Direction::South,
                Direction::South => Direction::North,
                _ => *self,
            },
        }
    }

    /// Rotate direction around Y axis (horizontal plane)
    pub fn rotate_y(&self, degrees: i32) -> Direction {
        let rotations = ((degrees % 360 + 360) % 360) / 90;

        let mut current = *self;
        for _ in 0..rotations {
            current = match current {
                Direction::North => Direction::East,
                Direction::East => Direction::South,
                Direction::South => Direction::West,
                Direction::West => Direction::North,
                _ => current, // Up and Down don't change
            };
        }
        current
    }

    /// Rotate direction around X axis
    pub fn rotate_x(&self, degrees: i32) -> Direction {
        let rotations = ((degrees % 360 + 360) % 360) / 90;

        let mut current = *self;
        for _ in 0..rotations {
            current = match current {
                Direction::Up => Direction::South,
                Direction::South => Direction::Down,
                Direction::Down => Direction::North,
                Direction::North => Direction::Up,
                _ => current, // East and West don't change
            };
        }
        current
    }

    /// Rotate direction around Z axis
    pub fn rotate_z(&self, degrees: i32) -> Direction {
        let rotations = ((degrees % 360 + 360) % 360) / 90;

        let mut current = *self;
        for _ in 0..rotations {
            current = match current {
                Direction::Up => Direction::West,
                Direction::West => Direction::Down,
                Direction::Down => Direction::East,
                Direction::East => Direction::Up,
                _ => current, // North and South don't change
            };
        }
        current
    }
}

/// Transform block state properties based on flip operation
pub fn transform_block_state_flip(block: &BlockState, axis: Axis) -> BlockState {
    let mut new_block = block.clone();

    // Transform 'facing' property
    if let Some(facing) = block.get_property("facing") {
        if let Some(dir) = Direction::from_str(facing) {
            let new_dir = dir.flip(axis);
            new_block.set_property("facing".to_string(), new_dir.to_str().to_string());
        }
    }

    // Transform 'axis' property
    if let Some(axis_val) = block.get_property("axis") {
        let new_axis = match (axis, axis_val.as_str()) {
            (Axis::X, "x") => "x", // X flip doesn't change x axis
            (Axis::Y, "y") => "y", // Y flip doesn't change y axis
            (Axis::Z, "z") => "z", // Z flip doesn't change z axis
            _ => axis_val.as_str(),
        };
        new_block.set_property("axis".to_string(), new_axis.to_string());
    }

    // Transform 'rotation' property (0-15, used by standing signs, banners, etc.)
    if let Some(rotation) = block.get_property("rotation") {
        if let Ok(rot_val) = rotation.parse::<i32>() {
            let new_rotation = match axis {
                Axis::Y => rot_val, // Y flip doesn't change rotation around Y
                Axis::X | Axis::Z => {
                    // Mirror the rotation
                    (16 - rot_val) % 16
                }
            };
            new_block.set_property("rotation".to_string(), new_rotation.to_string());
        }
    }

    // Transform directional properties for specific blocks
    transform_special_block_properties(&mut new_block, axis, false, 0);

    new_block
}

/// Transform block state properties based on rotation operation
pub fn transform_block_state_rotate(block: &BlockState, axis: Axis, degrees: i32) -> BlockState {
    let mut new_block = block.clone();

    // Transform 'facing' property
    if let Some(facing) = block.get_property("facing") {
        if let Some(dir) = Direction::from_str(facing) {
            let new_dir = match axis {
                Axis::Y => dir.rotate_y(degrees),
                Axis::X => dir.rotate_x(degrees),
                Axis::Z => dir.rotate_z(degrees),
            };
            new_block.set_property("facing".to_string(), new_dir.to_str().to_string());
        }
    }

    // Transform 'axis' property (logs, pillars, etc.)
    if let Some(axis_val) = block.get_property("axis") {
        let new_axis = rotate_axis_property(axis_val.as_str(), axis, degrees);
        new_block.set_property("axis".to_string(), new_axis);
    }

    // Transform 'rotation' property (0-15, used by standing signs, banners, etc.)
    if let Some(rotation) = block.get_property("rotation") {
        if let Ok(rot_val) = rotation.parse::<i32>() {
            let rotations = ((degrees % 360 + 360) % 360) / 90;
            let new_rotation = match axis {
                Axis::Y => (rot_val + rotations * 4) % 16,
                Axis::X | Axis::Z => rot_val, // Rotation around these axes doesn't change standing rotation
            };
            new_block.set_property("rotation".to_string(), new_rotation.to_string());
        }
    }

    // Transform directional properties for specific blocks
    transform_special_block_properties(&mut new_block, axis, true, degrees);

    new_block
}

/// Rotate axis property value
fn rotate_axis_property(axis_val: &str, rotation_axis: Axis, degrees: i32) -> String {
    let rotations = ((degrees % 360 + 360) % 360) / 90;

    if rotations == 0 || rotations == 2 {
        // 0° or 180° rotation
        return axis_val.to_string();
    }

    // 90° or 270° rotation
    let result = match rotation_axis {
        Axis::Y => match axis_val {
            "x" => "z",
            "z" => "x",
            "y" => "y",
            _ => axis_val,
        },
        Axis::X => match axis_val {
            "y" => "z",
            "z" => "y",
            "x" => "x",
            _ => axis_val,
        },
        Axis::Z => match axis_val {
            "x" => "y",
            "y" => "x",
            "z" => "z",
            _ => axis_val,
        },
    };
    result.to_string()
}

/// Handle special block properties (stairs, slabs, redstone, etc.)
fn transform_special_block_properties(
    block: &mut BlockState,
    axis: Axis,
    is_rotation: bool,
    degrees: i32,
) {
    // Handle stair shapes
    if let Some(shape) = block.get_property("shape") {
        if is_rotation && axis == Axis::Y {
            let new_shape = rotate_stair_shape(shape.as_str(), degrees);
            block.set_property("shape".to_string(), new_shape);
        }
    }

    // Handle redstone wire connections
    let mut wire_updates: Vec<(&'static str, String)> = Vec::new();
    let mut wire_removals: Vec<&'static str> = Vec::new();

    for direction in &["north", "south", "east", "west"] {
        if let Some(connection) = block.get_property(direction) {
            let connection_value = connection.clone();
            if is_rotation && axis == Axis::Y {
                let dir = Direction::from_str(direction).unwrap();
                let new_dir = dir.rotate_y(degrees);
                // Move the connection value to the new direction
                wire_removals.push(direction);
                wire_updates.push((new_dir.to_str(), connection_value));
            } else if !is_rotation {
                let dir = Direction::from_str(direction).unwrap();
                let new_dir = dir.flip(axis);
                if new_dir.to_str() != *direction {
                    // Swap connection values
                    wire_removals.push(direction);
                    wire_updates.push((new_dir.to_str(), connection_value));
                }
            }
        }
    }

    // Apply updates and removals
    for dir in wire_removals {
        block.remove_property(dir);
    }
    for (dir, value) in wire_updates {
        block.set_property(dir.to_string(), value);
    }

    // Handle door hinges (left/right)
    if let Some(hinge) = block.get_property("hinge") {
        if !is_rotation && (axis == Axis::X || axis == Axis::Z) {
            let new_hinge = match hinge.as_str() {
                "left" => "right",
                "right" => "left",
                _ => hinge.as_str(),
            };
            block.set_property("hinge".to_string(), new_hinge.to_string());
        }
    }
}

/// Rotate stair shape
fn rotate_stair_shape(shape: &str, degrees: i32) -> String {
    let rotations = ((degrees % 360 + 360) % 360) / 90;

    if rotations == 0 || rotations == 2 {
        return shape.to_string();
    }

    // For 90° or 270° rotations
    let result = match shape {
        "inner_left" => "inner_right",
        "inner_right" => "inner_left",
        "outer_left" => "outer_right",
        "outer_right" => "outer_left",
        _ => shape,
    };
    result.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_flip_x() {
        assert_eq!(Direction::East.flip(Axis::X), Direction::West);
        assert_eq!(Direction::West.flip(Axis::X), Direction::East);
        assert_eq!(Direction::North.flip(Axis::X), Direction::North);
        assert_eq!(Direction::Up.flip(Axis::X), Direction::Up);
    }

    #[test]
    fn test_direction_flip_y() {
        assert_eq!(Direction::Up.flip(Axis::Y), Direction::Down);
        assert_eq!(Direction::Down.flip(Axis::Y), Direction::Up);
        assert_eq!(Direction::North.flip(Axis::Y), Direction::North);
    }

    #[test]
    fn test_direction_flip_z() {
        assert_eq!(Direction::North.flip(Axis::Z), Direction::South);
        assert_eq!(Direction::South.flip(Axis::Z), Direction::North);
        assert_eq!(Direction::East.flip(Axis::Z), Direction::East);
    }

    #[test]
    fn test_direction_rotate_y() {
        assert_eq!(Direction::North.rotate_y(90), Direction::East);
        assert_eq!(Direction::East.rotate_y(90), Direction::South);
        assert_eq!(Direction::South.rotate_y(90), Direction::West);
        assert_eq!(Direction::West.rotate_y(90), Direction::North);
        assert_eq!(Direction::North.rotate_y(180), Direction::South);
        assert_eq!(Direction::Up.rotate_y(90), Direction::Up);
    }

    #[test]
    fn test_transform_facing_flip() {
        let mut block = BlockState::new("minecraft:lever".to_string());
        block.set_property("facing".to_string(), "east".to_string());

        let transformed = transform_block_state_flip(&block, Axis::X);
        assert_eq!(
            transformed.get_property("facing"),
            Some(&"west".to_string())
        );
    }

    #[test]
    fn test_transform_facing_rotate() {
        let mut block = BlockState::new("minecraft:lever".to_string());
        block.set_property("facing".to_string(), "north".to_string());

        let transformed = transform_block_state_rotate(&block, Axis::Y, 90);
        assert_eq!(
            transformed.get_property("facing"),
            Some(&"east".to_string())
        );

        let transformed_180 = transform_block_state_rotate(&block, Axis::Y, 180);
        assert_eq!(
            transformed_180.get_property("facing"),
            Some(&"south".to_string())
        );
    }

    #[test]
    fn test_rotate_axis_property() {
        assert_eq!(rotate_axis_property("x", Axis::Y, 90), "z".to_string());
        assert_eq!(rotate_axis_property("z", Axis::Y, 90), "x".to_string());
        assert_eq!(rotate_axis_property("y", Axis::Y, 90), "y".to_string());
        assert_eq!(rotate_axis_property("x", Axis::Y, 180), "x".to_string());
    }

    #[test]
    fn test_rotation_property() {
        let mut block = BlockState::new("minecraft:standing_sign".to_string());
        block.set_property("rotation".to_string(), "0".to_string());

        let transformed = transform_block_state_rotate(&block, Axis::Y, 90);
        assert_eq!(transformed.get_property("rotation"), Some(&"4".to_string()));

        let transformed_180 = transform_block_state_rotate(&block, Axis::Y, 180);
        assert_eq!(
            transformed_180.get_property("rotation"),
            Some(&"8".to_string())
        );
    }
}
