# Fresh plot-door vanilla checks

These are fresh checks performed on **harrison-work on 15 November 2026**, not
historical handoff measurements. Six source-preserving extractions were exercised
in Minecraft Java 1.21.11 and the current native engine. Five demonstrate the
specified door operation in vanilla; **two match the complete native cycle**
after the focused lamp/trapdoor notification fix, and **one is admitted as a
working solid door** (the other is hollow in its closed state).
All six native idle runs match their vanilla captures. Loading, rendering and
an idle pass are therefore insufficient to call these builds compatible.

| Extracted candidate | Vanilla functional result | Native first cycle difference |
|---|---|---|
| `part0556`, lamp 2×2 | Pass, two cycles | **Pass**, all 661 frames |
| Floor `part0004-floorpart0119`, 2×2 | Pass, with original empty droppers | Tick 22, sticky piston not extended |
| `part0117`, horizontal 6×6 | Pass, two cycles | Tick 91, wall west arm differs; observer mismatch at 29 fixed |
| `part0101`, signed 7×7 | Pass, two cycles | Tick 24, wall east arm differs |
| `part0622`, nonflush 5×5 | Pass after correcting aperture-depth oracle | Tick 42, concrete powder remains where vanilla has air |
| `part0465`, apparent 3×3 | Does not meet solid-door criterion: closed center stays air | **Pass**, all 661 frames after lamp notification fix |

The first differences are observations, not diagnoses. For example, the powder
case needs gravity/entity investigation, and the oracle does not census vanilla
runtime entities. No source block, inventory or control was repaired or replaced.
The 3×3 may be an intentionally hollow design; it is not admitted as a solid 3×3
door merely because its eight-block perimeter cycles successfully.

## Reusable control protocol

[oracle-specs.json](oracle-specs.json) pins source paths and SHA-256, source
bounds, actual lever positions, aperture geometry and panel materials. Coordinates
are original plot coordinates. The production exporter rebases by the source
bounding-box minimum; both vanilla and native simulation receive that minimum as
their origin, preserving absolute location and position-dependent redstone order.
Native DataVersion **4671** remains 4671; no datafix or version relabeling occurs.

Each positive scenario loads the original blocks and typed block entities with
InWorld semantics, seed zero, and no placement updates. It captures tick zero and
660 subsequent ticks. It pulls the actual lever immediately before ticks
**21, 181, 341 and 501**, then checks closed/open/closed/open at ticks
**180, 340, 500 and 660**. It requires:

- An initially empty aperture and unchanged pre-input state.
- Complete closure with the declared panel material and complete reopening.
- Both open endpoints return to the complete initial captured state, including
  comparator outputs and flat occupied inventory slots.
- Both closed endpoints agree, and every endpoint is stable for its last 16 ticks.
- Real non-control activity and observed moving-piston states.
- A separate fresh-world, unchanged-input 660-tick control remains unchanged.

The lamp 2×2 has three levers. The driver uses only `[166,56,2]`; the other two
start powered and remain unchanged as essential constant inputs. Its four
aperture cells are `[165..166,54..55,3]` and close with redstone lamps. The
floor-derived 2×2 keeps its two face-to-face droppers empty as saved.

For the 5×5, the original proposed assertion incorrectly required all quartz in
one z plane. Captured geometry showed a nonflush wall: outer rows at z129, next
rows at z130, middle row at z131. The revised oracle requires all 25 x/y passage
rays to contain quartz in the 3-deep aperture, and all 75 cells to be air when
open. This revision is explicitly marked `revised_after_capture`; the original
failed oracle/spec remains in the raw artifacts. No input or capture changed.

## Evidence and limits

[oracle-results.json](oracle-results.json) contains source and binary hashes,
capture hashes, first differing state/position/tick, matching-frame counts,
functional failures and admission decisions. Full original exports, scenarios,
logs, captures and audits are retained outside Git:

```text
Mac: Nucleation-schematic-archive/validation/plot-oracles-v1/
Laptop: /home/harrison/nucleation-perf/plot-validation/oracles/
```

The runs use the already verified Minecraft 1.21.11 runtime and Temurin
21.0.13+9, fresh GameTestServer worlds, no client and zero connected players.
The runner rechecks the pinned server/mappings/library hashes and original
schematic hash. Servers run sequentially. Rust compilation ran only on
harrison-work, with locked dependencies; these are debug correctness checks,
not performance measurements.

Native comparison covers every captured tick-end block state, comparator output
and flat occupied inventory slot inside the watch box, including tick zero.
Authored entities and engine runtime entities are rejected. It does **not** prove
vanilla runtime entity equality, nested item components, moving-block-entity
payload equality or intra-tick notification order. The watch extends three cells
beyond the exported box. The lamp case is eligible for a benchmark using this
specific control protocol and scope; this does not certify every control mode or
other builds of its size.

## Reproduction

Build the exporter and checker on harrison-work (the root agent coordinates
builds so binaries are not overwritten during comparisons):

```sh
CARGO_BUILD_JOBS=4 cargo build --locked --features mc-tick --example export_tick_oracle_fixture
CARGO_BUILD_JOBS=4 cargo build --locked -p mc-tick --example check_community_capture
python3 tools/check_tick_plot_oracles.py run \
  --specs docs/performance/schematic-candidates/plot-validation/oracle-specs.json \
  --archive /home/harrison/nucleation-perf/atlas-render/originals \
  --exporter "$PWD/target/debug/examples/export_tick_oracle_fixture" \
  --checker "$PWD/target/debug/examples/check_community_capture" \
  --runtime /home/harrison/nucleation-perf/execution-next/vanilla/runtime \
  --jdk /home/harrison/nucleation-perf/execution-next/vanilla/jdk-21.0.13+9 \
  --output /home/harrison/nucleation-perf/plot-validation/oracles/batch-new
```

Append `--case plot_part0556_lamp2x2` to run only the passing lamp door. The runner
preserves each failure and continues gathering evidence; inspect its result JSON
rather than treating command completion as a test pass. New runs copy the two
binaries into the artifact directory before starting, so another build cannot
silently change a comparison. Full captures and copied binaries stay out of Git.

```sh
python3 tools/check_tick_plot_oracles.py audit /path/to/batch/case
python3 tools/check_tick_plot_oracles.py report \
  --runs /path/to/plot-oracles-v1 \
  --output /path/to/oracle-results.json
```

`audit` exits nonzero on a failed functional assertion. For the explicitly
revised 5×5 audit, add `--specs .../oracle-specs.json --case plot_part0622_5x5
--output .../functional-revised.json`. The report prefers that retained revision
and preserves both native mismatches and the original run status.

The focused engine fix changes three lamp/trapdoor state writes from silent
writes to flag-2 shape notifications. Fresh three/four-block vanilla fixtures
reproduce the missing observer pulses independently of the doors. Both positive
captures and both idle controls now pass, alongside the established 640-tick
hipster golden and the 961-frame binary comparator capture. Original mismatch
evidence and checker hashes remain in `native_before_fix`; the report names the
new checker hash separately. The change has its own rollback commit.

A second fresh vanilla run of the horizontal 6×6 exactly matched the first
initial state and all 660 subsequent snapshots; fixture and actions also agreed.
[repeat6x6.json](repeat6x6.json) pins both captures. This supports the reproducibility
of the remaining wall mismatch, rather than turning that mismatch into a pass.

To recheck preserved captures after a focused engine change without silently
replacing the original evidence:

```sh
python3 tools/check_tick_plot_oracles.py recheck \
  --runs /home/harrison/nucleation-perf/plot-validation/oracles \
  --checker target/debug/examples/check_community_capture \
  --output /home/harrison/nucleation-perf/plot-validation/oracles/recheck-new \
  --extra-capture /home/harrison/nucleation-perf/schematic-candidates/vanilla-comparator/capture.json
```

Pass its `results.json` as `report --native-results` to produce the updated
summary while retaining the original native results.
