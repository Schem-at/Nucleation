# Diode timing and priority — verified from the game

Research notes for implementing torches, repeaters and comparators. **Not yet
implemented.** Every fact below was read from Minecraft 26.2's own bytecode
(the server jar ships unobfuscated), not from memory or community docs — these
are the details that decide whether piston-door timings come out right, and they
are exactly the ones that get misremembered.

Reference reading only; nothing is copied.

## Tick priority

Seven levels, matching `crate::schedule::TickPriority`:

```
EXTREMELY_HIGH(-3)  VERY_HIGH(-2)  HIGH(-1)  NORMAL(0)
LOW(1)  VERY_LOW(2)  EXTREMELY_LOW(3)
```

Lower runs first within a tick. **Priority is not decoration** — it is how the
game makes concurrent diode updates deterministic, and getting it wrong produces
timings that are right most of the time and wrong exactly where builds care.

## Repeaters and comparators share `DiodeBlock`

The scheduling decision lives in `DiodeBlock.checkTickOnNeighbor`, and the
priority is chosen from three cases:

| condition | priority |
|---|---|
| `shouldPrioritize(...)` is true | `EXTREMELY_HIGH` (-3) |
| currently powered (i.e. about to turn **off**) | `VERY_HIGH` (-2) |
| otherwise (turning **on**) | `HIGH` (-1) |

`shouldPrioritize` reads `FACING`, steps to the block **behind** the diode
(`FACING.getOpposite()` → `BlockPos.relative`), and asks `isDiode` whether that
neighbour is itself a diode facing this one. In other words: **a diode fed
directly by another diode jumps the queue.** That is the mechanism behind
"repeater priority", and it is why chains of repeaters resolve in a stable order.

There is also a guard before scheduling — the game checks whether a tick is
already pending for this position this tick and does not double-schedule.
`crate::schedule::TickQueue::has_pending_at` exists for exactly this; it must
actually be consulted, or delays silently double.

### Delays

- **Repeater**: `getDelay` reads the `DELAY` property (1–4) and multiplies by
  **2**, so 2 / 4 / 6 / 8 game ticks. The property is in *redstone* ticks; the
  scheduler works in game ticks.
- **Comparator**: fixed **2** game ticks.

### Locking

`DiodeBlock.isLocked` exists and repeaters override it — a repeater powered from
the side by another repeater or comparator ignores input changes entirely.
`checkTickOnNeighbor` returns early when locked, so a locked repeater schedules
nothing at all. Not modelled yet.

## Comparator priming

`ComparatorBlock` schedules with either `HIGH` or `NORMAL`, unlike a repeater
which never uses `NORMAL`. That split is the observable root of what the
community calls **comparator priming**: a comparator's pending tick can be
scheduled at a different priority than the diodes around it, so it resolves in a
different order relative to them than intuition suggests.

Do **not** implement this from the description above. Capture a trace first —
the `--break` actuation in `tools/gametest` is the tool for it — and encode the
observed ordering. This is precisely the class of behaviour where a plausible
reading produces something subtly wrong.

## Status update — now implemented and trace-verified

- **Quasi-connectivity** — confirmed by capture: a redstone block adjacent only to
  the space *above* a piston, touching it nowhere, extends it. Implemented.
- **Moving-block entity** — captured: blocks become `moving_piston` at tick 0 and
  resolve at tick 2, in the block-entities phase. Implemented as deferred writes
  applied in that phase.
- **Repeater locking** — implemented from `DiodeBlock.isLocked`; a powered diode on
  either perpendicular side makes the repeater schedule nothing.
- **Torch delay = 2** — confirmed by capture (torch flipped exactly two ticks after
  its repeater), no longer an assumption.

- **Comparator modes** — trace-confirmed for the pass-through case: a subtract
  comparator fed 15 from behind with no side input output 15, lighting dust at
  15 then 14. Compare/subtract arithmetic implemented; the *side-input* arithmetic
  follows the documented rules and still wants a trace of its own.
- **Torch burnout** — implemented. `RECENT_TOGGLE_TIMER` is 60, read as the literal
  `60L` in the class; `MAX_RECENT_TOGGLES` (8) is inlined by javac, so it is the
  conventional value rather than one read from the bytecode.

- **Comparator priming** — implemented, and the mechanism turned out to be
  concrete rather than folklore. `ComparatorBlock.checkTickOnNeighbor` schedules
  when `output != storedOutput || powered != shouldTurnOn`, where the stored value
  lives in a `ComparatorBlockEntity`. The block *state* only carries `powered` and
  `mode`, so it cannot express "on at strength 9" — hence the block entity, and
  hence priming: a comparator can hold a pending tick caused purely by a strength
  change. Its priority is `HIGH` when diode-fed and `NORMAL` otherwise, never the
  `VERY_HIGH`/`EXTREMELY_HIGH` a repeater uses, so a primed comparator always
  resolves after every repeater in the same tick.
- **Slime and honey** — trace-verified. Adhesion drags on every face, dragged
  blocks start push lines of their own, and slime does not stick to honey.
- **Immovable blocks** — one immovable block anywhere in a resolved structure
  cancels the entire push, including when reached through adhesion. Blocks already
  in motion (`moving_piston`) are immovable.

- **Comparator side-input arithmetic** — captured, no longer taken from
  documentation:

  ```text
  subtract  rear 15, side  0  -> 15
  subtract  rear 15, side 14  ->  1
  compare   rear 15, side 14  -> 15   (side loses, passes through)
  compare   rear 13, side 14  ->  0   (side wins, comparator unpowered)
  ```

Nothing on this page is unimplemented, and every rule on it is now backed by
either a captured trace or the game's own bytecode.

## Manual-engine session — note blocks, clicks, and boundary time

Everything below came out of running the first real community schematic
(`manual_engine.litematic`) end-to-end, verified against `NoteBlock`,
`ObserverBlock`, `PistonBaseBlock` and `PistonMovingBlockEntity` bytecode plus
the captures named in parentheses.

- **Note block** (`note_powered.json`, `note_click.json`) — `neighborChanged`
  compares `hasNeighborSignal` with `POWERED` and flips it **synchronously**; no
  scheduled tick anywhere. The note plays via `level.blockEvent(pos, this, 0, 0)`
  on the rising edge only, and only when the instrument can sound — air above,
  for the ordinary instruments. `useWithoutItem` cycles `NOTE` (0-24, wrapping),
  sets the block, then plays. The pitch change is a block-state change, which is
  what lets an observer see a click.
- **Observers emit from their back face only** — `ObserverBlock.getSignal`
  returns power only when the queried direction equals `FACING`. Reading an
  observer as an omnidirectional source made it power the note block it was
  watching, which re-triggered it forever. `VanillaRules` now carries per-state
  emission directions.
- **Boundary time** (`rep_boundary.json`, `note_click.json`, and every placement
  pulse) — actions between server ticks (structure placement, `--break`,
  `--use`) happen while the game time still reads the last *completed* tick, so
  anything they schedule fires one capture-tick sooner than an in-phase schedule.
  A repeater scheduled at the placement boundary turns on at trace tick 1; an
  observer clicked at a boundary pulses one tick after the click. Engine:
  `TickCtx::boundary`.
- **Placement updates every block from every side** —
  `StructureTemplate.placeInWorld` ends with a shape-update pass, which is why
  **every observer pulses once when a structure is placed**, whatever it faces
  (`manual_engine_settle.json`, ticks 1/3). `Simulation::settle` now notifies
  each placed block from all six directions.
- **Block events deduplicate** — `ServerLevel` keeps pending block events in an
  `ObjectLinkedOpenHashSet`: queueing an identical `(pos, id, param)` twice is a
  no-op. Load-bearing once placement sends six notifications per block.
- **Piston `triggerEvent` re-validates at dispatch** — an extend whose power
  vanished between queueing (phase 3) and dispatch (phase 7) is dropped; a
  retract whose power returned just re-marks `EXTENDED` with no updates
  (flag 2) and is unhandled. This is why a landed piston beside a dying observer
  pulse never extends (`manual_engine_settle.json`, ticks 2-3).
- **Move writes are silent** — `moveBlocks` writes placeholders and vacated
  slots with flags that suppress neighbour block updates (324/82/68/18), and the
  base's `EXTENDED=true` is written **after** the moves (flag 67, the one loud
  write). Consequence: a piston does not react to its own move — even when it
  pushes its own power source away — until the blocks land two ticks later.
- **Retraction travels like extension** — the *base* becomes
  `moving_piston` for two ticks: `extended=true -> moving_piston ->
  extended=false` (`manual_engine_settle.json`, ticks 3-5). An in-flight head is
  `finalTick`ed first, and a **source** block entity resolves to *air*, not to
  its head state.
- **Placeholder types** — `moveBlocks` sets only `FACING` on the placeholders it
  writes, so pushed *and pulled* blocks always ride `type=normal`; only the head
  slot's placeholder carries the piston's own type. Captured: a sticky pull
  wrote `moving_piston[...,type=normal]` over the sticky-typed head placeholder
  (`manual_engine_settle.json`, tick 6, at `[3,0,1]`).
- **Landing wakes the landed block** — `PistonMovingBlockEntity.finalTick` runs
  the landed state through `updateFromNeighbourShapes` (a shape update from all
  six sides) and then `neighborChanged`s the position itself. This is how an
  observer that was *moved* pulses two ticks after it lands, which the engine's
  cycle depends on.
- **`getNeighborSignal` skips two directions** — the facing direction at the
  piston's own position, and Down at the position above (the QC probe).
- **Chunk edges freeze block entities** — a moving piston pushed into a
  loaded-but-not-ticking chunk stays a placeholder indefinitely, and being
  immovable it then blocks further pushes. This is what stops the free-flying
  manual engine after two steps in the settle capture, and it is why
  `Simulation::set_ticking_bounds` exists.

Known simplification, deliberately accepted: the engine's single notification
channel does not deliver vanilla's *shape* updates from mid-move placeholder
writes (vanilla suppresses block updates but still fires shape updates there).
Nothing captured so far distinguishes the two; an observer watching a slot a
block moves *into* mid-flight would.

## Flying-machine session — conducted power and mid-pulse moves

`flying_machine.snbt` is a 6-block two-piston machine that does **not** fly:
the placement pulse pushes its front half one block east and it sits split in
two from tick 5 on. Reproducing that broken behaviour exactly
(`flying_machine.json`) forced three more mechanisms, each read from bytecode:

- **Strong power through conductors** — an observer *strongly* powers the block
  behind it, and `Level.getSignal` lets a conductor re-emit weak power on every
  face. That is how the observer drives a piston through a slime block it never
  touches. `VanillaRules` now carries `strong_into` (observers) and a
  capture-driven `conductors` list (slime is on it because this trace proves
  the signal crossed it; glass is not).
- **`updateNeighborsInFront`** — an observer's tick notifies the block it
  strongly powers *and that block's neighbours* on both pulse edges. Without
  this extra block of reach, the non-adjacent piston would never re-check and
  never retract.
- **`ObserverBlock.onPlace`** — a powered observer written into the world with
  no pending tick clears its own powered flag, silently (flag 18), then updates
  its front. This is how an observer pushed *mid-pulse* lands `powered=false`:
  its turn-off tick is stranded at the position it left. Engine:
  `BlockBehaviour::on_placed`, dispatched when a move lands — after the
  landing's shape updates, whose ordering matters (they must see the carried
  mid-pulse state so they do not start a pulse vanilla never starts).
- **Quiet placement exists, and dispatches nothing** —
  `StructurePlaceSettings.knownShape` skips the update pass entirely. Captured
  (`manual_engine_quiet_click.json`): the machine sits completely still — the
  QC-powered piston included, so per-block `onPlace` effects do not surface
  either — until the note block is clicked, and the click then runs exactly one
  activation from the as-built state. Engine equivalent: load without calling
  `settle`.
- **Settle order is placement order** — two racing piston triggers queue their
  block events in the order the placement pass walks the block list, and the
  first event to run moves the other piston's blocks out from under its event.
  The engine's settle dispatches in that same order.

## What to do next, in order

1. **Capture traces before writing any of it.** A structure per component:
   torch inversion, repeater at each of the four delays, repeater feeding
   repeater (to exercise `shouldPrioritize`), comparator in both modes, and a
   comparator primed by an adjacent diode.
2. Implement against those traces, one component at a time.
3. Only then pistons. Their block-event delay depends on this ordering being
   right, which is why they come last rather than first.

## Why this ordering matters for pistons

A piston's motion spans three phases — it decides in `BlockTicks`, starts moving
in `BlockEvents`, and finishes in `BlockEntities` (see [`crate::phase`]). The
tick on which it *decides* is chosen by the diode priorities above. Get the
priorities wrong and the piston fires on the wrong tick, and every door timing
downstream is wrong with it.
