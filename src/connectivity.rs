//! First-class connectivity flood-fill methods on [`UniversalSchematic`].
//!
//! These are thin, ergonomic wrappers around the generic [`crate::selection`]
//! flood-fill engine (a port of RedstoneTools' `//that` command), specialised
//! to the most common question: *"which non-air blocks are physically
//! connected?"*. They let you split an already-extracted schematic into its
//! physically-disconnected components as a cheap second pass, with no world
//! re-read.
//!
//! ## Connectivity ↔ `//that`
//!
//! The neighbour set is chosen with [`crate::selection::Connectivity`], whose
//! four variants map one-to-one onto `//that`'s offset sets:
//!
//! | `Connectivity` | neighbours | `//that` flag |
//! |----------------|-----------:|---------------|
//! | `Face`         | 6          | (default)     |
//! | `Edge`         | 14         | `-d`          |
//! | `EdgeMid`      | 18         | `-dd`         |
//! | `Corner`       | 26         | `-ddd`        |
//!
//! Each larger set is a strict superset of the previous, so a component found
//! at `Face` is always contained in the component found at `Corner` from the
//! same seed. A build split only by pure diagonal contact merges at `Corner`;
//! a build separated by a ≥1-block air gap stays split even at `Corner`
//! (the Moore neighbourhood reaches at most one cell per step).
//!
//! This module is generic geometry over "non-air blocks" and carries no
//! knowledge of any particular extraction or tagging scheme.

use crate::selection::{
    connected_components_collect, flood, iter_bounds, Component, Connectivity, Limits, NotAirMask,
};
use crate::block_position::BlockPosition;
use crate::universal_schematic::UniversalSchematic;

impl UniversalSchematic {
    /// Select the connected non-air component containing `seed`, exactly as
    /// RedstoneTools' `//that` does: a BFS flood-fill over non-air blocks using
    /// the neighbour set for `conn`.
    ///
    /// Returns a [`Component`] with the reached [`Component::blocks`] (BFS order
    /// from the seed) and their tight [`Component::bounds`]. If `seed` is air
    /// (or out of bounds), the returned component has zero blocks.
    ///
    /// See the [module docs](self) for the `conn` ↔ `//that` mapping.
    pub fn select_connected(&self, seed: (i32, i32, i32), conn: Connectivity) -> Component {
        let mask = NotAirMask::new(self);
        flood(
            BlockPosition::new(seed.0, seed.1, seed.2),
            &mask,
            conn,
            &Limits::unbounded(),
        )
    }

    /// Label every non-air block into physically-connected components using the
    /// neighbour set for `conn` — a repeated `//that` flood-fill over the whole
    /// schematic that touches each block at most once (shared visited set).
    ///
    /// Components are returned sorted largest-first (by block count), so
    /// `components[0]` is the dominant build. This is the primitive for a
    /// second-pass build splitter: run it on an extracted schematic and inspect
    /// how many substantial components come back.
    ///
    /// See the [module docs](self) for the `conn` ↔ `//that` mapping.
    pub fn connected_components(&self, conn: Connectivity) -> Vec<Component> {
        let mask = NotAirMask::new(self);
        let bounds = self.get_bounding_box();
        let mut comps =
            connected_components_collect(iter_bounds(&bounds), &mask, conn, &Limits::unbounded());
        comps.sort_by(|a, b| b.blocks.len().cmp(&a.blocks.len()));
        comps
    }
}

#[cfg(test)]
mod tests {
    use crate::selection::Connectivity;
    use crate::UniversalSchematic;

    fn place(s: &mut UniversalSchematic, x: i32, y: i32, z: i32) {
        s.set_block_str(x, y, z, "minecraft:stone");
    }

    #[test]
    fn face_touching_blobs_are_one_component_everywhere() {
        // Two blocks sharing a face are connected at every connectivity.
        let mut s = UniversalSchematic::new("t".into());
        place(&mut s, 0, 0, 0);
        place(&mut s, 1, 0, 0);
        for conn in [
            Connectivity::Face,
            Connectivity::Edge,
            Connectivity::EdgeMid,
            Connectivity::Corner,
        ] {
            assert_eq!(s.connected_components(conn).len(), 1, "{:?}", conn);
        }
    }

    #[test]
    fn diagonal_only_touch_splits_at_face_merges_at_corner() {
        // Two blobs whose only contact is the pure corner diagonal (1,1,1).
        let mut s = UniversalSchematic::new("t".into());
        place(&mut s, 0, 0, 0);
        place(&mut s, 1, 1, 1);
        // Face / Edge / EdgeMid sets never include the (1,1,1) corner.
        assert_eq!(s.connected_components(Connectivity::Face).len(), 2);
        assert_eq!(s.connected_components(Connectivity::Edge).len(), 2);
        assert_eq!(s.connected_components(Connectivity::EdgeMid).len(), 2);
        // The corner diagonal heals the touch -> single component.
        assert_eq!(s.connected_components(Connectivity::Corner).len(), 1);
    }

    #[test]
    fn edge_diagonal_touch_merges_at_edge_not_face() {
        // Contact via an edge diagonal (1,1,0): split at Face, merged from Edge up.
        let mut s = UniversalSchematic::new("t".into());
        place(&mut s, 0, 0, 0);
        place(&mut s, 1, 1, 0);
        assert_eq!(s.connected_components(Connectivity::Face).len(), 2);
        assert_eq!(s.connected_components(Connectivity::Edge).len(), 1);
        assert_eq!(s.connected_components(Connectivity::Corner).len(), 1);
    }

    #[test]
    fn one_block_gap_splits_at_every_connectivity() {
        // A 1-block air gap (distance 2 along X) is uncrossable even by the
        // 26-neighbour Moore set (reach is one cell per step).
        let mut s = UniversalSchematic::new("t".into());
        place(&mut s, 0, 0, 0);
        place(&mut s, 2, 0, 0);
        for conn in [
            Connectivity::Face,
            Connectivity::Edge,
            Connectivity::EdgeMid,
            Connectivity::Corner,
        ] {
            assert_eq!(s.connected_components(conn).len(), 2, "{:?}", conn);
        }
    }

    #[test]
    fn components_sorted_largest_first() {
        let mut s = UniversalSchematic::new("t".into());
        // Big blob (a 2x2x2 = 8) and a lone block, disconnected.
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    place(&mut s, x, y, z);
                }
            }
        }
        place(&mut s, 20, 0, 0);
        let comps = s.connected_components(Connectivity::Corner);
        assert_eq!(comps.len(), 2);
        assert_eq!(comps[0].blocks.len(), 8); // dominant first
        assert_eq!(comps[1].blocks.len(), 1);
    }

    #[test]
    fn select_connected_returns_seed_component_and_ignores_air_seed() {
        let mut s = UniversalSchematic::new("t".into());
        place(&mut s, 0, 0, 0);
        place(&mut s, 1, 0, 0);
        place(&mut s, 5, 0, 0); // separate blob
        let comp = s.select_connected((0, 0, 0), Connectivity::Face);
        assert_eq!(comp.blocks.len(), 2);
        // Air seed -> empty component.
        assert_eq!(
            s.select_connected((0, 3, 0), Connectivity::Corner).blocks.len(),
            0
        );
    }

    #[test]
    fn negative_coordinates_are_handled() {
        let mut s = UniversalSchematic::new("t".into());
        place(&mut s, -5, -5, -5);
        place(&mut s, -4, -5, -5);
        let comps = s.connected_components(Connectivity::Face);
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].blocks.len(), 2);
        assert_eq!(comps[0].bounds.min, (-5, -5, -5));
        assert_eq!(comps[0].bounds.max, (-4, -5, -5));
    }
}
