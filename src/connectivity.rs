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

    /// Split this schematic into one standalone [`UniversalSchematic`] per
    /// physically-connected component (see [`UniversalSchematic::connected_components`]
    /// for the `conn` ↔ `//that` mapping), largest-first.
    ///
    /// Each returned piece contains *exactly* that component's non-air
    /// blocks — full block state (properties) and any attached block entity
    /// (chest contents, sign text, etc.) travel with their coordinate into
    /// the correct piece. Original world coordinates are preserved (pieces
    /// are **not** re-origined), so the split is information-preserving and
    /// reversible: overlaying every returned piece back onto an empty
    /// schematic reproduces the input exactly. Each piece's `metadata.name`
    /// is the original name with a `#N` suffix (1-based, in output order);
    /// the rest of the top-level metadata is carried over unchanged.
    ///
    /// A fully-connected input returns a single-element `Vec` containing a
    /// block-identical clone (modulo the `#1` name suffix).
    pub fn split_connected(&self, conn: Connectivity) -> Vec<UniversalSchematic> {
        let base_name = self
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "schematic".to_string());

        self.connected_components(conn)
            .into_iter()
            .enumerate()
            .map(|(i, comp)| {
                let mut piece = UniversalSchematic::new(format!("{base_name}#{}", i + 1));
                piece.metadata = self.metadata.clone();
                piece.metadata.name = Some(format!("{base_name}#{}", i + 1));

                for pos in &comp.blocks {
                    if let Some(block) = self.get_block(pos.x, pos.y, pos.z) {
                        piece.set_block(pos.x, pos.y, pos.z, &block.clone());
                    }
                    if let Some(entity) = self.get_block_entity_owned(*pos) {
                        piece.set_block_entity(*pos, entity);
                    }
                }

                // set_block's incremental `expand_to_fit` pads the storage
                // region well beyond the placed blocks (a perf tradeoff for
                // incremental writes). Compact down to the tight content
                // bounds so the piece is a well-formed standalone schematic:
                // a correct (non-padded) bounding box, and `get_block` no
                // longer reports phantom air outside the component.
                piece.default_region = piece.default_region.to_compact();

                piece
            })
            .collect()
    }

    /// Like [`UniversalSchematic::split_connected`], but drops components
    /// with fewer than `min_blocks` blocks. Tiny fragments are simply
    /// discarded, not merged into a neighbouring piece — if you need
    /// attach-to-nearest behaviour, build it on top of this.
    pub fn split_connected_min(
        &self,
        conn: Connectivity,
        min_blocks: usize,
    ) -> Vec<UniversalSchematic> {
        self.split_connected(conn)
            .into_iter()
            .filter(|piece| piece.total_blocks() as usize >= min_blocks)
            .collect()
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

    // ── split_connected ─────────────────────────────────────────────────

    use crate::block_position::BlockPosition;
    use std::collections::HashMap;

    #[test]
    fn split_connected_two_blobs_routes_blocks_and_block_entity_to_correct_piece() {
        let mut s = UniversalSchematic::new("base".into());
        // Blob A: two stone blocks far from blob B.
        place(&mut s, 0, 0, 0);
        place(&mut s, 1, 0, 0);
        // Blob B: a chest (with NBT) plus a neighbouring stone block.
        let mut nbt = HashMap::new();
        nbt.insert("CustomName".to_string(), "\"Loot\"".to_string());
        s.set_block_with_nbt(50, 0, 0, "minecraft:chest", nbt).unwrap();
        place(&mut s, 51, 0, 0);

        let pieces = s.split_connected(Connectivity::Face);
        assert_eq!(pieces.len(), 2);

        // Largest-first: both components are size 2, so order is by
        // discovery (iter_bounds scan order) — identify by content instead.
        let piece_a = pieces
            .iter()
            .find(|p| p.get_block(0, 0, 0).is_some())
            .expect("piece containing blob A");
        let piece_b = pieces
            .iter()
            .find(|p| p.get_block(50, 0, 0).is_some())
            .expect("piece containing blob B");

        // Piece A has exactly blob A's blocks, no block entity.
        assert_eq!(piece_a.total_blocks(), 2);
        assert_eq!(
            piece_a.get_block(1, 0, 0).map(|b| b.get_name()),
            Some("minecraft:stone")
        );
        assert!(piece_a.get_block(50, 0, 0).is_none());
        assert!(piece_a
            .get_block_entity_owned(BlockPosition::new(50, 0, 0))
            .is_none());

        // Piece B has exactly blob B's blocks, and the chest's NBT travelled
        // with it.
        assert_eq!(piece_b.total_blocks(), 2);
        assert_eq!(
            piece_b.get_block(51, 0, 0).map(|b| b.get_name()),
            Some("minecraft:stone")
        );
        assert!(piece_b.get_block(0, 0, 0).is_none());
        let chest_entity = piece_b
            .get_block_entity_owned(BlockPosition::new(50, 0, 0))
            .expect("chest block entity should travel with its block");
        assert_eq!(chest_entity.id, "minecraft:chest");
        assert!(piece_b
            .get_block_entity_owned(BlockPosition::new(0, 0, 0))
            .is_none());
    }

    #[test]
    fn split_connected_honors_connectivity_choice() {
        // Pure corner diagonal touch: split at Face, merged at Corner.
        let mut s = UniversalSchematic::new("diag".into());
        place(&mut s, 0, 0, 0);
        place(&mut s, 1, 1, 1);

        assert_eq!(s.split_connected(Connectivity::Face).len(), 2);
        assert_eq!(s.split_connected(Connectivity::Corner).len(), 1);
    }

    #[test]
    fn split_connected_block_conservation_round_trip() {
        // Mixed schematic: a blockstate with properties in one component,
        // a plain block in a disconnected component, negative coordinates.
        let mut s = UniversalSchematic::new("mixed".into());
        s.set_block_from_string(-3, -3, -3, "minecraft:oak_stairs[facing=north,half=top]")
            .unwrap();
        place(&mut s, -3, -3, -2); // face-connects to the stairs block
        place(&mut s, 10, 10, 10); // disconnected singleton

        // Collect the original multiset of (pos, block-string).
        let mut original: Vec<((i32, i32, i32), String)> = Vec::new();
        for x in -4..=11 {
            for y in -4..=11 {
                for z in -4..=11 {
                    if let Some(b) = s.get_block(x, y, z) {
                        if b.get_name() != "minecraft:air" {
                            original.push(((x, y, z), b.to_string()));
                        }
                    }
                }
            }
        }
        original.sort();

        let pieces = s.split_connected(Connectivity::Face);
        assert_eq!(pieces.len(), 2);

        let mut recombined: Vec<((i32, i32, i32), String)> = Vec::new();
        for piece in &pieces {
            let bounds = piece.get_bounding_box();
            for x in bounds.min.0..=bounds.max.0 {
                for y in bounds.min.1..=bounds.max.1 {
                    for z in bounds.min.2..=bounds.max.2 {
                        if let Some(b) = piece.get_block(x, y, z) {
                            if b.get_name() != "minecraft:air" {
                                recombined.push(((x, y, z), b.to_string()));
                            }
                        }
                    }
                }
            }
        }
        recombined.sort();

        assert_eq!(original, recombined, "no block lost, duplicated, or mutated");
    }

    #[test]
    fn split_connected_min_drops_tiny_fragments() {
        let mut s = UniversalSchematic::new("frag".into());
        // Main mass: 5 blocks.
        for x in 0..5 {
            place(&mut s, x, 0, 0);
        }
        // Tiny disconnected fragment: 2 blocks.
        place(&mut s, 100, 0, 0);
        place(&mut s, 101, 0, 0);

        let all = s.split_connected(Connectivity::Face);
        assert_eq!(all.len(), 2);

        let filtered = s.split_connected_min(Connectivity::Face, 3);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].total_blocks(), 5);
    }

    #[test]
    fn split_connected_single_component_returns_one_block_identical_piece() {
        let mut s = UniversalSchematic::new("solo".into());
        for x in 0..3 {
            for y in 0..3 {
                place(&mut s, x, y, 0);
            }
        }
        let pieces = s.split_connected(Connectivity::Face);
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].total_blocks(), s.total_blocks());
        // `s.get_bounding_box()` is the padded storage bbox (perf tradeoff of
        // incremental `set_block`); the split piece is compacted, so compare
        // against the tight content bounds instead.
        assert_eq!(
            pieces[0].get_bounding_box(),
            s.default_region.get_tight_bounds().unwrap()
        );
        for x in 0..3 {
            for y in 0..3 {
                assert_eq!(
                    pieces[0].get_block(x, y, 0).map(|b| b.to_string()),
                    s.get_block(x, y, 0).map(|b| b.to_string())
                );
            }
        }
    }
}
