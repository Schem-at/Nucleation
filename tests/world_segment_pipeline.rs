//! Task 6: end-to-end synthetic pipeline test.
//!
//! Drives a multi-tile world through the whole runner
//! (source -> segment -> stitch -> score -> identity -> materialize) twice,
//! proving cross-tile stitching, tiering, determinism, order-independence and
//! cross-snapshot identity all work together — not just in isolation.
//!
//! No clock, no RNG, no `HashMap`/`HashSet` reaching output. `extracted_at` is
//! always a fixed literal passed on `SegmentJob`, never `SystemTime::now()`.

#![cfg(feature = "world-segment")]

use std::collections::BTreeSet;

use nucleation::BlockState;
use nucleation::world_segment::ids::TileId;
use nucleation::world_segment::identity::PriorBuild;
use nucleation::world_segment::partition::PartitionIndex;
use nucleation::world_segment::profile::WorldProfile;
use nucleation::world_segment::provenance::Provenance;
use nucleation::world_segment::runner::{SegmentJob, WorldSegmenter};
use nucleation::world_segment::score::{ScoreConfig, Tier};
use nucleation::world_segment::segment::SegConfig;
use nucleation::world_segment::source::{region_tile_bounds, Access, TileError, TileSource};
use nucleation::world_segment::tile::{TileBounds, VoxelTile};

/// An in-memory forward source yielding a fixed list of pre-built tiles, in
/// whatever order the caller supplied them. Mirrors the `MemSource` pattern
/// already established in `tests/world_segment_sources.rs`.
struct MemSource {
    tiles: Vec<VoxelTile>,
}

impl TileSource for MemSource {
    fn access(&self) -> Access {
        Access::Forward
    }
    fn tile_ids(&self) -> Result<Vec<TileId>, TileError> {
        Ok(self.tiles.iter().map(|t| t.id()).collect())
    }
    fn tile(&self, _id: TileId) -> Result<Option<VoxelTile>, TileError> {
        Err(TileError::NotRandomAccess)
    }
    fn for_each_tile(
        &self,
        f: &mut dyn FnMut(VoxelTile) -> Result<(), TileError>,
    ) -> Result<(), TileError> {
        for t in &self.tiles {
            let fresh =
                VoxelTile::from_blocks(t.id(), *t.bounds(), t.blocks().map(|(p, b)| (p, b.clone())));
            // Honor the `TileError::Stop` early-termination sentinel: stop
            // iterating and report success, never propagate `Stop` itself.
            match f(fresh) {
                Ok(()) => {}
                Err(TileError::Stop) => return Ok(()),
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

fn profile() -> WorldProfile {
    WorldProfile::new(
        BTreeSet::from(["minecraft:stone".to_string()]),
        (-64, -60),
    )
}

fn seg_config() -> SegConfig {
    SegConfig::default() // cell_size 4, closing_radius 2, min_cluster_blocks 1
}

// ---------------------------------------------------------------------------
// Test 1: cross-tile stitch
// ---------------------------------------------------------------------------

/// Two adjacent Minecraft regions, TileId {0,0} and {1,0}. Each holds one half
/// of a redstone build straddling the x = 511/512 seam, plus a substrate slab.
///
/// With `cell_size = 4`, `closing_radius = 2` (`SegConfig::default()`), the
/// margin band is `2R+1 = 5` cells wide from each tile face, and cells merge
/// across the seam up to Chebyshev cell-distance `2R+1 = 5` apart. Placing the
/// halves at world x in [500,509] (tile 0) and [512,521] (tile 1) keeps both
/// well inside their tile's margin band (band starts at local cell 123 of 128
/// for tile 0's right face, at local cell 0..4 for tile 1's left face) and
/// puts their nearest cells (global cell 127 vs. 128) one cell apart — see
/// `stitch::tests::adjacent_clusters_across_a_seam_join` for the same
/// arithmetic pinned directly against `StitchState`.
fn cross_tile_world() -> MemSource {
    let (id0, bounds0) = region_tile_bounds(0, 0, -64, 63);
    let (id1, bounds1) = region_tile_bounds(1, 0, -64, 63);

    let mut blocks0: Vec<((i32, i32, i32), BlockState)> = Vec::new();
    // Substrate slab in tile 0.
    for x in 0..16 {
        for z in 0..16 {
            blocks0.push(((x, -62, z), BlockState::new("minecraft:stone")));
        }
    }
    // Left half of the spanning build: 10 blocks near the tile's right edge.
    for x in 500..510 {
        blocks0.push(((x, 10, 250), BlockState::new("minecraft:redstone_wire")));
    }

    let mut blocks1: Vec<((i32, i32, i32), BlockState)> = Vec::new();
    // Substrate slab in tile 1.
    for x in 512..528 {
        for z in 0..16 {
            blocks1.push(((x, -62, z), BlockState::new("minecraft:stone")));
        }
    }
    // Right half of the spanning build: 10 blocks near the tile's left edge.
    for x in 512..522 {
        blocks1.push(((x, 10, 250), BlockState::new("minecraft:redstone_wire")));
    }

    MemSource {
        tiles: vec![
            VoxelTile::from_blocks(id0, bounds0, blocks0.into_iter()),
            VoxelTile::from_blocks(id1, bounds1, blocks1.into_iter()),
        ],
    }
}

#[test]
fn cross_tile_build_stitches_into_one() {
    let source = cross_tile_world();
    let profile = profile();
    let partitions = PartitionIndex::new(vec![]);
    let job = SegmentJob {
        config: seg_config(),
        // debris_max_blocks lowered well below the spanning build's 20 blocks
        // so it is unambiguously not scored as Debris — the point of this
        // test is the stitch, not the tiering, so tier is asserted only to
        // rule out the trivial "it's debris and nobody looked" reading.
        score_config: ScoreConfig { debris_max_blocks: 5, ..ScoreConfig::default() },
        source_id: "src-stitch".to_string(),
        snapshot_id: "snap-1".to_string(),
        min_y: -64,
        max_y: 63,
        extracted_at: 1_700_000_000,
        match_iou: 0.5,
    };

    let out = WorldSegmenter::run(&source, &profile, &partitions, &job, &[]);

    assert_eq!(
        out.len(),
        1,
        "the substrate slabs vanish and the two build halves stitch into exactly one build; \
         got {} builds: {:?}",
        out.len(),
        out.iter().map(|b| (b.provenance.world_bbox, b.provenance.block_count)).collect::<Vec<_>>()
    );
    let mb = &out[0];
    assert_eq!(mb.provenance.block_count, 20, "10 blocks from each tile half");
    assert_eq!(
        mb.provenance.world_bbox,
        ((500, 10, 250), (521, 10, 250)),
        "the bbox must span both tiles' x-range, proving the two halves were stitched, \
         not just independently discovered"
    );
    assert_ne!(mb.provenance.tier, Tier::Debris, "20 blocks exceeds the lowered debris threshold");
    assert_eq!(mb.provenance.cluster_count, 2, "one absorbed cluster from each tile");
}

// ---------------------------------------------------------------------------
// Test 2: substrate removed, debris tiered but not dropped
// ---------------------------------------------------------------------------

#[test]
fn substrate_removed_debris_tiered_not_dropped() {
    let bounds = TileBounds { min: (0, -64, 0), max: (255, 63, 255) };
    let id = TileId { x: 0, z: 0 };

    let mut blocks: Vec<((i32, i32, i32), BlockState)> = Vec::new();
    // Substrate slab, 16x16, inside the profile's y band.
    for x in 0..16 {
        for z in 0..16 {
            blocks.push(((x, -62, z), BlockState::new("minecraft:stone")));
        }
    }
    // A real build: a dense 15x10 sheet of redstone wire, far from the slab
    // and from the speck below (150 blocks, one contiguous cluster).
    for x in 0..15 {
        for z in 0..10 {
            blocks.push(((100 + x, 10, 100 + z), BlockState::new("minecraft:redstone_wire")));
        }
    }
    // A 1-block speck, far from everything above (beyond the closing
    // distance in every direction), so it stays its own cluster.
    blocks.push(((200, 10, 200), BlockState::new("minecraft:redstone_wire")));

    let source = MemSource { tiles: vec![VoxelTile::from_blocks(id, bounds, blocks.into_iter())] };
    let profile = profile();
    let partitions = PartitionIndex::new(vec![]);
    let job = SegmentJob {
        config: seg_config(),
        score_config: ScoreConfig::default(), // debris_max_blocks 100
        source_id: "src-debris".to_string(),
        snapshot_id: "snap-1".to_string(),
        min_y: -64,
        max_y: 63,
        extracted_at: 1_700_000_000,
        match_iou: 0.5,
    };

    let out = WorldSegmenter::run(&source, &profile, &partitions, &job, &[]);

    assert_eq!(out.len(), 2, "the slab contributes zero builds; the real build and the speck both survive");

    let real = out.iter().find(|b| b.provenance.block_count == 150).expect("the real build must be present");
    let speck = out.iter().find(|b| b.provenance.block_count == 1).expect("the speck must be present, not dropped");

    assert_ne!(real.provenance.tier, Tier::Debris, "150 blocks exceeds debris_max_blocks (100)");
    assert_eq!(speck.provenance.tier, Tier::Debris, "a 1-block cluster is debris");

    // The slab is gone: neither surviving build contains stone, and no build
    // has anything close to the slab's 256-block volume.
    assert_eq!(
        real.schematic.get_block(0, 0, 0).map(|b| b.get_name().to_string()),
        Some("minecraft:redstone_wire".to_string())
    );
    assert!(
        out.iter().all(|b| b.provenance.block_count != 256),
        "the substrate slab must not appear as a materialized build"
    );
}

// ---------------------------------------------------------------------------
// Tests 3 & 4: determinism and order-independence
// ---------------------------------------------------------------------------

/// Two region tiles, each holding a substrate slab plus one build placed near
/// the tile's *center* — far outside the margin band on every side — so the
/// two builds can never stitch across the seam. This isolates order effects
/// on the runner's own bookkeeping (the `blocks_by_cluster` map, the
/// `StitchState` merge sequence, `match_snapshots`) from the stitch geometry
/// already covered by `cross_tile_build_stitches_into_one`.
///
/// The two builds are deliberately different sizes (3 vs. 6 blocks). A bug
/// that mixed up which tile's blocks landed under which `ClusterId` — e.g. a
/// `for_each_tile` that let a shared buffer leak between iterations instead
/// of handing over fresh per-tile block maps, or a runner that iterated
/// `blocks_by_cluster` through a `HashMap` instead of `BTreeMap` and let
/// iteration order leak into which blocks got attributed to which build —
/// would, on the reversed run, either attach the wrong block set to a
/// `ClusterId` (changing `fingerprint`) or attach it under the wrong
/// `stable_build_id` seed. Same-cardinality-but-wrong-content mistakes are
/// exactly what comparing the *set* of `(stable_build_id, fingerprint)`
/// pairs (not just `out.len()`) is designed to catch; two same-sized,
/// interchangeable builds would let such a swap slip through invisibly,
/// which is why the two builds here differ in size.
fn order_test_world(reversed: bool) -> MemSource {
    let (id0, bounds0) = region_tile_bounds(0, 0, -64, 63);
    let (id1, bounds1) = region_tile_bounds(1, 0, -64, 63);

    let mut blocks0: Vec<((i32, i32, i32), BlockState)> = Vec::new();
    for x in 0..16 {
        for z in 0..16 {
            blocks0.push(((x, -62, z), BlockState::new("minecraft:stone")));
        }
    }
    // 3-block build, dead center of region 0 (x in [0,511]).
    blocks0.push(((250, 10, 250), BlockState::new("minecraft:redstone_wire")));
    blocks0.push(((251, 10, 250), BlockState::new("minecraft:redstone_wire")));
    blocks0.push(((252, 10, 250), BlockState::new("minecraft:repeater")));

    let mut blocks1: Vec<((i32, i32, i32), BlockState)> = Vec::new();
    for x in 512..528 {
        for z in 0..16 {
            blocks1.push(((x, -62, z), BlockState::new("minecraft:stone")));
        }
    }
    // 6-block build, dead center of region 1 (x in [512,1023]).
    for i in 0..6 {
        blocks1.push(((512 + 250 + i, 10, 250), BlockState::new("minecraft:redstone_wire")));
    }

    let tile0 = VoxelTile::from_blocks(id0, bounds0, blocks0.into_iter());
    let tile1 = VoxelTile::from_blocks(id1, bounds1, blocks1.into_iter());

    let tiles = if reversed { vec![tile1, tile0] } else { vec![tile0, tile1] };
    MemSource { tiles }
}

fn order_test_job() -> SegmentJob {
    SegmentJob {
        config: seg_config(),
        score_config: ScoreConfig::default(),
        source_id: "src-order".to_string(),
        snapshot_id: "snap-1".to_string(),
        min_y: -64,
        max_y: 63,
        extracted_at: 1_700_000_123,
        match_iou: 0.5,
    }
}

#[test]
fn two_identical_runs_are_byte_identical() {
    let profile = profile();
    let partitions = PartitionIndex::new(vec![]);
    let job = order_test_job();

    let mut a: Vec<Provenance> =
        WorldSegmenter::run(&order_test_world(false), &profile, &partitions, &job, &[])
            .into_iter()
            .map(|b| b.provenance)
            .collect();
    let mut b: Vec<Provenance> =
        WorldSegmenter::run(&order_test_world(false), &profile, &partitions, &job, &[])
            .into_iter()
            .map(|b| b.provenance)
            .collect();

    a.sort_by_key(|p| p.stable_build_id);
    b.sort_by_key(|p| p.stable_build_id);

    assert_eq!(a.len(), 2, "the two center builds must not stitch across the seam");
    // Full provenance equality, not merely a count or a stable-id set: this
    // also pins `fingerprint`, `world_bbox`, `block_count`, `tier`,
    // `config_hash` and `profile_hash` being identical across runs. A clock
    // read in `materialize` (instead of using `job.extracted_at`) or a
    // `HashMap` anywhere upstream of these fields would make this fail even
    // though both runs are, by construction, given identical input.
    assert_eq!(a, b, "two runs of the same world with the same extracted_at must be byte-identical");
}

#[test]
fn tile_order_does_not_change_the_result() {
    let profile = profile();
    let partitions = PartitionIndex::new(vec![]);
    let job = order_test_job();

    let forward = WorldSegmenter::run(&order_test_world(false), &profile, &partitions, &job, &[]);
    let backward = WorldSegmenter::run(&order_test_world(true), &profile, &partitions, &job, &[]);

    assert_eq!(forward.len(), 2);
    assert_eq!(backward.len(), 2);

    let forward_set: BTreeSet<(nucleation::world_segment::provenance::StableBuildId, u128)> =
        forward.iter().map(|b| (b.provenance.stable_build_id, b.provenance.fingerprint)).collect();
    let backward_set: BTreeSet<(nucleation::world_segment::provenance::StableBuildId, u128)> =
        backward.iter().map(|b| (b.provenance.stable_build_id, b.provenance.fingerprint)).collect();

    assert_eq!(
        forward_set, backward_set,
        "reversing tile order must not change the set of (stable_build_id, fingerprint) pairs"
    );

    // Pin the concrete block counts too, so a bug that preserved the right
    // *set* of ids by accident (e.g. by swapping both id AND content
    // together) cannot slip past a same-size coincidence: the two builds
    // differ in size (3 vs. 6 blocks), so a genuine mix-up changes which
    // fingerprint pairs with which stable id, which the set comparison above
    // already catches — this is a second, independent confirmation via a
    // human-legible signal.
    let mut counts: Vec<u64> = forward.iter().map(|b| b.provenance.block_count).collect();
    counts.sort();
    assert_eq!(counts, vec![3, 6]);
    let mut counts_rev: Vec<u64> = backward.iter().map(|b| b.provenance.block_count).collect();
    counts_rev.sort();
    assert_eq!(counts_rev, vec![3, 6]);
}

// ---------------------------------------------------------------------------
// Test 5: cross-snapshot identity
// ---------------------------------------------------------------------------

#[test]
fn edited_build_inherits_stable_id_new_fingerprint() {
    let bounds = TileBounds { min: (0, -64, 0), max: (63, 63, 63) };
    let id = TileId { x: 0, z: 0 };
    let profile = profile();
    let partitions = PartitionIndex::new(vec![]);

    // Snapshot 1: a 10-block line.
    let mut blocks1: Vec<((i32, i32, i32), BlockState)> = Vec::new();
    for x in 0..10 {
        blocks1.push(((x, 10, 10), BlockState::new("minecraft:redstone_wire")));
    }
    let source1 = MemSource { tiles: vec![VoxelTile::from_blocks(id, bounds, blocks1.into_iter())] };
    let job1 = SegmentJob {
        config: seg_config(),
        score_config: ScoreConfig::default(),
        source_id: "src-identity".to_string(),
        snapshot_id: "snap-1".to_string(),
        min_y: -64,
        max_y: 63,
        extracted_at: 1_700_000_000,
        match_iou: 0.5,
    };
    let out1 = WorldSegmenter::run(&source1, &profile, &partitions, &job1, &[]);
    assert_eq!(out1.len(), 1);
    let snap1 = &out1[0];

    let prior = vec![PriorBuild {
        stable_id: snap1.provenance.stable_build_id,
        bbox: snap1.provenance.world_bbox,
        block_count: snap1.provenance.block_count,
    }];

    // Snapshot 2: same world, same build location, edited — 5 more blocks
    // appended to the same line. bbox grows from x:[0,9] to x:[0,14]
    // (volume 10 -> 15); the old bbox is fully contained in the new one, so
    // IoU = 10/15 ≈ 0.667, comfortably above the 0.5 threshold, and this is
    // the *only* current build, so `match_snapshots` takes the one-prior,
    // one-current branch: `Outcome::Same`, identity inherited unchanged.
    let mut blocks2: Vec<((i32, i32, i32), BlockState)> = Vec::new();
    for x in 0..15 {
        blocks2.push(((x, 10, 10), BlockState::new("minecraft:redstone_wire")));
    }
    let source2 = MemSource { tiles: vec![VoxelTile::from_blocks(id, bounds, blocks2.into_iter())] };
    let job2 = SegmentJob { snapshot_id: "snap-2".to_string(), extracted_at: 1_700_001_000, ..job1 };
    let out2 = WorldSegmenter::run(&source2, &profile, &partitions, &job2, &prior);
    assert_eq!(out2.len(), 1);
    let snap2 = &out2[0];

    assert_eq!(
        snap2.provenance.stable_build_id, snap1.provenance.stable_build_id,
        "the edited build must inherit snapshot 1's stable_build_id"
    );
    assert_ne!(
        snap2.provenance.fingerprint, snap1.provenance.fingerprint,
        "the edit (5 more blocks) must be detected: the fingerprint must differ"
    );
    assert_eq!(snap1.provenance.block_count, 10);
    assert_eq!(snap2.provenance.block_count, 15, "the extra blocks must actually be present");
}
