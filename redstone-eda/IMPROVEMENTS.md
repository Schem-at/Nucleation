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


## Material model: transparent supports + crossing tiles (LANDED)

Probed and landed per `notes-material-model.md` (the user's spec; PROBED
table now recorded there).  `probe_materials.py` (7/7 controls) pinned the
mc-tick facts: glass / stained glass / slab-top SUPPORT dust but conduct no
weak power and never cut a diagonal (the cut rule runs on CONDUCTIVITY);
slab-bottom and air are vanilla-illegal supports the sim happily ticks
anyway (static-audit concern); and the **transparent diode** — a 1-y step
whose upper dust sits on a non-conductor conducts UP only, so every
bidirectional climb must keep its upper dust on solid.

Landed on that table (`materials.py`, all sim-gated):

- **Predicate refactor (2026-08-08)**: `materials.py` is now architected as
  primitive predicates (`sturdy`, `conducts`, `cuts_diagonal ==
  conducts`) + laws (`step_conducts` — the transparent diode;
  `cap_is_harmful` — only if a diagonal is in use) + cell choosers
  (`pick_support`, `separator`) that COMPUTE every pattern cell from its
  constraints and refuse over-constrained cells.  Consequence: transparent
  appears ONLY where a diagonal must survive; support cells the
  computation shows unconstrained went glass -> solid, and all gates
  re-passed (empirical confirmation of the model).
- **`half_slope_2line`** — two independent lines climbing 1 y per 2 x in
  adjacent z-rows, interleaved at 1-y offset: solid under climb-uppers
  (diode law), solid caps severing the cross-line diagonal at the 1-y
  columns (legal by the cap law: that dust is flat there).  Verified 4
  combos x both directions (9/9) **including the negative control:
  transparent caps mix the levels**, so the cap is proven load-bearing,
  not decorative.
- **`crossing_parity`** + **`crossing_dipunder`** — two verified 90-degree
  crossing tile families; in both, a block-sandwich station's ENTRY block
  doubles as the isolation (it cuts the through dust's up-diagonals and
  blocks the trunk's down-read).  Parity: crossing bus at +1 y rides its
  entry block over the through dust.  Dip-under: the crossing bus steps
  down 1 onto glass, runs under the entry block (which needs no support —
  that is what frees the cell), steps back up; both steps keep uppers on
  solid so the tile conducts both ways.  `crossing_tiles.py` verifies 4
  combos per tile (conduction + per-bit isolation) and saves
  `crossing_tiles.schem` baked at rest.
- **Model/checker integration**: `nets.py` gained material classes
  (`material`, conductor-backed `is_solid`, `step_conducts_down`);
  `audit.py` gained `is_sturdy` (floor components legal on glass/slab-top,
  still illegal on slab-bottom/air); `router.py` `dust_ok` accepts existing
  transparent supports; glass/slab states interned in `rs.EXTRA_STATES`.
  Full gate suite re-run green (test_router, rca2 32/32, demo1,
  accumulator, seg7 16/16).
- **Deferred**: router *exploitation* of transparent supports (placing
  glass, diode-aware descent — a naive down-move guard measurably broke
  A*/emitter interplay and was backed out, see note in `router.py`); wiring
  the crossing tiles into a bus router on crossing demand; influence-map
  bookkeeping (notes-material-model.md §4).

## Dense vertical buses + 8x8 crossing (LANDED)

The material model cashed out as a bus form (`bus8_probe.py`,
`bus8_cross.py`; showcase `bus8_run.schem`, `bus_cross8.schem`):

- **2y-pitch vertical bus, 1 block wide (v2)**: bit n dust at y=1+2n over
  full SOLID separator layers — NO transparent blocks: a straight run has
  no diagonals in use, so the cap law says a conductor layer is harmless
  (probed P1: caps over a live run conduct + dust/solid/dust isolated).
  Cross-section for 8 bits: **1 wide x 16 tall = 16 cells** (2 cells/bit).
  The flat 2z-pitch form spends 15x2 = 30+ cells of cross-section and 15
  columns of ground footprint for the same 8 bits — the vertical form is
  ~2x denser in section and 15x narrower on the ground, and it leaves the
  ground plane free for logic.
- **Refresh in-stack (v2)**: block-sandwich stations in ALIGNED columns
  (every bit's entry at the same x; v1's diagonal stagger removed — the
  repeater floor capping the bit below's live dust is exactly probe P1,
  harmless; P2 shows even v1's solid 'lid' floors were never electrically
  load-bearing).  Max clean pitch stays the probed 18 (15 dust + 3
  station); verified 96/96 over a 40-block run with two refresh stages,
  incl. the ss1 worst case (bit 7).
- **8x8 crossing v2 at CANONICAL levels** (`bus_cross8.schem`): both buses
  at the SAME levels y=2+2n — a bus's level never depends on crossing
  history (composability).  Bus B dips 1 down, passes under A's level
  through its own station (entry blocks interleaving A's in the shared
  column), climbs back up; the dip's two slope-transition supports per bit
  are the ONLY glass (each sits above the lower dust of the bit below's
  step diagonal; the paired solids are diode + cross-bit sever — the
  alternation, now probed at stack scale).  Core 3x7 ground x 17 tall, 16
  repeaters (1/bit, doubling as the refresh), 512 blocks.  Verified
  432/432 (27 patterns x 16 outputs) incl. 8 random joint patterns.
- **v1 tower kept** (`bus_cross8_tower.schem`, 432/432): 3x3-ground core,
  zero glass — genuine wins — but B shifts to even levels (the
  composability defect v2 removes).  A flat 8x8 crossing would need 64
  pairwise crossing tiles over a ~16x16 ground area.
- **Deferred**: bus-router integration (stamping the crossing on demand);
  T-junctions/taps off a vertical bus (exit-block strong-power leak makes
  mid-run taps nontrivial).

## Form-pivot adapters + HexAnalog bus form (LANDED, 2026-08-09)

Two template families closing the bus-form gaps (`pivot_tiles.py` +
`pivot_tiles.md` cell listings for the Rust port; `compositor/
hexanalog_bus8.py`; four showcase pieces):

- **NEW PROBED PHYSICS — the pointing law** (`probe_pivot.py`, 5/5;
  `materials.dust_drives_block`): dust weak-powers only the blocks on its
  connection axes, so a run that turns a corner does NOT fire a station
  entry block around the corner; strong exit blocks light dust on any
  side.  Found the honest way: the first v2h/flat90 layouts failed
  exactly at their corner-entered stations (75/96, 72/96) while the
  straight-entered h2v scored 96/96 — the law explains all three.
- **pivot_v2h / pivot_h2v**: vertical(2y) <-> flat(2z) adapters — fan
  column (separator stack) + per-lane station + uniform-budget 1y/1x
  staircase; 14-step descent probed; 96/96 each.  Distinct tiles (the
  repeater direction is the only difference).
- **pivot_flat90**: concentric flat corner, order preserved in the travel
  frame; in-leg + out-leg stations per the pointing law; 96/96.
- **hexanalog_bus8**: 8 bits as TWO analog wires (nibble per wire), with a
  probed comparator-SANDWICH trunk station (exact ss through dust ->
  block -> comparator -> block -> dust); 256/256 bytes exhaustive in
  Gray order.  Cross-section 2 wires vs binary's 16 cells; the cost is
  codec area (~40x40 per end) and 1 gt per trunk comparator.
- **Deferred**: router-side stamping of these tiles; a DENSE pivot
  (interleaved 8-bit staircase at 1y separation via the alternation law)
  was sketched but is over-constrained at aligned columns — needs the
  half_slope stagger, left for a follow-up; analog trunk corners (the
  comparator chain turning 90°) unprobed; nibble bands sit 60 z apart
  because the encoder decay lanes sprawl — packing codecs is open.


What is actually weak, drawn from the session docs, the deferred lists in
`demos/README.md` / `showcase/README.md`, `CORE_PROPOSALS.md`, and
`ROUTING_CRATE_DESIGN.md`. P1 = blocks or silently corrupts real use;
P2 = limits scale/quality; P3 = worth doing when touched.

## Typed HDL cells: CellContract from the compiler (LANDED)

`nucleation-hdl` now derives a `CellContract` at compile time
(`contract.rs`): vector-port grouping (`a[i]` and `aN` conventions) into
typed word ports, levers/probes mapped per bit, nearest-face + bus-pitch
geometry, bounds keepout, and an **estimated** delay table (levelization
depth x 2rt — ordering-grade, not measured). The hand-rolled JSON is
schema-checked by nucleation: `bridge::hdl::cell_contract` parses it into
the real `io_contract::CellContract` (round-trip asserted in tests), and
`Hdl.compile_blif_contract` exposes it in every binding.

Execution is typed end-to-end: `BackendCircuitExecutor::for_cell(schem,
contract, extra_states)` over `McTickBackend` — cmp4 verified 64 sampled
cases by port name/word only; `hdl/typed_demo.py` drives seg7 16/16 from
Python with every coordinate read from the contract. Remaining gaps:

- **P2 Bridge the typed executor.** Python interprets the contract itself;
  a `TypedCell` opaque (load schematic + contract, set/read by name) would
  remove that duplication in every binding.
- **P2 Measured characterization.** Replace the estimated `delays_rt` with
  a scripted executor sweep per port pair, and measure `drive_strength`;
  then flip the contract to measured and set `initial_state` from bake.
- **P3 `bridge-full` omits `hdl`/`routing`** — wheels silently lose the
  surface unless `NUCLEATION_FEATURES="bridge-full,hdl,routing"` is set
  (bit the wheel rebuild here; same class as the P1 wheel-drift note).

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
  router/checkers/STA are ported; ~~the PLA compiler, cell library, and the
  verification harness are still Python-only~~ **DONE for the HDL flow**:
  `crates/nucleation-hdl` ports the whole BLIF -> PLA pipeline
  (`hdl/hdl2redstone.py` + `build_ppa.py`: parse/fold, QM, peephole,
  levelise, slice packing, full geometry emission) plus the mc-tick
  verification harness (feature `mc-tick`), bridged as `Hdl.compile_blif` /
  `Hdl.compile_blif_report` in every binding; `hdl2redstone.py --rust`
  proves parity in-sim against the Python reference model. Still
  Python-only: the hand-written generators (`build_ppa.py`'s Kogge-Stone
  netlist, ALU/mult), the cell library, and audit/nets/timing for the HDL
  flow (the routing crate carries its own ports of those). The Python files
  remain the spec and executable tests.
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

## Genlib mapping onto comparator cells (LANDED, 2026-08-09)

`genlib_map.py`: yosys `abc -genlib` maps Verilog onto a library of seven
VERIFIED flat cells (torch INV, comparator-subtract AND2/XOR2, repeater-join
OR2, and inverted tails), with areas/delays in the genlib taken from the
measured fragments.  Levelized column placement; the A* maze router carries
the inter-cell nets; exhaustive in-sim verification against a pure-Python
eval of the mapped netlist gates every schematic before it may be saved.
Saved (verified + baked): `showcase/genlib_seg7.schem`,
`showcase/genlib_cmp4.schem`.

Measured against the PLA fabric's tracked artifacts (same measurement:
non-air blocks, tight bbox, structural STA):

| design | fabric | blocks | bbox | volume | crit path | verified |
|---|---|---|---|---|---|---|
| seg7 | PLA (hdl/) | 8627 | 254x8x75 | 152400 | 35 rt | 16/16 |
| seg7 | genlib cells | **6880** (-20%) | 128x7x100 | **89600** (-41%) | 35 rt | 16/16 |
| cmp4 | PLA (hdl/) | 13873 | 271x8x135 | 292680 | 43 rt | 256/256 |
| cmp4 | genlib cells | **3560** (**3.9x**) | 149x7x71 | **74053** (**4.0x**) | **28 rt** (-35%) | 256/256 |

Honest reading: the ~3x-density hypothesis holds on cmp4 (3.9x blocks, 4.0x
volume, 35% faster) where the PLA's dual-rail overhead dominates; seg7 —
whose PLA is a single dense truth table, the shape PLAs are best at — gains
only 20% blocks / 41% volume.  Cell area is a rounding error (814/558
blocks); ROUTING is the fabric cost, so channel geometry sets density.
TRICKS.md (community corpus) was reviewed; no new idiom was adopted this
round — the comparator-subtract AND/XOR and block-sandwich stations that
make the cells flat were already probed in cells.py/probe_station.py.

Fabric physics the composition run flushed out (all now router rules, each
found as a real in-sim failure of a 45-gate build):

- **Analog arrivals kill comparator cells**: a wire legally delivers ss1;
  a comparator cell fed at ss3 output ss0.  Every cell input port now has
  a repeater directly behind it, so cells always see the verified
  lever-strength condition (this is why cell delays grew ~1 rt).
- **Own-net repeater rings at fabric scale**: a same-net branch grazing its
  trunk on both sides of a repeater closes a self-sustaining ring that
  LATCHES (nets.check is blind: one net).  `Router.move_ok` hook: new dust
  may touch own-net dust only via its path predecessor or the destination
  aperture.
- **Strong-source injection**: station ENTRY blocks, torch-ladder torches
  and their strong-powered blocks, and PI LEVERS all write 15 into any
  adjacent foreign dust; none are in nets.py's dust-only model.  All are
  now strong-claimed at emission (and ladders refuse to stand beside
  existing foreign components — `ladder_clear`).
- **Self-conflicting flyovers**: a path crossing 1 above its own earlier
  segment lays its support ON the lower dust; crossing 2 above CAPS the
  lower dust's diagonal (found dead at a corridor mouth).  Path post-check
  + veto-retry.
- **Port pockets seal**: parked foreign trunks legally fence a port's only
  approach.  Fix stack: reserved self-refreshing 5-deep approach corridors
  (mouth dust -> repeater -> port), soft cost halos near mouths, and hard
  per-cell obstruction boxes (foreign nets may not route inside a cell's
  claimed region at all).
- **Trunk strength tracking**: `Router.ss` records estimated signal
  strength per emitted dust; branches only start from cells >= ss4 and
  emit resumes the refresh budget from the branch cell's recorded value.

Caveat, recorded honestly: composition correctness is layout-sensitive —
one spacing combination (COL_CHANNEL=16/CLEAR_Z=6) produced a 13/16 build
while both looser and tighter combos verify 16/16, so at least one hazard
class is still unmodelled.  The exhaustive verification gate is what makes
the flow trustworthy; do not ship an unverified layout.
