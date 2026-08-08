# Redstone EDA

Electronic design automation for Minecraft redstone: **Verilog in, exhaustively
sim-verified `.schem` out**, plus the place-and-route stack that makes it
possible — a Python toolchain (the spec and test vectors) backed by Rust crates
(`pnr-core`, `nucleation-routing`) and a generated `Routing` bridge surface in
every language Nucleation ships.

Every artifact in this tree was verified by its generator before saving, in
mc-tick (the vanilla-accurate tick simulator), and saved **baked at rest** —
levers off, circuit quiescent, settled block states written back — so what you
paste into a world is exactly what the simulator proved.

## Quickstart

Build the Python wheel from this branch (`feat/redstone-eda`); the `routing`
feature is NOT part of `bridge-full` and must be named explicitly:

```sh
python3 -m venv ~/eda-venv
NUCLEATION_FEATURES=bridge-full,routing ~/eda-venv/bin/pip install ./bindings/python
```

Then run a demo, or compile Verilog end to end:

```sh
cd redstone-eda/demos
~/eda-venv/bin/python demo1_route.py        # bridge routing + DRC + in-sim conduction proof

cd ..                                        # needs yosys on PATH (brew install yosys)
~/eda-venv/bin/python hdl/hdl2redstone.py --verilog hdl/seg7.v --top seg7 --out hdl/seg7.schem
```

See `demos/README.md` for the full wheel-build gotchas (stale vendored copy
under `bindings/python/rust/`, `EXTRA_STATES` interning, bounding-box traps).

## Directory map

| Path | What it is |
|---|---|
| `rs.py` | The `Build` DSL: full block-state strings, colour palette, save/sim helpers, `EXTRA_STATES` interning |
| `cells.py`, `rca_cells.py` | The verified comparator-cell library (half/full adder truth-table cells) and the N-bit ripple-carry generator |
| `router.py`, `nets.py`, `audit.py`, `timing.py`, `test_router.py` | Python prototypes of router, dust-adjacency/short checker, structural support audit, and STA — the spec and test vectors for `crates/nucleation-routing` |
| `build_adder.py`, `build_ppa.py`, `build_alu.py`, `mult4.py` | Netlist builders: RCA, the Kogge-Stone PLA compiler (channel-routed rails), 8-bit ALU, stacked 4x4 multiplier |
| `seq_cells.py`, `seq_counter.py`, `seq_register4.py`, `seq_probe.py`, `seq_README.md` | Sequential logic: repeater-lock latch, master-slave DFF (characterized: setup 0 / hold 3 / clk->Q 10 gt), register, synchronous counter |
| `probe_comp.py`, `probe_rep_side.py`, `probe_vert.py`, `probe_station.py`, `probe_materials.py` | Micro-probes that settled physics questions (comparator sides, repeater side-locks, vertical conduction, block-sandwich repeater/comparator stations + refresh-pitch floors, the material table: glass/slab supports, conductivity-based cuts, the transparent diode) |
| `materials.py`, `crossing_tiles.py`, `notes-material-model.md` | Material-aware patterns on the probed table: the 2-line half slope (transparent/solid alternation, 1-y pitch), and the two 90-degree crossing tiles (y-parity and dip-under; a station's entry block doubles as the isolation) |
| `hdl/` | The Verilog flow: yosys -> BLIF -> `hdl2redstone.py` -> dual-rail PLA stages -> verified `.schem` (seg7, cmp4, popcnt4 examples) |
| `compositor/` | Compositor MVP: compose characterized cells (register + adder = accumulator), mixed-level `functional_sim.py`, HexAnalog (4 bits on one wire's signal strength), annealing demo |
| `demos/` | Four end-to-end bridge demos: route+DRC+conduction, introspection (`conduction_trace`/`read_probes`/`bake_to`), cell composition, analysis (`drc`/`sta`/repeater-cycle) |
| `showcase/` | The artifact gallery (below) + self-verifying wrapper scripts |
| `*.schem` (root) | Verified artifacts saved by the builders (rca4, Kogge-Stone 32-bit, ALU, multiplier) |
| `CORE_PROPOSALS.md` | What computational redstone asked of Nucleation core (much of it since built: introspection, bake, STA, LVS) |
| `ROUTING_CRATE_DESIGN.md` | The crate design: fabric model, rule set, DRC/LVS/STA story, roadmap, bus/sequential/mixed-level design briefs |

## The gallery

Twelve pieces, 217,826 blocks, 1,156 verification cases/checks. Full
verification-evidence table in `showcase/README.md`; regenerate any piece with
`~/eda-venv/bin/python showcase/showcase_<name>.py` (each exits non-zero unless
its verification is perfect).

| Piece | Blocks | Demonstrates | Evidence |
|---|---|---|---|
| `router_gallery` | 239 | Bridge routing: obstacle dive, vertical via, 3-net braid, shared-trunk fork | 7/7 conduct, braid isolation, DRC 0 |
| `adder4_cells` | 988 | Dense comparator-cell RCA, carry by abutment | exhaustive 512/512 |
| `mult4x4_stacked` | 27,222 | 4 stacked planes, 3D maze-routed inter-plane nets | exhaustive 256/256 |
| `kogge_stone_32bit` | 154,152 | Flagship: 32-bit prefix adder from the PLA compiler | 54/54 (47 random + 7 corners) |
| `alu8` | 30,052 | 8-bit 4-op ALU (ADD/SUB/AND/XOR) | 144/144 |
| `bus_riser8` | 168 | 8-bit bus routed up two levels, one `route_net` per bit | 8/8, skew-matched 4 ticks |
| `dff` | 70 | MS rising-edge DFF from repeater locks | 11-pt protocol + 24 random steps, characterized |
| `register4` | 280 | 4-bit register, clock chained by abutment | 16 write + 16 hold rounds |
| `counter4` | 1,973 | The sequential loop closed: register + FA increment | 24 steps mod 16, min period 100 gt |
| `accumulator4` | 2,059 | Compositor MVP composition | 24/24 clocked steps, DRC clean, LVS opens=0 |
| `hexanalog_trunk` | 556 | 4 bits on one wire's signal strength + decoder | encode/decode 16/16 exhaustive |
| `crossing_tiles` | 67 | Both 90° bus-crossing tiles (y-parity + glass dip-under), station entry block as isolation | 4 combos x 2 tiles, per-bit conduction + isolation; regen `crossing_tiles.py` |

## Architecture

```
Verilog ──yosys──> BLIF ──hdl2redstone──> PLA stages ──build_ppa──> blocks
                                                 │
   cells.py / seq_cells.py  (truth-tabled comparator cells, DFF)
                                                 │
   router / nets / audit / timing   (place-and-route + static checks)
                                                 │
   mc-tick simulation: exhaustive/sampled vs pure-Python model of the netlist
                                                 │
                                     bake at rest ──> .schem
```

- **Compiler** (`build_ppa.py`, `hdl/hdl2redstone.py`) — dual-rail PLA:
  OR-of-AND over positive rails, torch inverters only in the input stage,
  Quine-McCluskey covers, levelised single-hop buffering, west->east slice
  discipline.
- **Cells** (`cells.py`, `seq_cells.py`) — truth-tabled comparator cells
  stamped at pitch; carry/clock chains connect by abutment (zero routing).
- **Router** (`router.py` -> `nucleation-routing`) — A* over the redstone
  fabric with signal budget (decay, refresh every 5, stair cap 4), torch-ladder
  vias, support emission, repeater insertion.
- **Checkers** — structural support audit (`audit.py`), dust-adjacency short
  check with cut diagonals (`nets.py`), DRC (shorts/support/decay/repeater
  cycles), LVS (intended netlist vs extracted conduction: opens, shorts, rings).
- **STA** (`timing.py` -> `sta`) — torch 1 rt, repeater = delay, dust free;
  critical path without running the sim, cross-checked against measured
  `tick_count()` deltas.
- **Bridge** (`src/bridge/routing.rs`) — `Routing.route_net` / `route_all` /
  `drc` / `lvs` / `sta`, JSON in/out, generated for all seven languages.

The full design rationale — every rule paid for by a real in-sim bug — is in
`ROUTING_CRATE_DESIGN.md`.

## Verification philosophy

Nothing in this tree is trusted because it "looks right":

- **Every artifact is gated.** Builders exit non-zero unless verification is
  perfect; showcase wrappers parse the verification line and refuse anything
  short of a full score.
- **Per-node checking, not just outputs.** The HDL flow compares *every*
  rail/lane probe against a pure-Python evaluation of the prim graph for every
  input assignment — an open circuit is a perfectly quiescent circuit, so
  output-only checks pass dead builds.
- **mc-tick is the golden model.** All conduction proofs run in the
  vanilla-accurate tick simulator (itself verified against captures from the
  game); static checks (DRC/LVS/STA) accelerate debugging but the sim has the
  final word.
- **Baked at rest.** Saved schematics carry their settled quiescent state and
  reload InWorld-quiescent in 0 gt.

## Rust crate map

| Component | Where | Role |
|---|---|---|
| `pnr-core` | `crates/pnr-core` | Fabric-agnostic PnR: grid/A*, negotiated congestion, placement annealing, union-find net check, STA delay DAG |
| `nucleation-routing` | `crates/nucleation-routing` | The redstone fabric: block states, transactional `Workspace`, nets/audit/budget/via, rules, `route`/`route_all`, `CellTemplate`/`BusSpec`/regions, DRC, LVS, STA |
| `routing` feature seam | `src/routing.rs` | Re-exports + schematic-level entry points (`route_all_schematic`, `lvs_schematic`, ...) |
| Bridge surface | `src/bridge/routing.rs` | `Routing.*` for Python/JS/C/C++/Kotlin/PHP via Diplomat; JSON-in/JSON-out |
| IO contracts | `src/io_contract/` | `CellContract` faces, physical sidecar, buses, DEF-style routing regions, Insign vocabulary, `compile_io_contracts_json` |
| Sim introspection | mc-tick bridge | `conduction_trace`, `read_probes`, `bake_to`, `last_error_detail` — the debugging loop the Python flow runs on |
| `SimBackend` | `src/simulation/typed_executor/backend.rs` | Backend trait under `TypedCircuitExecutor` (mc-tick / MCHPRS); the routing crates themselves stay sim-free |

## What's next

See `IMPROVEMENTS.md` for the honest, prioritized list (bridge gaps, density,
robustness, performance, architecture, process).
