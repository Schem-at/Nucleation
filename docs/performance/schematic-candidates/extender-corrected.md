# Corrected extender: full 149-block extension and return

Verified 18 November 2026. The newly supplied
`NanoLovesExtendingHisStickyPiston (1).litematic` works in the pinned Minecraft
Java **1.21.11** reference and the current Atlas engine. No further engine
change was needed after the wall-shape implementation.

The replacement adds **3,269 barrier blocks** and replaces **12 honey blocks
with white concrete**. This verifies the combined supplied correction. The
earlier [blocked-payload report](extender-endpoint.md) applies to the old file;
its push-limit controls did not establish the missing-power fault in the
intended design. Original files remain unchanged and outside Git.

| Checkpoint | Expected and observed payload | Result |
|---|---|---|
| Initial | 12 white concrete at `x147,y3,z8..19` | Matches |
| After first note interaction, tick 2,000 | All 12 at `x−2,y3,z8..19` | Full 149-block extension |
| After second interaction, tick 4,000 | All 12 back at `x147,y3,z8..19` | Complete return |

Inputs use the actual note block at local `(0,3,0)`, immediately before ticks
21 and 2,001. The full final block layout equals its initial layout except
the note block's pitch, which changes from 9 to 11 after two interactions.
Both independent vanilla endpoints have zero pending block ticks, fluid ticks,
block events, rescheduled events, and moving pistons in the watch.

The WASM run first changes the payload at tick 254, and becomes quiescent at
ticks 1,368 and 3,348. At 20 TPS the initial delay before payload movement is
about 12 seconds, with roughly 67 seconds from each click to quiescence.
These are simulated tick counts, not native performance measurements.

## Evidence and scope

Two fresh playerless vanilla captures ran on **harrison-work**: 2,000 ticks
for extension, and 4,000 ticks for extension plus return. Every server tick
executes; storage is sampled every 20 ticks. The production Atlas WASM importer
matches every full watched block-state frame, including air:

- Extension: **101 matching frames**, 2,982,996 recorded block changes.
- Cycle: **201 matching frames**, 6,433,092 recorded block changes.
- Real Chrome/Atlas `World`: initial, extended and returned worlds match vanilla;
  render mirrors match the simulation at all three checkpoints. Both endpoints
  are quiescent with zero remaining animation flights and no browser errors.

This establishes sampled tick-end block-state parity and concrete functional
outputs. It does not claim every-tick or intra-tick equality, moving-block-entity
NBT equality, inventory/entity parity, or performance improvement. The original
has no saved entities or block entities. Source **DV5023** is preserved; the
reference runs with explicit `exact_states_no_datafix` against 1.21.11/DV4671.
All watched chunks plus the halo are forced to tick before placement. The
watch is `[-4,-1,-1]..[300,4,23]`, translated by `[0,100,0]` on the server.

The engine WASM is unchanged, built previously on harrison-work with
`bridge,mc-tick,meshing`:
`f3469a884704684e445770d814e8b9ac98450d7c9658ff26b4e5cbd904519d74`.
No Rust compilation or Rust test run was needed for this source correction.
The thumbnail used the existing harrison-work native renderer and 26.3 visual
assets; that asset version is separate from the physics reference.

## Reproduction and regression assertions

[The source-pinned protocol](validation/extender-corrected-protocol.json) defines
the note interactions and all 12 output positions at three checkpoints. The
browser verifier checks these positions against vanilla before launching, then
against the live engine and render mirror. It requires quiescence and no
animation flights at each functional endpoint. Merely matching a stationary
vanilla build cannot pass these output assertions.
A deliberate incorrect `x−1` endpoint was rejected by the vanilla preflight
before browser launch. The new archive entry also loads directly in Atlas with
all 13,090 blocks and no browser errors.

The scenario generator imports the byte-identical source using the production
WASM importer. Its output was compared structurally with the scenario used for
the fresh capture. It refuses a different source hash and existing output file.

```sh
node tools/gametest/prepare_corrected_extender.mjs \
  apps/sim-lab-wasm/public/engine \
  '/path/NanoLovesExtendingHisStickyPiston (1).litematic' \
  /external/extender-cycle-scenario.json

# On harrison-work, using the retained pinned runtime/JDK:
python3 tools/gametest/run_community_capture12111.py \
  --runtime /home/harrison/nucleation-perf/execution-next/vanilla/runtime \
  --jdk /home/harrison/nucleation-perf/execution-next/vanilla/jdk-21.0.13+9 \
  --scenario /external/extender-cycle-scenario.json \
  --source-file '/path/NanoLovesExtendingHisStickyPiston (1).litematic' \
  --output /external/extender-cycle-capture --timeout 600

node tools/check_tick_block_capture_wasm.mjs \
  apps/sim-lab-wasm/public/engine /external/extender-cycle-capture/capture.json \
  '/path/NanoLovesExtendingHisStickyPiston (1).litematic'

# With Atlas serving on 127.0.0.1:8767 (CHROME_PATH may select installed Chrome):
node apps/sim-lab-wasm/verify-circuit-endpoint.mjs \
  '/path/NanoLovesExtendingHisStickyPiston (1).litematic' \
  /external/extender-cycle-capture/capture.json /external/browser-result.json \
  docs/performance/schematic-candidates/validation/extender-corrected-protocol.json
```

The separate extension capture uses the same generated scenario with
`capture_ticks: 2000` and just the first action. Compressed `.json.gz` captures
are also accepted by both comparison tools.

## Retained artifacts

[Compact results and hashes](validation/extender-corrected.json) include vanilla
activity counts, both comparisons, and browser checkpoint results. Source hash:
`ef09579af4818c2629b2aa894a3e7074ee620d0cdea8bb082baebee79c26bbee`.

Raw sources, expanded scenarios, captures, detailed source diff, logs, and
thumbnail remain outside Git under
`Nucleation-schematic-archive/validation/extender-corrected/`. Reference captures
also remain on harrison-work in
`/home/harrison/nucleation-perf/sticky-piston-note-trigger/`.

The local archive retains the original by hash and adds a separate corrected
entry, without replacing other designs:
[open the corrected extender in Atlas](http://127.0.0.1:8767/#entry=ef09579af4818c2629b2aa894a3e7074ee620d0cdea8bb082baebee79c26bbee).
It is a functional benchmark candidate; no timing comparison was run here.

The browser verification commands above use the [historical viewer test checkout](https://github.com/Schem-at/Nucleation/tree/6d1f5f1f/apps/sim-lab-wasm).
