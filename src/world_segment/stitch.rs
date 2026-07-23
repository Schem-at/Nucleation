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
        bounds.min.0.div_euclid(c) + local.0,
        bounds.min.1.div_euclid(c) + local.1,
        bounds.min.2.div_euclid(c) + local.2,
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

    /// Plain walk to the root; no path compression, since `find` takes `&self`.
    /// Absent ids are their own root (defensive).
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

    /// Combine two stitch states: union each forest, concatenate cluster
    /// payloads and margin entries, then re-resolve cross-tile adjacencies.
    /// Associative, commutative and idempotent (see module docs).
    pub fn merge(mut a: StitchState, b: StitchState, closing_radius: u32) -> StitchState {
        // Fold b's forest and payloads into a.
        for (id, c) in b.clusters {
            a.parent.entry(id).or_insert(id);
            a.clusters.entry(id).or_insert(c);
        }
        for (child, parent) in b.parent {
            // Re-apply b's unions through a's smaller-id-wins rule.
            a.union(child, parent);
        }
        a.margin.extend(b.margin);
        a.margin.sort_by(|x, y| x.cell.cmp(&y.cell).then(x.cluster.cmp(&y.cluster)));
        a.margin.dedup();
        a.resolve_adjacencies(closing_radius);
        a
    }

    fn resolve_adjacencies(&mut self, closing_radius: u32) {
        let r = (2 * closing_radius + 1) as i32;
        // Spatial index: global cell -> entries there.
        let mut index: BTreeMap<GlobalCell, Vec<(ClusterId, Option<String>)>> = BTreeMap::new();
        for e in &self.margin {
            index.entry(e.cell).or_default().push((e.cluster, e.partition.clone()));
        }
        // Collect unions first (do not mutate the forest while iterating).
        let mut to_union: Vec<(ClusterId, ClusterId)> = Vec::new();
        for e in &self.margin {
            for dx in -r..=r {
                for dy in -r..=r {
                    for dz in -r..=r {
                        let n = (e.cell.0 + dx, e.cell.1 + dy, e.cell.2 + dz);
                        if let Some(others) = index.get(&n) {
                            for (oc, op) in others {
                                if *oc != e.cluster && *op == e.partition {
                                    to_union.push((e.cluster, *oc));
                                }
                            }
                        }
                    }
                }
            }
        }
        for (a, b) in to_union {
            self.union(a, b);
        }
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

    fn seg_with(tile: TileId, id: ClusterId, cell: (i32,i32,i32), part: Option<&str>) -> TileSegments {
        TileSegments {
            tile_id: tile,
            clusters: vec![Cluster{ id, bbox:((0,0,0),(1,1,1)), block_count:5, cell_count:1,
                                    partition_id: part.map(|s| s.to_string()) }],
            margin: vec![MarginCell{ cell, cluster:id, partition: part.map(|s| s.to_string()) }],
        }
    }

    #[test]
    fn adjacent_clusters_across_a_seam_join() {
        // Tile (0,0) right edge cell and tile (1,0) left edge cell, one global
        // cell apart -> within 2R+1 -> same build.
        let a = cid(TileId{x:0,z:0}, (127,1,1));
        let b = cid(TileId{x:1,z:0}, (0,1,1));
        // region 0 local x=127 -> global 127; region 1 local x=0 -> global 128. Distance 1.
        let sa = StitchState::from(&seg_with(TileId{x:0,z:0}, a, (127,1,1), None), 4, -64);
        let sb = StitchState::from(&seg_with(TileId{x:1,z:0}, b, (0,1,1), None), 4, -64);
        let m = StitchState::merge(sa, sb, 2);
        assert_eq!(m.find(a), m.find(b), "one global cell apart must join");
    }

    #[test]
    fn distant_clusters_do_not_join() {
        let a = cid(TileId{x:0,z:0}, (0,1,1));
        let b = cid(TileId{x:1,z:0}, (100,1,1)); // far inside region 1 -> global 228, distance >> 2R+1
        let sa = StitchState::from(&seg_with(TileId{x:0,z:0}, a, (0,1,1), None), 4, -64);
        let sb = StitchState::from(&seg_with(TileId{x:1,z:0}, b, (100,1,1), None), 4, -64);
        let m = StitchState::merge(sa, sb, 2);
        assert_ne!(m.find(a), m.find(b));
    }

    #[test]
    fn clusters_in_different_partitions_never_join() {
        // Same geometry as the joining case, but different partitions.
        let a = cid(TileId{x:0,z:0}, (127,1,1));
        let b = cid(TileId{x:1,z:0}, (0,1,1));
        let sa = StitchState::from(&seg_with(TileId{x:0,z:0}, a, (127,1,1), Some("L")), 4, -64);
        let sb = StitchState::from(&seg_with(TileId{x:1,z:0}, b, (0,1,1), Some("R")), 4, -64);
        let m = StitchState::merge(sa, sb, 2);
        assert_ne!(m.find(a), m.find(b), "different partitions must not union");
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
        // Negative region: region -1 origin x = -512, /4 = -128.
        assert_eq!(to_global((0,0,0), TileId{x:-1,z:0}, 4, -64).0, -128);
        // Non-divisor cell_size on a negative region: this is the case that
        // discriminates truncating `/` from `div_euclid`. Region -1 origin
        // x = -512; -512.div_euclid(6) == -86, but -512 / 6 == -85
        // (truncates toward zero). Only div_euclid gives -86.
        assert_eq!(to_global((0,0,0), TileId{x:-1,z:0}, 6, -64).0, -86);
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
