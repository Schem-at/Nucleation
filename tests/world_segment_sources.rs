//! End-to-end: source streaming -> profile derivation -> segmentation, proving
//! the seam between a `TileSource` and the pure segmentation core is
//! order-independent.
//!
//! `region_tile_bounds` / `chunk_region` / `parse_region_coords` /
//! `region_outside_border` already have unit tests colocated with their
//! definitions (`src/world_segment/source.rs`, `world_source.rs`,
//! `targz_source.rs`); this file does not duplicate them. No small real
//! `.mca`/tarball world fixture exists under `tests/fixtures/` at the time of
//! writing, so the dir-vs-expected parity case mentioned in the task brief is
//! not added — noted here rather than skipped silently.

#![cfg(feature = "world-segment")]

use std::collections::{BTreeMap, BTreeSet};

use nucleation::BlockState;
use nucleation::world_segment::ids::TileId;
use nucleation::world_segment::partition::PartitionIndex;
use nucleation::world_segment::profile::{ProfileParams, WorldProfile};
use nucleation::world_segment::segment::{segment_tile, SegConfig};
use nucleation::world_segment::source::{region_tile_bounds, Access, TileError, TileSource, REGION_BLOCKS};
use nucleation::world_segment::tile::{TileBounds, VoxelTile};

/// An in-memory forward source, to exercise the trait without file I/O.
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
            // Clone-construct a fresh tile to hand ownership to the callback.
            // `TileBounds` is `Clone, Copy` (see `tile.rs`), so `*t.bounds()`
            // is a plain copy, not a move out of a borrow — this keeps the
            // borrow checker happy without changing what the brief intended.
            let fresh =
                VoxelTile::from_blocks(t.id(), *t.bounds(), t.blocks().map(|(p, b)| (p, b.clone())));
            f(fresh)?;
        }
        Ok(())
    }
}

fn flat_tile(id: TileId, ox: i32, oz: i32) -> VoxelTile {
    let mut blocks = vec![];
    for x in 0..16 {
        for z in 0..16 {
            for y in -64..=-61 {
                blocks.push(((ox + x, y, oz + z), BlockState::new("minecraft:stone")));
            }
        }
    }
    blocks.push(((ox + 3, -59, oz + 3), BlockState::new("minecraft:redstone_wire")));
    VoxelTile::from_blocks(
        id,
        TileBounds { min: (ox, -64, oz), max: (ox + 15, 63, oz + 15) },
        blocks.into_iter(),
    )
}

#[test]
fn derive_then_segment_end_to_end() {
    let tile = flat_tile(TileId { x: 0, z: 0 }, 0, 0);
    let profile = WorldProfile::derive(&[flat_tile(TileId { x: 0, z: 0 }, 0, 0)], &ProfileParams::default());
    // The stone slab is substrate; the redstone survives as one cluster.
    let cfg = SegConfig::default();
    let segs = segment_tile(&tile, &profile, &cfg, &PartitionIndex::new(vec![]));
    assert_eq!(segs.clusters.len(), 1, "slab removed, one build remains");
    assert!(profile.substrate_palette.contains("minecraft:stone"));
}

/// A tile at Minecraft-region granularity (via the real `region_tile_bounds`
/// helper), with a small stone slab near its own origin plus one artificial
/// block per `(dx, y, dz)` offset in `builds` (`dx`/`dz` relative to the
/// tile's own origin, `y` absolute).
///
/// Widening the bounds to a full 512-block region (rather than reusing
/// `flat_tile`'s 16-block bounds) is what lets `builds` place blocks far
/// enough apart to land in separate clusters within a single tile.
fn region_tile(region_x: i32, region_z: i32, builds: &[(i32, i32, i32)]) -> VoxelTile {
    let (id, bounds) = region_tile_bounds(region_x, region_z, -64, 63);
    let ox = region_x * REGION_BLOCKS;
    let oz = region_z * REGION_BLOCKS;

    let mut blocks = vec![];
    for x in 0..16 {
        for z in 0..16 {
            for y in -64..=-61 {
                blocks.push(((ox + x, y, oz + z), BlockState::new("minecraft:stone")));
            }
        }
    }
    for (dx, y, dz) in builds {
        blocks.push(((ox + dx, *y, oz + dz), BlockState::new("minecraft:redstone_wire")));
    }
    VoxelTile::from_blocks(id, bounds, blocks.into_iter())
}

/// Per-tile cluster shape (sorted bboxes), keyed by `TileId` in a `BTreeMap`
/// so the comparison itself cannot be order-sensitive.
fn stream_shape(
    src: &MemSource,
    profile: &WorldProfile,
    cfg: &SegConfig,
    idx: &PartitionIndex,
) -> BTreeMap<TileId, Vec<((i32, i32, i32), (i32, i32, i32))>> {
    let mut out: BTreeMap<TileId, Vec<((i32, i32, i32), (i32, i32, i32))>> = BTreeMap::new();
    src.for_each_tile(&mut |t| {
        let segs = segment_tile(&t, profile, cfg, idx);
        let mut bboxes: Vec<_> = segs.clusters.iter().map(|c| c.bbox).collect();
        bboxes.sort();
        out.insert(segs.tile_id, bboxes);
        Ok(())
    })
    .unwrap();
    out
}

/// Streaming two *different* tiles (one 1-cluster tile, one 2-cluster tile)
/// through `MemSource::for_each_tile` in each order must yield the identical
/// per-tile cluster shape either way.
///
/// This is deliberately not a single-tile or aggregate-count-only check:
///
/// - A single tile, or two tiles that both reduce to one trivial cluster,
///   cannot distinguish "processed in order A" from "processed in order B" —
///   reversing a list of interchangeable results is invisible. Using tiles
///   with *different* cluster counts (1 vs. 2) and comparing the full
///   `TileId -> bboxes` map (not just `sum(cluster counts)`) means a
///   regression that swapped which tile's blocks landed under which `TileId`
///   — e.g. a `for_each_tile` that let a loop variable or shared buffer leak
///   between iterations instead of cloning fresh per tile — would flip which
///   `TileId` reports 1 cluster and which reports 2, or attach the wrong
///   bboxes to a `TileId`, and the map comparison below would catch it even
///   though the *aggregate* count would still read 3 either way.
#[test]
fn source_streaming_is_order_independent() {
    let profile = WorldProfile::new(BTreeSet::from(["minecraft:stone".to_string()]), (-64, -61));
    let cfg = SegConfig::default();
    let idx = PartitionIndex::new(vec![]);

    // Region (0,0): one build block -> one cluster.
    // Region (1,0): two build blocks ~390 blocks apart -> two clusters (well
    // beyond the default closing distance of a few tens of blocks).
    let one_cluster_builds = [(10, -59, 10)];
    let two_cluster_builds = [(10, -59, 10), (400, -59, 400)];

    let forward = MemSource {
        tiles: vec![
            region_tile(0, 0, &one_cluster_builds),
            region_tile(1, 0, &two_cluster_builds),
        ],
    };
    let backward = MemSource {
        tiles: vec![
            region_tile(1, 0, &two_cluster_builds),
            region_tile(0, 0, &one_cluster_builds),
        ],
    };

    assert_eq!(forward.access(), Access::Forward);
    assert!(matches!(forward.tile(TileId { x: 0, z: 0 }), Err(TileError::NotRandomAccess)));

    let forward_shape = stream_shape(&forward, &profile, &cfg, &idx);
    let backward_shape = stream_shape(&backward, &profile, &cfg, &idx);

    assert_eq!(
        forward_shape, backward_shape,
        "streaming the same tiles in reverse order must not change which tile owns which clusters"
    );

    // Pin the concrete content, not just that the two runs agree with each
    // other (two runs of a badly broken pipeline could still agree with
    // *themselves*).
    let mut expected: BTreeMap<TileId, Vec<((i32, i32, i32), (i32, i32, i32))>> = BTreeMap::new();
    expected.insert(TileId { x: 0, z: 0 }, vec![((10, -59, 10), (10, -59, 10))]);
    expected.insert(
        TileId { x: 1, z: 0 },
        vec![((522, -59, 10), (522, -59, 10)), ((912, -59, 400), (912, -59, 400))],
    );
    assert_eq!(forward_shape, expected);

    let total: usize = forward_shape.values().map(|v| v.len()).sum();
    assert_eq!(total, 3, "one tile contributes 1 cluster, the other contributes 2");
}
