# Isolated WorldRecordPlot archive

The plot is extracted with the production `UniversalSchematic::split_connected_attach_nearby(Connectivity::Corner, 16, 3)` part isolator. Every output component is retained, including non-door fragments and any entity-only quarantine. Piston-bearing components are labeled `door_candidate`; this is a geometric candidate label, not a functionality claim.

The untouched original remains in the local archive as `user-world-record-plot/WorldRecordPlot.litematic`, SHA-256 `5476ac349cd4529d7a237828633b9e7a995d70b8937f70e4836d2f9f8db66c1e`. Its native data version is 4671. The original selection spans `(0,0,0)` through `(283,207,294)`. See the [source survey](world-record-plot.md) for saved-state caveats and source evidence; its earlier six-neighbour shortlist is not the production extraction.

## Completed extraction, 15 November 2026

The fresh `harrison-work` run exported and read back all **893 components** successfully. Local registration verified every extracted file's BLAKE3 and retained all files under archive source `world-record-parts`, totalling 2,415,518 compressed bytes. The [summary](world-record-parts-summary.json) records counts and verification flags; the [compressed metadata catalog](catalog/world-record-parts.json.gz) includes the full extraction manifest and curator rows. The extraction manifest's SHA-256 is `fdf6264c58b474b32f49b6f881d01582aa6d146979e21da33f1af639874b1b3e`.

| Result | Count |
|---|---:|
| Exported components | 893 |
| Raw piston-bearing candidates | 749 |
| Other components | 144 |
| Large infrastructure components requiring separate review | 3 |
| Initial individual candidates after excluding infrastructure | 746 |
| Preserved non-air blocks | 458,409 |
| Preserved explicit cave/void-air states | 160 |
| Preserved block entities | 12,541 |
| Preserved top-level entities | 318 |
| Entity-only quarantine files | 0 |

The three large infrastructure components remain in the canonical 893-part partition. They connect mechanisms through shared plot floors and must not be presented as three individual doors. Excluding them gives 746 initial individual candidates; that count does not imply the floor-attached mechanisms have been individually isolated.

| Retained component | Non-air blocks | Piston-related blocks, including heads |
|---|---:|---:|
| `part0004` | 100,234 | 1,884 |
| `part0005` | 84,499 | 11 |
| `part0006` | 84,362 | 50 |

The review tags identify 99 components contacting a selection boundary, 617 with empty or absent saved inventory tags, 59 piston-bearing components without a saved activation control, and two components containing moving-piston states without corresponding block entities. These sets overlap. They guide inspection and do not establish that a candidate is broken or complete. All 318 top-level entities were assigned exactly once to an exported component, so no quarantine file was necessary in this run.

## Preservation and verification

The extractor keeps source coordinates and metadata and makes no inventory, control or mechanics repairs. Source block states and block entities must form an exact, disjoint partition. Every exported `.litematic` is read back and checked against the source. NBT comparisons sort compound keys and compare float bits, preserving meaningful NaNs, entity passengers and saved item data. The original source contains 458,409 non-air blocks, 160 explicit cave/void-air states, 12,541 block entities and 318 top-level entities; a successful aggregate check must recover all 458,569 explicit non-ordinary-air states and each block entity/entity exactly once.

The physical isolator excludes all three air types. The wrapper preserves the distinct cave/void-air states by attaching each to the nearest physical component, preferring a containing physical bounding box and breaking ties by nearest actual block and stable part index. Those states do not create connectivity bridges or influence entity assignment. The manifest distinguishes `non_air_blocks` (excluding ordinary, cave and void air), `explicit_air_states` and their sum `preserved_block_states`; ordinary air remains implicit.

Block-entity identity and positions are compared using the materialized `BlockEntity.id` and absolute `BlockEntity.position`. The format's duplicated `Id`/`id` and `Pos`/`x`/`y`/`z` transport wrappers are normalized to those authoritative fields; every other payload tag remains bit-exact. Relative wrappers necessarily change when an extracted component has a different region origin. The first exported component demonstrated this at source position `(0,4,76)`: its new origin `(0,2,63)` gives relative position `(0,2,13)`, while all 20 block entities' non-wrapper payloads were independently unchanged. Conflicting identity or coordinate aliases are rejected rather than ignored. A mismatch reports the first position and NBT path/type, including float bits.

The existing part isolator does not copy top-level entities. The extraction wrapper assigns each entity to a containing component box, breaking overlap ties by nearest actual block and then stable part order. Entities outside every component box are retained in a separate entity-only quarantine file. No entity is silently discarded, and no top-level or passenger coordinates are rebased. A deterministic source index accompanies every assignment.

Component IDs follow source bounds and block coordinates. Archive display titles use `Plot door #### · X×Y×Z`, `Plot fragment ####` or `Unassigned plot entities`; these derived labels do not replace the schematic's saved metadata. The manifest records per-part blocks, bounds, saved controls, containers, inventories, source sign text, exact source-boundary contacts and review tags. An `INCOMPLETE` marker remains if any export or verification fails; only a completed run produces the success manifest accepted by registration.

Litematica air padding, derived volume statistics and writer software headers are regenerated. The format's existing importer/exporter does not preserve arbitrary unknown tags or pending tick lists; the original file remains the archival authority. This source's surveyed pending block/fluid tick lists were empty. Lossless block/entity partition checks do not claim a byte-identical derivative or a complete logical mechanism.

## Local registration

Run the Rust compilation and extraction on the designated `harrison-work` laptop. Copy the output directory intact before registering it. The example refuses an existing output directory.

```bash
cargo build --locked --release --example isolate_archive_plot
target/release/examples/isolate_archive_plot \
  /path/to/WorldRecordPlot.litematic /path/to/new-extraction-directory

/path/to/python-with-blake3 tools/register_tick_plot_parts.py \
  --archive /path/to/Nucleation-schematic-archive \
  --extraction /path/to/new-extraction-directory \
  --check-only

/path/to/python-with-blake3 tools/register_tick_plot_parts.py \
  --archive /path/to/Nucleation-schematic-archive \
  --extraction /path/to/new-extraction-directory \
  --report-summary docs/performance/schematic-candidates/world-record-parts-summary.json \
  --report-catalog docs/performance/schematic-candidates/catalog/world-record-parts.json.gz
```

Registration verifies the pinned original SHA-256, the extractor's source and per-file BLAKE3 hashes, all successful read-back flags and aggregate entity assignment coverage. It then places each file in immutable SHA-256 object storage with an archive hardlink. The original outputs are not moved or modified. Existing destinations must have identical bytes; no files are deleted or overwritten. Registration produces source `world-record-parts`, its source/manifest metadata and `curation/world-record-parts.json`. The archive index can then consume these through its existing source manifest and curator join.

The Python `blake3` package or `--b3sum /path/to/b3sum` is required. `--source-file` allows the original to be relocated from the extraction host; both SHA-256 and BLAKE3 must identify the same archived source bytes. The optional reports contain metadata only, with a reproducible gzip header; local-only schematic binaries are not added to Git.

## Functional admission

Every registered part starts as `unvalidated`, `functional_status=unverified`, `simulation_status=not_run` and `benchmark_status=not_admitted`. Exact extraction/read-back evidence is kept separate from simulator or vanilla evidence. Only piston-bearing candidates receive the archive's `door` tag; other components and quarantine files remain available for inspection without being presented as doors.

Before admitting a candidate as a benchmark, inspect neighbouring dependencies and piston travel space, identify the intended aperture and control sequence, establish required inventory/entity contents, then capture complete activation, reset and idle controls in Minecraft 1.21.11. Plot-boundary contacts, absent controls, empty inventories, shared infrastructure and moving-piston states lacking block entities are review signals. A failure in an incomplete or outdated source is not automatically an engine regression. Any repaired variant needs separate provenance and hashes.

The extraction and local registration checks above completed successfully. Functional simulation, vanilla comparisons and benchmark admission remain separate future work for these newly extracted candidates.
