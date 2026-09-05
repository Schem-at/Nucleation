# Spatial progress for world extraction

`WorldSegmenter::try_run_streaming_observed` is the original deterministic streaming API plus a diagnostic `FnMut(SpatialObservation)` callback. The existing `try_run_streaming` and collecting APIs are unchanged and allocate no observations. Observations do not enter segmentation, scoring, stable identity or materialization.

The callback reports:

| Phase | Geometry | Identity / parents |
| --- | --- | --- |
| `candidate` | Tile-local membership cluster, after segmentation/refinement | Cluster ID; no parents |
| `stitched` | Global stitched group, before the caller's optional further splitting | Stable build ID; up to 64 cluster IDs and full `parent_count` |

Bounds are inclusive **world-space** `[[min_x,min_y,min_z],[max_x,max_y,max_z]]`, not schematic-local coordinates. Candidate geometry is provisional: the source can still fail, groups can merge across tiles, and the CLI can split materialized groups into multiple smaller schematics. The callback may report shapes before an error; it does not grant evidence of complete coverage.

With `segment_world --progress-json true`, protocol-1 NDJSON gains `spatial_shape` events whose `details` contain the observation. Candidate/group events stop after 20,000 observations and a single `spatial_limit_reached` event is emitted. Every written final schematic still produces `build_extracted`, now including `world_bbox` and the materialized group's `parent_id` alongside `stable_id`, `blocks`, `builds`, and `tier`. The final output catalogue and `extraction_completed` count are never truncated by this diagnostic limit.

Consumers should batch and bound event storage, fence by worker attempt/lease, hide failed or superseded attempts, and keep provisional geometry separate from completed catalogue observations. Group parents may be omitted by the diagnostic cap; `parent_count` discloses capped parent lists. Stable world/run identity, attempt and phase should scope UI identities. Only the completion manifest proves full coverage; receiving a piece event does not prove the entire extraction succeeded.

The 2D source map can use these links to highlight candidate → stitched-group → final-piece ancestry. Background images, plot-owner hints and Dynmap projections belong to the application, not this extraction protocol. They must not become implicit segmentation inputs.

## Explicit source gaps (0.10.17)

Snapshot extraction defaults to `--empty-region-policy reject`. An operator may explicitly select `acknowledge-zero-byte` to process around zero-byte MCA placeholders. The immutable manifest retains the original error and content-addressed bytes. Actual objects must still exist and pass their length and BLAKE3 checks; nonempty corrupt regions always fail.

The completion receipt includes `empty_region_policy`, `coverage` (`complete` or `acknowledged_gaps`), and sorted `acknowledged_zero_byte_regions` coordinate pairs. `complete: true` means the configured extraction completed, not that missing bytes were recovered. Consumers must surface the gaps, pin the policy in extraction identities, and never infer deletions inside unobserved regions. Coverage/boundary checks remain enabled.
