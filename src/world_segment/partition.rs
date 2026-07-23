//! Caller-supplied partition hints: named boxes a cluster may never span.
//!
//! Deliberately generic. This module has no idea what the boxes represent —
//! land parcels, plot claims, administrative regions. The caller assigns
//! meaning; segmentation only enforces the boundaries.

use serde::{Deserialize, Serialize};

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
    /// Crossing is allowed but recorded, for measuring how often it happens.
    Prefer,
    /// Hints are ignored entirely.
    Off,
}

/// Point-in-partition lookup.
///
/// Hints are sorted by id at construction so that index assignment is
/// independent of the order the caller supplied them in — otherwise two
/// workers given the same hints in different orders would disagree.
pub struct PartitionIndex {
    hints: Vec<PartitionHint>,
}

impl PartitionIndex {
    pub fn new(mut hints: Vec<PartitionHint>) -> Self {
        hints.sort_by(|a, b| a.id.cmp(&b.id));
        PartitionIndex { hints }
    }

    pub fn is_empty(&self) -> bool {
        self.hints.is_empty()
    }

    pub fn partition_at(&self, x: i32, y: i32, z: i32) -> Option<&str> {
        self.hints.iter().find(|h| h.contains(x, y, z)).map(|h| h.id.as_str())
    }

    /// Stable numeric handle for the partition at this point, for cheap
    /// per-cell comparison during dilation.
    pub fn id_index_at(&self, x: i32, y: i32, z: i32) -> Option<u32> {
        self.hints.iter().position(|h| h.contains(x, y, z)).map(|i| i as u32)
    }

    pub fn id_of_index(&self, index: u32) -> &str {
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
}
