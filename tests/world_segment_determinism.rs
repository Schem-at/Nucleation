//! The property the whole design rests on: same input + same config produces
//! byte-identical output, whatever order the work is done in.

#![cfg(feature = "world-segment")]

// `block_state` is a private module re-exported at the crate root, so external
// tests must use `nucleation::BlockState`, not `nucleation::block_state::…`.
use nucleation::BlockState;
use nucleation::world_segment::ids::TileId;
use nucleation::world_segment::partition::{PartitionHint, PartitionIndex};
use nucleation::world_segment::profile::WorldProfile;
use nucleation::world_segment::segment::{segment_tile, SegConfig, TileSegments};
use nucleation::world_segment::tile::{TileBounds, VoxelTile};

fn profile() -> WorldProfile {
    WorldProfile::new(
        ["minecraft:stone", "minecraft:bedrock"].iter().map(|s| s.to_string()).collect(),
        (-64, -50),
    )
}

fn bounds() -> TileBounds {
    TileBounds { min: (0, -64, 0), max: (255, 127, 255) }
}

/// A deterministic pseudo-random scene: no RNG, just an arithmetic walk so the
/// scene is identical on every run and on every machine.
fn scene() -> Vec<((i32, i32, i32), BlockState)> {
    let mut out = Vec::new();
    // Ground slab.
    for x in 0..64 {
        for z in 0..64 {
            out.push(((x, -60, z), BlockState::new("minecraft:stone")));
        }
    }
    // Several clumps of build blocks at varying separations.
    let mut seed: i64 = 12345;
    for i in 0..300 {
        seed = (seed * 1103515245 + 12345) % 2147483648;
        let x = (seed.unsigned_abs() % 240) as i32;
        seed = (seed * 1103515245 + 12345) % 2147483648;
        let z = (seed.unsigned_abs() % 240) as i32;
        let y = 10 + (i % 20);
        out.push(((x, y, z), BlockState::new("minecraft:redstone_wire")));
        out.push(((x + 1, y, z), BlockState::new("minecraft:repeater")));
    }
    out
}

fn config() -> SegConfig {
    SegConfig { cell_size: 4, closing_radius: 2, min_cluster_blocks: 1, ..SegConfig::default() }
}

fn run(blocks: Vec<((i32, i32, i32), BlockState)>, hints: &PartitionIndex) -> TileSegments {
    let tile = VoxelTile::from_blocks(TileId { x: 0, z: 0 }, bounds(), blocks.into_iter());
    segment_tile(&tile, &profile(), &config(), hints)
}

#[test]
fn identical_runs_produce_identical_output() {
    let empty = PartitionIndex::new(vec![]);
    let a = run(scene(), &empty);
    let b = run(scene(), &empty);
    assert_eq!(a, b);
}

#[test]
fn block_insertion_order_does_not_affect_output() {
    let empty = PartitionIndex::new(vec![]);
    let forward = run(scene(), &empty);

    let mut reversed = scene();
    reversed.reverse();
    let backward = run(reversed, &empty);

    assert_eq!(forward, backward, "reversing input order changed the result");
}

#[test]
fn interleaved_input_order_does_not_affect_output() {
    let empty = PartitionIndex::new(vec![]);
    let expected = run(scene(), &empty);

    // Deal the blocks into four piles and concatenate: a very different order,
    // same set.
    let all = scene();
    let mut shuffled = Vec::with_capacity(all.len());
    for offset in 0..4 {
        for (i, b) in all.iter().enumerate() {
            if i % 4 == offset {
                shuffled.push(b.clone());
            }
        }
    }
    assert_eq!(run(shuffled, &empty), expected);
}

#[test]
fn output_serializes_stably() {
    let empty = PartitionIndex::new(vec![]);
    let a = bincode::serialize(&run(scene(), &empty)).expect("serialize");
    let b = bincode::serialize(&run(scene(), &empty)).expect("serialize");
    assert_eq!(a, b, "serialized bytes must match exactly");
}

/// Weak by construction: these three hints are pairwise disjoint and have
/// unique ids, which is precisely the configuration in which hint order
/// *cannot* matter — every point matches at most one hint, so first-match
/// selection is unambiguous regardless of vector order, and `id_of_index`
/// resolves any index back through the same vector to the same id string.
/// This test would still pass even with `PartitionIndex::new`'s sort deleted
/// outright. See `overlapping_hint_order_does_not_affect_output` below for
/// the case that actually exercises the sort.
#[test]
fn disjoint_hint_order_does_not_affect_output() {
    use nucleation::world_segment::partition::PartitionPolicy;

    let hints = vec![
        PartitionHint { id: "p1".into(), bbox_xz: (0, 127, 0, 127), y_range: None },
        PartitionHint { id: "p2".into(), bbox_xz: (128, 255, 0, 127), y_range: None },
        PartitionHint { id: "p3".into(), bbox_xz: (0, 127, 128, 255), y_range: None },
    ];
    let mut reversed = hints.clone();
    reversed.reverse();

    let cfg = SegConfig { partition_policy: PartitionPolicy::HardCut, ..config() };
    let tile_a = VoxelTile::from_blocks(TileId { x: 0, z: 0 }, bounds(), scene().into_iter());
    let tile_b = VoxelTile::from_blocks(TileId { x: 0, z: 0 }, bounds(), scene().into_iter());

    let a = segment_tile(&tile_a, &profile(), &cfg, &PartitionIndex::new(hints));
    let b = segment_tile(&tile_b, &profile(), &cfg, &PartitionIndex::new(reversed));
    assert_eq!(a, b, "hint order must not reach the output");
}

/// The discriminating case the test above is missing: two distinct ids whose
/// boxes *overlap*. First-match selection means a point in the overlap
/// resolves to whichever hint is scanned first. `PartitionIndex::new` sorts
/// hints by (id, bbox_xz, y_range) precisely so that "first" is a property of
/// the hint content, not of the order the caller happened to supply them in.
///
/// `"a"` spans x in [0, 200] and `"b"` spans x in [100, 255] (both full z,
/// full y), so they overlap on x in [100, 200] — a span the `scene()` helper
/// definitely places blocks across. Without the sort, the forward-ordered
/// vector resolves that overlap to `"a"` and the reversed vector resolves it
/// to `"b"`, so clusters straddling the overlap get different
/// `partition_id`s and the two `TileSegments` differ. With the sort, both
/// orderings agree ("a" sorts before "b" regardless of input order) and the
/// assertion holds.
#[test]
fn overlapping_hint_order_does_not_affect_output() {
    use nucleation::world_segment::partition::PartitionPolicy;

    let hints = vec![
        PartitionHint { id: "a".into(), bbox_xz: (0, 200, 0, 255), y_range: None },
        PartitionHint { id: "b".into(), bbox_xz: (100, 255, 0, 255), y_range: None },
    ];
    let mut reversed = hints.clone();
    reversed.reverse();

    let cfg = SegConfig { partition_policy: PartitionPolicy::HardCut, ..config() };
    let tile_a = VoxelTile::from_blocks(TileId { x: 0, z: 0 }, bounds(), scene().into_iter());
    let tile_b = VoxelTile::from_blocks(TileId { x: 0, z: 0 }, bounds(), scene().into_iter());

    let a = segment_tile(&tile_a, &profile(), &cfg, &PartitionIndex::new(hints));
    let b = segment_tile(&tile_b, &profile(), &cfg, &PartitionIndex::new(reversed));
    assert_eq!(a, b, "overlapping hint order must not reach the output");
}

#[test]
fn no_cluster_spans_a_partition_boundary_under_hard_cut() {
    use nucleation::world_segment::partition::PartitionPolicy;

    let hints = vec![
        PartitionHint { id: "p1".into(), bbox_xz: (0, 127, 0, 255), y_range: None },
        PartitionHint { id: "p2".into(), bbox_xz: (128, 255, 0, 255), y_range: None },
    ];
    let cfg = SegConfig { partition_policy: PartitionPolicy::HardCut, ..config() };
    let tile = VoxelTile::from_blocks(TileId { x: 0, z: 0 }, bounds(), scene().into_iter());
    let segs = segment_tile(&tile, &profile(), &cfg, &PartitionIndex::new(hints));

    for c in &segs.clusters {
        let spans = c.bbox.0 .0 <= 127 && c.bbox.1 .0 >= 128;
        assert!(!spans, "cluster {} spans the boundary: {:?}", c.id, c.bbox);
    }
}
