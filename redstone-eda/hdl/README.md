# hdl/ — Verilog to verified redstone

A combinational HDL flow on top of the PLA compiler in `../build_ppa.py`:
Verilog in, exhaustively sim-verified, baked-at-rest `.schem` out.

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

## Examples (all verified exhaustively)

| design    | function                        | inputs | cases | result |
|-----------|---------------------------------|--------|-------|--------|
| seg7      | hex digit -> 7-segment decoder  | 4      | 16    | PASS   |
| cmp4      | 4-bit unsigned eq / lt / gt     | 8      | 256   | PASS   |
| popcnt4   | population count                | 4      | 16    | PASS   |

Intermediate BLIF lands in `hdl/build/` for inspection.

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

* **Combinational only** — `.latch` (and `.subckt`/`.gate`) are rejected.
  No flip-flops, no clocks.
* Exhaustive verification up to 12 inputs; sampled (edge + random) above.
* One primary input per slice in the input stage; nodes cap at 3 product
  terms per slice (wider SOPs pay an extra OR level).
* No retiming/sharing across polarities: `f` and `~f` are built as separate
  columns even when one would be cheaper as the other plus an inverter.

## Next steps

* ABC genlib mapping onto the verified comparator/adder *cells*
  (`../cells.py`, `../rca_cells.py`) instead of raw PLA columns — smaller
  builds, reuse of the cell library.
* Sequential support: map `$dff` onto a verified redstone latch cell, add a
  clock lever/driver and settle-per-phase simulation.
* Multi-PI packing in the input stage (2 levers/slice) and producer-aware
  slice ordering to cut the eastward drift.
