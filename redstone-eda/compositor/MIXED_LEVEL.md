# Mixed-level simulation (E3): characterized cells as behavioural models

## The idea

A verified cell is more than blocks: it is a **contract** — a boolean/word
function plus a measured delay table (the `CellTemplate` of
ROUTING_CRATE_DESIGN.md). Once both halves are trusted, a composition of
cells can be simulated at the FUNCTION level: step the netlist, not the
voxels. mc-tick remains the golden model that (a) produced the numbers and
(b) spot-checks the abstraction. That is mixed-level simulation: fast
behavioural stepping everywhere, tick-exact simulation only where the
abstraction is in doubt (new cell, new corridor, marginal timing).

## The delay tables are now real numbers, all measured in mc-tick (gt)

| cell / path                      | parameter        | value  |
|----------------------------------|------------------|--------|
| MS DFF (`seq_cells.py`)          | setup            | 0      |
|                                  | hold             | 3      |
|                                  | min CLK pulse    | 3      |
|                                  | clk -> Q         | 10     |
|                                  | min period       | 20     |
| register chain                   | clock skew / bit | 2      |
| FA cell (`Routing.sta`, demo4)   | sum arrivals     | 4/8/12/16 (2/4/6/8 rt) |
| sequential loop (counter4/accum) | min period       | 100    |
| accumulator external-B path      | min B settle     | 80 (measured sweep: 60 FAIL, 80 PASS) |

## Why function-level stepping is then exact

The clocked protocol turns continuous time into discrete legality checks:
if `pulse >= min_pulse + skew·bits`, `period >= loop min period`, and
`B settle >= 80`, then every D input is stable at every rising edge — so
`Q' = f(Q, B)` per edge is not an approximation, it is what the redstone
provably computes. All dynamics (glitching carry chains, analog comparator
lanes, repeater locks) are absorbed into the measured minima.

## Proof of concept: `functional_sim.py`

~30 lines, no voxels: checks the schedule against the table above, then
steps `Q <- (Q + B) mod 16` per edge. Cross-checked against the mc-tick
trace recorded by `accumulator.py` (`accumulator_trace.json`, 24 random-B
clocked steps on the baked-and-reloaded artifact):

```
functional sim vs mc-tick: 24/24 clock steps identical
   pulse width >= DFF min pulse + chain skew    OK
   period >= sequential-loop min period         OK
   B settle >= measured B-path setup            OK
   low phase >= clk->Q (capture visible)        OK
```

Identical outputs at every clock — the behavioural model and the 2 059-block
simulation agree step for step, at ~10^6x less work.

## Where this plugs into the roadmap

- `CellTemplate.delay_rt` + the DFF table are exactly the inputs STA needs
  for `data_arrival + setup <= clock_arrival` checks at every flop.
- The compositor can verify a composition behaviourally first (cheap,
  exhaustive), then run mc-tick only on the assembled artifact (the
  accumulator did both, and they agreed).
- The same split is the `SimBackend` trait's third backend: after
  `MchprsBackend` (fast voxel) and `McTickBackend` (vanilla-exact), a
  `FunctionalBackend` steps characterized cells — legal-schedule-or-refuse.

## Honest limits

- The tables cover the protocols they were measured under; a new topology
  (e.g. a longer feedback corridor) needs its loop period re-measured.
- Glitch behaviour is bounded, not modelled: the B-settle number exists
  precisely because lever flips launch carry glitches that take ~60-80 gt
  to wash out. Behavioural simulation is exact *given* legality — never
  use it to explore illegal schedules.
