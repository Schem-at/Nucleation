# Community schematic library and validation

Fresh work on 15 November 2026. Rust compilation and execution, and the Minecraft
reference servers, ran on **harrison-work**. Downloads, metadata curation and
offline indexing ran on the Mac. These are correctness results, not new timings.

The local archive is `/Users/harrison/RustroverProjects/Nucleation-schematic-archive`.
Launch Circuit Atlas with `python3 tools/serve_tick_archive.py --archive /path/to/archive`
and open `http://127.0.0.1:8767`. See [dashboard setup](../../../apps/schematic-atlas/README.md).
The simpler `index.html` also opens directly: the search, source, format and priority filters
work without a server. Original downloads, provenance, byte hashes, source
READMEs, saved DataVersions and alternate formats are retained separately.

## Coverage

- The original collection has **914 schematic files, 862 distinct schematic
  contents**, across 13 source groups. The verified first plot extraction adds
  **893 separately downloadable components**; see the extraction reports below.
- **3,864 additional text layouts**, mainly Redstonepedia; these require conversion
  and validation before use in the Java 1.21.11 suite.
- Three independent curators produced **57 shortlisted test proposals**:
  20 logic/memory, 20 movement, and 17 storage/crafting. The user-provided
  World Record Plot's initial 20 survey leads have been superseded by full-plot
  extraction using the existing part isolator.
- All selected downloads completed. Binary NBT files were statically parsed;
  the 33 Lithium SNBT fixtures received version scanning only.
- Fourteen Bedrock exports, source-labeled broken/non-looping machines, files
  without save versions, and newer-version designs remain explicitly tagged.
  Alternate exports that omit minecarts/entities are not treated as equivalent.

Detailed findings: [logic](curation-logic.md), [movement](curation-movement.md),
[storage](curation-storage.md). The [source snapshot](archive-sources.json),
[integrity audit](archive-audit.json), and compressed metadata in `catalog/`
back up the curated catalog without redistributing the schematic binaries.
Source licenses/attribution are recorded per collection; no universal license
for all designs is assumed.

## Executed checks

| Design | Result | Evidence and scope |
| --- | --- | --- |
| LRA LFF006 D latch | Functional pass | 18 hold/capture endpoint checks |
| LRA DECODE004 programmable decoder | Functional pass | 32 checks, all 16 input words twice; tests its four identical saved rows |
| LRA ADD005 8-bit CLE adder | Functional pass | 656 sum/carry checks over 328 operand/carry-input vectors |
| LRA COUNT001 counter/load | Quarantined | First increment after loading zero returned zero, expected one; cause unresolved |
| LRA OTHERCOMB017 binary comparator | Vanilla tick-end parity pass | 960 ticks, 961 frames; both reference and selected optimization modes match Minecraft 1.21.11 |
| SleeperPin interlocking 3×3 door | Vanilla functionality pass; engine blocked | Two cycles restore the initial state; idle control is stable. Nucleation rejects an unregistered copper block before ticking |

The original eight selected files passed the production importer/exporter with
their source DataVersions, dimensions and non-air block counts preserved.
Exported block-entity payloads were serialized with NBT types and round-tripped
through the SNBT parser; this is not an assertion about every original metadata
field. OTHERCOMB017 was imported separately as a ninth
native-4671 design. All nine contain zero authored entities.

The four initial logic checks used native source version 4189, InWorld loading,
128 setup ticks, then actual lever/button interactions. These are functional
checks, not fresh vanilla traces. Both the reference engine and the selected
eight optimization features produced **identical complete result records and
ordered-record fingerprints**, including the counter failure. Shadow assertions
for the wire plan and transition signatures were enabled. See
[logic differential results](validation/logic-differential.json).

The comparator reference uses its original native-4671 file, actual lever pulls,
40 input stages and 72 input actions. An independent integer oracle checks the
equal/less/greater lamps at stage endpoints, including asymmetric low/high-bit
inputs. See [functional results](validation/comparator-functional.json).
Two fresh Minecraft worlds produced identical captured state sequences.
The Rust checker matched every initial/tick-end block, comparator output and
occupied flat inventory slot inside the declared watch bounds. See
[reference](validation/comparator-reference.json),
[selected shadow](validation/comparator-selected.json), and
[repeatability](validation/comparator-repeat.json).
This is a finite input sample, not exhaustive testing of all 65,536 input pairs.

The door's positive capture and unchanged-input control each cover 320 ticks.
The exact nine-block copper panel closes twice; blocks, comparator outputs and
inventory slots return to their initial maps after both cycles. See
[door assertions](validation/door-functional.md). Its engine rejection is kept
in [compare-cycle.json](validation/compare-cycle.json); no blocks were silently
replaced to obtain a pass. Crafter recipe execution is also outside the current
engine's implemented behavior, so crafting candidates require separate work.

These captures do not establish intra-tick notification parity, moving block
entity payload parity, nested item components, or an independent vanilla runtime
entity census. The checker rejects authored entities and engine runtime entities.
The established hipster and other regression suites remain necessary.

## Reproduce and extend

See [validation-plan.md](validation-plan.md) for exact commands and admission
rules. The archive keeps complete captures, original files and generated inputs
under `validation/candidate-run/`. Git retains small result records, capture
hashes, compiler/host/binary metadata, source pins, the catalog and tools.
Build caches, servers and third-party schematic binaries stay outside Git.

No production simulation code or default optimization setting changed in this
checkpoint. The additional checkers are standalone examples and the new oracle
driver is separate from the existing fixed-case driver. The benchmark suite has
not yet been expanded to time all library candidates: admission requires a
finite workload, positive and negative controls, observable activity and the
appropriate functionality/reference checks. No imported-but-idle machine counts
as a successful benchmark.

## World Record Plot

The original user file is retained unchanged as a local-only source. Its native 4671
selection contains 458,409 non-air blocks in 284×208×295 cells, 12,541 block entities
and 318 top-level entities. See the [survey](world-record-plot.md) and
[overview map](world-record-plot-map.svg). The [full extraction](world-record-parts.md)
produced 893 files, including 746 smaller physical door candidates, three shared
floor groups, and 144 other components. Every exported file passed source-state,
block-entity payload and entity read-back checks. Candidate boundaries
do not prove their surrounding dependencies are present. Saved inventories are
mostly empty; edge-touching mechanisms may be cut by the approximate selection.
Five moving-piston blocks have no saved piston block entities. These fixture
limitations must be resolved before treating a failure as engine incompatibility.

Circuit Atlas exposes [textured isometric PNGs](thumbnails.md), metadata, evidence, captured-tick
playback and an explicit live-browser launch for supported-size Java inputs.
The full plot exceeds the browser engine's 8-million-cell limit, so its live
launch is disabled in favor of the extracted files. Live interaction uses the existing
SimLab engine; it is not a new vanilla certification or a native speed result.
