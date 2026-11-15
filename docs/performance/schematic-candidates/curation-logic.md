# Logic and memory candidate curation

The local machine-readable catalog is
[`Nucleation-schematic-archive/curation/logic.json`](../../../../Nucleation-schematic-archive/curation/logic.json).
Its 590 rows join the archive manifest by **`source` + `path`**. Every row retains
its pinned URL, SHA-256, source records, evidence paths, mechanism tags, proposed
functional oracle/control and readiness. This is a candidate library, not an
admitted benchmark suite or a performance result.

## Evidence and coverage

| Source | Curated attachments/layouts | Evidence |
| --- | ---: | --- |
| [Logical Redstone Archive](https://github.com/mattbatwings/Logical-Redstone-Archive/tree/6d054aa0a87d8f58f672625b3e6d0e727bd97016) | 215 | Downloaded entry JSON/READMEs and saved NBT where supported |
| [Soontech current](https://github.com/Soontech-Annals/SoonNowArchive/tree/d58447452e7a2df336a989c4e1c389f2ecea8f7f) | 28 | Decoder, encoder, data-storage, item-memory and logic entries |
| [Soontech archive](https://github.com/Soontech-Annals/Archive/tree/dd1ab242c66e3de3805a5acffdcd5ef1d87c7740) | 30 | Original machine-readable descriptions/specifications and NBT |
| [Redstonepedia](https://github.com/vjeux/redstonepedia/tree/86a5e4843d8effe81bf7bf411a554cef8b58a1c9) | 317 | Adder, compute, gates, memory, T-flip-flop and randomizer text layouts |

259 binary files had their saved blocks/NBT decoded with the bounded Python
readers; 18 attachments contain actual **DataVersion 4671**. Declared Minecraft
versions remain separate fields. Fourteen Bedrock `.mcstructure` files are
quarantined as a different edition/schema; failure to parse them as Java gzip
NBT is not evidence that the downloads are corrupted. Redstonepedia layouts
carry no native DataVersion and target **26.3** in their source repository:
conversion plus independent 1.21.11 checks are required.

Mechanism tags come from decoded blocks where available, otherwise source text
or layout palettes. `stationary-logic-candidate` means no saved piston blocks or
authored entities were found; it does not certify runtime behavior or exclude
inventory/comparator dependencies. Complexity bands use decoded non-air counts
(small <128, medium <2048, large otherwise), not timing predictions. Litematic
multiregion counts are summed before production-import overlap resolution.
Every curated payload's SHA-256 was rechecked against its downloaded manifest.

## Shortlist

The JSON contains exact paths and per-case controls. Counts below are saved
non-air blocks, and versions are inspected DataVersion integers. Source codes
are scoped to their archive; for example current Soontech DC002 is not the old
archive's DC02.

| Rank | Candidate | Blocks / DataVersion | Purpose and current gate |
| --- | --- | --- | --- |
| 1 | LRA LFF006 D latch | 25 / 4189 | Repeater-lock hold/capture; 18 engine checks passed |
| 2 | LRA ADD005 8-bit CLE adder | 1,217 / 4189 | Carry-heavy arithmetic; 656 engine checks across 328 vectors passed |
| 3 | LRA DECODE004 decoder | 188 / 4189 | 32 engine checks passed against its actual repeated programming |
| 4 | LRA OTHERCOMB017 binary comparator | 272 / 4671 | Native-version candidate with an exhaustive 8-bit integer oracle |
| 5 | LRA ALU003 ALU | 1,059 / 4671 | Native-version arithmetic/logic/shift mix; map operation table first |
| 6 | LRA HEX008 single hex equality cell | 17 / 4671 | All 16 strengths and live inventory-driven comparison |
| 7 | LRA LFF003 hex D latch | 16 / 4440 | Analogue hold/capture, beyond boolean power |
| 8 | LRA REGISTER001 register | 202 / 4189 | Simple read/write memory before larger RAM |
| 9 | LRA REGISTER002 tristate register | 230 / 4440 | Inverted controls and simultaneous read/write contract |
| 10 | LRA SHIFT003 bidirectional shift/load | 687 / 4189 | Deterministic pistonless sequential workload |
| 11 | LRA REGISTERFILE002 register file | 2,468 / 4189 | Address isolation and 64 stored bits |
| 12 | LRA RWM001 data memory | 79,999 / 4189 | Larger topology; validate controls before scaling |
| 13 | LRA RANDOM005 LFSR | 3,285 / 4440 | Repeatable feedback activity with a software sequence oracle |
| 14 | LRA COUNT002 up/down/load counter | 518 / 4189 | Direction and load sequencing; independently establish protocol |
| 15 | Soontech current DS001 600-bit RAM | 14,623 / 2975 | Streaming plus pistons/rails; not a pure stationary case |
| 16 | Soontech current DC002 6-bit decoder | 2,239 / 2730 | QC and update-order coverage |
| 17 | Soontech archive LC03 await latch | 15 / 2730 | Delayed pulse release; includes a piston |
| 18 | Soontech archive LC08 SIPO | 747 / 3218 | Same-tick input ordering; needs stronger reference probes |
| 19 | Redstonepedia `fa3d-36b` | Text layout / no native version | Explicit ports and eight full-adder combinations; conversion required |
| 20 | LRA COUNT001 counter/load | 320 / 4189 | Failure triage only; excluded pending diagnosis |

The three engine passes and counter failure are retained in
[`validation/candidate-run/logic-results.jsonl`](../../../../Nucleation-schematic-archive/validation/candidate-run/logic-results.jsonl),
with its hash and per-case ordered-record fingerprints in the catalog. They used
InWorld loading followed by 128 ticks and endpoint assertions. They are **not
new vanilla captures** and do not establish every intermediate state. COUNT001
passed its load checks, then its first increment after loading zero returned
zero instead of one. Engine behavior, input protocol and source-version handling
remain possible causes; no vanilla incompatibility has been established.

## Controls and warnings

- **Decoder programming:** DECODE004's four saved rows are identical. Its source
  one-hot claim is conditional on unique programming. Preserve and test the
  original pattern; separately label any uniquely programmed derivative.
- **Memory gating:** REGISTER002 uses inverted read/write signals and documents
  zero output when reading while writing. Its hoppers also matter to glass-tower
  geometry. Do not replace them with inert blocks to simplify the fixture.
- **Broken source:** LRA RWM002 cached serial RAM is marked `Broken` upstream.
  Retain it for diagnosis, exclude it from accepted scores. Upstream
  `Functional`, `Tested & Functional` or `robust` tags never become local passes.
- **Timing:** source “ticks” may mean redstone ticks or game ticks. Derive an
  explicit input schedule and measure reference latency; lamp persistence is not
  necessarily electrical output latency. LC08 explicitly distinguishes inputs
  within one tick, beyond what tick-end equality alone proves.
- **Inventory scope:** item memories and hex cells require item/cooldown audits.
  Current stack-size and component limitations are described in the
  [validation plan](validation-plan.md). The simple community oracle does not
  capture item components, entities or moving-piston BE progress.
- **Versions:** the native community oracle accepts 4671 only. Older saved files
  need verified forward loading; newer files must not be relabelled or datafixed
  backwards. Keeping the original version for engine motion semantics remains
  necessary even after conversion for a reference server.
- **Duplicates:** 44 curated rows share exact payloads elsewhere in these four
  source manifests. Preserve provenance aliases but avoid treating duplicate
  binaries or large generated parameter sweeps as independent benchmark cases.
  The global archive index may find additional duplicates.
- **Licenses:** retain source notices. LRA has no established blanket schematic
  license; old Soontech has an Unlicense, which does not automatically cover new
  Soontech additions. Redstonepedia has a root MIT license. Local collection does
  not by itself authorize republishing third-party schematics in Nucleation.

Each proposed workload needs a software truth/sequence oracle, a deliberately
disabled-input/clock control, repeatable active transitions, checkpoint replay,
and matched 1.21.11 snapshots within their declared scope. Input actions and real
stepping belong inside the benchmark timer; construction, restore, vector
generation and assertions do not. No row in this curation is yet classified as
an admitted benchmark.
