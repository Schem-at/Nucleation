# mc-tick roadmap

Where the engine stands and what is left. Ordered by leverage, not by size.

## The discipline that got us here

Every behaviour is derived from a **captured vanilla trace** or from the game's own
**unobfuscated bytecode** — never from memory. That is not ceremony. It has already
overturned four things I would otherwise have written wrong:

- dust settles *synchronously*, not one block per tick
- a piston's move takes *two ticks* and lands in the block-entities phase
- quasi-connectivity is real and reaches exactly one block up
- a repeater takes input on the side its `facing` names

Anything below that says "capture first" means it. The cost of a wrong guess here
is not a failing test; it is a plausible number that is silently off by a tick.

---

## Done since this was written

Items 1-4 are complete: the differ is wired (`tests/conformance.rs`), observers are
implemented, both inlined constants are captured, and `.snbt` loading plus a
`vanilla` descriptor-to-behaviour registry mean an arbitrary structure can be run
without hand-registering anything.

Four real defects fell out of the diff, none of which unit tests would have caught:
a vacated source becomes **air** rather than a placeholder; a position that is both
source and destination must not be cleared; a piston is **not part of its own
structure** (slime beside it dragged it along); and deferred writes were bypassing
the change log, so a movement landed while the trace ended two ticks early.

It also corrected a *method* error. Goldens captured by diffing snapshots between
ticks cannot know intra-tick order — what they record is the capture's scan order.
Comparing that strictly compares two arbitrary iteration orders, so both sides are
now canonicalised. An instrumented capture would know the real order and must not
be canonicalised; `Trace::canonicalize` says so.

---

## 1. ~~Wire the trace differ into the corpus~~ — done

**Why first:** captures are currently compared **by eye**. `mc-tick-trace` already
has `diff`, tolerance handling and divergence reporting, and `tools/gametest` can
produce traces on demand — but nothing joins them. Wiring them turns every capture
from a one-off investigation into a permanent regression test, and every subsequent
item on this list gets cheaper.

- Teach the corpus runner a `trace <name>.json` expectation.
- `UPDATE_GOLDEN=1` re-records.
- Commit the goldens captured so far: dust, torch+repeater, QC, slime, comparator
  side inputs, sticky pulse.

**Done when** breaking a delay constant fails a corpus case naming the exact tick.

## 2. ~~Observers~~ — done

**Not implemented, and most piston doors need one.** An observer watches the block
it faces and emits a short pulse on change. That pulse interacts directly with the
block-dropping behaviour already captured, so it is on the critical path for the
product rather than a nice-to-have.

Capture first: pulse length in game ticks, tick priority, and whether it fires on
its own placement.

## 3. ~~Retire the two assumed constants~~ — done

`MAX_PUSH_DEPTH` (12) and `MAX_RECENT_TOGGLES` (8) are javac-inlined, so they were
taken from convention and asserted in tests rather than read. Both are now cheap to
settle empirically:

- push depth: a 12-block column versus 13, one moves and one does not
- burnout: toggle a torch clock and count the toggles before it stalls

## 4. ~~Close the Rust side of the load/execute/verify loop~~ — done

The oracle loads `.snbt` and runs it; the engine cannot. Until it can, the two
sides are compared through hand-written corpus cases rather than the *same input*.

- A minimal SNBT structure reader. It must **not** pull in nucleation — that would
  cost the 0.7s edit-test loop, which is the substrate everything else rests on.
  Either a small shared crate or a reader local to mc-tick.
- Then `load foo.snbt` in the corpus runner, which currently fails loudly on
  purpose.
- This is also the natural **UniversalSchematic** touchpoint: conversion at the
  boundary, with the engine keeping its interned `u16` states.

## 4b. ~~Run the manual engine~~ — done

**The manual engine runs end-to-end, twice over.** Two goldens now conform
tick-for-tick:

- `manual_engine_settle.json` — placement alone runs the machine two full
  9-game-tick steps (placement pulses every observer, which acts as one
  trigger); it stops at tick 21 against its own blocks frozen at a chunk edge.
- `manual_engine_click.json` — the padded variant: placement cycle, quiescence,
  then a `--use` click on the note block at tick 30 runs a complete second
  activation through tick 55.

What fell out of making that pass (each verified in bytecode + capture; the
details live in `src/redstone_components.md` under "Manual-engine session"):

- Note blocks implemented: synchronous `powered`, block-event play, click
  cycles the pitch. `use_block`/`on_used` is the new player-input path.
- **Boundary time**: actions between ticks schedule with "now" = the last
  completed tick, one tick sooner than in-phase schedules (`TickCtx::boundary`).
  This corrected a latent off-by-one for every boundary actuation.
- Observers emit from their **back face only** (`VanillaRules` emission
  directions); placement shape-updates every block from all six sides, so every
  observer pulses once at placement; moved blocks re-examine their world when
  they land.
- Pistons re-validate power at event dispatch; move writes are silent; the
  retracting *base* is a two-tick `moving_piston`; pushed/pulled placeholders
  are always `type=normal`; block events deduplicate like vanilla's set.
- `Simulation::set_ticking_bounds` models chunk-edge freezing, which is what
  stops a free-flying machine in a capture.

`tools/gametest/capture.sh` now wraps the whole staging-and-capture flow, and
`TraceCapture` has `--use x,y,z` / `--use-tick N` (the exact
`GameTestHelper.useBlock` click sequence with an equivalent mock player).

### The original plan, for reference

**The concrete first real-schematic target is downloaded and waiting:**
`tools/gametest/samples/manual_engine.litematic` — a "2-Step 9gt Manual Engine",
a real community slimestone flying-machine engine. Small and self-contained
(5×3×3), which is why it beats `trencher.litematic` (15×378×21, too big for a first
run) as the first end-to-end conformance case.

Contents (via `cargo run --example dump_schematic <file>`):

```
1  note_block          <- the manual trigger; a PLAYER clicks it to start the engine
3  observer            done
3  piston + 1 sticky   done
4  slime_block         done (adhesion)
1  redstone_block      done
3  white_stained_glass inert; add to vanilla.rs INERT list
```

The pistons, sticky pull, slime adhesion and observers are already implemented and
trace-verified. **Two things block the run, and they are exactly items on the
near-term list above:**

### (a) Note blocks
Flagged historically as mishandled ("flagged for no apparent reason"). Capture
first — what a note block does on a redstone-power change, and whether/what it emits
or schedules. It plays a note via a **block event** (like a piston), so it likely
belongs in the `BlockEvents` phase. Get a trace of a redstone-block → note-block
before writing any Rust.

### (b) Player interaction — a new input path
This engine is *manual*: it does nothing until a player **right-clicks the note
block**, which is a use-block action. The simulation has no notion of this yet.

- It belongs in **`Phase::PlayerInputs`** (phase 10, currently a named no-op) —
  which is correct, and is why an input applied "now" is only observed next tick.
- Needs a public API on `Simulation`, e.g. `use_block(pos)`, that injects the
  interaction and lets the block's behaviour respond.
- `BlockBehaviour` needs a `fn on_used(...)` hook (default no-op), mirroring
  vanilla's `useWithoutItem`.
- **Capture the click.** Author a minimal structure (note block + the engine's first
  observer/piston), drive `TraceCapture` while performing a use-block on the note
  block, and record what the click produces. TraceCapture currently supports
  `--break`/`--pulse` only; it will need a `--use x,y,z` actuation that calls the
  block's use handler on the given tick. `useWithoutItem` on the server side is the
  method to invoke.

### Sequence for the next agent
1. `manual_engine.litematic` → SNBT. nucleation can already do this
   (`nucleation::formats::structure_snbt::to_structure_snbt`); wire a tiny converter
   or extend `examples/dump_schematic.rs`. Drop the `.snbt` in
   `tools/gametest/pack/data/nucleation/structure/` and in
   `crates/mc-tick/tests/corpus/structures/`.
2. Add `--use` actuation to `TraceCapture.java` (calls `useWithoutItem`).
3. Capture note-block behaviour in isolation → implement `NoteBlock` → conformance.
4. Capture the note-block click → add `Simulation::use_block` + `on_used` +
   `Phase::PlayerInputs` handling → conformance.
5. Run the whole `manual_engine` through `tests/conformance.rs`. Expect it to name
   any still-missing block loudly (that is the design) — `white_stained_glass` at
   least needs adding to `vanilla.rs`'s `INERT` list.

Everything needed to *verify* each step already exists — this is capture-then-
implement, per the discipline, not new infrastructure.

## 5. Milestone A — containers and item *transfer* (CURRENT)

Hoppers, droppers, dispensers and inventories, **without** free-flying items:
deterministic, integer-state, no physics. Two structural problems come first,
both tooling, because the capture-first discipline goes blind exactly where
item logic lives without them:

1. **Inventory changes are invisible to the capture.** TraceCapture diffs block
   states; a hopper pulling an item changes only block-entity NBT. The oracle
   must snapshot container contents per tick and the trace format needs an
   inventory-changed event kind.
2. **Block-entity tick order.** Two adjacent hoppers transfer differently
   depending on which ticks first — `tickBlockEntities` order is a new ordering
   domain, same class of problem as the phase order was. Read the iteration
   order from bytecode, pin it with a discriminating capture (a two-hopper
   race), then model it. This is where the engine's block-entities phase stops
   being just deferred piston writes.

Sequenced for the fastest verified loop:

- [x] **Comparator reads containers** — done. `Inventory` +
      `analog_signal` (the exact `getRedstoneSignalFromContainer` formula,
      `floor(fullness*14) + (any ? 1 : 0)`; max-stack assumed 64, documented),
      `Structure` parses block-entity `Items`, `VanillaRules` carries container
      slot counts, and the comparator's `getInputSignal` container path is
      implemented from bytecode — direct rear override, plus reading *through*
      one conductor when the rear signal is under 15. `Checkpoint` now carries
      **all** mutable state (moves, toggles, comparator memory, inventories).
      Goldens: `comparator_barrel.json` (3 stacks -> signal 2 -> on at t1),
      `comparator_barrel_off.json` (empty barrel turns a lit comparator off).
      Known gap, deliberate: `redstone_power` has no inventory view, so a
      container-fed comparator's *strength* is invisible to that path — nothing
      consumes it yet (dust is not integrated); revisit with dust.
- [x] **Capture upgrade** — done. TraceCapture diffs container block-entity
      contents slot-by-slot each tick (`"<count>x <id>"` strings, matching the
      engine's rendering); `EventKind::InventoryChanged` in mc-tick-trace; the
      engine logs slot changes through `TickCtx::set_inventory_slot`, which
      also notifies the blocks around the container the way vanilla's
      `updateNeighbourForOutputSignal` does.
- [x] **Hopper** — done, all from `HopperBlockEntity` bytecode and pinned by
      capture: 8gt cooldown (`hopper_pull.json`: transfers at ticks 0/8/16),
      eject-then-suck order, one item per move into the first
      empty-or-mergeable slot, the `enabled` gate (`hopper_locked.json`:
      breaking the power flips enabled and transfers the same tick), and the
      **block-entity tick order** with the destination-cooldown rule
      (`hopper_race.json`: an empty hopper receiving from an earlier-ordered
      hopper forwards after 7 ticks, not 8 — the `tickedGameTime` comparison,
      including the never-ticked sentinel). `comparator_drain.json` closes the
      loop: a comparator follows a barrel while a hopper drains it, going dark
      2gt after the last item.
- [x] **Dropper / dispenser** — done. `DispenserBlock.neighborChanged` from
      bytecode: `hasNeighborSignal(pos) || hasNeighborSignal(above)` (full QC),
      rising edge schedules 4gt and flips TRIGGERED silently.
      `dropper_fill.json`: one item into the barrel in front (boundary
      schedule fires at tick 3); `dropper_into_barrel.json`: with no container
      in front the item leaves as an entity — Milestone B's territory — and
      the engine models exactly the container-visible decrement. Known
      simplification: vanilla picks a random occupied slot; the engine takes
      the first, identical whenever at most one slot is occupied.

Still on the board, small and orthogonal: **buttons and pressure plates**
(pulse lengths differ per material), redstone lamp, target block.

## 6. Milestone B — item entities — DONE (dry land)

Item entities live. The decisive method discovery: **structures author item
entities directly** (`entities` list, `minecraft:item` with `Pos`/`Motion`/
`Item`), which makes physics captures completely RNG-free — and the physics
turned out to conform not merely within tolerance but to the diff's 1e-6 on
the first run, because the engine mirrors vanilla's arithmetic types
(`f32` drag widened to `f64` exactly where the bytecode widens).

- **Capture**: `--entities` diffs item entities per tick; `entity_moved` /
  `entity_removed` in mc-tick-trace; emission is by **position** change on
  both sides, which is what makes a resting item silent (its velocity
  oscillates as gravity accumulates and collisions flush it — invisible by
  construction).
- **Physics**, all from `ItemEntity.tick` bytecode (`item_fall.json`):
  gravity 0.04 before the move; drag ×0.98f (horizontal also × block friction
  when grounded, 0.6 default / 0.8 slime); the −0.5 landing bounce; the
  `(tickCount + id) % 4` rest skip; despawn at 6000. Collision is Y-then-
  larger-horizontal axis clipping against full cubes, clipped axes zeroing
  their velocity.
- **Hopper vacuum** (`item_into_hopper.json`): the suck column is the full
  block from y+11/16 to y+2; a whole stack absorbs at once; a full block
  above the hopper blocks suction; partial absorbs modify both sides but do
  not take the cooldown (vanilla returns success only on full consumption).
- **Merging** (`item_merge.json`): ±0.5 horizontal, every 2 ticks while
  crossing block boundaries and every 40 at rest; the larger stack survives;
  over-full merges refuse.
- **RNG policy** (`dropper_eject.json`): vanilla jitters spawn velocity
  (`triangle(mean, 0.103)`, speed 0.2..0.3); the engine spawns at the
  distribution means, deterministically. Trajectory conformance for RNG
  spawns uses a tolerance sized to the jitter bounds over a short flight;
  container-visible effects stay exact. Deterministic runs never involve RNG
  at all.

Still open from the original Milestone B slate: **water-stream item motion**
(fluids, section 7) and player pickup (needs players).

## 7. Fluids

Water and lava flow, levels, sources, and the **flow vector field** — joined
to item motion once Milestone B lands.

## 8. Remaining entities

1. **Minecarts** — rails, momentum, curves, powered rails
2. **Armor stands** — mostly static, but needed for interaction and collision

Entity motion is float-based, so the differ's tolerance mode finally earns its
keep. Traces will need entity events, which the format already defines but the
capture tool does not yet emit.

## 9. Throughput

Only once correctness is pinned by the differ, so optimisation cannot drift.
The likely first targets are the ones deliberately left simple:

- `TickQueue`'s `BTreeMap` → bucketed ring
- `has_pending_at` is a linear scan
- `resolve_push` uses `Vec::contains`
- parallelism **across** simulations, never within one tick

## 10. Product surface

Expose step/run/reset/checkpoint through `src/bridge/` to all six languages, then
build piston-door timing on top: actuate, run to quiescence, report the tick counts
and a per-component timeline. The trace machinery already produces the timeline.

---

## Sequencing

```
1 differ ─┬─► 2 observers ──► 5 components ──► 10 product
          ├─► 3 constants
          └─► 4 snbt loading ──► 6 block entities ──► 7 fluids ──► 8 entities
                                                                     │
                                                        9 throughput ┘
```

Items 1–4 are each roughly a session. 7 and 8 are the large ones and should not be
started until the differ makes regressions visible, or they will quietly break the
redstone that already works.
