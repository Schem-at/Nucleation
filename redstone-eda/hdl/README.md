# hdl/ — Verilog to verified redstone

An HDL flow on top of the PLA compiler in `../build_ppa.py`: Verilog in,
sim-verified, baked-at-rest `.schem` out. Combinational designs verify
exhaustively; **sequential designs (`always @(posedge clk)`) compile through
the built-in Rust pipeline** to a DFF bank + clock spine and verify with a
measured fixed-tick clocked protocol (see "Sequential" below).

```
Verilog ──yosys──> BLIF ──hdl2redstone.py──> PLA stages ──bp.PPA──> blocks
  (synth -lut 4;     (.names          (dual-rail values,     │
   write_blif)        truth tables)    QM covers, buffers)    ▼
                                              audit + net-short check
                                                      │
                                     mc-tick sim, exhaustive vs pure-Python
                                     eval of the same prim graph
                                                      │
                                            bake at rest ──> .schem
```

## Usage

```sh
cd redstone-eda
python3 hdl/hdl2redstone.py --verilog hdl/seg7.v    --top seg7    --out hdl/seg7.schem
python3 hdl/hdl2redstone.py --verilog hdl/cmp4.v    --top cmp4    --out hdl/cmp4.schem
python3 hdl/hdl2redstone.py --verilog hdl/popcnt4.v --top popcnt4 --out hdl/popcnt4.schem
```

Needs `yosys` on PATH (`brew install yosys`) and the `nucleation` Python
module. `--blif x.blif` skips yosys and consumes a hand-made BLIF;
`--no-sim` stops after the structural audit + net-short check; `--cases N`
samples instead of exhausting (automatic above 12 inputs).

## Built-in Rust compiler (`--rust`)

The whole BLIF -> PLA pipeline is BUILT IN now: `crates/nucleation-hdl` is a
line-by-line Rust port of this file + `../build_ppa.py` (parse/fold, QM
off-set covers, peephole, levelise + buffers, slice packing, rails/
inverters/columns/routes/lids), exposed to every binding as the `Hdl`
opaque — `Hdl.compile_blif(blif, name, bake) -> Schematic` and
`Hdl.compile_blif_report(blif, name) -> JSON` (stats, probe and lever
coordinates). Its own gate lives in
`cargo test -p nucleation-hdl --features mc-tick` (seg7 16/16, popcnt4
16/16, cmp4 256/256, exhaustive, vs the pure-Rust prim-graph eval).

```sh
python3 hdl/hdl2redstone.py --blif hdl/build/seg7.blif --top seg7 --rust
```

`--rust` is the parity proof: the SAME BLIF goes through the Rust compiler,
and the Rust-built geometry is driven in mc-tick against THIS file's Python
reference model. This Python path stays as the executable reference.

Because the core compile path is dependency-free and wasm-clean
(`cargo check --target wasm32-unknown-unknown -p nucleation-hdl`), a fully
in-browser flow is one step away: YoWASP's `yosys.wasm` (the upstream
yosys built for WebAssembly, `pip/npm install yowasp-yosys`) synthesises
Verilog to BLIF client-side, and `Hdl.compile_blif` in the wasm binding
turns that BLIF into a schematic without a server round-trip.

## Typed cells (`CellContract`)

Every compile now derives a **typed-cell contract** alongside the schematic:
`Hdl.compile_blif_contract(blif, name) -> JSON` in every binding, in the
exact serde shape of nucleation's `io_contract::CellContract`.

* Vector ports group back into words — both `a[0]..a[3]` (yosys) and
  `a0..a3` conventions parse; index = bit significance (LSB first); single
  bits are `Boolean`, words are `UnsignedInt` (unsigned by default).
* Input port positions are the drive levers, output positions the dust
  probes, with a `face` (nearest lateral bounds face) and `direction` per
  port; uniformly pitched words also get a `buses` entry (spec + bit0).
* The `physical` sidecar carries the build bounds as a keepout, a per
  (input, output) `delays_rt` table **estimated** as levelization depth x
  2rt (not measured characterization), and `paste_safe: false` until proven.

In Rust the pair (schematic, contract) is directly executable:
`BackendCircuitExecutor::for_cell(schem, &contract, extra_states)` wraps the
mc-tick oracle so `set_input("a", Value::U32(11))` drives the lever bank and
`read_output("f")` decodes the probe word — no coordinates in caller code
(gate: cmp4 verified over 64 sampled cases this way, `src/bridge/hdl.rs`).
Note the cell must be **baked** (or settled with `TickSettleMode.Placement`)
before driving: the backend trusts saved block states.

From Python there is no bridged executor surface yet; `typed_demo.py`
interprets the contract directly (positions from the JSON, zero hardcoded
coordinates) over `TickSimulation` and drives seg7 by name: `set d=0xB,
read seg` — 16/16. Wheel builds need
`NUCLEATION_FEATURES="bridge-full,hdl,routing"` (bridge-full alone omits
the Hdl surface).

## Sequential (`.latch` -> DFF bank, `--rust`)

`crates/nucleation-hdl` accepts `.latch <d> <q> re <clk> [<init>]` (what
yosys emits for `always @(posedge clk)` registers; `dffunmap` in the yosys
recipe legalises enable-DFFs to plain latches + muxes). The compile:

* **Latch Q nets become stage-0 input rails** without levers ("ext" rails),
  so the combinational fabric reads state exactly like a primary input
  (both polarities, inverter torch and all).
* **D nets are buffer-raised to the top level**, so the **DFF bank — the
  last stage band** — is fed by ordinary next-stage routes onto bank rails;
  each DFF taps its rail with a y3 flyover spur whose support blocks double
  as caps over every rail row crossed.
* **The bank stamps the verified 13x4x7 master-slave repeater-lock DFF**
  from `../seq_cells.py` (frozen as template data in
  `crates/nucleation-hdl/src/seq.rs`; characterization: setup 0 / hold 3 /
  min pulse 3 / clk->Q 10 / min period 20 gt).
* **Clock = a dedicated y1 spine** north of the DFF row: one lever (the
  contract's clock port), dense repeaters (>= every 8 cells, ~2 gt skew
  each), one dust branch per DFF by adjacency. The clock net never enters
  the dual-rail fabric (compile error if it feeds logic).
* **Q wraps around the fabric** — east out of the cell, south, along the
  west side (x < 0), north past stage 0, and into its input rail's west-end
  dust from the north. Corridor rows/columns are pitched 2 apart with
  depths ordered by slice, which makes the whole wrap planar by
  construction (the counter4 feedback-corridor argument, generalised).
* **Initial state is baked by construction**: the slave repeater + its lock
  path are authored at the declared init (BLIF init 1 bakes powered dust
  levels down the Q path), so the placement settle converges to the
  declared state before any edge — and since a locked slave cuts every
  feedback loop, the at-rest build is quiescent. The schematic IS the
  reset state; there is no reset pin.

Verification (`verify_clocked` in Rust, the same protocol in
`hdl2redstone.py --rust` against an independent raw-BLIF + latch-state
model): reset-by-bake, then per step **fixed-tick only** — set input
levers, *measure* the input-to-edge margin by polling the D-port dust
against the model (never assume a settle), pulse the clock lever (40 gt
high), measure the post-edge margin on Q rails + outputs. Measured
periods: toggle1 72 gt, fsm 130 gt, counter4 140 gt, uart_tx 562 gt.

```sh
python3 hdl/hdl2redstone.py --verilog ../crates/nucleation-hdl/tests/data/counter4.v \
    --top counter4 --rust --steps 24
```

The contract gains the clock as a real Boolean input port (position = the
spine lever) plus a `sequential` sidecar: `clock_port`, the characterized
DFF table, `est_min_period_gt`, and `initial_state` per latch (unknown keys
— the shared `CellContract` parser ignores the sidecar).

## Examples

| design    | function                        | kind         | cases              | result |
|-----------|---------------------------------|--------------|--------------------|--------|
| seg7      | hex digit -> 7-segment decoder  | comb         | 16 exhaustive      | PASS   |
| cmp4      | 4-bit unsigned eq / lt / gt     | comb         | 256 exhaustive     | PASS   |
| popcnt4   | population count                | comb         | 16 exhaustive      | PASS   |
| counter4  | 4-bit counter                   | seq, 4 DFF   | 24 steps exact     | PASS   |
| fsm       | Moore "11" detector             | seq, 2 DFF   | 30 steps           | PASS   |
| toggle1   | toggler, **baked init Q=1**     | seq, 1 DFF   | boots at 1 + 12    | PASS   |
| uart_tx   | 8N1 serializer, /2 baud divider | seq, 15 DFF  | full frame of 0xA5 | PASS   |

Sequential sources + BLIFs are checked in at
`crates/nucleation-hdl/tests/data/` (the Rust crate's own gate drives them:
`cargo test -p nucleation-hdl --features mc-tick`). uart_tx's frame is
verified bit-by-bit: the in-sim run matches the model every step, and the
model's tx tape is asserted to spell start, LSB-first 0xA5, stop, each held
2 clocks. Intermediate BLIF lands in `hdl/build/` for inspection.

## How the mapping works

* **yosys** (`synth -lut 4; write_blif`) reduces any combinational Verilog to
  `.names` truth tables of <= 4 inputs.
* **Constants are folded** out of the netlist (yosys's `$true/$false/$undef`
  nets and anything they imply).
* **Dual rail**: the PLA columns compute OR-of-AND over *positive* rails only;
  torch inverters exist only in the input stage (one lever + complement per
  primary input, exactly the `pg`-stage idiom).  A literal used complemented
  anywhere becomes its own value: for node `f`, `~f` is a second PLA node
  computing the Quine-McCluskey-minimised cover of the off-set over the same
  inputs.  Covers with more than 3 product terms split into an OR tree
  (3 columns is one slice's budget).
* **Levelised, single-hop**: every prim sits at level `1 + max(inputs)` and
  buffer nodes are inserted so each net crosses exactly one stage boundary.
  All routes are therefore next-stage hops on channels 0/1 — the far-route
  corridor conflicts that constrain hand-written netlists can't occur.
* **West->east discipline**: rails only conduct west to east, so slice
  assignment forces every consumer's slice >= its producers' slices
  (nodes cascade eastward; width grows with fan-out, fine at this scale).
* **Verification** drives the levers one at a time (settle after each),
  reads every rail/lane probe, and compares *all* of them — not just the
  outputs — against a pure-Python evaluation of the prim graph, for every
  input assignment.  Then levers are returned to 0, simulator state is baked
  into the schematic, and the `.schem` is saved.

## Limits

* Sequential designs: **single clock domain, rising edge only** (`re`);
  `.subckt`/`.gate` still rejected. The **pure-Python geometry path stays
  combinational** — `.latch` routes to `--rust`. BLIF init 2/3
  (don't-care/unknown) bakes at 0.
* The clock net may only drive latches — a clock feeding `.names` logic is
  a compile error (gate it into an enable-mux instead, which is what yosys
  does anyway).
* Exhaustive verification up to 12 inputs; sampled (edge + random) above.
* One primary input per slice in the input stage; nodes cap at 3 product
  terms per slice (wider SOPs pay an extra OR level).
* No retiming/sharing across polarities: `f` and `~f` are built as separate
  columns even when one would be cheaper as the other plus an inverter.

## Next steps

* ABC genlib mapping onto the verified comparator/adder *cells*
  (`../cells.py`, `../rca_cells.py`) instead of raw PLA columns — smaller
  builds, reuse of the cell library.
* Multi-PI packing in the input stage (2 levers/slice) and producer-aware
  slice ordering to cut the eastward drift.
* Sequential: multiple clock domains / falling edge (second spine +
  inverted lock phases), a Python-side geometry port of the DFF bank, and
  driving compiled sequential cells through `BackendCircuitExecutor`
  (needs a clock-aware step API on the executor).
