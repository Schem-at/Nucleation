# Immutable world snapshots and the Laravel worker contract

Build the owned standalone worker for the queue host's architecture:

```sh
cargo build --release --no-default-features --features world-segment,store-fs,store-ssh --bin segment_world
```

Existing `--example segment_world` builds remain supported. The library's existing
streaming API is preserved; workers should use `WorldSegmenter::try_run_streaming`
so source errors cannot be confused with an empty result.

## Index once

```sh
segment_world index world.tar.gz /shared/world-snapshots \
  --source-id ore:build --dimension minecraft:overworld \
  --world-prefix build/region --capture-unreadable true \
  --previous-manifest /shared/world-snapshots/manifests/PREVIOUS_HASH.json
```

Omit `--previous-manifest` for the first observation. Inputs can be an Anvil world
directory, an uncompressed/gzip/zstd tar archive (detected by magic), or an enabled
Store URI. Explicit `--world-prefix` selects the region directory when there is
more than one world/dimension. Symlinks and ambiguous directories are rejected.

Schema-1 manifests contain source identity, dimension, and a canonical region map.
Each region references `objects/<BLAKE3>.mca` containing its exact original bytes,
and absolute `cx,cz` keys containing semantic chunk hashes. Semantic hashes include
sorted absolute block positions, canonical block states/properties, and block-entity
positions, ids, and stable NBT. Anvil timestamps, sector layout, and compression do
not change extraction dependencies. Deleted chunks/regions do change dependencies.

The completed manifest is itself addressed by BLAKE3 at `manifests/<hash>.json`.
It is written only after traversal completes. Repeated identical regions reuse the
previous semantic index and verify/repair their stored raw object. Empty sources
are rejected. Regions are capped at 128 MiB. Strict indexing rejects malformed data;
the explicit `--capture-unreadable true` policy stores raw unreadable region bytes
with an `error` field instead. Such regions are archival evidence, never empty space.

Index progress is newline-delimited JSON on stdout (`protocol: 1`):
`snapshot_started`, `region_indexed`, `snapshot_completed`. Fields include regions,
reused_regions, unreadable_regions, chunks, bytes, current region, and final manifest
key/hash. Indexing is region-incremental but still traverses a new monolithic archive.

## Extract an observation

```sh
segment_world /shared/world-snapshots/manifests/HASH.json /shared/run/area \
  84 84 171 171 --snapshot-store /shared/world-snapshots \
  --source-id ore:build --snapshot-id CALLER_SNAPSHOT_SHA256 \
  --dimension minecraft:overworld --extracted-at 0 --progress-json true
```

Pin the complete effective extraction configuration, substrate (or its inference
inputs), partition contents, and executable hash in the orchestrator. The original
archive can be unavailable: `SnapshotTiles` reads and verifies intersecting raw
objects from the immutable Store. Empty rectangles in a valid snapshot are valid
observations. Corrupt/missing referenced objects and intersecting unreadable regions
fail the run before any builds are emitted.

Snapshot CLI extraction uses `CoverageCheckedTiles`. Non-substrate blocks near an
artificial XZ cut cause an actionable error, before small-component filtering.
The guard band is 16 blocks plus the configured component join gap. Caller-supplied
hard partitions fully contained in the rectangle are accepted as intentional cuts.
Supply verified whole plots, or widen/re-align the rectangle to an empty margin.
Unverified legacy directory/archive extraction never sets `complete: true`.

Extraction progress wraps details in `{protocol:1,event,details}`. Events are
`extraction_started`, `profile_ready`, `build_extracted`, `extraction_completed`.
Human-readable stdout also remains; consumers must ignore non-protocol lines and
drain stdout/stderr with bounded buffers. `completion.json` is the durable result:
`protocol`, `complete`, `source_hash`, inclusive XZ `bounds`, `builds`, catalog key,
profile hash, and segmentation stats. `builds` counts the final output pieces;
`stats.builds` counts the pre-component-split groups. They may differ.

Laravel stages each extraction attempt separately, requires successful process exit
and a matching complete result, seals every output file by SHA-256, then promotes
the directory. Consumers must never import a partial directory merely because some
`.schem` files or a catalog exist. Content hashes and provenance are separate: a new
world observation may explicitly reuse an older extraction artifact.

## Scope

This is block/build observation storage, not a complete Minecraft backup format.
Entity regions, POI, player data, and `level.dat` are not semantic build inputs.
Keep the original backup for full-world restoration. Use consistent backup
directories/Store prefixes; an actively written Minecraft world is not an atomic
snapshot. No object garbage collection or deployment is performed by these APIs.

Regression command:

```sh
cargo test --no-default-features --features world-segment,store-fs,store-ssh --lib world_segment
cargo test --no-default-features --features world-segment,store-fs,store-ssh --test world_snapshot
```
