# Floor-attached WorldRecordPlot candidates

This second pass addresses the three large infrastructure components retained by the [canonical 893-part extraction](world-record-parts.md). It creates additional local derivatives without modifying those parents or replacing their exact source partition. Physical candidates still need independent functional and vanilla checks.

The fresh `harrison-work` run on 15 November 2026 exported and verified **566 components: 57 piston-bearing candidates and 509 other components**. All files are registered locally under `world-record-floor-parts`; no entity quarantine was needed. The [summary](world-record-floor-parts-summary.json) and [complete extraction metadata](catalog/world-record-floor-parts-extraction.json.gz) record the successful checks and parent links. The extraction manifest SHA-256 is `2bd0d7181678905054760ad59ea68ad21ce32aeac51026faa764f9db0f411f00`.

Together with the 746 individual first-pass candidates, this yields **803 geometric door candidates**. This count excludes the three giant floor clusters and does not assert working, complete or unique logical doors. The 893 original first-pass files remain unchanged and available as the canonical partition.

The connectivity mask is deliberately narrow:

| Parent | Exact platform Y | Masked original cells | Remaining non-air blocks |
|---|---:|---:|---:|
| `part0004` | 50 | 83,774 | 16,460 |
| `part0005` | 101 | 83,775 | 724 |
| `part0006` | 152 | 83,780 | 582 |

Completed parent coverage was:

| Parent | Child files | Piston candidates | Block entities | Top-level entities | Context instances | Unique context cells |
|---|---:|---:|---:|---:|---:|---:|
| `part0004` | 286 | 32 | 847 | 4 | 5,472 | 4,891 |
| `part0005` | 156 | 5 | 271 | 14 | 1,871 | 1,756 |
| `part0006` | 124 | 20 | 170 | 2 | 1,549 | 1,386 |
| Total | 566 | 57 | 1,288 | 20 | 8,892 | 8,033 |

All 17,766 nonplatform states were preserved exactly once; none of these three parents contained explicit cave/void-air states. The restored floor context contributes 8,892 cell instances from 8,033 unique source cells, so 859 instances are deliberate overlap. The other 243,296 masked platform cells remain only in the unchanged canonical parents. The derived files contain 26,658 non-air block instances in total; that number includes repeated context and must not be interpreted as additional unique source content. Their total compressed size is 816,331 bytes.

Only `minecraft:black_stained_glass`, `minecraft:white_stained_glass` and `minecraft:birch_planks` at that parent's exact platform Y are excluded from connectivity. Other materials, including the command block at `(201,50,152)`, remain. Those same materials at any other height remain. Holes are never filled, and inventories or controls are never repaired.

The masked copy uses the existing `split_connected_attach_nearby(Connectivity::Corner, 16, 3)` isolator. After grouping, each candidate receives only original masked platform cells inside its frozen nonplatform bounding rectangle plus a one-block horizontal halo, clipped to the parent's bounds. Restoring context after grouping prevents the floor from joining the candidates again. All group outputs are retained, including small fragments. Piston-bearing groups are labeled `door_candidate`; the other groups remain `other_component`.

Each restored context cell carries its original state. The manifest records every context position, its count and content hash, the precise mask, halo and parent hashes. Context can overlap between children and must not be summed as a disjoint partition. The full original platform remains in its unchanged canonical parent, including cells unused by any child. A restored floor under a floating component is source preview context, not proof of required support or complete piston travel space.

All **nonplatform** block states, including cave/void-air states, form an exact disjoint partition across children. Their block entities and top-level entities are preserved exactly once. Entities outside every residual physical component box are retained in explicit entity-only quarantine outputs. Every exported file must be an exact state subset of its raw parent and pass read-back checks. Metadata and native data version 4671 remain unchanged; coordinates remain in the original source selection. Block-entity wrapper coordinates use the same documented absolute-position normalization as the canonical extractor, while every other NBT payload bit, float bit and nested passenger is retained.

Nonplatform selection-boundary contacts and restored-context boundary contacts are recorded separately. Floor context touching an edge does not, by itself, establish that the mechanism was cut off. Adjacent component dependencies, floor/support requirements, activation controls, saved inventory contents and outdated mechanics all remain review items. A derivative is not admitted to a benchmark simply because its extraction passed.

The completed review counts are 52 components with nonplatform selection-boundary contacts and 54 with context-boundary contacts. Among the 57 piston-bearing candidates, four contact a source boundary, eight lack a saved activation control and nine have empty or absent saved inventory tags. These sets overlap. All 566 derivatives retain the floor-context and adjacent-dependency review flags; no runtime or vanilla functionality claim was added.

## Reproduction

Compile and run the Rust example on `harrison-work`; the input is the completed first-pass extraction directory. Output must be a new directory.

```bash
cargo build --locked --release --example isolate_archive_floor_parts
target/release/examples/isolate_archive_floor_parts \
  /path/to/world-record-plot-v2 /path/to/new-floor-extraction

/path/to/python-with-blake3 tools/register_tick_floor_parts.py \
  --archive /path/to/Nucleation-schematic-archive \
  --extraction /path/to/new-floor-extraction \
  --check-only

/path/to/python-with-blake3 tools/register_tick_floor_parts.py \
  --archive /path/to/Nucleation-schematic-archive \
  --extraction /path/to/new-floor-extraction \
  --report-summary docs/performance/schematic-candidates/world-record-floor-parts-summary.json \
  --report-catalog docs/performance/schematic-candidates/catalog/world-record-floor-parts-extraction.json.gz
```

Registration checks the original plot and canonical parent hashes, every exported BLAKE3, successful read-back flags, all nonplatform totals, unique entity ownership and bounded context accounting. It registers source `world-record-floor-parts` with immutable SHA-256 object storage and hardlinks. Reruns require byte-identical destinations; no originals are overwritten or removed. Schematic binaries stay local, while optional reproducible reports contain metadata only.

The production extractor's 566 groups and 57 piston-bearing candidates matched the earlier static preview. Every file passed source-subset and aggregate nonplatform ownership checks, metadata and NBT read-back checks, then local hash verification and registration. Functional testing and benchmark admission remain separate work.
