//! Neighbour-connectivity for flood selection.
//!
//! Mirrors RedstoneTools' `That.kt` `Offsets`: callers pick how aggressive the
//! flood should be at merging diagonally-adjacent blocks.

/// Which neighbours count as "connected" during a flood select.
///
/// In ascending order of permissiveness — `Face` is strict (6-neighbour),
/// `Corner` is the full Moore neighbourhood (26).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Connectivity {
    /// 6 face-adjacent neighbours. Strictest — two builds touching only at
    /// an edge or corner stay separate.
    Face,
    /// 14 neighbours: faces + the 8 edge-diagonals in the top/bottom layers
    /// (`-d` in `/that`).
    Edge,
    /// 18 neighbours: `Edge` plus the 4 mid-layer X/Z diagonals (`-dd`).
    EdgeMid,
    /// 26 neighbours: full Moore neighbourhood (`-ddd`).
    Corner,
}

impl Connectivity {
    /// Offsets to apply to a position to enumerate its neighbours.
    pub fn offsets(self) -> &'static [(i32, i32, i32)] {
        match self {
            Connectivity::Face => &FACE,
            Connectivity::Edge => &EDGE,
            Connectivity::EdgeMid => &EDGE_MID,
            Connectivity::Corner => &CORNER,
        }
    }
}

const FACE: [(i32, i32, i32); 6] = [
    (1, 0, 0),
    (-1, 0, 0),
    (0, 1, 0),
    (0, -1, 0),
    (0, 0, 1),
    (0, 0, -1),
];

// `Edge` = FACE + 8 edge-diagonals (top + bottom layers, no mid-layer).
// Matches That.kt `Offsets.DIAG`.
const EDGE: [(i32, i32, i32); 14] = [
    // face
    (1, 0, 0),
    (-1, 0, 0),
    (0, 1, 0),
    (0, -1, 0),
    (0, 0, 1),
    (0, 0, -1),
    // top layer
    (1, 1, 0),
    (-1, 1, 0),
    (0, 1, 1),
    (0, 1, -1),
    // bottom layer
    (1, -1, 0),
    (-1, -1, 0),
    (0, -1, 1),
    (0, -1, -1),
];

// `EdgeMid` = EDGE + 4 mid-layer X/Z diagonals. Matches `Offsets.VERY_DIAG`.
const EDGE_MID: [(i32, i32, i32); 18] = [
    (1, 0, 0),
    (-1, 0, 0),
    (0, 1, 0),
    (0, -1, 0),
    (0, 0, 1),
    (0, 0, -1),
    (1, 1, 0),
    (-1, 1, 0),
    (0, 1, 1),
    (0, 1, -1),
    (1, -1, 0),
    (-1, -1, 0),
    (0, -1, 1),
    (0, -1, -1),
    // mid layer
    (1, 0, 1),
    (-1, 0, 1),
    (1, 0, -1),
    (-1, 0, -1),
];

// `Corner` = EDGE_MID + 8 corner-diagonals. Full 26-neighbour Moore set.
// Matches `Offsets.VERY_VERY_DIAG`.
const CORNER: [(i32, i32, i32); 26] = [
    (1, 0, 0),
    (-1, 0, 0),
    (0, 1, 0),
    (0, -1, 0),
    (0, 0, 1),
    (0, 0, -1),
    (1, 1, 0),
    (-1, 1, 0),
    (0, 1, 1),
    (0, 1, -1),
    (1, -1, 0),
    (-1, -1, 0),
    (0, -1, 1),
    (0, -1, -1),
    (1, 0, 1),
    (-1, 0, 1),
    (1, 0, -1),
    (-1, 0, -1),
    // top corners
    (1, 1, 1),
    (-1, 1, 1),
    (1, 1, -1),
    (-1, 1, -1),
    // bottom corners
    (1, -1, 1),
    (-1, -1, 1),
    (1, -1, -1),
    (-1, -1, -1),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_counts() {
        assert_eq!(Connectivity::Face.offsets().len(), 6);
        assert_eq!(Connectivity::Edge.offsets().len(), 14);
        assert_eq!(Connectivity::EdgeMid.offsets().len(), 18);
        assert_eq!(Connectivity::Corner.offsets().len(), 26);
    }

    #[test]
    fn no_duplicate_offsets() {
        for conn in [
            Connectivity::Face,
            Connectivity::Edge,
            Connectivity::EdgeMid,
            Connectivity::Corner,
        ] {
            let mut seen: Vec<_> = conn.offsets().to_vec();
            seen.sort();
            let len = seen.len();
            seen.dedup();
            assert_eq!(seen.len(), len, "duplicates in {:?}", conn);
        }
    }

    #[test]
    fn no_zero_offset() {
        for conn in [
            Connectivity::Face,
            Connectivity::Edge,
            Connectivity::EdgeMid,
            Connectivity::Corner,
        ] {
            for &o in conn.offsets() {
                assert_ne!(o, (0, 0, 0), "(0,0,0) is self, not a neighbour");
            }
        }
    }
}
