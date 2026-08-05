# A dropped observer was deaf to the world it woke up in

**Fixed.** `crates/mc-tick/src/piston.rs`, the short-pulse drop. Regression
test: `observer_diagonal_piston.test.json`. Ground truth:
`tools/gametest/work/obs3x3.json` (26.2, `--known-shape`, note block used at
t5/t15/t25) — the engine now matches it on **all 31 ticks**.

## The machine

`tests/corpus/structures/observer_diagonal_piston.snbt`, 7x2x1:

```
y1:  .   .   .   obs>  [P<  obs>  note
y0:  ▓   [P<  .    .    .    .     .
```

A counter. Each note-block click toggles the upper observer between `(3,1,0)`
and `(2,1,0)`. Only at `(2,1,0)` does its output face point into `(1,1,0)`,
quasi-powering the sticky piston at `(1,0,0)`, which throws the gray concrete
off `(0,0,0)`. Hence "the block moves every third click".

## Symptom

The lower piston never fired. Under `MC_TICK_TRACE_EVENTS` it was not being
ignored — it queued an extend and then *refused* it:

```
[t18] queue  (1, 0, 0) id=0 on sticky_piston[extended=false,facing=west]
[t18] run    (1, 0, 0) id=0 on sticky_piston[extended=false,facing=west]
[t18] refuse (1, 0, 0) id=0
```

That is `PistonBaseBlock.triggerEvent`'s `!shouldExtend && id == 0` re-check:
nothing was powering `(1,1,0)`, because the observer that had just landed at
`(2,1,0)` never pulsed.

## Root cause

`MC_TICK_TRACE_WRITE=2,1,0` showed the landing happening in the **block-events**
phase, not the block-entities landing loop:

```
[t16/block_events] set_shape_only 2 1 0 -> moving_piston[facing=west,type=normal]
[t18/block_events] set(flag3)     2 1 0 -> observer[facing=east,powered=false]
```

Two code paths land a moved block. The ordinary two-tick landing
(`sim.rs`, the pending-move drain) runs `updateFromNeighbourShapes` on it —
`UpdateEntry::own_shapes`. The **short-pulse drop** in `piston.rs` — where a
block still travelling toward a retracting head is finalised where it is
instead of being pulled — did not.

`ObserverBlock.updateShape` is the *only* thing that calls `startSignal`; an
observer is deaf to plain neighbour updates. So an observer dropped by that
path never scheduled its pulse.

This also explains why the first cycle looked correct. `updateShape` fires
`startSignal` only when the observer lands **unpowered**:

| cycle | lands as | vanilla | why |
|---|---|---|---|
| t3  | `powered=true` (pushed mid-pulse) | no pulse | `!POWERED` is false |
| t18 | `powered=false` | pulse at t20 | signal starts, +2 ticks |

We matched t3 by accident — with no shape update at all, the outcome is the
same — and diverged at t18, where the shape update was the whole point.

## The fix

`updateFromNeighbourShapes` before `onPlace`, drained rather than merely
queued. The order is load-bearing in both directions: an observer dropped
mid-pulse must still be carrying `powered` when the shape update reaches it,
so it does *not* re-schedule, and only then does `onPlace` clear the flag.
Clearing first would start a pulse vanilla never starts — which is exactly
what `sim.rs`'s landing loop already documents for the other path.

## Method notes

Two traps, both of which cost real time here:

- The oracle trace is a per-tick **snapshot diff**, not a write log. Comparing
  it against the probe's execution-ordered log invents divergences that are
  not there. Collapse both to net-state-per-tick first; that is what
  `netdiff` does, and it took the diff from "16 differing ticks" to the six
  that were real.
- **Match the settle mode.** `--known-shape` (oracle) is `quiet` (engine), and
  it fires a placement pulse that starts the counter. Comparing it against
  `in-world` put the two runs a full cycle out of phase and made every tick
  look wrong.

`machine_probe` is `#[ignore]`d — without `-- --ignored` it silently prints
nothing and looks like a trace facility that does not work.
