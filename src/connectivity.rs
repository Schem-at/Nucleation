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

use crate::block_position::BlockPosition;
use crate::selection::{
    connected_components_collect, flood, iter_bounds, Component, Connectivity, Limits, NotAirMask,
};
use crate::universal_schematic::UniversalSchematic;

fn component_box_air_gap(a: &Component, b: &Component) -> u32 {
    let axis_gap = |a0: i32, a1: i32, b0: i32, b1: i32| -> u32 {
        if a1 < b0 {
            b0.saturating_sub(a1).saturating_sub(1) as u32
        } else if b1 < a0 {
            a0.saturating_sub(b1).saturating_sub(1) as u32
        } else {
            0
        }
    };
    axis_gap(
        a.bounds.min.0,
        a.bounds.max.0,
        b.bounds.min.0,
        b.bounds.max.0,
    )
    .max(axis_gap(
        a.bounds.min.1,
        a.bounds.max.1,
        b.bounds.min.1,
        b.bounds.max.1,
    ))
    .max(axis_gap(
        a.bounds.min.2,
        a.bounds.max.2,
        b.bounds.min.2,
        b.bounds.max.2,
    ))
}

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

    /// **Lossless** connected-component split for extraction pipelines.
    ///
    /// Like [`UniversalSchematic::split_connected`], but instead of *dropping*
    /// sub-threshold fragments (as [`UniversalSchematic::split_connected_min`]
    /// does) it **attaches every fragment smaller than `min_blocks` to its
    /// nearest "core"** — a component with at least `min_blocks` blocks. No
    /// block, block state, or block entity is ever lost: the union of the
    /// returned pieces equals the input exactly (block-conserving).
    ///
    /// Semantics:
    /// * Components with `>= min_blocks` blocks are **cores**; the rest are
    ///   **fragments**.
    /// * If there are **0 or 1 cores**, the whole schematic is returned as a
    ///   single piece (nothing is split off, nothing is dropped). This is the
    ///   guard that keeps a redstone build — which shatters into many small
    ///   substrate-subtracted fragments under a block-level flood-fill — from
    ///   being torn apart: with no *second* substantial core, it stays whole.
    /// * With **≥2 cores**, each fragment is assigned to the core with the
    ///   nearest **bounding-box centroid** (Euclidean distance between centroid
    ///   points; cheap and stable, ties broken by core order which is
    ///   largest-first). Each core plus its attached fragments becomes one
    ///   piece, largest-core-first, `#N`-suffixed like `split_connected`.
    ///
    /// `min_blocks == 0` makes every component a core (equivalent to
    /// [`UniversalSchematic::split_connected`]).
    pub fn split_connected_attach(
        &self,
        conn: Connectivity,
        min_blocks: usize,
    ) -> Vec<UniversalSchematic> {
        let base_name = self
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "schematic".to_string());

        let comps = self.connected_components(conn);

        // bbox centroid of a component (float, for nearest-core assignment).
        let centroid = |c: &Component| -> (f64, f64, f64) {
            (
                (c.bounds.min.0 as f64 + c.bounds.max.0 as f64) / 2.0,
                (c.bounds.min.1 as f64 + c.bounds.max.1 as f64) / 2.0,
                (c.bounds.min.2 as f64 + c.bounds.max.2 as f64) / 2.0,
            )
        };

        // Partition into cores (>= min_blocks) and fragments, preserving the
        // largest-first order from `connected_components`.
        let mut core_idx: Vec<usize> = Vec::new();
        let mut frag_idx: Vec<usize> = Vec::new();
        for (i, c) in comps.iter().enumerate() {
            if c.blocks.len() >= min_blocks {
                core_idx.push(i);
            } else {
                frag_idx.push(i);
            }
        }

        // Materialize a piece from a set of source positions.
        let materialize = |positions: &[BlockPosition], name: String| -> UniversalSchematic {
            let mut piece = UniversalSchematic::new(name.clone());
            piece.metadata = self.metadata.clone();
            piece.metadata.name = Some(name);
            for pos in positions {
                if let Some(block) = self.get_block(pos.x, pos.y, pos.z) {
                    piece.set_block(pos.x, pos.y, pos.z, &block.clone());
                }
                if let Some(entity) = self.get_block_entity_owned(*pos) {
                    piece.set_block_entity(*pos, entity);
                }
            }
            piece.default_region = piece.default_region.to_compact();
            piece
        };

        // 0 or 1 core: return the whole schematic as one piece (lossless, no
        // shatter). This is the guard for redstone-style builds.
        if core_idx.len() <= 1 {
            let mut all: Vec<BlockPosition> = Vec::new();
            for c in &comps {
                all.extend(c.blocks.iter().cloned());
            }
            return vec![materialize(&all, format!("{base_name}#1"))];
        }

        // ≥2 cores: seed each core's bucket with its own blocks, then attach
        // each fragment to the nearest core by centroid distance.
        let core_centroids: Vec<(f64, f64, f64)> =
            core_idx.iter().map(|&i| centroid(&comps[i])).collect();

        let mut buckets: Vec<Vec<BlockPosition>> =
            core_idx.iter().map(|&i| comps[i].blocks.clone()).collect();

        for &fi in &frag_idx {
            let fc = centroid(&comps[fi]);
            let nearest = core_centroids
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let da = (a.0 - fc.0).powi(2) + (a.1 - fc.1).powi(2) + (a.2 - fc.2).powi(2);
                    let db = (b.0 - fc.0).powi(2) + (b.1 - fc.1).powi(2) + (b.2 - fc.2).powi(2);
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i)
                .unwrap_or(0);
            buckets[nearest].extend(comps[fi].blocks.iter().cloned());
        }

        buckets
            .into_iter()
            .enumerate()
            .map(|(i, positions)| materialize(&positions, format!("{base_name}#{}", i + 1)))
            .collect()
    }

    /// Losslessly split disconnected builds while reuniting nearby loose parts.
    ///
    /// The initial components use `conn` exactly like [`Self::split_connected`].
    /// Components whose tight bounding boxes are separated by at most
    /// `max_air_gap` empty blocks are then grouped (transitively) into one
    /// output piece. Unlike [`Self::split_connected_attach`], this decision is
    /// independent of block count: a small machine a long way from a larger
    /// one remains a standalone schematic, while a detached torch or wire a
    /// couple of blocks from its machine stays with that machine.
    ///
    /// `max_air_gap == 0` still reunites components whose bounding boxes touch
    /// or overlap on every axis. Output is deterministic, largest source
    /// component first, and block/block-entity conserving.
    pub fn split_connected_by_gap(
        &self,
        conn: Connectivity,
        max_air_gap: u32,
    ) -> Vec<UniversalSchematic> {
        let base_name = self
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "schematic".to_string());
        let comps = self.connected_components(conn);
        if comps.is_empty() {
            return Vec::new();
        }

        // A tiny deterministic union-find is enough here: this is a cheap
        // post-pass over component boxes, not over world voxels.
        let mut parent: Vec<usize> = (0..comps.len()).collect();
        fn root(parent: &mut [usize], mut i: usize) -> usize {
            while parent[i] != i {
                parent[i] = parent[parent[i]];
                i = parent[i];
            }
            i
        }
        for i in 0..comps.len() {
            for j in (i + 1)..comps.len() {
                if component_box_air_gap(&comps[i], &comps[j]) <= max_air_gap {
                    let ri = root(&mut parent, i);
                    let rj = root(&mut parent, j);
                    if ri != rj {
                        let keep = ri.min(rj);
                        parent[ri] = keep;
                        parent[rj] = keep;
                    }
                }
            }
        }

        let mut groups = std::collections::BTreeMap::<usize, Vec<usize>>::new();
        for i in 0..comps.len() {
            let r = root(&mut parent, i);
            groups.entry(r).or_default().push(i);
        }
        let mut groups = groups.into_values().collect::<Vec<_>>();
        groups.sort_by_key(|indices| indices[0]);

        groups
            .into_iter()
            .enumerate()
            .map(|(piece_index, component_indices)| {
                let name = format!("{base_name}#{}", piece_index + 1);
                let mut piece = UniversalSchematic::new(name.clone());
                piece.metadata = self.metadata.clone();
                piece.metadata.name = Some(name);
                for component_index in component_indices {
                    for pos in &comps[component_index].blocks {
                        if let Some(block) = self.get_block(pos.x, pos.y, pos.z) {
                            piece.set_block(pos.x, pos.y, pos.z, &block.clone());
                        }
                        if let Some(entity) = self.get_block_entity_owned(*pos) {
                            piece.set_block_entity(*pos, entity);
                        }
                    }
                }
                piece.default_region = piece.default_region.to_compact();
                piece
            })
            .collect()
    }

    /// Losslessly split independent builds and attach only nearby tiny parts.
    ///
    /// Every connected component with at least `min_standalone_blocks` is an
    /// independent output core, even when another core is spatially nearby.
    /// A smaller component attaches directly to its nearest core only when the
    /// two tight bounding boxes are separated by at most `max_air_gap` empty
    /// blocks. Otherwise the small component remains its own output.
    ///
    /// Attachment is deliberately **non-transitive**: a fragment can never
    /// bridge two cores, and a chain of nearby fragments cannot collapse a
    /// plot full of disconnected machines into one schematic. With no cores,
    /// every connected component remains standalone. Output is deterministic,
    /// largest source component first, and conserves all blocks and block
    /// entities.
    ///
    /// A `min_standalone_blocks` value of `0` makes every component a core, so
    /// this is exactly equivalent to [`Self::split_connected`] and
    /// `max_air_gap` has no effect.
    pub fn split_connected_attach_nearby(
        &self,
        conn: Connectivity,
        min_standalone_blocks: usize,
        max_air_gap: u32,
    ) -> Vec<UniversalSchematic> {
        let base_name = self
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "schematic".to_string());
        let comps = self.connected_components(conn);
        if comps.is_empty() {
            return Vec::new();
        }

        let core_indices = comps
            .iter()
            .enumerate()
            .filter_map(|(index, component)| {
                (component.blocks.len() >= min_standalone_blocks).then_some(index)
            })
            .collect::<Vec<_>>();
        let mut groups = core_indices
            .iter()
            .map(|&index| (index, vec![index]))
            .collect::<Vec<_>>();

        for fragment_index in 0..comps.len() {
            if comps[fragment_index].blocks.len() >= min_standalone_blocks {
                continue;
            }
            let nearest = core_indices
                .iter()
                .enumerate()
                .filter_map(|(group_index, &core_index)| {
                    let gap = component_box_air_gap(&comps[fragment_index], &comps[core_index]);
                    (gap <= max_air_gap).then_some((gap, core_index, group_index))
                })
                .min();
            if let Some((_gap, _core_index, group_index)) = nearest {
                groups[group_index].1.push(fragment_index);
            } else {
                groups.push((fragment_index, vec![fragment_index]));
            }
        }
        groups.sort_by_key(|(anchor, _)| *anchor);

        groups
            .into_iter()
            .enumerate()
            .map(|(piece_index, (_anchor, component_indices))| {
                let name = format!("{base_name}#{}", piece_index + 1);
                let mut piece = UniversalSchematic::new(name.clone());
                piece.metadata = self.metadata.clone();
                piece.metadata.name = Some(name);
                for component_index in component_indices {
                    for pos in &comps[component_index].blocks {
                        if let Some(block) = self.get_block(pos.x, pos.y, pos.z) {
                            piece.set_block(pos.x, pos.y, pos.z, &block.clone());
                        }
                        if let Some(entity) = self.get_block_entity_owned(*pos) {
                            piece.set_block_entity(*pos, entity);
                        }
                    }
                }
                piece.default_region = piece.default_region.to_compact();
                piece
            })
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
            s.select_connected((0, 3, 0), Connectivity::Corner)
                .blocks
                .len(),
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
        s.set_block_with_nbt(50, 0, 0, "minecraft:chest", nbt)
            .unwrap();
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

        assert_eq!(
            original, recombined,
            "no block lost, duplicated, or mutated"
        );
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

    // ── split_connected_attach (lossless) ───────────────────────────────

    #[test]
    fn split_connected_attach_two_cores_conserves_blocks_and_attaches_nearest() {
        let mut s = UniversalSchematic::new("attach".into());
        // Core A: 5 blocks near origin.
        for x in 0..5 {
            place(&mut s, x, 0, 0);
        }
        // Core B: 5 blocks far away.
        for x in 0..5 {
            place(&mut s, 100 + x, 0, 0);
        }
        // Tiny 1-block fragment, disconnected from both, closest to core B.
        place(&mut s, 110, 0, 10);

        let pieces = s.split_connected_attach(Connectivity::Corner, 3);
        assert_eq!(pieces.len(), 2, "two substantial cores -> two pieces");

        // Block conservation: nothing dropped or duplicated.
        let total: i64 = pieces.iter().map(|p| p.total_blocks() as i64).sum();
        assert_eq!(total, s.total_blocks() as i64, "no block lost");

        // Fragment attached to the nearer core (B), not A.
        let piece_b = pieces
            .iter()
            .find(|p| p.get_block(100, 0, 0).is_some())
            .expect("piece with core B");
        assert!(
            piece_b.get_block(110, 0, 10).is_some(),
            "fragment attached to nearest core B"
        );
        let piece_a = pieces
            .iter()
            .find(|p| p.get_block(0, 0, 0).is_some())
            .expect("piece with core A");
        assert!(
            piece_a.get_block(110, 0, 10).is_none(),
            "fragment did not go to core A"
        );
    }

    #[test]
    fn split_connected_attach_single_core_stays_whole_and_lossless() {
        // A redstone-style build shatters into one dominant mass plus many
        // small fragments; with only one core the guard keeps it whole and
        // loses nothing.
        let mut s = UniversalSchematic::new("redstone".into());
        for x in 0..8 {
            place(&mut s, x, 0, 0); // one core (size 8)
        }
        // scattered sub-threshold fragments
        place(&mut s, 50, 0, 0);
        place(&mut s, 60, 5, 0);
        place(&mut s, 70, 0, 9);

        let pieces = s.split_connected_attach(Connectivity::Corner, 4);
        assert_eq!(pieces.len(), 1, "one core -> single whole piece");
        assert_eq!(
            pieces[0].total_blocks(),
            s.total_blocks(),
            "no fragment dropped"
        );
    }

    #[test]
    fn split_connected_by_gap_keeps_small_distant_machines_independent() {
        let mut s = UniversalSchematic::new("three-machines".into());
        // Deliberately make only one machine larger than the old 128-block
        // attachment threshold. Size must not decide whether a distant build
        // gets its own schematic.
        for x in 0..6 {
            for y in 0..5 {
                for z in 0..5 {
                    place(&mut s, x, y, z); // 150 blocks
                }
            }
        }
        for x in 30..33 {
            for y in 0..3 {
                place(&mut s, x, y, 0); // 9 blocks
            }
        }
        for x in 60..62 {
            for y in 0..2 {
                place(&mut s, x, y, 0); // 4 blocks
            }
        }

        let pieces = s.split_connected_by_gap(Connectivity::Corner, 3);
        assert_eq!(pieces.len(), 3);
        assert_eq!(pieces[0].total_blocks(), 150);
        assert_eq!(pieces[1].total_blocks(), 9);
        assert_eq!(pieces[2].total_blocks(), 4);
        assert_eq!(
            pieces
                .iter()
                .map(UniversalSchematic::total_blocks)
                .sum::<i32>(),
            s.total_blocks(),
            "the split is lossless"
        );
    }

    #[test]
    fn split_connected_by_gap_reunites_nearby_detached_parts() {
        let mut s = UniversalSchematic::new("loose-parts".into());
        for x in 0..4 {
            place(&mut s, x, 0, 0);
        }
        // Two empty blocks between the main run and the loose part.
        place(&mut s, 6, 0, 0);
        // A truly separate machine.
        place(&mut s, 30, 0, 0);

        let pieces = s.split_connected_by_gap(Connectivity::Corner, 2);
        assert_eq!(pieces.len(), 2);
        assert_eq!(pieces[0].total_blocks(), 5);
        assert_eq!(pieces[1].total_blocks(), 1);
        assert!(pieces[0].get_block(6, 0, 0).is_some());
    }

    #[test]
    fn split_connected_attach_nearby_cannot_chain_independent_cores() {
        let mut s = UniversalSchematic::new("no-chain".into());
        for base in [0, 24, 48] {
            for x in base..(base + 20) {
                place(&mut s, x, 0, 0);
            }
        }
        // Each pair of cores has four empty blocks between their boxes. A
        // transitive proximity union would collapse all three at gap=4.
        let pieces = s.split_connected_attach_nearby(Connectivity::Corner, 16, 4);
        assert_eq!(pieces.len(), 3);
        assert!(pieces.iter().all(|piece| piece.total_blocks() == 20));
    }

    #[test]
    fn split_connected_attach_nearby_attaches_only_direct_tiny_fragments() {
        let mut s = UniversalSchematic::new("direct-fragments".into());
        for x in 0..20 {
            place(&mut s, x, 0, 0);
        }
        place(&mut s, 22, 0, 0); // two empty blocks from the core
        place(&mut s, 25, 0, 0); // two from the fragment, five from the core
        place(&mut s, 60, 0, 0); // wholly independent tiny build

        let pieces = s.split_connected_attach_nearby(Connectivity::Corner, 16, 2);
        assert_eq!(pieces.len(), 3);
        assert_eq!(pieces[0].total_blocks(), 21);
        assert_eq!(pieces[1].total_blocks(), 1, "fragments do not chain");
        assert_eq!(pieces[2].total_blocks(), 1, "distant tiny build survives");
        assert_eq!(
            pieces
                .iter()
                .map(UniversalSchematic::total_blocks)
                .sum::<i32>(),
            s.total_blocks()
        );
    }

    #[test]
    fn split_connected_attach_nearby_zero_threshold_is_exact() {
        let mut s = UniversalSchematic::new("literal-components".into());
        for base in [0, 10, 20] {
            place(&mut s, base, 0, 0);
            place(&mut s, base + 1, 0, 0);
        }

        let exact = s.split_connected(Connectivity::Corner);
        let nearby = s.split_connected_attach_nearby(Connectivity::Corner, 0, u32::MAX);

        assert_eq!(nearby.len(), 3, "every disconnected component is emitted");
        assert_eq!(
            nearby
                .iter()
                .map(UniversalSchematic::total_blocks)
                .collect::<Vec<_>>(),
            exact
                .iter()
                .map(UniversalSchematic::total_blocks)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            nearby
                .iter()
                .map(UniversalSchematic::total_blocks)
                .sum::<i32>(),
            s.total_blocks(),
            "exact splitting remains lossless"
        );
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
