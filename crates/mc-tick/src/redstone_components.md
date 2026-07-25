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

Still outstanding: **comparator priming**. It needs a trace of a comparator
scheduled at NORMAL alongside diodes at HIGH, and that ordering encoded — the one
behaviour on this page still deliberately unimplemented.

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
