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

## 7b. The door fixtures — the standing target (NEXT)

Five community doors arrived (`door_3x3_flush`, `door_4x4_sliding`,
`door_6x6_sliding`, `door_4x4_vault`, `door_tgm_4x4`) — every block they
contain loads and registers (`check_structure` reports zero gaps after the
material sweep: slabs, composter analog reads, targets, leaves, shulker
boxes, signs/heads/rods classified). All five have captured goldens
(placement settle + lever click), and **all five diverge** — the tests are
committed `#[ignore]`d as the standing target.

**Session update — the placement half is solved.** All five doors now
place **exactly** as vanilla does (zero block diffs each; the vault
started at 22), and tick 0 matches event-for-event on every one. Getting
there rebuilt the update model, all bytecode-verified:

- `CollectingNeighborUpdater` semantics: updates are *entries* (one
  `updateNeighborsAt` = six notifications in `UPDATE_ORDER`) run
  depth-first, entries queued mid-notification completing first.
- **Shape updates are a separate callback** (`BlockBehaviour::on_shape_update`).
  `ObserverBlock` overrides only `updateShape`; that is why placement
  pulses observers and not pistons. `set` sends neighbours then shapes;
  a flag-2 write (the dust evaluator's) still sends shapes.
- `DefaultRedstoneWireEvaluator` transcribed — per-wire recompute, then
  seven `updateNeighborsAt` entries in Java `HashSet` iteration order.
  The ideal fixed-point relaxation is gone.
- **Strength-aware power** (`Level.getSignal`/`getDirectSignalTo`):
  comparators emit their stored block-entity strength, diodes emit from
  one face, lit torches strongly power upward, and dust strongly powers
  everything it powers weakly.
- A retracted piston is a full collision cube but **not** a redstone
  conductor.
- Placement walks three sorted groups — solid, other, block-entities —
  keyed `(y, x, z)`, not file order.
- `RepeaterBlock`'s `LOCKED` and `NoteBlock`'s `INSTRUMENT` are derived
  properties the game rewrites on placement.

Two capture tools made it tractable and should be the first reach next
time: `--dump-placed` (diff a whole placed world) and
`--probe`/`--probe-tick` (ask the running game for
`getSignal`/`getDirectSignalTo`/`isRedstoneConductor` at a position, at
any tick).

### The vault door, traced to one contested column

Focused session on `door_4x4_vault`. Ticks 0, 2, 5 and 7 now match
vanilla exactly; the whole remaining failure is **one piston race**:

- The column at x=4 (and its mirror at x=9) has an **up** piston at the
  bottom, five concrete blocks, an air gap, and a **down** piston at the
  top. Both become powered on tick 1 by their own observers, both want
  the gap, and whichever's block event dispatches first moves the column
  and blocks the other. Vanilla's *down* piston wins; this engine's *up*
  piston does, which displaces the door frame by one block and is why
  the lever cascade at tick 10 — the slime extenders — never fires.
- Fixed on the way: **a comparator must notify when its stored strength
  changes even if `powered` does not** (`refreshOutputState` acts on
  `j != i`, writes flag 2, then calls `updateNeighborsInFront`). A
  comparator placed `powered=true` over a fresh block entity holding 0
  hits exactly that, and a whole branch of the door stayed dark. Also
  modelled: `placeInWorld`'s **setBlock loop**, whose shape updates let
  an observer schedule when a *later* block is placed — that reordered
  the tick-1 queue correctly for the x=9 side.
- **Confirmed from source and implemented**: `doBlockEvent` refuses an
  event whose position no longer holds the **Block** it was queued for
  (`state.is(event.block())` — properties may differ freely), so
  `BlockEvent` now carries that block and `StateRegistry` interns block
  identity alongside state identity.
- **Found while doing it**: `PistonBaseBlock.getPistonPushReaction`
  answers `BLOCK` while `EXTENDED`, so an **extended piston base cannot
  be pushed**. This engine's immovable list keys on block *name*, so a
  slime structure was happily dragging extended pistons along. Fixed.
- Still open, and the prime suspect for the race: **refused block events
  are rescheduled** (`blockEventsToReschedule`, re-added after the drain
  so they lead the next tick), and `doBlockEvent` drops an event whose
  position no longer holds the block it was queued for. Implementing the
  re-add alone regressed things badly (the 6x6 ran away to 151 ticks, a
  manual-engine golden grew a retraction), so the identity guard has to
  land with it. The guard is now in place, and the re-add is one
  commented-out line in `run_block_events` — but it still turns two
  manual-engine goldens red, and the cause is now localised:

    The engine queues a piston extend at (13,1,2) on tick 27, the tick
    that piston *lands*; vanilla never queues one. Probes confirm both
    agree the piston is quasi-powered there and that its push plan is 8
    blocks, so the difference is either the landing notification or the
    push resolution — not the power model. Today the stale event is
    refused on tick 28 and dropped, hiding it; with rescheduling it
    survives to tick 29 and fires. **Fix that queue, then uncomment the
    re-add.**

  **Session update.** `PistonStructureResolver` is now transcribed
  faithfully (backward sticky chains, perpendicular branching, collision
  reordering, the 12-limit applied at each step) instead of the earlier
  forward-only approximation. Asking the game with the new
  `--probe-push` flag also overturned a hand-written unit test: obsidian
  stuck to a slime block does **not** cancel a push (`addBlockLine`
  returns *true* for a block it cannot push, so the branch is abandoned,
  not fatal) — only an immovable block in the push **line** stops a
  piston. The engine and the test now agree with the game.

  **RESOLVED — the reschedule is live.** `--probe-push` at (13,1,2) on
  tick 27 answered it: vanilla refuses that push, so it never queues an
  extend. `PistonBaseBlock.checkIfExtend` runs the resolver *before*
  `level.blockEvent(...)`; this engine queued on power alone and only
  resolved at dispatch. Equivalent while refusals are dropped, wrong
  once they are rescheduled — the phantom event survived and fired two
  ticks later. Both are now faithful and all goldens stay green.

  **The doors' remaining failure is now a single question.** Probed on
  the vault door: at tick 0 *both* opposed pistons on the x=4 column have
  a resolvable push (`resolve=true`, 5 concrete); by tick 1 the up one is
  refused because the down one's placeholder already occupies the gap. So
  the winner is decided purely by which piston is notified first, and
  that traces to two observers, (3,2,1) at y=2 and (4,10,1) at y=10, both
  scheduled during placement. This engine fires the y=2 one first (walk
  order is `(y, x, z)`); vanilla evidently fires the y=10 one first.
  Neither schedules in the setBlock pass — (4,10,1) faces up into nothing
  and (3,2,1) watches a block placed before it — so both should land in
  the update pass in y order. Something about observer scheduling is
  still unmodelled, and that is the next thing to find.

  Historical note, now fixed: at the tick
  the manual engine's piston at (13,1,2) lands, vanilla's resolver
  refuses the push after collecting 7 blocks and ours accepts it with 8
  — the extra being the retracted piston at (10,0,1). Vanilla's
  `isPushable` tail reads `getDestroySpeed == -1 → false`, then
  `BLOCK → false`, `DESTROY → allowDestroy`, `PUSH_ONLY → dir == face`,
  default `→ !hasBlockEntity()`, none of which obviously rejects a
  retracted piston, so the difference is likely in *which line* is being
  walked (a branch's forward walk hitting something unpushable, which is
  fatal, versus a branch origin that is merely skipped). `--probe-push`
  at that position and tick is the way in.

  `cargo run -p mc-tick --example diff_vanilla` is the tool for this:
  per-tick MATCH/DIFFER against any golden, with `--region` to watch one
  mechanism and `--from/--to` to bound the range.

**What remains** is tick-time cascade order: the doors now diverge deep
into their opening sequences, and the vault's case is traced to two
opposed pistons racing — vanilla's scheduled-tick order picks one, the
engine picks the other. The next lead is `LevelTicks`' sub-tick ordering
against this engine's `(target, priority, sequence)` queue.

The original findings, for reference:

- The divergence starts **inside the placement settle**: vanilla's
  `DefaultRedstoneWireEvaluator` relaxes wire-by-wire, recursively, and its
  ordered transients are *latched* by the doors' torches, repeaters and
  comparators. The engine's ideal fixed-point wire (the documented deviation
  in `src/wire.rs`) produces the same final dust levels but different
  component timings — invisible to every small golden, decisive here.
- Reproducing it means reproducing vanilla's **update dispatch order**
  (`updateNeighborsAt`'s west/east/down/up/north/south, re-entrant through
  `setBlock`) and the placement pass's exact semantics — read
  `StructureTemplate.placeInWorld` and the wire evaluator's bytecode before
  writing anything.
- A second discovery: under `--known-shape` the 4x4 sliding door still
  self-fires a push at tick 0 (`door_4x4_sliding_q.json` in the capture
  workspace) — community litematics are saved in states that are not at
  rest, so quiet placement does not sidestep the problem.

## 7. Milestone C — closing the redstone surface — DONE (awaiting a real door)

The gap between "the engine runs slimestone machines and item logistics" and
"the engine runs *doors*" is mostly one component: **dust**. Ordered by
leverage:

- [ ] **Wire dust into the simulation.** `RedstoneNetwork` exists and is
      trace-verified standalone (synchronous settling, 15-block attenuation),
      but nothing dispatches it: dust in a structure is still a loud unknown.
      This is the largest single blocker for real doors, and it forces the
      strength-plumbing question the comparator work deferred: `redstone_power`
      has no inventory/aux view, so a container-fed comparator's *strength*
      cannot reach a dust line today. The likely shape: dust settling runs
      inside `propagate` (it is notification-driven and synchronous — the
      captures already prove there is no per-tick crawl), reading sources
      through an aux-aware strength query; `VanillaRules` grows dust states
      per power level. Discriminating captures to take first:
      dust ← comparator at partial strength (the barrel goldens extend
      naturally), dust → repeater/piston/note-block inputs, dust on top of a
      conductor (soft power), and a **locational** case where
      alternate-current's non-locational model would differ — the deviation
      the sources table has warned about since day one.

      The **solid-vs-transparent rules** live here too, and they are exactly
      where folklore is least trustworthy — capture every one:
      - dust climbing vs descending past **glass**: up works, down does not,
        which is the classic *glass diode*; same family, **slabs** — dust
        cannot step down onto a line below a top slab. These are one-way
        vertical connection rules, and a discriminating capture each way is
        cheap.
      - which blocks cut dust lines, conduct soft power, and support dust at
        all (glass supports dust but does not conduct; slabs depend on half).
- [ ] **Buttons and pressure plates.** Buttons are pure `use_block` (the input
      path exists): capture pulse lengths per material — stone and wood
      differ. Pressure plates now have something to press them: **item
      entities trigger wooden plates**, so a dropper → plate → piston chain is
      a fully deterministic capture with the existing tooling.
- [ ] **Redstone lamp** — the one block with an off-delay (4gt); on is
      immediate. Small, common in doors as the visible output.
- [ ] **Target block** — analog output; pairs with the dust strength work.

Everything above is implemented and conformance-pinned (dust line drops,
comparator strength through wire, soft power extending a piston through the
dust's floor block, both sides of the glass diode, stone/oak button
durations 20/30, the lamp's 4-tick off-delay, and an item pressing a wooden
plate). The wire model is the ideal fixed-point evaluator — the documented
deviation from vanilla's per-wire cascade, so locational-transient
contraptions are the known gap. Target block still open (needs projectiles).

**Done when**: a real community piston door (litematic, with dust) runs
end-to-end and its open/close timing conforms — the product's core claim.
Send one over Discord (the tooling can receive files now) and it becomes the
closing golden.

## 8. Milestone D — fluids — DONE (water)

Water flow, levels, sources, the flow vector field, item buoyancy and
currents, and bubble columns. Everything from `FlowingFluid`/`WaterFluid`
bytecode, pinned by five captures (`water_spread`, `water_hole`,
`item_float_water`, `item_stream`, `bubble` — all conforming at 1e-6):

- **Fluid ticks are their own queue** (`Simulation.fluids`), drained in the
  previously-empty `Phase::FluidTicks`, 5 gt water delay, boundary folding
  identical to block ticks. `src/fluid.rs` implements `getNewLiquid`
  (drop-off 1, falling-8 under any water, the 2-source infinite-water rule),
  `spread` (down first and stop, side spread gated on 3+ source neighbours or
  no hole below), and the slope search (`getSlopeFindDistance` = 4) — the
  `water_hole` golden proves a T-junction sends every drop toward the hole.
- **Item fluid physics**, bit-faithful: `EntityFluidInteraction` runs **twice
  per item tick** (baseTick and again near ItemEntity.tick's tail — the
  stream capture's velocities prove both 0.014 pushes); the fluid height is
  measured from the *raw* box floor while the skip test uses the 0.001-deflated
  box (that millimetre decides float-vs-sink in one-deep shallows);
  `setUnderwaterMovement` is ×0.99f horizontal with the 5e-4f nudge under
  0.06f. Flow vectors are `getFlow` exactly, f32 height arithmetic included.
- **Bubble columns**: per-cell clamps from `handleOnInsideBubbleColumn` /
  `OnAboveBubbleColumn` (±0.03/0.06/0.1 against −0.3/0.7/−0.9/1.8), applied
  during the move and — crucially — **also on rest-skipped ticks**
  (`applyEffectsFromBlocksForLastMovements`), which is what keeps a sunk item
  pinned to the floor of a drag column.
- **The rest-flush phase is id-dependent** (`(tickCount + id) % 4`): the
  harness now spawns authored items with the golden's raw server ids, and the
  capture's entity observation window (structure box + margin) is modelled so
  an item flying out of view reads as removed/re-appearing, as the capture
  sees it.

Still open, deliberately: **lava** (different delay/drop-off, mixing),
**waterlogged spread** (waterlogged states count as sources for neighbours
and item physics, but do not yet run their own spread tick),
`canPassThroughWall` face shapes (approximated by the full-cube table), and
flooding replaceable plants. Each becomes a capture when a build needs it.

## 9. Milestone E — remaining entities and the surfaces they ride (IN PROGRESS)

1. ~~**Item surfaces**~~ — DONE: ice/packed/frosted 0.98 and blue ice 0.989
   in the friction table (from `Blocks` static-init), cobwebs via
   `Entity.stuckSpeedMultiplier` (0.25, 0.05f, 0.25 — armed every touching
   tick, rest-skipped ones included), soul sand as the first partial-height
   solid (14/16, so it also stops conducting), and `Entity.move`'s 1e-7
   movement gate. Goldens: `item_ice`, `item_web`, `item_soulsand`.
2. ~~**Minecarts, stage 1**~~ — DONE: `OldMinecartBehavior` transcribed
   (`src/minecart.rs`) — the EXITS table from the static initialiser, chord
   projection, ×0.96/0.997 rail drag with vy zeroed, ±0.4 movement clamp
   (velocity itself caps at 2.0), powered +0.06 boost and 0.02 conductor
   launch, unpowered braking (×0.5, dead stop under 0.03), slope pull
   0.0078125, corner fixups, the 0.05 height correction, and `getPos`'s
   doubled y-delta (a slope spans its full block). `comeOffTrack` with the
   0.95f air drag. Carts spawn from structure entity lists and share the
   entity id counter. Goldens: `cart_flat`, `cart_boost`, `cart_brake`,
   `cart_slope`, `cart_curve`, and `cart_loop` (200 ticks around a powered
   circuit through all four corner shapes).

   Stage 2 so far: **dynamic rail powering** landed (`PoweredRail` — chains
   of at most 8 already-powered rails, `findPoweredRailSignal` transcribed;
   goldens `rails_chain` and `cart_launch`), and **levers** joined the
   component set (strong emission into the support block; golden
   `lever_lamp`). Still open: detector/activator rails, rail shape
   re-bending (authored shapes must be the natural fixed point — placement
   normalisation re-bends the rest, which the slope rig proved), cart-in-water
   (0.2 max speed, 0.95f slowdown, fifth slope pull), cart–cart collisions,
   hopper/chest carts, and riders (the 0.75 ridden factor and 0.997 drag are
   already stubbed in the transcription).
3. **Boats** — the other rideable surface: boat-on-ice item transport and
   boat-based mob/item alignment in sorters. Boat physics is `Boat.tick`
   (friction by surface, paddle-free drift for contraption use); Milestone D's
   water surface heights and flow field are the prerequisite, now in place.
4. **Armor stands** — mostly static, but needed for interaction and collision
5. **Player pickup** of items (needs a player model; hoppers ignore
   `pickupDelay`, players do not)

The entity capture/tolerance machinery from Milestone B carries over directly.

## 9b. Milestone F — TNT, explosions, and duplication

The precision showcase. Primed TNT is an entity (fuse 80, its own physics);
explosions are ray-based block destruction plus entity knockback; **TNT
minecarts** combine both. The reason this earns a milestone rather than a
bullet: **TNT duplication** — the mechanism tunnel bores and world-eaters run
on — is a pure update-order artifact (a piston moves a TNT block in the same
window it is ignited, so the block both becomes an entity and travels), and
getting it right is exactly the class of claim this engine exists to make.
Before implementing, study real dupers and tunnel-bore schematics to design
the discriminating captures; folklore about *why* duping works is abundant
and unreliable, the bytecode and traces are not. RNG note: explosion rays and
TNT entity spawn velocities are randomized — expect the Milestone B split
(deterministic engine policy + tolerance/effect-level conformance) to apply.

## 10. Throughput

Only once correctness is pinned by the differ, so optimisation cannot drift.
Benchmarked baseline (`bench_manual_engine`): 0.72 µs/active tick
(~70,000× real time), 12 ns quiescent — so the pressure is low. The likely
first targets are the ones deliberately left simple:

- `TickQueue`'s `BTreeMap` → bucketed ring
- `has_pending_at` is a linear scan
- `resolve_push` uses `Vec::contains`
- entity/inventory lookups are linear over small `Vec`s/`HashMap`s
- parallelism **across** simulations, never within one tick

## 11. Product surface — parallelisable now

Correctness is pinned well enough that this no longer needs to wait for C:

- Expose load/step/run/actuate (`place_block`, `use_block`)/checkpoint/
  restore/quiescence through `src/bridge/` to all six languages.
- Door timing on top: actuate → run to quiescence → tick counts plus a
  per-component timeline (the trace machinery already produces it).
- Rendering: `render_simulation_video` should learn to read the **entity
  log** — falling and ejected items animating alongside the blocks. The
  smooth-piston cast machinery extends directly (an item is a member with a
  per-tick position track instead of a lerp).

---

## Sequencing

```
A containers ──► B items ──► C dust + small components ──► D fluids ──► E entities
      (done)      (done)          │                                        │
                                  └─► 11 product surface ◄─────── 10 throughput
```

Milestones A and B each took a session. C's dust integration is the next
door-blocking brick; the product surface can proceed in parallel with any of
it, because every behaviour behind it is already conformance-pinned.
