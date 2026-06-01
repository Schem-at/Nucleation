//! Deterministic synthetic builds for fingerprint/footprint tests.
#![allow(dead_code)]

use crate::block_state::BlockState;
use crate::universal_schematic::UniversalSchematic;

pub fn filled_box(min: (i32, i32, i32), max: (i32, i32, i32), block: &str) -> UniversalSchematic {
    let mut s = UniversalSchematic::new("gen".to_string());
    let bs = BlockState::new(block);
    for x in min.0..=max.0 {
        for y in min.1..=max.1 {
            for z in min.2..=max.2 {
                s.set_block(x, y, z, &bs);
            }
        }
    }
    s
}

pub fn translated(src: &UniversalSchematic, by: (i32, i32, i32)) -> UniversalSchematic {
    let mut s = UniversalSchematic::new("gen".to_string());
    for (p, b) in src.iter_blocks() {
        s.set_block(p.x + by.0, p.y + by.1, p.z + by.2, b);
    }
    s
}

pub fn rotated_y(src: &UniversalSchematic, degrees: i32) -> UniversalSchematic {
    use crate::transforms::{transform_block_state_rotate, Axis};
    let d = ((degrees % 360) + 360) % 360;
    let mut s = UniversalSchematic::new("gen".to_string());
    for (p, b) in src.iter_blocks() {
        let (x, y, z) = (p.x, p.y, p.z);
        let np = match d {
            90 => (z, y, -x),
            180 => (-x, y, -z),
            270 => (-z, y, x),
            _ => (x, y, z),
        };
        let nb = transform_block_state_rotate(b, Axis::Y, d);
        s.set_block(np.0, np.1, np.2, &nb);
    }
    s
}
