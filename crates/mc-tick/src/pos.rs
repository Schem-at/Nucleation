//! Positions, directions, and the bounded region the simulation runs in.
//!
//! # Why the region is bounded
//!
//! The game's world is effectively infinite; a schematic is not. Simulating a
//! bounded region lets block storage be one dense `Vec` indexed by arithmetic,
//! which is what makes the tick loop fast and checkpoints trivial. The cost is
//! that behaviour at the boundary is not the game's behaviour — see
//! [`Bounds::contains`] and the note on out-of-bounds reads in
//! [`crate::world::World`].

/// A block position in world coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos {
    /// East/west.
    pub x: i32,
    /// Vertical.
    pub y: i32,
    /// North/south.
    pub z: i32,
}

impl Pos {
    /// A position.
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// This position offset one block in `dir`.
    pub const fn offset(self, dir: Dir) -> Self {
        let (dx, dy, dz) = dir.delta();
        Self {
            x: self.x + dx,
            y: self.y + dy,
            z: self.z + dz,
        }
    }
}

/// One of the six block faces.
///
/// The declaration order matches the game's `Direction` enum, because neighbour
/// update order is observable: a block that updates its neighbours does so in a
/// defined sequence, and redstone can tell the difference. Reordering this
/// changes simulation results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Dir {
    /// -Y
    Down = 0,
    /// +Y
    Up = 1,
    /// -Z
    North = 2,
    /// +Z
    South = 3,
    /// -X
    West = 4,
    /// +X
    East = 5,
}

/// Every direction, in the game's declaration order.
/// `NeighborUpdater.UPDATE_ORDER` — the order one `updateNeighborsAt` entry
/// dispatches its six notifications.
pub const UPDATE_ORDER: [Dir; 6] = [Dir::West, Dir::East, Dir::Down, Dir::Up, Dir::North, Dir::South];

/// `Block.UPDATE_SHAPE_ORDER` — the order a placed block receives its shape
/// updates in `updateFromNeighbourShapes`.
pub const UPDATE_SHAPE_ORDER: [Dir; 6] =
    [Dir::West, Dir::East, Dir::North, Dir::South, Dir::Down, Dir::Up];

/// `Direction.values()` — Java's enum order, where vanilla iterates it.
pub const JAVA_DIRECTIONS: [Dir; 6] =
    [Dir::Down, Dir::Up, Dir::North, Dir::South, Dir::West, Dir::East];

pub const ALL_DIRS: [Dir; 6] = [
    Dir::Down,
    Dir::Up,
    Dir::North,
    Dir::South,
    Dir::West,
    Dir::East,
];

impl Dir {
    /// The unit offset for this direction.
    pub const fn delta(self) -> (i32, i32, i32) {
        match self {
            Dir::Down => (0, -1, 0),
            Dir::Up => (0, 1, 0),
            Dir::North => (0, 0, -1),
            Dir::South => (0, 0, 1),
            Dir::West => (-1, 0, 0),
            Dir::East => (1, 0, 0),
        }
    }

    /// The direction facing the other way.
    pub const fn opposite(self) -> Dir {
        match self {
            Dir::Down => Dir::Up,
            Dir::Up => Dir::Down,
            Dir::North => Dir::South,
            Dir::South => Dir::North,
            Dir::West => Dir::East,
            Dir::East => Dir::West,
        }
    }
}

/// An inclusive axis-aligned region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds {
    /// Lowest corner, inclusive.
    pub min: Pos,
    /// Highest corner, inclusive.
    pub max: Pos,
}

impl Bounds {
    /// A region spanning `a` to `b` inclusive, in either order.
    pub fn new(a: Pos, b: Pos) -> Self {
        Self {
            min: Pos::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z)),
            max: Pos::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z)),
        }
    }

    /// Extent along each axis.
    pub fn size(&self) -> (u32, u32, u32) {
        (
            (self.max.x - self.min.x + 1) as u32,
            (self.max.y - self.min.y + 1) as u32,
            (self.max.z - self.min.z + 1) as u32,
        )
    }

    /// Number of blocks in the region.
    ///
    /// Returned as `u64` because a large region overflows `u32`, and silently
    /// wrapping here would corrupt every index derived from it.
    pub fn volume(&self) -> u64 {
        let (x, y, z) = self.size();
        u64::from(x) * u64::from(y) * u64::from(z)
    }

    /// Whether `pos` lies inside the region.
    pub fn contains(&self, pos: Pos) -> bool {
        pos.x >= self.min.x
            && pos.x <= self.max.x
            && pos.y >= self.min.y
            && pos.y <= self.max.y
            && pos.z >= self.min.z
            && pos.z <= self.max.z
    }

    /// The dense storage index for `pos`, or `None` if outside.
    ///
    /// Y-major then Z then X, matching the schematic formats this will be fed
    /// from, so bulk loads are sequential writes.
    pub fn index(&self, pos: Pos) -> Option<usize> {
        if !self.contains(pos) {
            return None;
        }
        let (sx, _sy, sz) = self.size();
        let dx = (pos.x - self.min.x) as usize;
        let dy = (pos.y - self.min.y) as usize;
        let dz = (pos.z - self.min.z) as usize;
        Some((dy * sz as usize + dz) * sx as usize + dx)
    }

    /// The position a storage index refers to. Inverse of [`Bounds::index`].
    pub fn position_of(&self, index: usize) -> Option<Pos> {
        if index as u64 >= self.volume() {
            return None;
        }
        let (sx, _sy, sz) = self.size();
        let (sx, sz) = (sx as usize, sz as usize);
        let dx = index % sx;
        let dz = (index / sx) % sz;
        let dy = index / (sx * sz);
        Some(Pos::new(
            self.min.x + dx as i32,
            self.min.y + dy as i32,
            self.min.z + dz as i32,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_deltas_and_opposites_agree() {
        for dir in ALL_DIRS {
            let there = Pos::new(0, 0, 0).offset(dir);
            let back = there.offset(dir.opposite());
            assert_eq!(back, Pos::new(0, 0, 0), "{dir:?} round trip");
        }
    }

    #[test]
    fn direction_order_matches_the_games_declaration_order() {
        // Neighbour update order is observable to redstone, so this order is a
        // behavioural contract, not a stylistic choice.
        assert_eq!(
            ALL_DIRS.map(|d| d as u8),
            [0, 1, 2, 3, 4, 5],
            "ALL_DIRS must follow the Dir discriminants"
        );
    }

    #[test]
    fn index_round_trips_for_every_position_in_a_region() {
        let bounds = Bounds::new(Pos::new(-3, 60, 7), Pos::new(2, 63, 11));
        let mut seen = vec![false; bounds.volume() as usize];

        for y in bounds.min.y..=bounds.max.y {
            for z in bounds.min.z..=bounds.max.z {
                for x in bounds.min.x..=bounds.max.x {
                    let pos = Pos::new(x, y, z);
                    let index = bounds.index(pos).expect("inside");
                    assert!(!seen[index], "index {index} used twice");
                    seen[index] = true;
                    assert_eq!(bounds.position_of(index), Some(pos));
                }
            }
        }
        assert!(seen.into_iter().all(|hit| hit), "indices must be dense");
    }

    #[test]
    fn out_of_bounds_positions_have_no_index() {
        let bounds = Bounds::new(Pos::new(0, 0, 0), Pos::new(1, 1, 1));
        assert!(bounds.index(Pos::new(2, 0, 0)).is_none());
        assert!(bounds.index(Pos::new(0, -1, 0)).is_none());
        assert!(!bounds.contains(Pos::new(0, 0, 2)));
    }

    #[test]
    fn bounds_accept_corners_in_any_order() {
        let a = Bounds::new(Pos::new(5, 5, 5), Pos::new(0, 0, 0));
        let b = Bounds::new(Pos::new(0, 0, 0), Pos::new(5, 5, 5));
        assert_eq!(a, b);
        assert_eq!(a.volume(), 216);
    }
}
