#![cfg(feature = "world-segment")]
//! merge must be associative, commutative and idempotent — the property that
//! lets partial stitches from many workers tree-reduce to one answer.

use nucleation::world_segment::ids::{ClusterId, ContentId, TileId};
use nucleation::world_segment::segment::{Cluster, MarginCell, TileSegments};
use nucleation::world_segment::stitch::{Build, StitchState};

fn cid(tile: TileId, anchor: (i32,i32,i32)) -> ClusterId {
    ClusterId::new(ContentId::of(&[b"t"]), tile, None, anchor)
}

// A deterministic chain of N tiles, each with one cluster one global cell from
// the next, so the whole chain should stitch into a single build.
fn chain(n: i32) -> Vec<StitchState> {
    (0..n).map(|i| {
        let id = cid(TileId{x:i, z:0}, (0,1,1));
        let seg = TileSegments {
            tile_id: TileId{x:i, z:0},
            // left edge cell (global 128*i) and right edge cell (global 128*i+127)
            clusters: vec![Cluster{ id, bbox:((0,0,0),(1,1,1)), block_count:1, cell_count:1, partition_id:None }],
            margin: vec![
                MarginCell{ cell:(0,1,1), cluster:id, partition:None },
                MarginCell{ cell:(127,1,1), cluster:id, partition:None },
            ],
        };
        StitchState::from(&seg, 4, -64)
    }).collect()
}

fn build_ids(mut b: Vec<Build>) -> Vec<ClusterId> {
    b.sort_by_key(|x| x.id);
    b.into_iter().map(|x| x.id).collect()
}

fn reduce_left(parts: Vec<StitchState>) -> StitchState {
    parts.into_iter().reduce(|a, b| StitchState::merge(a, b, 2)).unwrap()
}

#[test]
fn merge_is_commutative() {
    let mut p = chain(2);
    let (b, a) = (p.pop().unwrap(), p.pop().unwrap());
    let ab = StitchState::merge(a.clone(), b.clone(), 2).finish();
    let ba = StitchState::merge(b, a, 2).finish();
    let (ab_len, ba_len) = (ab.len(), ba.len());
    assert_eq!(build_ids(ab), build_ids(ba));
    assert_eq!(ab_len, 1, "the two-tile chain must collapse to one build");
    assert_eq!(ba_len, 1, "the two-tile chain must collapse to one build regardless of merge order");
}

#[test]
fn merge_is_associative() {
    // (a·b)·c == a·(b·c) for a 3-tile chain.
    let p = chain(3);
    let (a, b, c) = (p[0].clone(), p[1].clone(), p[2].clone());
    let left = StitchState::merge(StitchState::merge(a.clone(), b.clone(), 2), c.clone(), 2).finish();
    let right = StitchState::merge(a, StitchState::merge(b, c, 2), 2).finish();
    assert_eq!(left.len(), 1);
    assert_eq!(build_ids(left), build_ids(right));
}

#[test]
fn merge_is_idempotent() {
    let p = chain(2);
    let m = StitchState::merge(p[0].clone(), p[1].clone(), 2);
    let once = m.clone().finish();
    let twice = StitchState::merge(m.clone(), m, 2).finish();
    let (once_len, twice_len) = (once.len(), twice.len());
    let once_block_count = once[0].block_count;
    let twice_block_count = twice[0].block_count;
    assert_eq!(build_ids(once), build_ids(twice));
    assert_eq!(once_len, 1, "the two-tile chain must collapse to one build");
    assert_eq!(twice_len, 1, "re-merging the already-merged state must still collapse to one build");
    assert_eq!(once_block_count, twice_block_count, "idempotent merge must not double-count blocks");
}

#[test]
fn transitive_closure_when_endpoints_are_not_directly_adjacent() {
    // Regression for the classic incremental-resolve bug (lost transitive
    // closure). A 3-tile chain: C's cell is adjacent to B's, B's to A's, but
    // C is NOT adjacent to A. chain(3) realises exactly this: tile 0 occupies
    // global cells {0,127}, tile 1 {128,255}, tile 2 {256,383}. Tile 0<->1 and
    // tile 1<->2 touch at a 1-cell seam (within r=2R+1=5), but tile 0<->2 are
    // >=129 cells apart, far outside the neighbourhood. So A and C can only be
    // joined transitively through B via the union-find forest — never directly.
    // Both merge groupings must still collapse the chain to ONE build; an
    // incremental resolver that failed to probe b's full margin against the
    // accumulated index (or that only re-resolved genuinely-new cells) would
    // split A from C here.
    let p = chain(3);
    let (a, b, c) = (p[0].clone(), p[1].clone(), p[2].clone());
    let left = StitchState::merge(StitchState::merge(a.clone(), b.clone(), 2), c.clone(), 2).finish();
    let right = StitchState::merge(a, StitchState::merge(b, c, 2), 2).finish();
    assert_eq!(left.len(), 1, "(A·B)·C must join the whole chain via B");
    assert_eq!(right.len(), 1, "A·(B·C) must join the whole chain via B");
    assert_eq!(build_ids(left), build_ids(right));
}

#[test]
fn scale_sanity_folds_a_long_chain_into_one_build() {
    // Cheap scale check (not a benchmark). Sequentially fold a 200-tile chain.
    // The old full-rescan resolver re-probed every accumulated margin entry on
    // every merge -> O(total_margin * 125) per merge, quadratic over the fold.
    // The incremental resolver probes only each incoming tile's entries, so this
    // stays near-linear and must still yield exactly one connected build.
    let n = 200;
    let builds = reduce_left(chain(n)).finish();
    assert_eq!(builds.len(), 1, "a {n}-tile seam chain must stitch into one build");
}

#[test]
fn reduction_order_does_not_change_the_result() {
    // A 5-chain reduced left-to-right vs in a different grouping must agree.
    let p = chain(5);
    let left = reduce_left(p.clone()).finish();
    // Reduce as ((0,1),(2,3),4) then combine.
    let g1 = StitchState::merge(p[0].clone(), p[1].clone(), 2);
    let g2 = StitchState::merge(p[2].clone(), p[3].clone(), 2);
    let g = StitchState::merge(StitchState::merge(g1, g2, 2), p[4].clone(), 2).finish();
    assert_eq!(left.len(), 1, "the whole chain is one build");
    assert_eq!(build_ids(left), build_ids(g));
}
