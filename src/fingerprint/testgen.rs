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

/// Apply `k` deterministic edits (alternating add / remove / change) to a copy.
/// Returns (edited schematic, exact counts (adds, removes, changes)).
pub fn edited(src: &UniversalSchematic, k: usize) -> (UniversalSchematic, (usize, usize, usize)) {
    let mut s = UniversalSchematic::new("gen".to_string());
    // Only real (non-air) blocks are edit targets — `iter_blocks` includes the
    // region's air-padding cells, which must not be treated as build cells.
    let cells: Vec<(i32, i32, i32)> = src
        .iter_blocks()
        .filter(|(_, b)| b.get_name() != "minecraft:air")
        .map(|(p, _)| (p.x, p.y, p.z))
        .collect();
    for (p, b) in src.iter_blocks() {
        s.set_block(p.x, p.y, p.z, b);
    }
    let air = BlockState::new("minecraft:air");
    let other = BlockState::new("minecraft:glass");
    let (mut adds, mut removes, mut changes) = (0, 0, 0);
    let max_x = cells.iter().map(|c| c.0).max().unwrap_or(0);
    for i in 0..k {
        match i % 3 {
            0 => {
                s.set_block(max_x + 2 + i as i32, 0, 0, &other);
                adds += 1;
            }
            1 => {
                if let Some(c) = cells.get(i) {
                    s.set_block(c.0, c.1, c.2, &air);
                    removes += 1;
                }
            }
            _ => {
                if let Some(c) = cells.get(i) {
                    // Distinct target per change so they don't look like one
                    // consistent palette swap (which the diff would collapse).
                    let blk = if changes % 2 == 0 {
                        BlockState::new("minecraft:glass")
                    } else {
                        BlockState::new("minecraft:sand")
                    };
                    s.set_block(c.0, c.1, c.2, &blk);
                    changes += 1;
                }
            }
        }
    }
    (s, (adds, removes, changes))
}

/// Replace every block named `from` with `to` (a global palette swap).
pub fn repalette(src: &UniversalSchematic, from: &str, to: &str) -> UniversalSchematic {
    let mut s = UniversalSchematic::new("gen".to_string());
    let repl = BlockState::new(to);
    for (p, b) in src.iter_blocks() {
        if b.get_name() == from {
            s.set_block(p.x, p.y, p.z, &repl);
        } else {
            s.set_block(p.x, p.y, p.z, b);
        }
    }
    s
}
