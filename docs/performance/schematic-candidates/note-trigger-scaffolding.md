# Note-triggered piston mechanism: missing scaffolding updates

Checked on 17 November 2026. The supplied mechanism activates and continues
running in **vanilla Java 1.21.11**. Nucleation 0.12.3 accepts the input and
starts the first pistons, but its missing scaffolding support-state updates
stop propagation. The same failure occurs in the exact WASM binary served by
Circuit Atlas. **This report does not fix the engine or establish a completed
functional extension/retraction cycle.**

## Original and reference

- Original: `NanoLovesExtendingHisStickyPiston.litematic`, saved author `Purplers`.
- SHA-256: `e3f1466374535f9eda361b0745dd6a81aa090aba5a961e3b93944b0d320e457b`.
- Saved version: **26.3**, data version **5023**. This was identified from the
  installed official 26.3 client jar's `version.json`, not inferred from its name.
- One region, 297 × 4 × 22; 9,821 non-air blocks, no block entities or entities.
- Material counts: 5,198 observers, 1,788 sticky pistons, 1,337 normal pistons,
  596 scaffolding, 297 walls, 297 white concrete, 295 barriers, 12 honey blocks,
  and one note block.
- Trigger: empty-hand use of the note block at local `(0, 3, 0)`, changing
  `note=9` to `note=10`. The reference invokes the real
  `NoteBlock.useWithoutItem`, with a detached server player; no player connects
  or enters the level.

The retained physics reference is 1.21.11 (data version 4671). The source's 5023
metadata stays intact. An explicit `exact_states_no_datafix` scenario places its
block descriptors unchanged in 1.21.11, with origin `(0, 100, 0)`. The new mode
rejects block-entity payloads. All 9,821 descriptors must round-trip exactly
through vanilla's state parser and match the initial capture. An independent
NBT decode and the production schematic exporter agreed on every source cell.
This is a **1.21.11 compatibility experiment**, not a native 26.3 server run or
a general backward datafixer.

The pinned server, official mappings and extracted libraries passed the existing
runtime hash checks. Runs used Java 21.0.13 on **harrison-work**, with zero
connected players and seed 0. No Rust compilation was needed.

## First difference

The click occurs immediately before numbered tick 21. Initial states and the
first two activated tick-end frames match. At tick 23 the first mismatch is:

| Position | Vanilla 1.21.11 | Nucleation 0.12.3 |
|---|---|---|
| `(0, 1, 0)` | `scaffolding[bottom=true,distance=2,waterlogged=false]` | `scaffolding[bottom=false,distance=0,waterlogged=false]` |

The extending piston changes the scaffolding's support. Vanilla schedules its
support-distance/bottom update; the observer at `(1, 1, 0)` then detects that
state change and starts the next stage. Nucleation classifies scaffolding as
partial inert building material in `crates/mc-tick/src/vanilla.rs`, so it never
performs this transition. This is an absent block behaviour, not a cache
invalidation, note-click or renderer problem. Later differences may expose
additional mechanics; fixing the first mismatch does not prove the entire build.

| Same 160-tick protocol | Vanilla | Nucleation |
|---|---:|---:|
| Tick-end changed cells, summed over frames | 55,000 | 81 |
| Last tick with a block change | 160, still active | 36 |
| Idle control, changes without a click | 0 | 0 |

The idle comparison matched all 160 frames. The Atlas WASM also produced 81
ordered block records and stopped at tick 36, leaving the same scaffolding state
unchanged. Its binary SHA-256 is
`3e01cc423394ff67268e78d63b0fa771059c1a2020c2142ba45e8687660892b0`.

A longer vanilla run captured **1,100 ticks and 1,725,136 tick-end changed
cells** after one click. It was still active at tick 1,100 (899 moving-piston
cells), with no stable endpoint or exact repeated block frame in ticks 900–1100.
No changed cell touched the watch boundary. The 12 honey blocks remained at
their saved positions. This proves sustained activation, not the intended
completed stroke; the intended endpoint needs a separate functional assertion.

The first attempt to serialize the 1,100-tick capture exhausted Java's 2 GiB
heap while constructing its output string. The harness now streams JSON to a
writer. A fresh run completed and passed structural capture validation; the
failed attempt's logs remain retained separately.

## Small replayable failure

`tools/gametest/samples/scaffolding_note_trigger.snbt` is a hand-authored
eight-block native-4671 fixture: a note block, two observers, one piston, three
scaffolding blocks and a stone support. It is not a copy of the archived build.
Its paired scenario is
`tools/gametest/scenarios/scaffolding_note_trigger.json`.

Fresh vanilla capture reproduces the same first mismatch at tick **3**, after
the click before tick 1. Vanilla updates the scaffolding and pulses the side
observer; the published engine misses that pulse. It has 12 tick-end changes
versus the engine's 8. The original source-version difference is therefore not
needed to reproduce this missing mechanic.

On harrison-work, from the repository, with `RUNTIME`, `JDK` and a fresh `RUN`
directory selected:

```sh
python3 tools/gametest/run_community_capture12111.py \
  --runtime "$RUNTIME" --jdk "$JDK" \
  --scenario tools/gametest/scenarios/scaffolding_note_trigger.json \
  --source-file tools/gametest/samples/scaffolding_note_trigger.snbt \
  --output "$RUN"
/home/harrison/nucleation-perf/release-installed-venv/bin/python \
  tools/check_tick_schematic_capture.py \
  --source tools/gametest/samples/scaffolding_note_trigger.snbt \
  --capture "$RUN/capture.json" --output "$RUN/comparison.json"
```

The comparison **currently exits 1 with an expected, real mismatch**. Do not
interpret a structurally valid vanilla capture or successful import as parity.
The comparator checks source hash/version, identical initial state and every
captured tick-end block including air; it does not compare entities or intra-tick
notification order. Seven scenario validation tests also pass, covering explicit
cross-version opt-in, provenance preservation and rejected payloads/actions.

Original schematics, full captures, source inspection, binary metadata and logs
remain outside Git:

- Laptop: `/home/harrison/nucleation-perf/sticky-piston-note-trigger/`.
- Local archive: `Nucleation-schematic-archive/validation/sticky-piston-note-trigger/`.

The compact fixture and replay tools are version controlled. The next engine
change must implement scaffolding support updates and observer notifications,
validate their timing against this oracle, then replay the full original to
discover any subsequent divergence. The original remains a known failing
simulation candidate, not an admitted vanilla-parity benchmark.
