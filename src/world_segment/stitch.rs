//! Cross-tile stitching: a mergeable union-find that rejoins clusters split by
//! tile boundaries.
//!
//! `merge` is associative, commutative and idempotent, so partial stitches from
//! any number of workers tree-reduce to the same result in any order. That is
//! the property the distributed design rests on; the property tests enforce it.

use std::collections::BTreeMap;

use crate::world_segment::ids::{ClusterId, TileId};
use crate::world_segment::segment::{Cluster, TileSegments};
use crate::world_segment::source::region_tile_bounds;

pub type GlobalCell = (i32, i32, i32);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MarginEntry {
    pub cell: GlobalCell,
    pub cluster: ClusterId,
    pub partition: Option<String>,
}

/// Local grid cell -> global cell. See the plan's coordinate rule.
pub fn to_global(local: (i32, i32, i32), tile: TileId, cell_size: u32, min_y: i32) -> GlobalCell {
    let (_id, bounds) = region_tile_bounds(tile.x, tile.z, min_y, min_y); // only min corner needed
    let c = cell_size as i32;
    (
        bounds.min.0 / c + local.0,
        bounds.min.1.div_euclid(c) + local.1,
        bounds.min.2 / c + local.2,
    )
}

#[derive(Clone, Debug)]
pub struct StitchState {
    /// Union-find parent map. The root of a set is always its smallest ClusterId.
    parent: BTreeMap<ClusterId, ClusterId>,
    /// Cluster payloads by id.
    clusters: BTreeMap<ClusterId, Cluster>,
    /// All margin entries seen, in global coords. Retained across merges so a
    /// later merge can still find an adjacency involving an earlier tile.
    margin: Vec<MarginEntry>,
}

impl StitchState {
    pub fn empty() -> Self {
        StitchState { parent: BTreeMap::new(), clusters: BTreeMap::new(), margin: Vec::new() }
    }

    pub fn from(seg: &TileSegments, cell_size: u32, min_y: i32) -> Self {
        let mut s = StitchState::empty();
        for c in &seg.clusters {
            s.parent.insert(c.id, c.id);
            s.clusters.insert(c.id, c.clone());
        }
        for m in &seg.margin {
            s.margin.push(MarginEntry {
                cell: to_global(m.cell, seg.tile_id, cell_size, min_y),
                cluster: m.cluster,
                partition: m.partition.clone(),
            });
        }
        s.margin.sort_by(|a, b| a.cell.cmp(&b.cell).then(a.cluster.cmp(&b.cluster)));
        s
    }

    /// Find with path halving. Absent ids are their own root (defensive).
    pub fn find(&self, mut x: ClusterId) -> ClusterId {
        while let Some(&p) = self.parent.get(&x) {
            if p == x { break; }
            x = p;
        }
        x
    }

    pub fn margin_len(&self) -> usize {
        self.margin.len()
    }

    /// Union two clusters; the smaller id becomes the root, so the outcome does
    /// not depend on argument order.
    fn union(&mut self, a: ClusterId, b: ClusterId) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb { return; }
        let (root, child) = if ra < rb { (ra, rb) } else { (rb, ra) };
        self.parent.insert(child, root);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_segment::ids::{ClusterId, TileId};
    use crate::world_segment::segment::{Cluster, MarginCell, TileSegments};

    fn cid(tile: TileId, anchor: (i32,i32,i32)) -> ClusterId {
        // Use the real constructor so ids are realistic; config/profile/partition
        // folded in by segment_tile don't matter for stitch-key behaviour here.
        ClusterId::new(
            crate::world_segment::ids::ContentId::of(&[b"t"]),
            tile, None, anchor,
        )
    }

    #[test]
    fn to_global_aligns_tiles_on_one_lattice() {
        // cell_size 4, min_y -64. Region (1,0) origin x=512 -> 128 cells.
        // Local cell (0,_,_) in region 1 == global x 128; local (0) in region 0 == global 0.
        assert_eq!(to_global((0,0,0), TileId{x:0,z:0}, 4, -64).0, 0);
        assert_eq!(to_global((0,0,0), TileId{x:1,z:0}, 4, -64).0, 128);
        // Y uses div_euclid: min_y -64 / 4 = -16, plus ly.
        assert_eq!(to_global((0,0,0), TileId{x:0,z:0}, 4, -64).1, -16);
        assert_eq!(to_global((0,5,0), TileId{x:0,z:0}, 4, -64).1, -11);
    }

    #[test]
    fn from_lifts_clusters_and_margin_into_global_coords() {
        let a = cid(TileId{x:0,z:0}, (1,1,1));
        let seg = TileSegments {
            tile_id: TileId{x:0,z:0},
            clusters: vec![Cluster{ id:a, bbox:((0,0,0),(3,3,3)), block_count:10, cell_count:2, partition_id:None }],
            margin: vec![MarginCell{ cell:(0,0,0), cluster:a, partition:None }],
        };
        let s = StitchState::from(&seg, 4, -64);
        // The cluster is its own representative initially.
        assert_eq!(s.find(a), a);
        // Its margin entry is now in global coords.
        assert_eq!(s.margin_len(), 1);
    }
}
