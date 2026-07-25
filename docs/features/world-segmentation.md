# World segmentation: from a world save to individual builds

`nucleation::world_segment` turns a whole Minecraft world into a set of discrete,
individually addressable builds — each one a normal `UniversalSchematic` plus a
**provenance envelope** recording exactly where in the world it came from. It is
built for doing this *repeatably*: the same world bytes and the same configuration
produce byte-identical output, on any machine, in any order, every time.

Feature flag: `world-segment` (pulls in `tar` for the archive source).

```toml
nucleation = { version = "0.5", features = ["world-segment"] }
```

---

## What it does, in one pass

```text
world save ──► tiles ──► substrate removed ──► clusters ──► stitched builds
                                                             │
                                              scored (tiers) ┤
                                                             ▼
                                     UniversalSchematic + Provenance, per build
```

1. **Tile** the world (one region file = one tile by default).
2. **Subtract substrate** — the natural ground — so terrain stops gluing
   everything together.
3. **Cluster** what remains with morphological closing, so a machine, its
   floating wiring, and its support frame come out as *one* build instead of
   dozens of fragments.
4. **Stitch** clusters that cross tile boundaries back together.
5. **Score** every build into a tier (`Confident` / `Probable` / `Debris`).
   Debris is *kept*, never deleted — the machine orders the queue, a human
   decides worth.
6. **Materialize** each build into a local-origin schematic plus its
   `Provenance`.

Everything upstream of I/O is a pure function. There is no clock, no RNG, no
hash-map iteration order anywhere in the pipeline — that is what makes runs
reproducible and shardable.

## Quick start

```rust
use nucleation::world_segment::{
    ProfileParams, WorldProfile, PartitionIndex, PartitionPolicy,
    SegConfig, SegmentJob, ScoreConfig, WorldSegmenter, TileSource,
    TarGzSource, WorldSourceTiles,
};
use nucleation::formats::world_stream::WorldSource;

// 1. A tile source. A world directory gives random access…
let source = WorldSourceTiles::new(WorldSource::open_dir("world/".as_ref())?, -64, 320);
// …a .tar.gz backup streams forward-only:
// let source = TarGzSource::open("backup.tar.gz", -64, 320)?.with_world_border(8192);

// 2. Derive (or load a pinned) profile of the world's natural ground.
let mut samples = Vec::new();
source.for_each_tile(&mut |tile| {
    samples.push(tile);
    if samples.len() >= 24 {
        return Err(nucleation::world_segment::TileError::Stop); // enough — stop streaming
    }
    Ok(())
})?;
let profile = WorldProfile::derive(&samples, &ProfileParams {
    min_slab_coverage: 0.3,   // a Y level is "ground" if ≥30% of columns have a block there
    palette_min_share: 0.01,  // a material is "ground" only if it dominates the ground layers
    ..Default::default()
});

// 3. Optional: partition hints — named boxes a build may never span.
let partitions = PartitionIndex::new(vec![]); // none

// 4. Configure and run.
let job = SegmentJob {
    config: SegConfig {
        partition_policy: PartitionPolicy::Off,
        partition_floor_share: None,
        ..Default::default()          // cell_size 4, closing_radius 2
    },
    score_config: ScoreConfig::default(),
    source_id: "my-world".into(),     // opaque labels, yours to define
    snapshot_id: "2026-07-24".into(),
    min_y: -64, max_y: 320,
    extracted_at: 1_753_300_000,      // an input — never read from the clock
    match_iou: 0.5,
};

let source = WorldSourceTiles::new(WorldSource::open_dir("world/".as_ref())?, -64, 320);
let mut stats = WorldSegmenter::run_streaming(
    &source, &profile, &partitions, &job, &[],
    &mut |build| {
        // one build at a time: schematic + provenance, then dropped
        println!("{} — {} blocks, {:?}",
                 build.provenance.stable_build_id,
                 build.provenance.block_count,
                 build.provenance.tier);
    },
);
```

`WorldSegmenter::run(..)` is the same pipeline returning a `Vec<MaterializedBuild>`;
prefer `run_streaming` for whole worlds so you never hold every output schematic
at once.

---

## The pieces

### `TileSource` — where voxels come from

| Implementation | Access | Notes |
|---|---|---|
| `WorldSourceTiles` (dir / zip / mca bytes) | `Random` | tiles addressable by id, pull-scheduling friendly |
| `TarGzSource` | `Forward` | streams a `.tar.gz` backup once; cannot seek |

The one entry point every source supports is
`for_each_tile(&mut FnMut(VoxelTile) -> Result<(), TileError>)`. Returning
`Err(TileError::Stop)` from the callback ends iteration early and cleanly
(`for_each_tile` returns `Ok`): this is how you sample N tiles from a 1.6 GB
archive without paying for the rest of it.

`TarGzSource` filters junk aggressively and *reports* every skip on stderr rather
than silently dropping it: backup files (`*.mca.bak`, `r.X.Z.mca.<digits>.backup`),
entries outside `region/`, empty entries, region coordinates beyond ±120 000
(sign-extension artifacts in some server backups), and — if you call
`.with_world_border(n)` — regions entirely outside the border. A malformed region
or a corrupt chunk skips *that region* and keeps streaming; a callback error
aborts the run (that one is yours).

### `WorldProfile` — what counts as ground

Substrate is decided per block by two tests: the block's name is in the
**substrate palette** AND its Y is inside the **substrate band**. Both come from
`WorldProfile::derive(&samples, &params)`, which finds the near-solid ground slab
empirically:

- the **band** is the contiguous run of Y levels, from the lowest sampled level
  up, whose per-level column coverage is at least `min_slab_coverage`;
- the **palette** is the set of block names inside that band whose share of the
  band's blocks is at least `palette_min_share`.

The result is a small, serializable value with a stable `profile_hash()`.
**Pin it**: derive once, save it, and reuse it for every later run of the same
world — reproducibility then survives even future changes to the derivation
heuristic, and forward-only sources don't pay a second streaming pass.

Calibration guidance, learned on real worlds:

- **Sample representatively.** The first N tiles of an archive are usually the
  world's outskirts. Sample from the area you actually care about, or you will
  derive a band of just bedrock.
- **Player-modified worlds have porous ground.** On a heavily built (and dug)
  creative world, ground layers may only reach 30–50% column coverage.
  `min_slab_coverage: 0.3` is a better starting point than the pristine-world
  default of 0.9. Print per-level coverage from your samples when in doubt.
- **`palette_min_share` exists because players place blocks at ground level.**
  Without it, one redstone wire inside the band puts `redstone_wire` in the
  "ground" palette — and then substrate subtraction eats the bottom layer of
  every build. Dominance filtering (≥1% of band blocks) keeps the palette to
  actual ground materials.

### `SegConfig` — clustering

| Field | Default | Meaning |
|---|---|---|
| `cell_size` | 4 | occupancy-grid cell edge, blocks |
| `closing_radius` (R) | 2 | Chebyshev dilation radius, cells |
| `min_cluster_blocks` | 1 | clusters smaller than this are dropped **per tile, before stitching** |
| `partition_policy` | `Off` | see partition hints below |
| `partition_floor_share` | `None` | see partition floors below |

Two structures end up in the same build iff their occupied cells are within
Chebyshev **2R+1 cells** — with defaults, gaps up to roughly 20 blocks bridge,
wider gaps separate. Those are the only geometry knobs, and both have a physical
meaning you can explain: `cell_size` is resolution, `closing_radius` is "how far
apart can two parts of the same build float".

`ClusterId`s (and everything derived from them) are bound to a
`config_hash` folding the config, the profile, and the partition hints — outputs
produced under different settings can never collide or be confused in a cache.

### Partition hints — boundaries a build may not cross

If you know the world is divided into parcels (a plot grid, districts, claim
regions — any set of named boxes), pass them:

```rust
let hints = vec![PartitionHint {
    id: "12,-3".into(),                    // opaque, yours
    bbox_xz: (x0, x1, z0, z1),             // inclusive
    y_range: None,                         // None = full column
}];
let partitions = PartitionIndex::new(hints);
// SegConfig { partition_policy: PartitionPolicy::HardCut, .. }
```

Under `HardCut`, blocks are partitioned **per block** (boundaries need not align
with cells), each partition is clustered in isolation, and stitching will never
union clusters across differing partitions. Two adjacent builds on opposite
sides of a boundary stay two builds, however close. Each build records the
partition it fell in (`Cluster::partition_id`, `Provenance::partition_id`) — an
opaque join key back to whatever your boxes mean.

`Prefer` is currently inert (documented as such); `Off` ignores hints entirely.

**Partition floors.** In parcelled worlds, owners often floor their parcel with
a material of their choice. Globally that material is rare (so the profile's
palette can't catch it), but locally it is dominant — and a surviving floor
bridges everything on the parcel into one giant cluster.
`partition_floor_share: Some(0.3)` fixes this generically: per partition, any
material holding ≥30% of that partition's blocks *inside the substrate band* is
subtracted as its floor.

### Stitching and its algebra

Tiles are segmented independently; `StitchState` reunites builds that cross tile
boundaries. Its `merge` is **associative, commutative, and idempotent** (property
tested), so partial stitches can be combined in any order and any grouping —
including a tree reduction across machines. Sequential single-process merging is
what `WorldSegmenter` does; the algebra is what makes anything fancier possible
without changing results.

If you consume `TileSegments` directly: `MarginCell` entries carry their
partition, and a stitcher must never union margin entries whose partitions
differ — two entries can share a cell precisely because per-block partitioning
allows a cell to straddle a boundary.

### Scoring

`score(&build, &ScoreConfig)` assigns a tier from explainable signals (block
count, bbox volume, density, cluster count — each recorded on the result):

- `Debris` — at or below `debris_max_blocks` (default 100),
- `Confident` — at least `confident_min_blocks` **and** `confident_min_density`,
- `Probable` — everything between.

Scoring is per-build and pure (no percentiles over the whole set), so it shards
and re-runs freely. Nothing filters Debris out; it is a label for triage.

### Identity across snapshots

Re-extracting a newer save of the same world should *update* builds, not
duplicate them. `match_snapshots(current, prior, source_id, iou_threshold)`
matches by bounding-box IoU:

- no prior overlaps → `New` (fresh `StableBuildId`, deterministically seeded),
- exactly one ↔ one → `Same` (id inherited; a changed `fingerprint` on identical
  identity is your "this build was edited" signal),
- one prior, many current → the largest current inherits, the rest are
  `Split { inherits }` with fresh ids,
- many prior, one current → `Merge { from }`, inheriting from the largest prior.

All tie-breaks are content-ordered — input order never changes the outcome.
Spatial identity is deliberate: a build edited in place keeps its id (and your
curation attached to it); a content-hash identity would orphan it on every edit.

### Provenance

Every materialized build carries:

```text
stable_build_id · snapshot_build_id · source_id · snapshot_id · source fingerprint
world_bbox · origin_offset (local (0,0,0) → world coords)
block_count · cluster_count · tier · signals
config_hash · profile_hash · extracted_at
```

`block_count` describes the schematic actually produced. `extracted_at` is a
caller-supplied timestamp — the library never reads the clock, so identical
inputs give byte-identical envelopes. Store envelopes outside the schematic
binary; they are the queryable index (which box, which snapshot, which
partition) that later attribution or cataloguing can join against without
parsing blocks.

---

## Gotchas

- **Forward-only sources can't rewind.** Deriving a profile and then running
  means opening a `TarGzSource` twice. Pin the profile to pay the sampling pass
  once, ever.
- **`TileError::Stop` is the only early exit.** Without it, "give me 3 tiles"
  still streams the whole archive.
- **`min_cluster_blocks` filters per tile, *before* stitching.** A large build
  split by a tile edge into two sub-threshold fragments would vanish entirely.
  Leave it at 1 and let tiers do the triage.
- **Memory scales with the world's artificial blocks.** The runner holds
  per-cluster blocks until each build materializes (a 1.6 GB / 845-region world
  peaked around 16 GB). Use `run_streaming` so outputs don't stack on top.
- **Bounding boxes are inclusive** everywhere (`bbox_xz`, `world_bbox`, IoU).
- **A `Provenance` with a different `config_hash`/`profile_hash` is a different
  extraction.** Don't compare fingerprints across configs and expect stability.
- **Debris is data.** 1-block specks come out labeled, not deleted; drop them at
  the consumer if you must, knowing what you dropped.
- **Skips are stderr lines** (one per rejected archive entry). Capture stderr if
  you need an audit of what was filtered.

## Example use cases

- **Catalogue a creative server's map**: stream the nightly backup, HardCut on
  the plot grid, and publish each non-debris build as a schematic with its
  provenance row — 845 regions became ~4,500 addressable builds in ~30 minutes
  in our validation run, deterministically.
- **Incremental snapshots**: keep each run's provenance; feed it as `prior` to
  the next run. Edited builds keep their `StableBuildId` with a new fingerprint;
  new builds mint ids; splits and merges are labeled as such — a version history
  of a living world.
- **Non-world voxel data**: anything you can voxelize into tiles can be
  segmented — the pipeline never asks where the voxels came from.
- **Distributed extraction** (advanced): segment tiles on many workers, ship
  `TileSegments` + serialized `StitchState`s, tree-reduce with `merge` — the
  algebra guarantees the same answer as a single-threaded fold.

## FFI / bindings

The runner is exposed through the generated bindings surface (`bridge` feature):
job/hints/profile handles, a directory-based run entry, and per-build accessors
(hex `StableBuildId`, hex fingerprint, tier, bbox, block counts, schematic
writing). Errors cross the boundary as `Result`s; panics do not.
