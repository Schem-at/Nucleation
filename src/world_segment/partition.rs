//! Caller-supplied partition hints: named boxes a cluster may never span.
//!
//! Deliberately generic. This module has no idea what the boxes represent —
//! land parcels, plot claims, administrative regions. The caller assigns
//! meaning; segmentation only enforces the boundaries.

use serde::{Deserialize, Serialize};

use crate::world_segment::ids::ContentId;

/// An axis-aligned region in XZ, optionally bounded in Y.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PartitionHint {
    /// Opaque caller id. Carried through to provenance as a join key.
    pub id: String,
    /// Inclusive `(x0, x1, z0, z1)`.
    pub bbox_xz: (i32, i32, i32, i32),
    /// Inclusive `(y0, y1)`. `None` means the full column.
    pub y_range: Option<(i32, i32)>,
}

impl PartitionHint {
    pub fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        let (x0, x1, z0, z1) = self.bbox_xz;
        if x < x0 || x > x1 || z < z0 || z > z1 {
            return false;
        }
        match self.y_range {
            Some((y0, y1)) => y >= y0 && y <= y1,
            None => true,
        }
    }
}

/// How segmentation treats hint boundaries.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum PartitionPolicy {
    /// A cluster may never span a boundary. Dilation does not propagate
    /// between differing partitions.
    HardCut,
    /// Intended: crossing is allowed but recorded, for measuring how often it
    /// happens.
    ///
    /// **Not implemented yet.** Crossing-recording does not exist, so
    /// segmentation currently treats `Prefer` exactly like [`Off`]: hints are
    /// ignored, clusters may span boundaries freely, and every cluster comes
    /// back with `partition_id: None`. Nothing is recorded. Do not select
    /// `Prefer` expecting crossing data — you will get silence, not zero
    /// crossings.
    ///
    /// [`Off`]: PartitionPolicy::Off
    Prefer,
    /// Hints are ignored entirely.
    Off,
}

/// Point-in-partition lookup.
///
/// Hints are sorted at construction by a total order over their full
/// content — `id`, then `bbox_xz`, then `y_range` — so that index assignment
/// is independent of the order the caller supplied them in, even when
/// multiple hints share the same `id`. `PartitionHint` does not require ids
/// to be unique, and a plain sort by `id` alone (a *stable* sort) would
/// leave same-id hints in their original relative order, letting input
/// order leak into which hint — and which `u32` index — a point resolves
/// to. Sorting by the full tuple means ties can only remain between hints
/// that are entirely identical, which are genuinely interchangeable, so two
/// workers given the same hints in different orders always agree.
pub struct PartitionIndex {
    hints: Vec<PartitionHint>,
}

impl PartitionIndex {
    pub fn new(mut hints: Vec<PartitionHint>) -> Self {
        hints.sort_by(|a, b| {
            a.id.cmp(&b.id)
                .then_with(|| a.bbox_xz.cmp(&b.bbox_xz))
                .then_with(|| a.y_range.cmp(&b.y_range))
        });
        PartitionIndex { hints }
    }

    pub fn is_empty(&self) -> bool {
        self.hints.is_empty()
    }

    /// Content digest of every hint this index holds.
    ///
    /// Folded into [`SegConfig::config_hash`] so that hint *geometry* is part
    /// of cluster identity, not just the hint *name* that
    /// [`ClusterId`](crate::world_segment::ids::ClusterId) records. Two runs
    /// whose hints share ids but cover different extents — `"L"` over
    /// `x 0..=61` versus `"L"` over `x 0..=40` — segment differently while
    /// agreeing on tile, partition name and anchor cell, so without this they
    /// mint the same id for genuinely different clusters.
    ///
    /// Order-independent: [`PartitionIndex::new`] has already sorted the hints
    /// by their full content, so two callers who supply the same hints in
    /// different orders digest to the same value. Every part is length-framed
    /// by [`ContentId::of`], so hints cannot alias through concatenation
    /// (`id: "ab"` next to `id: "c"` differs from `id: "a"` next to `id: "bc"`),
    /// and the count is included so a prefix of the hint list cannot collide
    /// with the whole.
    ///
    /// [`SegConfig::config_hash`]: crate::world_segment::segment::SegConfig::config_hash
    pub fn hints_hash(&self) -> ContentId {
        let mut parts: Vec<Vec<u8>> = Vec::with_capacity(self.hints.len() * 8 + 2);
        parts.push(b"parthints.v1".to_vec());
        parts.push((self.hints.len() as u64).to_le_bytes().to_vec());
        for h in &self.hints {
            parts.push(h.id.as_bytes().to_vec());
            let (x0, x1, z0, z1) = h.bbox_xz;
            for v in [x0, x1, z0, z1] {
                parts.push(v.to_le_bytes().to_vec());
            }
            // `None` gets its own presence byte rather than a sentinel range,
            // so an unbounded column cannot collide with a hint that genuinely
            // names `(i32::MIN, i32::MAX)`.
            match h.y_range {
                Some((y0, y1)) => {
                    parts.push(vec![1]);
                    parts.push(y0.to_le_bytes().to_vec());
                    parts.push(y1.to_le_bytes().to_vec());
                }
                None => {
                    parts.push(vec![0]);
                    parts.push(Vec::new());
                    parts.push(Vec::new());
                }
            }
        }
        let refs: Vec<&[u8]> = parts.iter().map(|p| p.as_slice()).collect();
        ContentId::of(&refs)
    }

    pub fn partition_at(&self, x: i32, y: i32, z: i32) -> Option<&str> {
        self.hints.iter().find(|h| h.contains(x, y, z)).map(|h| h.id.as_str())
    }

    /// Stable numeric handle for the partition at this point, for cheap
    /// per-cell comparison during dilation.
    pub fn id_index_at(&self, x: i32, y: i32, z: i32) -> Option<u32> {
        self.hints.iter().position(|h| h.contains(x, y, z)).map(|i| i as u32)
    }

    /// Looks up the id for a previously-obtained index.
    ///
    /// # Invariant
    /// `index` must have come from [`PartitionIndex::id_index_at`] called on
    /// this same instance. Passing any other value (e.g. an index obtained
    /// from a differently-constructed `PartitionIndex`, or one at all
    /// out-of-range) is a precondition violation and panics.
    pub fn id_of_index(&self, index: u32) -> &str {
        debug_assert!(
            (index as usize) < self.hints.len(),
            "id_of_index: index {index} out of range for {} hints; index must come from id_index_at on this same PartitionIndex",
            self.hints.len()
        );
        &self.hints[index as usize].id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hints() -> Vec<PartitionHint> {
        vec![
            PartitionHint { id: "a".into(), bbox_xz: (0, 9, 0, 9), y_range: None },
            PartitionHint { id: "b".into(), bbox_xz: (11, 20, 0, 9), y_range: None },
            PartitionHint { id: "c".into(), bbox_xz: (0, 9, 11, 20), y_range: Some((0, 10)) },
        ]
    }

    #[test]
    fn point_inside_a_hint_resolves_to_it() {
        let idx = PartitionIndex::new(hints());
        assert_eq!(idx.partition_at(5, 100, 5), Some("a"));
        assert_eq!(idx.partition_at(15, 100, 5), Some("b"));
    }

    #[test]
    fn bbox_edges_are_inclusive() {
        let idx = PartitionIndex::new(hints());
        assert_eq!(idx.partition_at(0, 0, 0), Some("a"));
        assert_eq!(idx.partition_at(9, 0, 9), Some("a"));
    }

    #[test]
    fn gaps_between_hints_resolve_to_none() {
        // x = 10 is the road between plots a and b.
        assert_eq!(PartitionIndex::new(hints()).partition_at(10, 0, 5), None);
    }

    #[test]
    fn y_range_is_respected_when_present() {
        let idx = PartitionIndex::new(hints());
        assert_eq!(idx.partition_at(5, 5, 15), Some("c"));
        assert_eq!(idx.partition_at(5, 50, 15), None, "outside c's y_range");
    }

    #[test]
    fn none_y_range_means_full_column() {
        let idx = PartitionIndex::new(hints());
        assert_eq!(idx.partition_at(5, -1000, 5), Some("a"));
        assert_eq!(idx.partition_at(5, 1000, 5), Some("a"));
    }

    #[test]
    fn id_index_is_stable_and_matches_sorted_id_order() {
        // Index assignment must not depend on input order.
        let mut shuffled = hints();
        shuffled.reverse();
        let a = PartitionIndex::new(hints());
        let b = PartitionIndex::new(shuffled);
        assert_eq!(a.id_index_at(5, 0, 5), b.id_index_at(5, 0, 5));
        assert_eq!(a.id_index_at(15, 0, 5), b.id_index_at(15, 0, 5));
    }

    #[test]
    fn hints_hash_is_order_independent_but_content_sensitive() {
        let mut reversed = hints();
        reversed.reverse();
        assert_eq!(
            PartitionIndex::new(hints()).hints_hash(),
            PartitionIndex::new(reversed).hints_hash(),
            "hints are sorted at construction, so input order cannot reach the digest"
        );

        // Same ids, different boxes: the case that used to slip through
        // `config_hash` entirely.
        let narrower = vec![
            PartitionHint { id: "a".into(), bbox_xz: (0, 8, 0, 9), y_range: None },
            PartitionHint { id: "b".into(), bbox_xz: (11, 20, 0, 9), y_range: None },
            PartitionHint { id: "c".into(), bbox_xz: (0, 9, 11, 20), y_range: Some((0, 10)) },
        ];
        assert_ne!(
            PartitionIndex::new(hints()).hints_hash(),
            PartitionIndex::new(narrower).hints_hash(),
            "hint geometry must be part of the digest"
        );

        // An unbounded column must not collide with an explicitly full one.
        let full = vec![PartitionHint {
            id: "a".into(),
            bbox_xz: (0, 9, 0, 9),
            y_range: Some((i32::MIN, i32::MAX)),
        }];
        let unbounded =
            vec![PartitionHint { id: "a".into(), bbox_xz: (0, 9, 0, 9), y_range: None }];
        assert_ne!(
            PartitionIndex::new(full).hints_hash(),
            PartitionIndex::new(unbounded).hints_hash()
        );

        // Ids must not alias through concatenation.
        let ab_c = vec![
            PartitionHint { id: "ab".into(), bbox_xz: (0, 0, 0, 0), y_range: None },
            PartitionHint { id: "c".into(), bbox_xz: (0, 0, 0, 0), y_range: None },
        ];
        let a_bc = vec![
            PartitionHint { id: "a".into(), bbox_xz: (0, 0, 0, 0), y_range: None },
            PartitionHint { id: "bc".into(), bbox_xz: (0, 0, 0, 0), y_range: None },
        ];
        assert_ne!(PartitionIndex::new(ab_c).hints_hash(), PartitionIndex::new(a_bc).hints_hash());

        // A prefix of the hint list must not collide with the whole.
        assert_ne!(
            PartitionIndex::new(hints()).hints_hash(),
            PartitionIndex::new(hints()[..2].to_vec()).hints_hash()
        );
    }

    #[test]
    fn duplicate_ids_with_different_boxes_resolve_the_same_regardless_of_input_order() {
        // Two hints sharing the same id but occupying disjoint boxes. A
        // sort keyed on `id` alone is stable and would leave these in
        // whatever relative order the caller passed them in, so which one
        // `partition_at`/`id_index_at` report first (and which `u32` index
        // it gets) would depend on input order. Sorting on the full content
        // tuple (id, bbox_xz, y_range) fixes a single order regardless of
        // how the caller supplied them.
        let dup_first = PartitionHint { id: "dup".into(), bbox_xz: (0, 4, 0, 4), y_range: None };
        let dup_second = PartitionHint { id: "dup".into(), bbox_xz: (10, 14, 0, 4), y_range: None };

        let forward = vec![dup_first.clone(), dup_second.clone()];
        let mut reversed = forward.clone();
        reversed.reverse();

        let idx_forward = PartitionIndex::new(forward);
        let idx_reversed = PartitionIndex::new(reversed);

        // Point inside the first box.
        assert_eq!(idx_forward.partition_at(2, 0, 2), idx_reversed.partition_at(2, 0, 2));
        assert_eq!(idx_forward.id_index_at(2, 0, 2), idx_reversed.id_index_at(2, 0, 2));

        // Point inside the second box.
        assert_eq!(idx_forward.partition_at(12, 0, 2), idx_reversed.partition_at(12, 0, 2));
        assert_eq!(idx_forward.id_index_at(12, 0, 2), idx_reversed.id_index_at(12, 0, 2));

        // Both boxes resolve to the same id string either way.
        assert_eq!(idx_forward.partition_at(2, 0, 2), Some("dup"));
        assert_eq!(idx_forward.partition_at(12, 0, 2), Some("dup"));
    }
}
