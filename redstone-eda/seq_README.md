# Sequential cells (Phase 3 A3): repeater-lock latch, MS DFF, register, counter

Everything below is verified in mc-tick with **fixed-tick stepping**
(`sim.sim.run(n)`), never `run_until_quiescent` after the initial placement
settle of a freshly authored build.  All numbers are exact (mc-tick is
deterministic).  gt = game ticks, 2 gt = 1 redstone tick.

## Probe findings (`seq_probe.py`, all PASS)

Repeater-lock semantics in mc-tick:

- A repeater whose SIDE is entered by a **powered** repeater or comparator
  reads `locked=true` and its output freezes at the stored value.  Both side
  orientations work (locker to the south facing north-out, and to the north
  facing south-out); a comparator locks identically to a repeater.
- Retention verified both ways: lock-while-high holds output HIGH against
  D falling (40 gt), lock-while-low holds LOW against D rising.  Releasing
  the lock resumes tracking within one repeater delay.
- **Baked state survives save -> reload**: a locked repeater frozen HIGH with
  its input lever OFF (a state that contradicts its input) was baked back
  into a schematic, saved to `.schem`, reloaded:
  - `InWorld` settle: **quiescent in 0 gt**, `locked=true`, `powered=true`,
    frozen output intact -- the FPGA-bitstream deployment model works.
  - `Placement` settle: locked flags are re-derived, the sim reaches
    quiescence after 2 gt with the stored state **kept** -- this gadget is
    also `paste_safe` (recorded property, not assumed for other cells).

## Cells (`seq_cells.py`, all PASS)

- **D latch**: data repeater + side locker; transparent when EN=0, holds
  when EN=1 (5-point fixed-tick protocol).
- **MS DFF** (13x4x7 cell, pitch 7): master lock <- CLK, slave lock <-
  NOT(CLK) via wall-torch inverter.  On the rising edge the master locks
  2 gt BEFORE the slave opens, so capture is glitch-free; a designed y3
  bridge carries Q over the cell's own clock column (cap on a straight flat
  run -- legal).  Ports: D west, Q east (buffered), clk_in north / clk_out
  south chained by abutment with one refresh repeater per cell.
  Verified: 11-point protocol + 24-step random D/CLK sequence vs model;
  baked at Q=0 AND Q=1 -> InWorld reload quiescent in 0 gt with correct Q,
  still clocks after reload; both bakes also survive Placement (paste_safe).

## DFF characterization (empirical, exact)

| parameter                | value  | note                                    |
|--------------------------|--------|-----------------------------------------|
| setup (D before edge)    | 0 gt   | D and CLK may toggle in the same gap    |
| hold (D after edge)      | 3 gt   | D may change 3 gt after the edge        |
| min CLK pulse width      | 3 gt   | narrower pulses never engage the locks  |
| clk -> Q                 | 10 gt  | rising edge to Q, through the Q buffer  |
| min period (DFF alone)   | 20 gt  | 10-cycle alternating-D toggling test    |
| clock skew per chained cell | 2 gt | one refresh repeater per cell           |

## Composition

- **register4** (`seq_register4.py`): 4 DFF stamps at pitch 7, clock by
  abutment.  Boundary rows z=-1/z=5 hold nothing but the x=10 clock column
  (the rca_cells seam rule).  16 random write + hold-scramble rounds PASS;
  baked at Q=0 -> InWorld quiescent in 0 gt.
- **counter4** (`seq_counter.py`): register + the existing FA cell column as
  increment (b lanes idle, cin tied high by a torch), Q -> a and sum -> D
  feedback corridors with diagonal-guard blocks severing every dust
  diagonal into foreign nets (alias-aware `nets.check` clean).  Settles to
  Q=0, D=1.  **24 clocked steps count 1..24 mod 16 PASS**; baked at Q=0 ->
  InWorld reload quiescent in 0 gt and counts 5 further steps PASS.
  Functional clock: HIGH 30 / LOW 110 gt.  Measured min period: **100 gt**
  (16-step sweep; 90 gt fails -- the loop clk->Q + Q corridor + FA ripple +
  sum route must settle before the next rising edge).

## Artifacts (showcase/, colour-coded, baked at Q=0, InWorld-quiescent-0)

| file             | blocks | size     |
|------------------|--------|----------|
| `dff.schem`      | 70     | 13x4x7   |
| `register4.schem`| 280    | 13x4x28  |
| `counter4.schem` | 1973   | 50x6x57  |
