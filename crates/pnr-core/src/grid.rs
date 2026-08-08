//! Integer 3-D grid positions and axis-aligned boxes.

/// A cell position on the integer grid. Ordering is lexicographic
/// `(x, y, z)`, which gives every ordered container over positions a
/// deterministic iteration order.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Pos {
    /// X coordinate.
    pub x: i32,
    /// Y coordinate (vertical).
    pub y: i32,
    /// Z coordinate.
    pub z: i32,
}

impl Pos {
    /// Construct a position.
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Pos { x, y, z }
    }

    /// This position displaced by `(dx, dy, dz)`.
    pub const fn offset(self, dx: i32, dy: i32, dz: i32) -> Self {
        Pos::new(self.x + dx, self.y + dy, self.z + dz)
    }

    /// Manhattan (L1) distance to `other`.
    pub fn manhattan(self, other: Pos) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y) + self.z.abs_diff(other.z)
    }
}

impl From<(i32, i32, i32)> for Pos {
    fn from((x, y, z): (i32, i32, i32)) -> Self {
        Pos::new(x, y, z)
    }
}

/// An inclusive axis-aligned box.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Aabb {
    /// Minimum corner (inclusive).
    pub min: Pos,
    /// Maximum corner (inclusive).
    pub max: Pos,
}

impl Aabb {
    /// Construct from two corners; the corners are normalized so callers may
    /// pass them in any order.
    pub fn new(a: Pos, b: Pos) -> Self {
        Aabb {
            min: Pos::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z)),
            max: Pos::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z)),
        }
    }

    /// Whether `p` lies inside the box (inclusive bounds).
    pub fn contains(&self, p: Pos) -> bool {
        self.min.x <= p.x
            && p.x <= self.max.x
            && self.min.y <= p.y
            && p.y <= self.max.y
            && self.min.z <= p.z
            && p.z <= self.max.z
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manhattan_and_contains() {
        let a = Pos::new(0, 1, 2);
        let b = Pos::new(3, -1, 2);
        assert_eq!(a.manhattan(b), 5);
        let bb = Aabb::new(b, a);
        assert!(bb.contains(Pos::new(1, 0, 2)));
        assert!(!bb.contains(Pos::new(1, 2, 2)));
    }
}
