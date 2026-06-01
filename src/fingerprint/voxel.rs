//! Dense occupancy grid from a tokenized build (shared by the footprint and the
//! phase-2 diff aligner). Normalized to the build's min corner.

use crate::fingerprint::FingerprintSpec;
use crate::universal_schematic::UniversalSchematic;

pub struct Grid {
    pub dims: [usize; 3], // x, y, z
    pub data: Vec<f32>,   // row-major, x-fastest; 1.0 = occupied
}

pub fn occupancy_grid(schem: &UniversalSchematic, spec: &FingerprintSpec) -> Grid {
    let cells: Vec<(i32, i32, i32)> = schem
        .iter_blocks()
        .filter(|(_, b)| spec.blocks.tokenize(b).is_some())
        .map(|(p, _)| (p.x, p.y, p.z))
        .collect();
    if cells.is_empty() {
        return Grid {
            dims: [0, 0, 0],
            data: vec![],
        };
    }
    let mn = cells.iter().fold((i32::MAX, i32::MAX, i32::MAX), |m, p| {
        (m.0.min(p.0), m.1.min(p.1), m.2.min(p.2))
    });
    let mx = cells.iter().fold((i32::MIN, i32::MIN, i32::MIN), |m, p| {
        (m.0.max(p.0), m.1.max(p.1), m.2.max(p.2))
    });
    let dims = [
        (mx.0 - mn.0 + 1) as usize,
        (mx.1 - mn.1 + 1) as usize,
        (mx.2 - mn.2 + 1) as usize,
    ];
    let mut data = vec![0.0f32; dims[0] * dims[1] * dims[2]];
    for (x, y, z) in cells {
        let (i, j, k) = (
            (x - mn.0) as usize,
            (y - mn.1) as usize,
            (z - mn.2) as usize,
        );
        data[i + dims[0] * (j + dims[1] * k)] = 1.0;
    }
    Grid { dims, data }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::testgen::filled_box;
    use crate::fingerprint::FingerprintSpec;

    #[test]
    fn voxelizes_occupancy() {
        let s = filled_box((5, 5, 5), (6, 5, 5), "minecraft:stone"); // 2 cells
        let grid = occupancy_grid(&s, &FingerprintSpec::structural());
        assert_eq!(grid.dims, [2, 1, 1]);
        assert_eq!(grid.data.iter().filter(|&&v| v != 0.0).count(), 2);
    }
}
