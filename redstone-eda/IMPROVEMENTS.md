# Improvements — honest and prioritized

## Circuit-level optimizations (LANDED)

Three user-suggested optimizations, probe-verified in mc-tick
(`probe_station.py`, 20/20 expectations) and landed behind the full gate
suite (rca2 32/32, rca4 512/512, adder4 512/512 + reload re-check, demos
1–4, seg7 16/16, cmp4 256/256, popcnt4 16/16, seq_probe, accumulator 24/24,
test_router incl. a station-exercising net).

**1. Block-sandwich repeater stations** (`router.py` emit): dust → block →
repeater → block → dust.  Probed semantics: the entry block fires the
repeater from a ss1 dust arrival (15-dust max run, same as inline); the exit
block re-emits a strong fresh 15 — so one repeater covers 18 cells of run
instead of 16, and the blocks need no supports.  Hazards, all probed: the
exit block powers EVERY adjacent dust at 15 (leak — `dust_ok` now keeps
later routes out of its 6-neighbourhood via `Router.strong`); dust atop
either block joins the net diagonally; and the trunk dust must be a straight
line flanked only by solids — any connectable neighbour (dust, repeater,
torch), own net included, bends its shape off the block face and kills the
drive (this exact case broke the FA cell's cin route against a port-stub
repeater before `station_ok` learned to check it).  Emission falls back to
an inline repeater wherever `station_ok` says no.

**Comparator variant for analog trunks — better than expected**: probed
dust(ssN) → block → comparator(compare) → block → dust reads back EXACTLY N
(tested N=12 and N=8; through-block rear read and through-block re-emit each
preserve ss).  So block-sandwich comparator stations ARE usable on HexAnalog
trunks; the "comparator needs a direct back input" worry is false in
mc-tick.  Not yet wired into any emitter — recorded here as verified physics.

**2. Refresh intervals at measured max pitch, margin 2** (was REFRESH=5 /
run 6 / run_path 5 — debugging-era):
- Tap-free routes: a repeater fires from ss1, so true max is 15 dust between
  refreshes; set 14 (last dust at ss3).  `router.REFRESH`,
  `build_ppa.PPA.ROUTE_REPEAT`, `build_adder.run_path`.
- First refresh stays within 5–6 cells (source strength unknown at a cell
  port), and a refresh is banked at the last straight cell before a
  repeater-free tail (stairs/landings) — at pitch 14 a tail could otherwise
  start on a nearly-spent budget and die.
- **Rails: RAIL_REPEAT=12 is NOT over-conservative — unchanged.**  A rail is
  tapped from the side; the tap dust reads rail−1 and a tap on a ss1 rail
  cell never flips its torch (probe G).  So rails need ss≥2 everywhere:
  worst gap 13 (12 + bad-column skip) → min rail ss 4 = floor 2 + margin 2.
  Same reason stations don't pay on rails: the tap floor binds, not the
  repeater's input floor.

**3. Double-inversion elimination** (`hdl/hdl2redstone.py`
`Compiler.peephole`, before `levelise` — levelise's own buffers are
structural): collapses every pure buffer node (OR-of-AND over one literal)
by aliasing.  Dual-rail construction turns a BLIF NOT into a buffer of the
complement rail, so NOT(NOT(x)) chains collapse to nothing and consumers
take the already-available polarity rail — this IS the polarity-aware
selection.  Proven on a synthetic 4-deep NOT chain (4 nodes collapsed,
sim-verified 4/4); on yosys-synthesized seg7/cmp4/popcnt4 it removes 0 —
`opt_clean` already leaves no such chains, honest result.  The Kogge-Stone
netlist's `i < s` pass-through buffers are deliberately NOT touched: they
re-drive each prefix stage's rails and removing them changes the far-route
channel discipline.

Measured on the regenerated, fully re-verified artifacts (total block count
is invariant — a repeater swaps 1:1 with a dust cell; the win is repeater
count, i.e. delay and STA):

| artifact | blocks | repeaters | torches | STA critical path |
|---|---|---|---|---|
| ripple_carry_adder_4bit | 3362 → 3362 | 85 → **82** (−3.5%) | 112 | n/a (non-PPA) |
| rca4_cells | 988 → 988 | 84 → **80** (−4.8%) | 0 | n/a (comparator cells) |
| hdl/seg7 | 8627 → 8627 | 340 → **316** (−7.1%) | 167 | 37 → **35** rt |
| hdl/cmp4 | 13873 → 13873 | 610 → **532** (−12.8%) | 190 | 48 → **43** rt (−10%) |
| hdl/popcnt4 | 4419 → 4419 | 181 → **163** (−9.9%) | 99 | 29 → **27** rt |

ks32 / alu8 / mult4x4 were NOT regenerated (multi-hour sweeps; their tracked
schems remain artifacts of the old, more conservative pitch and stay valid).
`hdl2redstone` now also prints the STA critical path on every run.

Deferred from this round: stations in `build_ppa.run`/rails (STA repeater
attribution walks dust labels and would miss station repeaters; rails don't
benefit anyway — see above), and comparator stations on HexAnalog trunks.


What is actually weak, drawn from the session docs, the deferred lists in
`demos/README.md` / `showcase/README.md`, `CORE_PROPOSALS.md`, and
`ROUTING_CRATE_DESIGN.md`. P1 = blocks or silently corrupts real use;
P2 = limits scale/quality; P3 = worth doing when touched.

## Bridge gaps

- **P1 `route_all` air-endpoint handling + order sensitivity.** Endpoints in
  air are not handled uniformly, and results depend on the order nets are
  submitted — negotiated congestion exists in `pnr-core` but the bridge path
  does not surface a deterministic, order-independent contract. Fix the
  contract first, then document it.
- **P1 `spacing` / `direction_bias` accepted but unenforced.** The options
  parse and are silently ignored. Either enforce them in the fabric rules or
  reject them; accepting-and-ignoring is the worst outcome.
- **P2 LVS merges-through-components semantics.** The extractor merges nets
  through components in ways the intended-netlist side does not expect;
  callers must know which merges are real. Needs a spelled-out semantics doc +
  tests, likely a per-component merge policy.
- **P2 Bridge `sta` has no net labels** on a bare schematic, so per-net
  repeater delays contribute 0 through this path (known-deferred). Label-aware
  STA and short checking need the native `Workspace` bridged — see
  Architecture.
- **P3 Python error detail.** The wheel exposes
  `TickSimulation.last_error_detail` instead of `NucleationError.detail()`
  (nanobind template does not emit enum methods). Ugly but workable; fix in
  the generator.

## Density

- **P2 Cells are unannealed.** The placement annealer exists (`pnr-core`)
  but every artifact uses hand-pitched placements. Wire it to `CellAbstract`s
  with HPWL + overlap + STA cost, accept only if `route_all` + DRC pass
  (roadmap C2).
- **P2 The FA cell (22x5x13) has slack** — it was the first one that worked,
  not the smallest. A compaction pass over the cell library pays off in every
  composed build.
- **P3 `SLICE_W` is conservative.** The PLA slice budget (3 product terms,
  one PI per input slice) leaves area on the table; multi-PI packing and
  producer-aware slice ordering are already sketched in `hdl/README.md`.
- **P3 Sparse Kogge-Stone / HexAnalog arithmetic revisit.** The 32-bit KS is
  dense; sparse prefix trees and the verified HexAnalog trunk (4 bits on one
  wire) could cut interconnect dramatically (roadmap E4).

## Robustness

- **P1 Compare-mode comparators are unverified.** The cell library and
  HexAnalog work leans on subtract-mode behaviour that was probe-verified;
  compare mode has no probe suite. Anything that starts using it inherits
  untested physics.
- **P1 The bridge workspace is label-blind; clearance discipline is manual.**
  Multi-net bridge pieces stay isolated only by 2-block clearance convention,
  proven after the fact in-sim. Halos / net labels should move into
  `Workspace` proper so DRC enforces isolation statically instead of
  culturally.
- **P2 Mixed-level delay tables are protocol-scoped.** The characterized
  numbers (`compositor/MIXED_LEVEL.md`) are valid only under the measured
  protocols; new topologies need re-measurement, and glitch behaviour is
  bounded, not modelled. Never use behavioural sim to explore illegal
  schedules — enforce that in the API, not in prose.

## Performance

- **P2 Verification throughput.** The big sweeps (ks32, alu, mult) are
  multi-hour. Parallel sim instances, incremental verification (only re-prove
  what changed), and the `redstone_connectivity`-style static continuity check
  (`CORE_PROPOSALS.md` §6) would each cut hours.
- **P3 Batched probes are underused.** `read_probes` exists precisely to
  batch, but several Python helpers still read one block per call inside
  settle loops.

## Architecture

- **P2 Migrate the Python toolchain into the crates, progressively.** The
  router/checkers/STA are ported; the PLA compiler, cell library, and the
  verification harness are still Python-only. The Python files are the spec —
  keep them as executable tests while the crates absorb the logic
  (`ROUTING_CRATE_DESIGN.md` roadmap).
- **P2 Genlib mapping onto the comparator cells.** yosys ABC mapping onto the
  verified cell library instead of raw PLA columns should shrink HDL output
  roughly 3x and reuses characterized cells (roadmap E2).
- **P2 Sequential HDL.** `.latch` is rejected today; map `$dff` onto the
  characterized DFF cell + clock driver + settle-per-phase protocol (roadmap
  A3/E2). The cell, register, and counter already exist and are characterized.
- **P3 Clock trees.** Clock currently chains by abutment (2 gt skew/bit);
  wider designs need a distribution tree with skew budgeting in STA.
- **P3 Steiner nets.** Multi-terminal nets are incremental joins today;
  proper Steiner trees are the known-missing router feature (design doc
  "missing for state of the art" #6).

## Process

- **P1 CI for the demo suite.** demo1–4, `rca_cells --bits 2`, `seg7`,
  `seq_probe`, `accumulator` all run in minutes and gate the whole stack;
  none run in CI. This move was verified by hand — that should be a workflow.
- **P1 Wheel-drift detection.** A stale venv wheel silently lacked
  `Routing.lvs` during the post-move verification (AttributeError deep in a
  40-minute run). Scripts should assert the bridge surface they need at
  import time, and the wheel should expose a build fingerprint.
- **P2 Golden-file tests for schems.** The showcase pieces are deterministic;
  regenerate-and-diff (or fingerprint-compare) against the tracked `.schem`s
  would catch regressions without re-running verification sweeps.
