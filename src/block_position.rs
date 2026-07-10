use serde::{Deserialize, Serialize};
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

// Core implementation available to all users of the library
impl BlockPosition {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        BlockPosition { x, y, z }
    }

    pub fn to_tuple(&self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }

    pub fn from_tuple(tuple: (i32, i32, i32)) -> Self {
        BlockPosition::new(tuple.0, tuple.1, tuple.2)
    }
}
