//! Coarse occupancy grid and morphological dilation.
//!
//! Segmentation runs on a grid `1/C^3` the size of the tile, which is what
//! makes closing affordable over a whole world.

/// A sparse set of occupied cells of edge `cell_size`.
///
/// `BTreeSet` rather than a dense `Vec<bool>`: after substrate subtraction the
/// data is overwhelmingly empty, so a dense array over the tile volume would
/// scan ~1.5M cells per pass on a real region to find a handful of occupied
/// ones. The ordered set also gives sorted iteration for free, which is what
/// component labelling relies on to stay order-independent.
#[derive(Clone)]
pub struct OccupancyGrid {
    /// World coordinate of the low corner of cell `(0, 0, 0)`.
    origin: (i32, i32, i32),
    dims: (usize, usize, usize),
    cell_size: u32,
    occupied: std::collections::BTreeSet<(i32, i32, i32)>,
}

impl OccupancyGrid {
    pub fn new(origin: (i32, i32, i32), dims: (usize, usize, usize), cell_size: u32) -> Self {
        assert!(cell_size > 0, "cell_size must be positive");
        OccupancyGrid {
            origin,
            dims,
            cell_size,
            occupied: std::collections::BTreeSet::new(),
        }
    }

    pub fn dims(&self) -> (usize, usize, usize) {
        self.dims
    }

    pub fn cell_size(&self) -> u32 {
        self.cell_size
    }

    pub fn count(&self) -> usize {
        self.occupied.len()
    }

    /// Cell containing a world coordinate.
    ///
    /// Uses floor division: `-1 / 4` must be `-1`, not `0`. Rust's `/`
    /// truncates toward zero, which would fold the first negative cell into
    /// the first positive one and silently merge structures across the origin.
    pub fn cell_of(&self, x: i32, y: i32, z: i32) -> (i32, i32, i32) {
        let c = self.cell_size as i32;
        (
            (x - self.origin.0).div_euclid(c),
            (y - self.origin.1).div_euclid(c),
            (z - self.origin.2).div_euclid(c),
        )
    }

    fn in_bounds(&self, cell: (i32, i32, i32)) -> bool {
        cell.0 >= 0
            && cell.1 >= 0
            && cell.2 >= 0
            && (cell.0 as usize) < self.dims.0
            && (cell.1 as usize) < self.dims.1
            && (cell.2 as usize) < self.dims.2
    }

    /// Mark the cell containing this world coordinate. Out-of-range is ignored.
    pub fn mark(&mut self, x: i32, y: i32, z: i32) {
        let cell = self.cell_of(x, y, z);
        self.mark_cell(cell);
    }

    pub fn mark_cell(&mut self, cell: (i32, i32, i32)) {
        if self.in_bounds(cell) {
            self.occupied.insert(cell);
        }
    }

    pub fn is_occupied(&self, cell: (i32, i32, i32)) -> bool {
        self.occupied.contains(&cell)
    }

    /// Occupied cells in ascending `(x, y, z)` order.
    ///
    /// `BTreeSet` iteration is already sorted, so this is O(occupied) and the
    /// ordering guarantee costs nothing.
    pub fn occupied_cells(&self) -> impl Iterator<Item = (i32, i32, i32)> + '_ {
        self.occupied.iter().copied()
    }

    /// Chebyshev dilation by `radius` cells — a cube kernel.
    pub fn dilated(&self, radius: u32) -> OccupancyGrid {
        let mut out = OccupancyGrid::new(self.origin, self.dims, self.cell_size);
        if radius == 0 {
            out.occupied = self.occupied.clone();
            return out;
        }
        let r = radius as i32;
        for cell in &self.occupied {
            for dx in -r..=r {
                for dy in -r..=r {
                    for dz in -r..=r {
                        out.mark_cell((cell.0 + dx, cell.1 + dy, cell.2 + dz));
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> OccupancyGrid {
        // 64^3 blocks at cell size 4 -> 16^3 cells, origin at world (0,0,0).
        OccupancyGrid::new((0, 0, 0), (16, 16, 16), 4)
    }

    #[test]
    fn cell_of_floors_toward_negative_infinity() {
        let g = OccupancyGrid::new((0, 0, 0), (32, 32, 32), 4);
        assert_eq!(g.cell_of(0, 0, 0), (0, 0, 0));
        assert_eq!(g.cell_of(3, 3, 3), (0, 0, 0));
        assert_eq!(g.cell_of(4, 0, 0), (1, 0, 0));
        // -1 must land in cell -1, not cell 0: integer division truncates
        // toward zero and would be wrong here.
        assert_eq!(g.cell_of(-1, 0, 0), (-1, 0, 0));
        assert_eq!(g.cell_of(-4, 0, 0), (-1, 0, 0));
        assert_eq!(g.cell_of(-5, 0, 0), (-2, 0, 0));
    }

    #[test]
    fn marking_blocks_in_the_same_cell_yields_one_cell() {
        let mut g = grid();
        g.mark(0, 0, 0);
        g.mark(1, 1, 1);
        g.mark(3, 3, 3);
        assert_eq!(g.count(), 1);
        assert!(g.is_occupied((0, 0, 0)));
    }

    #[test]
    fn occupied_cells_are_sorted() {
        let mut g = grid();
        g.mark(40, 0, 0);
        g.mark(0, 0, 0);
        g.mark(20, 0, 0);
        let cells: Vec<_> = g.occupied_cells().collect();
        assert_eq!(cells, vec![(0, 0, 0), (5, 0, 0), (10, 0, 0)]);
    }

    #[test]
    fn dilate_by_one_grows_a_single_cell_into_a_3x3x3_cube() {
        let mut g = grid();
        g.mark(20, 20, 20); // cell (5,5,5)
        let d = g.dilated(1);
        assert_eq!(d.count(), 27);
        assert!(d.is_occupied((4, 4, 4)));
        assert!(d.is_occupied((6, 6, 6)));
        assert!(!d.is_occupied((3, 5, 5)));
    }

    #[test]
    fn dilate_by_zero_is_identity() {
        let mut g = grid();
        g.mark(20, 20, 20);
        assert_eq!(g.dilated(0).count(), 1);
    }

    #[test]
    fn dilation_clamps_at_grid_edges() {
        let mut g = grid();
        g.mark(0, 0, 0); // corner cell (0,0,0)
        let d = g.dilated(1);
        // Only the in-bounds octant survives: 2*2*2.
        assert_eq!(d.count(), 8);
    }
}
