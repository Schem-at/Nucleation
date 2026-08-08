# Improvements — honest and prioritized

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
