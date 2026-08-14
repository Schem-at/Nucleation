//! End-to-end confirmation on the ACTUAL flagged build `d44bbed2`.
//!
//! Loads the real `.schem`, feeds its non-air voxels through `segment_tile`
//! with the driver-default split config (min_component_blocks 4096, share 0.40,
//! gap 2 cells), and asserts the morphological-closing merge is undone into
//! exactly two substantial build segments (build A ~184k, build B ~179k), with
//! the minor fragments/floor attaching to the nearest seed rather than becoming
//! their own segments. Not part of the module suite's contract — a throwaway
//! confirmation harness kept in the tree per request.
#![cfg(feature = "world-segment")]

use nucleation::world_segment::ids::TileId;
use nucleation::world_segment::partition::{PartitionIndex, PartitionPolicy};
use nucleation::world_segment::profile::WorldProfile;
use nucleation::world_segment::segment::{segment_tile, DisconnectedSplit, SegConfig};
use nucleation::world_segment::tile::{TileBounds, VoxelTile};
use nucleation::{BlockState, UniversalSchematic};
use std::collections::BTreeSet;

const SCHEM: &str = "wol-project/m10-full/d44bbed2cdad55d93551056046accb7b.schem";

#[test]
#[ignore = "requires the local ORE d44bbed2 corpus fixture"]
fn d44bbed2_real_voxels_split_into_two_substantial_builds() {
    let data = std::fs::read(SCHEM).expect("read d44bbed2.schem");
    let schem = UniversalSchematic::from_schematic(&data).expect("parse d44bbed2.schem");

    let air = ["minecraft:air", "minecraft:cave_air", "minecraft:void_air"];
    let mut blocks: Vec<((i32, i32, i32), BlockState)> = Vec::new();
    let (mut mnx, mut mny, mut mnz) = (i32::MAX, i32::MAX, i32::MAX);
    let (mut mxx, mut mxy, mut mxz) = (i32::MIN, i32::MIN, i32::MIN);
    for (pos, state) in schem.iter_blocks() {
        if air.contains(&state.get_name()) {
            continue;
        }
        let p = (pos.x, pos.y, pos.z);
        mnx = mnx.min(p.0);
        mny = mny.min(p.1);
        mnz = mnz.min(p.2);
        mxx = mxx.max(p.0);
        mxy = mxy.max(p.1);
        mxz = mxz.max(p.2);
        blocks.push((p, state.clone()));
    }
    let total = blocks.len() as u64;
    let bounds = TileBounds {
        min: (mnx, mny, mnz),
        max: (mxx, mxy, mxz),
    };
    let tile = VoxelTile::from_blocks(TileId { x: 0, z: 0 }, bounds, blocks.into_iter());

    // Empty substrate palette: nothing is dropped as ground, so every non-air
    // voxel is segmented — the floor slab included, exactly the state that
    // produced the over-merged build in the real run.
    let profile = WorldProfile::new(BTreeSet::new(), (mny, mny));

    // Driver defaults: cell 4, R 2, split at 4096 / 0.40 / 2 cells.
    let cfg_on = SegConfig {
        cell_size: 4,
        closing_radius: 2,
        min_cluster_blocks: 1,
        partition_policy: PartitionPolicy::Off,
        split_disconnected: Some(DisconnectedSplit::default()),
        ..SegConfig::default()
    };
    let cfg_off = SegConfig {
        split_disconnected: None,
        ..cfg_on.clone()
    };
    let no_hints = PartitionIndex::new(vec![]);

    let off = segment_tile(&tile, &profile, &cfg_off, &no_hints);
    let on = segment_tile(&tile, &profile, &cfg_on, &no_hints);

    let mut counts_on: Vec<u64> = on.clusters.iter().map(|c| c.block_count).collect();
    counts_on.sort_unstable_by(|a, b| b.cmp(a));
    let bboxes: Vec<_> = {
        let mut cs: Vec<&nucleation::world_segment::segment::Cluster> =
            on.clusters.iter().collect();
        cs.sort_by_key(|c| std::cmp::Reverse(c.block_count));
        cs.iter().take(2).map(|c| (c.block_count, c.bbox)).collect()
    };

    eprintln!("D44 non-air voxels = {total}");
    eprintln!("D44 segments WITHOUT split = {}", off.clusters.len());
    eprintln!("D44 segments WITH split    = {}", on.clusters.len());
    eprintln!("D44 WITH-split block counts (desc) = {counts_on:?}");
    eprintln!("D44 top-2 (blocks, bbox) = {bboxes:?}");

    // The bug: without the split the whole plot is one merged build.
    assert_eq!(
        off.clusters.len(),
        1,
        "closing merges d44bbed2 into one build"
    );

    // The fix: exactly two substantial (>= 50k-block) build segments, matching
    // build A (~184k) and build B (~179k); everything else attached to a seed.
    let substantial: Vec<u64> = counts_on.iter().copied().filter(|&c| c >= 50_000).collect();
    assert_eq!(
        substantial.len(),
        2,
        "exactly two substantial build segments after split"
    );
    assert!(substantial[0] >= 150_000, "build A is plot-scale");
    assert!(substantial[1] >= 150_000, "build B is plot-scale");
    // No block is lost: the split partitions the whole merged cluster.
    assert_eq!(
        counts_on.iter().sum::<u64>(),
        total,
        "every block keeps a home"
    );
}
