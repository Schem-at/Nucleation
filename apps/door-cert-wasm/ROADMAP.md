# Door validator — ideas from the Purplers conversation

Captured 2026-07-29 from Nano ↔ Purplers. Purplers is describing what the
competitive piston-door scene actually measures, which is worth more than any
metric we would invent ourselves. Ordered by value, not by effort.

---

## 1. The timings are currently wrong — measure the doorway, not the quiet

**"yea those opening and closing times are wrong lol" / "It's settle times"**

Today `open_ticks` is the last tick carrying *any* block change after the lever
click — i.e. when the whole machine goes quiet. That is not when the door
opened. A door whose panels clear in 8 ticks but whose tape keeps shuffling for
another 12 reports 20.

The fix is now cheap because the passage extractor exists (`src/lib/aperture.ts`):
we know exactly which cells are the doorway.

- **Open time** = first tick at which every passage cell is clear.
- **Close time** = first tick at which the passage is blocked again.
- Keep settle time as a *separate, honestly-named* stat: "quiet at t=N" already
  appears on the activity chart, and the gap between "open" and "quiet" is
  itself interesting (a door that opens fast and thrashes afterwards).

**This is the single highest-value fix on the list** — every number on the
certificate hangs off it, and the door community will spot it instantly.

## 2. Reset time (Purplers' algorithm)

The time before the input can be used *again*. There is no closed-form way to
compute it without understanding the door, so measure it empirically:

```
toggle input → wait X ticks → toggle input → run to quiescence
does the world match the state it started in?
  no  → X += 1, retry
  yes → X is the reset time
```

Two directions, and they differ:

- **opening reset**: `open → closed → open`
- **closing reset**: `closed → open → closed`

Purplers suggests two separate runs; Nano's variant is one run of four toggles
(`open → closed → open → closed`) which gets both. Worth benchmarking both —
the four-toggle version is cheaper but couples the two measurements, so if a
door is asymmetric the second half starts from a state the first half chose.

Implementation notes:
- Start X at 0 and walk up; cap it (say 200) and report "does not reset" beyond.
- Checkpoint/restore (already on `TickSimulation`) makes each trial cheap —
  snapshot once at the cycle start and restore per attempt instead of rebuilding.
- With ~4,000 evals/sec headroom in the browser this is nothing.

### 2b. Negative reset time

**"run a check to see if the reset time is negative — meaning the lever can be
operated again before the door has finished closing/opening. it's quite rare
for doors to have that, but some do."**

Falls straight out of §2: compare reset time against open/close time. If reset <
open, the door accepts a re-trigger mid-stroke — a genuinely impressive property
and worth a badge. We already know the vault survives interrupted strokes
(conformance covers it), so the machinery is there.

## 3. Input detection beyond levers — BUILT

`detectInputs()` in `src/worker/certify.worker.ts`. Candidates are levers,
buttons (any material), note blocks and pressure plates; each is actuated in a
throwaway simulation and the trial is scored by `aperture()` — did a walkable
passage open? Not "did anything move": playing a note block in `fast tgm 4x4`
BUDs a piston and shuffles four cells, and in the 6 × 6 it briefly leaves two
`minecraft:moving_piston` placeholders, so a movement test alone puts three
drivers on a one-lever door. Rarity orders the queue and nothing else; every
candidate is still tried. A second control counts as a second input only if it
opens the SAME passage.

Still open: the engine delivers no power from a floor `stone_button` or
`stone_pressure_plate` under `TickSettleMode.InWorld` — the button's own
20-tick press/release is simulated correctly, but the door never moves. So no
button or plate door can certify yet; the detector is ready for one.

Original notes below. Real builds use:

- **Levers** — the common case.
- **Buttons** — both as inputs *and* to redirect redstone. Cannot be
  distinguished statically; brute force.
- **Note blocks** — "a pain"; brute force.
- **Levers used as redirectors** — rare but real ("I know I did").

**Brute force is the right answer** ("there's scenarios where the simplest
solution is the best"): for each candidate input, checkpoint → actuate → run →
does the passage change state? The one that opens the door is the input. Cheap,
robust, no heuristics to get wrong. Report ambiguity if several qualify (a door
with two independent inputs is a real thing).

### 3b. Odd-one-out heuristic (optimisation, not truth)

**"check the blocks for an odd one out — for example, 1 block of lime
wool/concrete amongst loads of cyan"**

A good *ordering* hint for the brute force so the likely input is tried first —
not a detector on its own. Rank candidates by material rarity within the build.

## 4. Qualifier tags — what pro door makers actually compete on

**"observerless, slimeblockless, and cycle-less are the 3 main qualifiers that
pro door makers use atm"** — plus **dustless**.

- **Observerless** — trivial from the census we already compute.
- **Slimeless** — must check **honey and slime** together; honey is the usual
  substitute and a "slimeless" door using honey is not what the tag means.
  Check the exact community definition before shipping the badge.
- **Dustless** — trivial from the census.
- **Cycle-less** — "does it run a piston tape or not". Not trivial, but
  detectable: a cycling door has pistons that fire repeatedly on a period while
  idle or during the stroke. We already detect flight periods in the GA via
  rise-gap analysis; the same shape of test applies to piston-event periodicity
  in the change log. Report as: cycle-less / runs a tape of period N.

These four are one-line badges on the certificate and immediately legible to the
target audience.

## 5. Entities: boats, minecarts, mobs

**"Next step getting doors with boats and minecarts working" / "eventually
you'll have to throw mobs in there too"**

Minecarts already simulate (rails, powered-rail chains, cart physics are all in
the engine and oracle-verified). Boats and mobs are not modelled at all.

Order of attack: minecart doors first (should mostly work today — worth simply
trying one), boats next (entity physics, water interaction), mobs last (AI is a
different kind of problem and probably out of scope; a mob used as a redstone
component is usually stationary — a pressure-plate weight or a comparator-read
container — which may be tractable without real AI).

## 6. X-ray / propagation view — BUILT

**Nano: "an X-ray view for both redstone updates and just events, so you can see
things propagate through the door, even on the subtick — for example see the
updates propagate through leaves."**
**Purplers: "make the blocks translucent and then highlight them when they
receive an update."**
**Nano: "and color them depending on the tick phase or update type."**

The engine already emits exactly this data — `MC_TICK_TRACE_UPDATES` traces every
neighbour update, and the notify log records the block *at dispatch time*, which
is what makes intra-tick propagation legible at all. Nothing new is needed
engine-side; it needs a bridge surface (per-update JSON: tick, phase, position,
kind, source) and a rendering mode.

Design sketch:
- Whole build drawn translucent; a cell flares when it receives an update.
- Colour by **tick phase** (the engine has `Phase`/`PHASE_ORDER`) or by update
  **kind** (neighbour vs shape vs block-event) — user's choice of channel.
- A sub-tick scrubber: step *within* a tick through the update queue, not just
  between ticks. This is the part no other tool can do, because it requires an
  engine that models update order faithfully — which is precisely what we spent
  the project building.

This is the most differentiated feature on the list. Everything else measures a
door; this one *explains* it.

**Shipped** as a mode on the existing replay (`src/components/MeshReplay.tsx`,
`src/lib/xray.ts`, recorded in `certify.worker.ts`), verified by
`scripts/verify-xray.mjs`. Notes worth keeping:

- The engine's `updatesHeatJson` / `updatesWaveJson` are the drawable views;
  the raw `updatesJson` (15.8 MB per 6x6 cycle) is never fetched. A whole
  cycle is 0.88 MB of heat + 1.85 MB of waves, packed to ~1.5 MB of typed
  arrays and transferred, not copied.
- `record_updates(false)` **drops** the log. Read the JSON before switching
  the recorder off.
- Four tick phases fire in a door, and four categorical hues cannot pass the
  all-pairs colour-vision gate — so the fourth (`boundary`, the out-of-tick
  lever action) is drawn as a hueless wireframe cage instead of a fifth
  colour. See the note in `lib/xray.ts`.
- Flares are opaque depth-tested marks, not glow: additive clips a five-deep
  build to white and max-blending invents hues at overlaps.

---

## Notes

- The classifier was called "a bit buggy" in this conversation — the aperture
  extraction has since been rewritten around the open passage (Harrison's idea),
  which fixed the 4×4 vault reading 6×5. Worth re-showing.
- Everything here runs in the browser with no backend, which is the constraint
  that makes brute-force approaches attractive: compute is free and local.
