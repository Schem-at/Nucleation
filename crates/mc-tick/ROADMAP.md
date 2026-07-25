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

## 1. Wire the trace differ into the corpus — *do this first*

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

## 2. Observers — the gap that matters most for doors

**Not implemented, and most piston doors need one.** An observer watches the block
it faces and emits a short pulse on change. That pulse interacts directly with the
block-dropping behaviour already captured, so it is on the critical path for the
product rather than a nice-to-have.

Capture first: pulse length in game ticks, tick priority, and whether it fires on
its own placement.

## 3. Retire the two assumed constants

`MAX_PUSH_DEPTH` (12) and `MAX_RECENT_TOGGLES` (8) are javac-inlined, so they were
taken from convention and asserted in tests rather than read. Both are now cheap to
settle empirically:

- push depth: a 12-block column versus 13, one moves and one does not
- burnout: toggle a torch clock and count the toggles before it stalls

## 4. Close the Rust side of the load/execute/verify loop

The oracle loads `.snbt` and runs it; the engine cannot. Until it can, the two
sides are compared through hand-written corpus cases rather than the *same input*.

- A minimal SNBT structure reader. It must **not** pull in nucleation — that would
  cost the 0.7s edit-test loop, which is the substrate everything else rests on.
  Either a small shared crate or a reader local to mc-tick.
- Then `load foo.snbt` in the corpus runner, which currently fails loudly on
  purpose.
- This is also the natural **UniversalSchematic** touchpoint: conversion at the
  boundary, with the engine keeping its interned `u16` states.

## 5. Remaining redstone components

Each is small once the scheduler is right; the work is capturing, not coding.
Roughly in order of how often doors use them:

| component | note |
|---|---|
| buttons / pressure plates | pulse lengths differ per material |
| redstone lamp | has an off-delay, unlike most blocks |
| target block | analogue output |
| dispenser / dropper | needs container contents |
| rails, tripwire, daylight sensor | rarely in doors |

## 6. Block entities, properly

Moving pistons are modelled as deferred writes. That is correct for *timing* and is
what the captures show, but it is not a block-entity model. It becomes limiting
when containers arrive (a dropper's inventory has to live somewhere), so it is
sequenced with item handling rather than before it.

## 7. Fluids — prerequisite for entities

Water and lava flow, levels, sources, and the **flow vector field**. Not an aside:
item alignment in water streams is that field acting on entity motion, so this
comes before items rather than after.

## 8. Entities

In the order originally specified:

1. **Items** — gravity, drag, water-flow response, merging, despawn
2. **Minecarts** — rails, momentum, curves, powered rails
3. **Armor stands** — mostly static, but needed for interaction and collision

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
