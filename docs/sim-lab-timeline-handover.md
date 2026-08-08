# Sim-lab timeline & GLB export — handover

Parked 2026-08-08 with Stage 3b four tasks of six complete. Everything below
is committed on `master`; the working tree is clean.

**Spec:** `docs/superpowers/specs/2026-08-06-sim-lab-event-timeline-glb-export-design.md`
**Plan (Stage 3b):** `docs/superpowers/plans/2026-08-08-timeline-strip-and-export.md`
**Ledger** (per-task rulings, deferred minors, review outcomes):
`.superpowers/sdd/2026-08-08-timeline-strip-and-export/progress.md` — git-ignored
scratch, so read it before any `git clean -fdx`.

## What the goal was

Record a run, see on a timeline *when* things happened, select a span or a
detected cycle, and export that range as an animated GLB — off the main
thread. Stages 1–3a built the library and bridge for this; Stage 3b is the
app.

## What landed

| Commit | What |
|---|---|
| `b12d2396` | bindings regenerated with real JS booleans |
| `4419483d` | Record/Stop; `drainChanges` honours a refused clear |
| `b82b1d9c` | a missing `clearChanges` no longer masquerades as a successful one |
| `09a8d185` | `changes_json_from(n)` — drain reads only the new tail |
| `5e09d738` | the activity strip: one column per *active* tick |
| `7fd58c5e` | input-only ticks are visible (were rendering at zero height) |
| `398d1933` | drag to select a span; `find cycles`; click a cycle to select it |
| `6cb57c4c` | `gen-bindings.sh` fails if `diplomat-tool` predates the bool fix |

Numbers worth keeping: the per-frame drain went from **10.3 ms and climbing
at tick 200 (14× growth) to a flat 0.1 ms**. That regression had reappeared
specifically *while recording* — the one situation where a session runs
longest — because `clear_changes()` rightly refuses while a timeline is
active, and the only read available was the whole log.

## What remains

**Task 5 — export in a worker.** `export-worker.ts` owns its own engine and
pack; schematic + timeline JSON in, transferred GLB out; spawned lazily.
**Task 6 — export probe + handover.** `verify-export.mjs` asserting
*structure*, not bytes.

Then a **whole-branch review**. Do not skip it. The worst defect of this
whole project was invisible to every per-task review and obvious to the
whole-branch one: two individually-correct commits that together corrupted a
recording. Prompt it for *interactions*, not diffs.

Extract a task brief with:
`.claude/plugins/…/subagent-driven-development/scripts/task-brief <plan> 5`

**Note:** the plan's Task 5/6 "verify by hand"/"by eye" steps are not
executable by an agent. Both prior tasks replaced them with Playwright probes
against `window.simlab` (`stripprobe.mjs`, `stripselectprobe.mjs`, pattern in
`verify-handover.mjs`). Do the same rather than skipping.

**Already known about Task 5:** the mesher is **not byte-deterministic** —
three runs of the same input produced three different SHA-1s. Any "same GLB"
assertion is unsatisfiable; assert structure (node counts, track counts,
channel kinds) instead. `to_animated_glb` already exists and already emits the
Blender-shaped scene; Task 5 is wiring, not a new glTF path.

## Open questions and traps

### `detect_cycles` finds whole-scene translation, not a moving machine

**The most important thing on this page.** The plan named BB as the fixture
for `find cycles`; BB detects nothing, even after 8000 driven ticks. This is
**not** a bug and **not** a fixture quirk — it is a design limitation,
confirmed by code reading and by the detector's own doc comment ("translated
matches subtract only one bounding-box origin").

`StateFrame::from_blocks` (`crates/mc-tick/src/timeline.rs:62-89`) computes a
single `origin` as the min corner over *every* non-air block in the frame.
`same_translated` (line 95-108) and `detect_cycles`'s candidate filter
(`first.origin != digest.origin`, line 393-396) both key off that one
whole-world origin. So a machine that flies *inside* a larger static housing
barely shifts that origin, and the translated comparison never lines up.

Consequence: `find cycles` will silently return `translated: null` for the
common case of a moving machine in a static frame — which is most realistic
redstone builds, including the plan's own named fixture. Exact cycles (a
build returning bit-for-bit to a previous state, e.g. a door opening and
closing) do work.

**Decide this before Task 5**, because "export a cycle" is a headline feature
and the export range depends on it. The fix, if wanted, is per-object or
per-region motion detection rather than one scene-wide origin — a library
change in `mc-tick`, not an app change. Rated Important by review.

Task 4 proved the plumbing on a `6x6_sliding_door` fixture (open+close
returns exact in 30 ticks) and disclosed the substitution rather than
reporting a green run against a fixture that quietly proved nothing.

**BB never quiesces.** Any assertion of the form "nothing happens, so nothing
changes" needs a settling fixture; BB cannot prove it.

**`apps/sim-lab-wasm/public/engine/` is a copy of `bindings/`** with nothing
guarding drift. A stale copy silently disagrees with the source of truth.
This is the same class of trap as the diplomat one just closed, and it is
still open. If a binding call is missing at runtime, rebuild:
`NUCLEATION_WASM_FEATURES=bridge,simulation,meshing,mc-tick ./tools/package-npm.sh dist/npm-mctick`
then `cd apps/sim-lab-wasm && npm run sync-engine`.

**Build the narrowest feature combination, not just the union.** Building only
the union has twice hidden cross-boundary breakage here. `tools/prepush.py`
now gates this.

**Open findings from Task 4's review** — approved with these outstanding,
deliberately not fixed because the branch was being parked. Fix them at the
whole-branch review or as Task 5 opens:

- *Important, untested:* the drag uses no `setPointerCapture` and handles no
  `pointercancel`. A cancelled gesture leaves `dragging` set until the next
  `pointerdown` (self-healing but wrong), and **touch/pen dragging likely
  does not work at all** — a touch `pointerdown` implicitly captures to its
  target, so sibling columns never see `pointerenter`. `stripselectprobe.mjs`
  only drives `page.mouse`, so this is unverified rather than known-broken.
- *Important, untested:* the **translated-cycle render path was never
  exercised with a truthy value** — the door fixture only ever yields
  `translated: null`. The code mirrors the tested `exact` block, so risk is
  low, but it is unproven. This is downstream of the `detect_cycles`
  limitation above: no available fixture produces a translated cycle.

**Deferred minors** (triage at the whole-branch review; full text in the
ledger): `findCycles`'s bare `catch { setCycles(null) }` makes a real bridge
or JSON failure indistinguishable from the legitimate null-cycle case, and
unlike `pollActivity`'s catch it isn't documented as deliberately silent; the
drag-commit effect re-subscribes its `pointerup` listener on every column
crossed (churn, not incorrectness); `stopRecording()`'s `finally` flips
`recording = false` even if `stopTimeline` throws for an unrelated reason;
`drainprobe.mjs`'s phase-2 timing gate has a 0.5 ms floor that is coarse
against ~0.1 ms harness quantisation; the delivery-conservation check cannot
distinguish a bug that drops and duplicates by exactly offsetting amounts.

## Invariants earned the hard way

- **Never clear anything you have not successfully read and parsed.**
  `drainChanges` reads → parses → builds its result → *then* clears. Two
  review rounds were spent on that ordering.
- **A run timeline is a seed plus the change log.** Clearing the log
  mid-recording leaves replay reconstructing a confidently wrong world with
  nothing to signal it. That is why `clear_changes()` returns `false` rather
  than obliging.
- **The cursor moves only on a known result** — `0` on an explicit `true`,
  `from + all.length` on an explicit `false`, untouched otherwise. `all` is
  the *tail*, not the whole log; using `all.length` there walks the cursor
  backwards and re-delivers a growing overlap forever.
- **Presence in `activity.ticks` must imply visibility.** A tick enters that
  list on changes *or* inputs *or* pistons; anything rendered at zero height
  is also unselectable.
- **Selection identity is exact ticks, never column indices.** Buckets
  re-partition under the 250 ms poll.
