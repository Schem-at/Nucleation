# Extender endpoint: the original payload is blocked

Verified 17 November 2026. `NanoLovesExtendingHisStickyPiston.litematic` does
**not extend its payload in the pinned vanilla Java 1.21.11 reference**. A fresh
2,000-tick run ends with all 12 honey blocks at their original positions, no
moving pistons, and zero pending block ticks, fluid ticks, block events or
rescheduled block events. The installed Atlas engine matches that endpoint.
The earlier 1,100-tick trace match established parity during activity, not a
working full extension.

## Cause and controlled checks

The payload is a bar of 12 honey blocks at `(147,3,8..19)`. Immediately behind
it are 12 rows of west-facing sticky piston bases starting at `x148`. Only the
front piston at `(148,3,19)` initially receives the activation sequence, through
the observer/barrier below it. Its load includes the honey bar **and the other
11 adjacent piston bases**: a connected group of 23 movable blocks, exceeding
the 12-block push limit.

A hand-authored native-4671 fixture isolates this without the long sequencer.
It powers just the last piston in each of four otherwise independent banks.
Fresh vanilla capture and the native/WASM simulators agree at every one of its
24 ticks:

| Control | Connected load seen by the powered piston | Result |
|---|---:|---|
| 12 honey, 12 adjacent bases | 23 | Blocked |
| 12 honey, only the powered base | 12 | Honey moves one block |
| One honey, 12 adjacent bases | 1 | Honey moves one block |
| 12 honey, powered base plus one passive base | 13 | Blocked |

The same isolation was also performed on in-memory copies of the complete
original with direct power at `(148,2,19)`: removing either the 11 additional
honey blocks or the 11 passive front bases allows the first push. Original
schematic bytes were preserved.

## Full runs and endpoint assertions

Both fresh full-run references execute every server tick and use the real
note-block interaction immediately before tick 21. They record initial state,
every 20th tick, and tick 2,000: **101 full frames per run**. They additionally
record final level-wide block/fluid tick and block-event counts, plus the number
of moving-piston states inside the watch. This avoids mistaking an aliased
20-tick animation cycle for a stopped mechanism.

| Build | Start honey | Final honey | Pending queues / moving pistons |
|---|---|---|---|
| Original, unchanged | 12 at `x147,y3,z8..19` | Same 12 positions | All zero |
| Diagnostic: retain only honey `(147,3,19)` | One at `x147` | `(1,3,19)` | All zero |

The diagnostic therefore travels **146 blocks west** in vanilla and in the
installed WASM. It isolates the excessive load; it is not a validated replacement
for the original. A second-click WASM probe moves that honey to `x19`, not back
to its initial `x147`. No extension/retraction cycle or intended 149-block stroke
is claimed for this altered schematic.

Both cases also ran for all 2,000 ticks in a fresh Chrome instance through the
live Atlas World. Both final simulation states match the vanilla endpoints;
both final render mirrors match the simulation, with no remaining piston flight
objects or browser errors. The UI has no additional endpoint discrepancy in
these runs.

The original changes only seven cells at rest: note 9 becomes note 10, the two
observer strips at `x146/147` shift one cell south, and their walls become tall.
The upper piston bank is unchanged. The exact WASM run becomes quiescent at tick
1,368; vanilla is proven clear of the measured pending work at tick 2,000, not
assigned an exact stopping tick from sparse samples.

Source metadata remains **26.3 / DV5023**. Reference metadata remains
**1.21.11 / DV4671**, with explicit exact-state loading and no datafixing. This
report does not claim a test against the 26.3 server. The watch spans
`[-4,-1,-1]..[300,4,23]`, including the inferred westward endpoint with margin.
Every watched chunk plus a halo is forced and checked for entity-ticking
readiness before placement; no players connect.

## Reproduction and retained evidence

`tools/gametest/scenarios/honey_piston_bank.json` and its paired SNBT are the
small authored fixture. Its committed golden is
`crates/mc-tick/tests/corpus/extender12111/honey_piston_bank.json`.
`cargo test --locked -p mc-tick --test honey_piston_bank` passes both the golden
replay and explicit functional 12/13/23-block assertions on **harrison-work**.

The capture harness now accepts optional `capture_stride`, preserving the
existing contiguous schema when omitted. Sparse output includes the exact tick
list and an explicit limited scope. Sixteen Python tests pass, including
sampling validation and unchanged defaults. Actual remote Java runs validate
both the sparse and ordinary paths. The WASM replay executes all intermediate
ticks and applies inputs on unsampled ticks, rather than skipping the tick-21
activation. The native contiguous-capture checker is unchanged.

```sh
python3 tools/gametest/run_community_capture12111.py \
  --runtime "$RUNTIME" --jdk "$JDK" --scenario "$SCENARIO" \
  --source-file "$SOURCE" --output "$FRESH_RUN"
node --max-old-space-size=4096 tools/check_tick_block_capture_wasm.mjs \
  apps/sim-lab-wasm/public/engine "$CAPTURE" "$SOURCE"
# From apps/sim-lab-wasm, exercise the actual browser World and render mirror:
CHROME_PATH="$CHROME" node verify-circuit-endpoint.mjs "$SOURCE" "$CAPTURE" "$RESULT"
```

The source SHA-256 is
`e3f1466374535f9eda361b0745dd6a81aa090aba5a961e3b93944b0d320e457b`;
the diagnostic SHA-256 is
`9a9567106d42715bc91f3caa2e84b6904f34246529f913ec93ce4e2def6db891`.
The served WASM remains the wall-fix binary
`f3469a884704684e445770d814e8b9ac98450d7c9658ff26b4e5cbd904519d74`.
This investigation changes verification tooling/tests, not simulation mechanics.

Raw schematics, captures, compiler/server logs, source hashes, scripts and
in-memory control results remain outside Git under
`Nucleation-schematic-archive/validation/extender-endpoint/` and on harrison-work
under `nucleation-perf/sticky-piston-note-trigger/`. Compact results are committed
in `validation/extender-endpoint.json`. No packages were released.

Sparse comparisons establish equality of sampled tick-end block states and the
endpoint, not every intermediate state, moving-block-entity payload or runtime
entity. Native small-control comparisons also check captured comparator outputs
and occupied inventory slots. The final activity counters do not census entities.
