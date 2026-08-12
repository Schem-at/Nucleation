# World segmentation: from a world save to individual builds

`nucleation::world_segment` turns a whole Minecraft world into a set of discrete,
individually addressable builds — each one a normal `UniversalSchematic` plus a
**provenance envelope** recording exactly where in the world it came from. It is
built for doing this *repeatably*: the same world bytes and the same configuration
produce byte-identical output, on any machine, in any order, every time.

Feature flag: `world-segment` (pulls in `tar` and `zstd` for archive sources).

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
    TarArchiveSource, WorldSourceTiles,
};
use nucleation::formats::world_stream::WorldSource;

// 1. A tile source. A world directory gives random access…
let source = WorldSourceTiles::new(WorldSource::open_dir("world/".as_ref())?, -64, 320);
// …a .tar, .tar.gz, or .tar.zst backup streams forward-only. Compression is
// detected from magic bytes, and a rectangle clips blocks before tile storage:
// let source = TarArchiveSource::open("backup.tar.zst", -64, 320)?
//     .with_world_rect(-1024, -1024, 1024, 1024);

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

## Python API

The wheel exposes the same native Rust pipeline; Python only marshals arguments.
For a local world directory, the complete call is:

```python
from pathlib import Path
import nucleation

world = "/srv/minecraft/world"
out = Path("builds")
out.mkdir(exist_ok=True)

profile = nucleation.WsProfile.derive_from_dir(
    world, -64, 320, 24, 0.75,
)
hints = nucleation.WsPartitionHints.create()
job = nucleation.WsSegmentJob.create(
    4, 2, 1,                 # cell size, closing radius, minimum tile cluster
    "map:creative", "save-2026-08-12",
    -64, 320, 1_786_453_837, # Y range and caller-supplied timestamp
    0.5, False,              # snapshot IoU and hard-cut partitions
)
result = nucleation.WsRunResult.run_dir(job, hints, profile, world)

for index in range(result.build_count()):
    result.write_schem_to(index, str(out / f"{result.stable_id_hex(index)}.schem"))
```

An already-extracted schematic can use the separation-aware final splitter
directly from Python. It is lossless and does not treat small machines as
accessories:

```python
schematic = nucleation.Schematic.open("combined.schem")
pieces = schematic.split_connected_attach_nearby(16, 3)

for index in range(pieces.len()):
    pieces.piece(index).save(f"machine-{index + 1}.schem")
```

`WsRunResult` holds a bounded/local run's outputs for convenient inspection.
For a whole remote world, use `examples/distributed_world_extract.py`: Python
schedules resumable centre-out shards while the compiled `segment_world` worker
reads only intersecting MCA files from the input `Store` and streams outputs to
the output `Store`. Neither layer copies or loads the complete map.

---

## The pieces

### `TileSource` — where voxels come from

| Implementation | Access | Notes |
|---|---|---|
| `WorldSourceTiles` (dir / zip / mca bytes) | `Random` | tiles addressable by id, pull-scheduling friendly |
| `StoreRegionTiles` (`region/` keys in any Store) | `Random` | remote compute reads only intersecting MCA files; one region buffered at a time |
| `TarArchiveSource` (`TarGzSource` compatibility alias) | `Forward` | streams `.tar`, `.tar.gz`, or `.tar.zst`; cannot seek |

The one entry point every source supports is
`for_each_tile(&mut FnMut(VoxelTile) -> Result<(), TileError>)`. Returning
`Err(TileError::Stop)` from the callback ends iteration early and cleanly
(`for_each_tile` returns `Ok`): this is how you sample N tiles from a 1.6 GB
archive without paying for the rest of it.

`TarArchiveSource` filters junk aggressively and *reports* every skip on stderr rather
than silently dropping it: backup files (`*.mca.bak`, `r.X.Z.mca.<digits>.backup`),
entries outside `region/`, empty entries, region coordinates beyond ±120 000
(sign-extension artifacts in some server backups), and — if you call
`.with_world_border(n)` — regions entirely outside the border. An inclusive
`.with_world_rect(min_x, min_z, max_x, max_z)` also clips blocks before they are
stored, and `quiet_filtered_entries()` suppresses benign out-of-scope messages.
A malformed region
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
| `partition_dense_layer_coverage` | `None` | subtract a nearly full partition layer even when it uses many materials |
| `split_disconnected` | `None` | optionally undo a closing that fused substantial disconnected builds |
| `drop_unpartitioned` | `false` | with `HardCut`, omit roads/gutters outside all hints |

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

Patterned or deliberately mixed-material floors may have no single dominant
material. `partition_dense_layer_coverage: Some(0.8)` subtracts an exact Y layer
when at least 80% of the partition footprint is occupied, regardless of its
palette. Only the dense layer is removed; sparse machinery above it is kept.

`PartitionIndex` spatially indexes arbitrary boxes by world region. A global
uniform grid with tens of thousands of cells therefore remains practical, and
every distributed shard can receive the same complete hint set. This matters
for identity: the complete hint geometry is part of `config_hash`, so workers
must share it even when each worker reads only a small world rectangle.

The compiled worker also accepts `--partition-hints FILE.json`. The file is an
array of inclusive rectangles with an opaque `id`; optional `y0`/`y1` bounds
limit a partition vertically. Additional scalar fields are attribution data:

```json
[
  {
    "id": "plot:12,-3",
    "x0": 2817, "x1": 3071,
    "z0": -1023, "z1": -769,
    "owner": "ExamplePlayer",
    "trusted": "BuilderOne,BuilderTwo",
    "members": "*",
    "alias": "decoder-lab"
  }
]
```

The worker normalizes those fields and embeds them in each matching
schematic's standard `SchematicProvenance.attributes` as
`nucleation:partition_<field>`. It also records
`nucleation:partition_catalog_hash`, the content hash of the exact JSON bytes.
Thus ownership is an auditable snapshot rather than an unversioned assertion.
If one logical partition is represented by multiple rectangles, repeat its
`id`; scalar values are combined deterministically. Geometry-only fields are
not duplicated into every output.

### Compiled worker and Python control plane

`examples/segment_world.rs` is a compiled extraction worker. It accepts a world
directory or tar archive, an inclusive rectangle, either uniform-grid or
arbitrary partition hints, pinned-substrate settings, a detached-component
attachment policy, and an output Store URL. It writes:

```text
schematics/<stable-build-id>.schem
provenance/<stable-build-id>.json
catalog/x<min>_<max>_z<min>_<max>.jsonl
```

Inputs can also be a Store URL plus `--world-prefix`: `StoreRegionTiles` lists
the remote `region/` directory, intersects it with the shard before transfer,
and buffers only one MCA file. The compute node never downloads or loads the
whole world. All `.schem` writes go through `Store`, so the same binary targets a local
folder, `ssh://` host, S3/MinIO, Redis, Postgres, or a callback-backed host.
The schematic and the sidecar both carry the standard provenance contract.
Catalog JSONL also exposes `partition_metadata` and `partition_catalog_hash`
for querying without opening a schematic.

After materialization the worker supports three lossless component policies:

- `exact` emits every disconnected 26-neighbour component as an independent
  schematic. Size and gap thresholds do not apply.
- `nearby` (the default) keeps every component with at least
  `--component-min-blocks` blocks (default 16) independent. A smaller fragment
  attaches directly only to a substantial component within
  `--component-join-gap` blocks; attachment is non-transitive, so fragments
  cannot chain independent builds back together.
- `nearest` is the conservative legacy policy: every component below the
  threshold attaches to its closest substantial component, regardless of
  distance. It is useful when disconnected fixtures are known to belong to one
  assembly, but a high threshold can under-count a plot containing many builds.

The worker folds the selected mode and thresholds into the output `config_hash`,
records them as namespaced provenance attributes, and derives split identities
from world bounds plus the exact piece fingerprint rather than component
ordering. The uniform-grid and arbitrary-partition Python schedulers both
default to `nearby`, 16 blocks, and a three-block direct attachment gap.

For orchestration, `examples/distributed_world_extract.py` divides a global
grid into deterministic rectangular shards, runs one or more compiled workers,
logs each shard, and records local completion markers for restart. Its optional
`--work-bounds` assigns a subset to one machine while `--grid-bounds` stays
identical on every machine. Shards are scheduled centre-out so populated map
areas produce useful output before empty world-border catalogs. The state
directory also pins the semantic job
configuration and worker-binary SHA-256, so a changed splitter or setting cannot
silently reuse stale completion markers. Python is the control plane; Rust
remains the hot Anvil parsing and segmentation path.

For non-uniform claims or merged plots, use
`examples/distributed_rect_extract.py`. Each compute host receives the same
partition catalogue plus its own list of pairwise-disjoint work rectangles.
The state manifest pins the binary, catalogue, and rectangle-list SHA-256s.
Choose work boundaries that do not cross a logical partition; the compute host
then reads only intersecting MCA files through `StoreRegionTiles`, one region at
a time, while output streams directly to the configured Store.

For literal component-per-schematic extraction, pass
`--component-attach-mode exact`. This is intentionally explicit: redstone
assemblies can contain detached lamps, piston heads, or other loose parts, so
the default `nearby` policy is usually the better definition of a logical build.

### Analysing an extracted corpus

`examples/analyze_schematic_corpus.rs` profiles an existing collection without
copying it to the compute machine or loading it all into memory. Feed it an
uncompressed tar stream (local or over SSH); it parses one schematic at a time
and writes JSONL with compressed size, tight dimensions, non-air block count,
bounding-box density, palette name/state counts, palette entropy, dominant
material share, a redstone/mechanism share, and both 6- and 26-neighbour
connectivity metrics:

```bash
ssh storage-host 'tar -C /data/extraction -cf - schematics' \
  | cargo run --release --features world-segment \
      --example analyze_schematic_corpus -- metrics.jsonl run-summary.json
```

`run-summary.json` records the processed and failed counts, total compressed
bytes and blocks, and the 50 most common block names. The stream keeps network
and memory behaviour bounded: only the current `.schem` payload and its occupied
coordinate set are resident. Since an extractor may still be writing, the run
is a point-in-time corpus sample rather than a transactionally frozen snapshot.

### Curating registry and ranking views

Do not make extraction lossy merely to improve a catalogue. A barrel, lamp, or
wire fragment may be a legitimate part of a disconnected assembly even when it
is not independently useful. Keep raw `schematics/`, `provenance/`, and
`catalog/` intact, then use the Python `CurationPolicy` layer to make a derived
view before registry batching, owner ranking, or other publication:

```python
from pathlib import Path
from nucleation import CurationPolicy, curate_corpus

policy = CurationPolicy.minima(
    min_blocks=2,
    min_palette_names=2,
    name="ore-sanity-v1",
)
curated = curate_corpus(
    Path("/data/ore-builds"),
    Path("/data/ore-builds/curation/ore-sanity-v1"),
    policy,
)
```

The output contains `accepted-ids.txt`, `rejected.jsonl`, `policy.json`, and
`summary.json`. Rejections include all matching reasons; policies have stable
content IDs. `write_registry_archives` and `write_top_owner_archives` consume
the accepted view and embed that policy ID in their indexes. Raw schematics are
never moved or deleted. Add `MetricRule` entries for analyser/catalogue fields,
or named Python predicates for project-specific sanity checks. Predicate names
enter the content ID; keep predicate source under version control with the run.

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
inputs give byte-identical envelopes. Materialization also embeds the common
[`SchematicProvenance`](schematic-provenance.md) record in every schematic.
Keep the JSONL envelopes as a queryable index (which box, snapshot, and
partition) that attribution or cataloguing can join without opening every file.

---

## Gotchas

- **Forward-only sources can't rewind.** Deriving a profile and then running
  means opening a `TarArchiveSource` twice. Pin the profile to pay the sampling pass
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
