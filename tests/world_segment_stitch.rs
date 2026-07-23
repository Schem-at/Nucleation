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
    assert_eq!(build_ids(ab), build_ids(ba));
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
    assert_eq!(build_ids(once), build_ids(twice));
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
